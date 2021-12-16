use near_sdk::BorshStorageKey;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::{env, ext_contract, near_bindgen, log, AccountId, Promise, PromiseResult, Balance};
use near_sdk::json_types::U128;


#[ext_contract(ext_interest_rate_model)]
pub trait InterestRateModel {
    fn get_borrow_rate(&self, underlying_balance: Balance, total_borrows: Balance, total_reserve: Balance) -> U128;
}

#[ext_contract()]
pub trait ExtSelf {
    fn callback_promise_result(&self) -> U128;
}

#[derive(BorshStorageKey, BorshSerialize)]
pub enum StorageKeys 
{
    SupportedMarkets,
    InterestRateModels,
    BorrowCaps,
    Prices,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Controller 
{
    supported_markets: LookupSet<AccountId>,
    interest_rate_models: LookupMap<AccountId, AccountId>,
    borrow_caps: LookupMap<AccountId, u128>,
    prices: LookupMap<AccountId, U128>
}

impl Default for Controller
{
    fn default() -> Self {
        Self {
            supported_markets: LookupSet::new(StorageKeys::SupportedMarkets),
            interest_rate_models: LookupMap::new(StorageKeys::InterestRateModels),
            borrow_caps: LookupMap::new(StorageKeys::BorrowCaps),
            prices: LookupMap::new(StorageKeys::Prices),
        }
    }
}

#[near_bindgen]
impl Controller {
    pub fn callback_promise_result(&self) -> U128 {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ASSERT|callback_promise_result:promise_results_count"
        );

        match env::promise_result(0) {
            PromiseResult::NotReady =>  unreachable!(),
            PromiseResult::Failed => env::panic_str("PANIC|callback_promise_result:PromiseResult::Failed"),
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result).unwrap()
        }
    }

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

    pub fn get_interest_rate(&mut self, dtoken_address : AccountId, underlying_balance : Balance, total_borrows : Balance, total_reserve : Balance ) -> Promise
    {
        assert!(self.interest_rate_models.contains_key(&dtoken_address));

        let interest_rate_model_address = self.interest_rate_models.get(&dtoken_address).unwrap();

        ext_interest_rate_model::get_borrow_rate(
            underlying_balance,
            total_borrows,
            total_reserve,
            interest_rate_model_address,
            0,                         // attached yocto NEAR
            5_000_000_000_000.into(),   // attached gas
        )
        .then(ext_self::callback_promise_result(
            env::current_account_id(), // this contract's account id
            0,                        // yocto NEAR to attach to the callback
            6_000_000_000_000.into()   // gas to attach to the callback
        ))
    }

    pub fn set_borrow_cap(&mut self, dtoken_address : AccountId, decimal : u128 )
    {
        self.borrow_caps.insert(&dtoken_address, &decimal);
    }

    pub fn has_collaterall(&mut self, user_address: AccountId ) -> bool
    {
        // calling dToken contract
        true
    }

    pub fn set_price(&mut self, dtoken_address: AccountId, price: U128) {
        self.prices.insert(&dtoken_address, &price);
    }

    pub fn get_price(&self, dtoken_address: AccountId) -> U128 {
        assert!(self.interest_rate_models.contains_key(&dtoken_address), "price for token not found");

        return self.prices.get(&dtoken_address).unwrap().into();
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