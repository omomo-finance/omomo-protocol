mod borrow;
mod ft;
mod repay;
mod supply;
mod withdraw;
mod common;
mod config;
mod constants;

pub use crate::borrow::*;
pub use crate::ft::*;
pub use crate::repay::*;
pub use crate::supply::*;
pub use crate::withdraw::*;
pub use crate::common::*;
pub use crate::config::*;
pub use crate::constants::*;

use near_sdk::json_types::U128;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::collections::{UnorderedMap, LazyOption};
use near_sdk::{
    env, ext_contract, near_bindgen, AccountId, BorshStorageKey, Balance,
    Promise, PromiseResult, PromiseOrValue, log, Gas
};
use near_contract_standards::fungible_token::FungibleToken;

pub type TokenAmount = u128;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    Borrows,
    Config
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    ///  Exchange rate in case of zero supplies
    initial_exchange_rate: u128,

    /// Total sum of supplied tokens
    total_supplies: TokenAmount,

    /// Total sum of borrowed tokens
    total_borrows: TokenAmount,

    /// Account Id -> Token's amount
    borrows: UnorderedMap<AccountId, TokenAmount>,

    /// Address of underlying token
    underlying_token: AccountId,

    /// Pointer for contract token
    token: FungibleToken,

    /// Contract configuration object
    config: LazyOption<Config>
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Token contract should be initialized before usage")
    }
}

#[ext_contract(underline_token)]
trait UnderlineTokenInterface {
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: Balance, memo: Option<String>);
    fn ft_transfer_call(&mut self, receiver_id: AccountId, amount: Balance, memo: Option<String>);
    fn ft_resolve_transfer(&self, account_id: AccountId) -> U128;
}

#[ext_contract(controller)]
trait ControllerInterface {
    fn increase_supplies(&mut self, account_id: AccountId, amount: Balance) -> Promise;
    fn decrease_supplies(&mut self, account_id: AccountId, amount: Balance) -> Promise;
}

#[ext_contract(ext_self)]
trait InternalTokenInterface {
    fn supply_balance_of_callback(&mut self, amount: Balance);
    fn controller_increase_supplies_callback(&mut self, amount: Balance) -> PromiseOrValue<U128>;
    fn supply_ft_transfer_call_callback(&mut self, amount: Balance);
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        Self {
            initial_exchange_rate: config.initial_exchange_rate.clone(),
            total_supplies: 0,
            total_borrows: 0,
            borrows: UnorderedMap::new(StorageKey::Borrows),
            underlying_token: config.underlying_token_id.clone(),
            token: FungibleToken::new(b"t".to_vec()),
            config: LazyOption::new(StorageKey::Config, Some(&config))
        }
    }
}