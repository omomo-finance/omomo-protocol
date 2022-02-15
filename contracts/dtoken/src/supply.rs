use crate::*;

use near_sdk::{env, is_promise_success};

#[near_bindgen]
impl Contract {
    pub fn supply(&mut self, amount: Balance) -> Promise {
        return underline_token::ft_balance_of(
            env::current_account_id(),
            self.underlying_token.clone(),
            NO_DEPOSIT,
            TGAS * 20u64,
        )
            .then(ext_self::supply_balance_of_callback(
                amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                TGAS * 60u64,
            ));
    }

    #[allow(dead_code)]
    fn supply_balance_of_callback(&mut self, amount: Balance) -> Promise {
        let promise_success: bool = is_promise_success();

        assert_eq!(
            promise_success,
            true,
            "Supply has failed on receiving UToken balance_of: Account {} deposits {}",
            env::predecessor_account_id(),
            amount
        );

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<u128>(&result)
                .unwrap()
                .into(),
        };
        let token_amount: u128;
        let exchange_rate = self.get_exchange_rate(balance_of);
        token_amount = amount * exchange_rate;

        // Cross-contract call to market token
        underline_token::ft_transfer_call(
            env::current_account_id(),
            amount,
            Some(format!("Supply with token_amount {}", token_amount)),
            self.underlying_token.clone(),
            NO_DEPOSIT,
            TGAS * 20u64, //
            // TODO: move this value into constants lib
        )
            .then(ext_self::supply_ft_transfer_call_callback(
                token_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                TGAS * 60u64,
            ))
    }

    #[allow(dead_code)]
    fn supply_ft_transfer_call_callback(&mut self, amount: Balance) -> bool {
        let promise_success: bool = is_promise_success();
        if promise_success {
            self.mint(&env::signer_account_id(), amount)
        }
        promise_success
    }

    #[allow(dead_code)]
    fn controller_increase_supplies_callback(&mut self, amount: Balance) -> PromiseOrValue<U128> {
        let promise_success: bool = is_promise_success();
        if promise_success {
            let total_supplies = self.total_supplies;
            self.total_supplies = total_supplies + amount;
        }
        PromiseOrValue::Value(U128(self.total_supplies))
    }

    #[allow(dead_code)]
    fn mint(&mut self, account_id: &AccountId, amount: Balance) {
        if !self.token.accounts.contains_key(&account_id.clone()) {
            self.token.internal_register_account(&account_id.clone());
        }
        self.token.internal_deposit(&account_id, amount);
    }
}
