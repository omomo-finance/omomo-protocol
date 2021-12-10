use bigdecimal::{BigDecimal, ToPrimitive};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{near_bindgen, Balance};
use std::str::FromStr;

//near-sdk-3.1.0 AccountId => String used in FungibleToken
//near-sdk-4.0.0-pre.4 AccountId => struct used in project
//it line for version conflict fix
pub type AccountId = String;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Dtoken {
    initial_exchange_rate: u128,
    total_supply: u128,
    total_reserve: u128,
    total_borrows: u128,
    balance_of: UnorderedMap<AccountId, BigDecimal>,
    debt_of: UnorderedMap<AccountId, BigDecimal>,
    token: FungibleToken,
}

impl Default for Dtoken {
    fn default() -> Self {
        Self {
            initial_exchange_rate: 0,
            total_supply: 0,
            total_reserve: 0,
            total_borrows: 0,
            balance_of: UnorderedMap::new(b"s".to_vec()),
            debt_of: UnorderedMap::new(b"s".to_vec()),
            token: FungibleToken::new(b"a".to_vec()),
        }
    }
}

#[near_bindgen]
impl Dtoken {
    pub fn supply(amount: Balance) {
        //TODO: supply implementation
    }

    pub fn withdraw(amount: Balance) {
        //TODO: withdraw implementation
    }

    pub fn borrow(amount: Balance) {
        //TODO: borrow implementation
    }

    pub fn repay() {
        //TODO: repay implementation
    }

    pub fn add_reserve(amount: Balance) {
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

    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        self.token.internal_unwrap_balance_of(account_id)
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        self.token.internal_deposit(account_id, amount);
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        self.token.internal_withdraw(account_id, amount);
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        self.token
            .internal_transfer(sender_id, receiver_id, amount, memo);
    }

    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        self.token.internal_register_account(account_id);
    }

    fn mint(&mut self, account_id: &AccountId, amount: Balance) {
        self.internal_deposit(account_id, amount);
    }

    fn burn(&mut self, account_id: &AccountId, amount: Balance) {
        self.internal_withdraw(account_id, amount);
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
