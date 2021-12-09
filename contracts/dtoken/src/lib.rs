use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, Balance, near_bindgen};
use near_sdk::collections::UnorderedMap;
use bigdecimal::{BigDecimal, ToPrimitive};
use std::str::FromStr;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Dtoken {
    initial_exchange_rate: u128,
    total_supply: u128,
    total_reserve: u128,
    total_borrows: u128,
    balance_of: UnorderedMap<AccountId, BigDecimal>,
    debt_of: UnorderedMap<AccountId, BigDecimal>
}

impl Default for Dtoken {
    fn default() -> Self {
        Self {
            initial_exchange_rate: 0,
            total_supply: 0,
            total_reserve: 0,
            total_borrows: 0,
            balance_of: UnorderedMap::new(b"s".to_vec()),
            debt_of: UnorderedMap::new(b"s".to_vec())
        }
    }
}

#[near_bindgen]
impl Dtoken {

    pub fn supply( amount: Balance ) {
        //TODO: supply implementation
    }

    pub fn withdraw( amount: Balance ) {
        //TODO: withdraw implementation
    }

    pub fn borrow( amount: Balance ) {
        //TODO: borrow implementation
    }

    pub fn repay() {
        //TODO: repay implementation
    }

    pub fn add_reserve( amount: Balance ) {
        //TODO: add_reserve implementation
    }

    pub fn get_exchange_rate() -> u128 {
        //TODO: get exchange rate by formula
        BigDecimal::from_str("1.0").unwrap().to_u128().unwrap()
    }

    pub fn get_total_reserve() -> u128 {
        Dtoken::default().total_reserve
    }

    pub fn get_total_borrows() -> u128 {
        Dtoken::default().total_borrows
    }

    pub fn get_underlying_balance() -> u128 {
        BigDecimal::from_str("1.2").unwrap().to_u128().unwrap()
    }
}

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

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
