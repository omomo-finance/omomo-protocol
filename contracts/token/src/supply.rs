use crate::*;

use near_sdk::{env, is_promise_success, Gas, Balance, Promise, PromiseResult, PromiseOrValue};

pub const NO_DEPOSIT: Balance = 0;
pub const ONE_YOCTO: Balance = 1;
pub const TGAS: Gas = near_sdk::Gas::ONE_TERA;

#[near_bindgen]
impl Contract {

    // TODO: move to common.rs
    pub fn get_exchange_rate(&self, underlying_balance: Balance) -> Balance {
        let mut exchange_rate: u128;
        exchange_rate = self.initial_exchange_rate;
        if self.token.total_supply > 0 {
            exchange_rate = (underlying_balance + self.total_borrows - self.total_supplies)
                / self.token.total_supply;
        }
        return exchange_rate;
    }

    // TODO: Amount in token
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
        let mut balance_of: Balance = 0;

        if promise_success {
            // TODO: research how to use promise_result_as_success();
            balance_of = match env::promise_result(0) {
                PromiseResult::NotReady => 0,
                PromiseResult::Failed => 0,
                PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<u128>(&result)
                    .unwrap()
                    .into(),
            };
        } else {
            // TODO: implement fault behaviour
        }

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
            TGAS * 20u64, // TODO: move this value into constants lib
        ).then(ext_self::supply_ft_transfer_call_callback(
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
        if !self
            .token
            .accounts
            .contains_key(&account_id.clone())
        {
            self.token
                .internal_register_account(&account_id.clone());
        }
        self.token.internal_deposit(&account_id, amount);
    }

}
