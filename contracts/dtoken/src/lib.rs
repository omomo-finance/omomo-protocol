use bigdecimal::{BigDecimal, ToPrimitive};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::{env, ext_contract, near_bindgen, serializer, AccountId, Balance, Gas};
use std::str::FromStr;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Dtoken {
    initial_exchange_rate: u128,
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
            total_reserve: 0,
            total_borrows: 0,
            balance_of: UnorderedMap::new(b"s".to_vec()),
            debt_of: UnorderedMap::new(b"s".to_vec()),
            token: FungibleToken::new(b"a".to_vec()),
        }
    }
}

#[ext_contract(weth_token)]
trait WethTokenInterface {
    fn internal_deposit(&mut self, account_id: AccountId, amount: Balance);
    fn internal_withdraw(&mut self, account_id: AccountId, amount: Balance);
}

const WETH_TOKEN_ACCOUNT_ID: &str = "dev-1639393437256-47963104973950";
const NO_DEPOSIT: Balance = 0;
const BASE_GAS: Gas = Gas(5_000_000_000_000);

#[near_bindgen]
impl Dtoken {
    pub fn supply(&mut self, amount: Balance) {
        let dtoken_account_id = env::current_account_id();
        let signer_account_id = env::predecessor_account_id();
        let weth_token_account_id: AccountId =
            AccountId::new_unchecked(WETH_TOKEN_ACCOUNT_ID.to_string());
        weth_token::internal_withdraw(
            signer_account_id.clone(),
            amount,
            weth_token_account_id.clone(),
            NO_DEPOSIT,
            BASE_GAS,
        );
        weth_token::internal_deposit(
            dtoken_account_id.clone(),
            amount,
            weth_token_account_id.clone(),
            NO_DEPOSIT,
            BASE_GAS,
        );
        self.mint(&signer_account_id, amount);
    }

    pub fn withdraw(&mut self, amount: Balance) {
        let dtoken_account_id = env::current_account_id();
        let signer_account_id = env::predecessor_account_id();
        let weth_token_account_id: AccountId =
            AccountId::new_unchecked(WETH_TOKEN_ACCOUNT_ID.to_string());
        weth_token::internal_withdraw(
            dtoken_account_id.clone(),
            amount.clone(),
            weth_token_account_id.clone(),
            NO_DEPOSIT,
            BASE_GAS,
        );
        let ex_rate = self.get_exchange_rate();
        weth_token::internal_deposit(
            signer_account_id.clone(),
            amount * ex_rate,
            weth_token_account_id.clone(),
            NO_DEPOSIT,
            BASE_GAS,
        );
        self.burn(&signer_account_id, amount);
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

    pub fn get_exchange_rate(&self) -> u128 {
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
        self.token
            .internal_unwrap_balance_of(&account_id.to_string())
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        self.token.internal_deposit(&account_id.to_string(), amount);
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        self.token
            .internal_withdraw(&account_id.to_string(), amount);
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        self.token.internal_transfer(
            &sender_id.to_string(),
            &receiver_id.to_string(),
            amount,
            memo,
        );
    }

    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        self.token
            .internal_register_account(&account_id.to_string());
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
