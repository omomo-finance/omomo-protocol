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
    pub fn add_market( _underlying_address : AccountId )
    {

    }

    pub fn add_market_( _dtoken_address : AccountId )
    {

    }
    
    pub fn supply_allowed( _dtoken_address : AccountId, _user_address : AccountId, _amount : i8 ) -> bool
    {
        false
    }

    pub fn borrow_allowed( _dtoken_address : AccountId, _user_address : AccountId, _amount : i8 ) -> bool
    {
        false
    }

    pub fn set_interest_rate_model( _dtoken_address : AccountId, _interest_rate_model_address : AccountId )
    {

    }

    pub fn get_interest_rate( _dtoken_address : AccountId ) -> i8
    {
        0
    }

    pub fn set_borrow_cap( _dtoken_address : AccountId, _decimal : i8 )
    {

    }

    pub fn has_collaterall( _user_address : AccountId ) -> bool
    {
        false
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