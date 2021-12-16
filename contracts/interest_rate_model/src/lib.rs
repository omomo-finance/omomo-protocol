use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::{near_bindgen, Balance};

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct InterestRateModel {}

#[near_bindgen]
impl InterestRateModel {

    pub fn get_borrow_rate(
        underlying_balance: Balance,
        total_borrows: Balance,
        total_reserve: Balance,
    ) -> U128 {
        55.into()
    }

    pub fn get_supply_rate(
        underlying_balance: Balance,
        total_borrows: Balance,
        total_reserve: Balance,
        reserve_factor: i32,
    ) -> U128 {
        66.into()
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
