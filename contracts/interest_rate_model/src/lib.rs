use bigdecimal::{BigDecimal, ToPrimitive};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{log, near_bindgen, Balance};
use std::str::FromStr;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct InterestRateModel {}

#[near_bindgen]
impl InterestRateModel {
    pub fn get_borrow_rate(
        underlying_balance: Balance,
        total_borrows: Balance,
        total_reserve: Balance,
    ) -> u128 {
        BigDecimal::from_str("1.1").unwrap().to_u128().unwrap()
    }

    pub fn get_supply_rate(
        underlying_balance: Balance,
        total_borrows: Balance,
        total_reserve: Balance,
        reserve_factor: i32,
    ) -> u128 {
        BigDecimal::from_str("0.9").unwrap().to_u128().unwrap()
    }
}

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder.predecessor_account_id(predecessor);
        builder
    }

    // TESTS HERE
}
