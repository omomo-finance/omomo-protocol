use crate::*;

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
            self.terra_gas(160),
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

        let exchange_rate: f32 = self.get_exchange_rate(WBalance::from(balance_of)) as f32 / RATIO_DECIMALS as f32;
        let supply_rate: Balance = self.get_supply_rate(U128(balance_of), U128(self.total_borrows), U128(self.total_reserves), U128(self.model.reserve_factor));
        let token_amount: Balance = (Balance::from(dtoken_amount) as f32 / exchange_rate) as u128;
        let token_return_amount: Balance = token_amount * supply_rate / RATIO_DECIMALS;

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
            token_return_amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(120),
        ));
    }

    pub fn withdraw_supplies_callback(
        &mut self,
        user_account: AccountId,
        token_amount: WBalance,
        dtoken_amount: WBalance,
        token_return_amount: WBalance,
    ) ->Promise {
        let promise_success: bool = is_promise_success();

        assert_eq!(promise_success, true, "Withdraw supplies has been failed");

        // Cross-contract call to market token
        underlying_token::ft_transfer(
            user_account,
            token_return_amount,
            Some(format!("Withdraw with token_amount {}", Balance::from(token_return_amount))),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(40),
        )
        .then(ext_self::withdraw_ft_transfer_call_callback(
            token_amount.into(),
            dtoken_amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(50),
        ))
    }

    pub fn withdraw_ft_transfer_call_callback(
        &mut self,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    )  {
        if is_promise_success() {
            self.burn(&env::signer_account_id(), dtoken_amount);
        } else {

            log!(
                "withdraw_ft_transfer_fallback - user {}, tokens {}, dtokens {}",
                env::signer_account_id(),
                Balance::from(token_amount),
                Balance::from(dtoken_amount)
            );
    
            controller::increase_supplies(
                env::signer_account_id(),
                self.get_contract_address(),
                token_amount,
                self.get_controller_address(),
                NO_DEPOSIT,
                self.terra_gas(10),
            )
            .then(ext_self::withdraw_increase_supplies_callback(
                token_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ));
        }
    }

    pub fn withdraw_increase_supplies_callback(
        &mut self,
        token_amount: WBalance,
    ) -> bool {
        if is_promise_success(){
            log!("Token amount: {} was succesfully increased after transfer fail for account {}", Balance::from(token_amount), env::signer_account_id());
        } 
        else {
            log!("Failed to increase supplies for {}", env::signer_account_id());

            // Account should be marked
        }

        return is_promise_success();
    }
}


