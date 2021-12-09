use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::near_bindgen;
use near_sdk::AccountId;

#[near_bindgen]
#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct Controller {
    // SETUP CONTRACT STATE
}

#[near_bindgen]
impl Controller {
    pub fn add_market( underlying_address : AccountId )
    {

    }

    pub fn add_market_( dtoken_address : AccountId )
    {

    }
    
    pub fn supply_allowed( dtoken_address : AccountId, user_address : AccountId, amount : u128 ) -> bool
    {
        true
    }

    pub fn borrow_allowed( dtoken_address : AccountId, user_address : AccountId, amount : u128 ) -> bool
    {
        true
    }

    pub fn set_interest_rate_model( dtoken_address : AccountId, interest_rate_model_address : AccountId )
    {

    }

    pub fn get_interest_rate( dtoken_address : AccountId ) -> u128
    {
        1
    }

    pub fn set_borrow_cap( dtoken_address : AccountId, decimal : u128 )
    {

    }

    pub fn has_collaterall( user_address : AccountId ) -> bool
    {
        true
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