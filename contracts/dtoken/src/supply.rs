use crate::*;

impl Contract {
    pub fn supply_ft_transfer_fallback(&mut self, user_account: AccountId, token_amount: WBalance) {
        log!(
            "user {}, tokens {}",
            user_account,
            Balance::from(token_amount),
        );
        //TODO: implement fallback method
    }
}

#[near_bindgen]
impl Contract {

    pub fn supply(&mut self, amount: Balance) -> Promise {
        return underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS * 20u64,
        )
        .then(ext_self::supply_balance_of_callback(
            amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            TGAS * 60u64,
        ));
    }

    #[allow(dead_code)]
    fn supply_balance_of_callback(&mut self, amount: WBalance) -> Promise {
        let promise_success: bool = is_promise_success();

        assert_eq!(
            promise_success,
            true,
            "Supply has failed on receiving UToken balance_of: Account {} deposits {}",
            env::signer_account_id(),
            Balance::from(amount)
        );

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<u128>(&result)
                .unwrap()
                .into(),
        };
        let token_amount: Balance;
        let exchange_rate: Balance = self.get_exchange_rate(balance_of.into());
        token_amount = Balance::from(amount) * exchange_rate;

        // Cross-contract call to market token
        underlying_token::ft_transfer_call(
            self.get_contract_address(),
            amount,
            Some(format!("Supply with token_amount {}", token_amount)),
            format!(
                "Supply to {} from {} with token_amount {}",
                env::current_account_id(),
                self.get_contract_address(),
                token_amount
            ),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            TGAS * 40u64,
            // TODO: move this value into constants lib
        )
        .then(ext_self::supply_ft_transfer_call_callback(
            token_amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            TGAS * 60u64,
        ))
    }

    #[allow(dead_code)]
    fn supply_ft_transfer_call_callback(&mut self, token_amount: WBalance) -> bool {
        let promise_success: bool = is_promise_success();
        if promise_success {
            self.mint(&env::signer_account_id(), token_amount);
        } else {
            self.supply_ft_transfer_fallback(env::signer_account_id(), token_amount);
        }
        return promise_success;
    }

    #[allow(dead_code)]
    fn controller_increase_supplies_callback(&mut self, amount: WBalance) -> PromiseOrValue<U128> {
        let promise_success: bool = is_promise_success();
        if promise_success {
            let total_reserves = self.total_reserves;
            self.total_reserves = total_reserves + Balance::from(amount);
        }
        PromiseOrValue::Value(U128(self.total_reserves))
    }
}
