use std::collections::{HashSet, HashMap};
use near_sdk::BorshStorageKey;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::near_bindgen;
use near_sdk::AccountId;


#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys 
{
    supported_markets,
    interest_rate_models,
    borrow_caps
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Controller 
{
    supported_markets: LookupSet<AccountId>,
    interest_rate_models: LookupMap<AccountId, AccountId>,
    borrow_caps: LookupMap<AccountId, u128>
}

impl Default for Controller
{
    fn default() -> Self 
    {
        Self
        {
            supported_markets: LookupSet::new(StorageKeys::supported_markets),
            interest_rate_models: LookupMap::new(StorageKeys::interest_rate_models),
            borrow_caps: LookupMap::new(StorageKeys::borrow_caps),
        }
    }
}

#[near_bindgen]
impl Controller {

    pub fn add_market(&mut self,  dtoken_address : AccountId )
    {
        self.supported_markets.insert(&dtoken_address);
    }
    
    pub fn supply_allowed(&mut self, dtoken_address : AccountId, user_address : AccountId, amount : u128 ) -> bool
    {
        true
    }

    pub fn borrow_allowed(&mut self, dtoken_address : AccountId, user_address : AccountId, amount : u128 ) -> bool
    {
        true
    }

    pub fn set_interest_rate_model(&mut self, dtoken_address : AccountId, interest_rate_model_address : AccountId )
    {
        self.interest_rate_models.insert(&dtoken_address, &interest_rate_model_address);
    }

    pub fn get_interest_rate(&mut self, dtoken_address : AccountId ) -> u128
    {
        assert!(!self.interest_rate_models.contains_key(&dtoken_address));
        //self.interest_rate_models.get(&dtoken_address).unwrap()
        1
    }

    pub fn set_borrow_cap(&mut self, dtoken_address : AccountId, decimal : u128 )
    {
        self.borrow_caps.insert(&dtoken_address, &decimal);
    }

    pub fn has_collaterall(&mut self, user_address : AccountId ) -> bool
    {
        // calling dToken contract
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