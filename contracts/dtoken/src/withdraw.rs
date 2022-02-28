use crate::*;

impl Contract {
    pub fn withdraw_ft_transfer_fallback(
        &mut self,
        user_account: AccountId,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) {
        log!(
            "withdraw_ft_transfer_fallback - user {}, tokens {}, dtokens {}",
            user_account,
            Balance::from(token_amount),
            Balance::from(dtoken_amount)
        );
        //TODO: implement fallback method
    }
}

#[near_bindgen]
impl Contract {

    pub fn withdraw(&mut self, dtoken_amount: WBalance) -> Promise {
        return underlying_token::ft_balance_of(
            self.get_contract_address(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::withdraw_balance_of_callback(
            Balance::from(dtoken_amount),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(140),
        ));
    }

    pub fn withdraw_balance_of_callback(&mut self, dtoken_amount: Balance) -> Promise {
        let promise_success: bool = is_promise_success();
        assert_eq!(
            promise_success,
            true,
            "Withdraw has failed on receiving UToken balance_of: Account {} token {}",
            env::signer_account_id(),
            self.get_underlying_contract_address()
        );
        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let exchange_rate: Balance = self.get_exchange_rate(WBalance::from(balance_of));
        let token_amount: Balance = Balance::from(dtoken_amount) / exchange_rate;

        return controller::withdraw_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount.into(),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::withdraw_supplies_callback(
            env::signer_account_id(),
            token_amount.into(),
            dtoken_amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(70),
        ));
    }

    pub fn withdraw_supplies_callback(
        &mut self,
        user_account: AccountId,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) ->Promise {
        let promise_success: bool = is_promise_success();

        assert_eq!(promise_success, true, "Withdraw supplies has been failed");

        // Cross-contract call to market token
        underlying_token::ft_transfer(
            user_account,
            token_amount,
            Some(format!("Withdraw with token_amount {}", Balance::from(token_amount))),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(40),
        )
        .then(ext_self::withdraw_ft_transfer_call_callback(
            dtoken_amount.into(),
            token_amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(10),
        ))
    }

    pub fn withdraw_ft_transfer_call_callback(
        &mut self,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) -> bool {
        let promise_success: bool = is_promise_success();

        if promise_success {
            self.burn(&env::signer_account_id(), dtoken_amount);
        } else {
            self.withdraw_ft_transfer_fallback(
                env::signer_account_id(),
                token_amount,
                dtoken_amount,
            );
        }
        return promise_success;
    }
}
