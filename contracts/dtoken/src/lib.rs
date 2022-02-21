mod borrow;
mod common;
mod config;
mod ft;
mod repay;
mod supply;
mod withdraw;
mod interest_model;

pub use crate::borrow::*;
pub use crate::common::*;
pub use crate::config::*;
pub use crate::ft::*;
pub use crate::repay::*;
pub use crate::supply::*;
pub use crate::withdraw::*;
pub use crate::interest_model::*;


#[allow(unused_imports)]
use general::*;

use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, ext_contract, is_promise_success, log, near_bindgen, AccountId, Balance, BorshStorageKey, Gas, Promise, PromiseOrValue, PromiseResult, BlockHeight};

pub type TokenAmount = u128;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    Borrows,
    Config,
    Actions
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    ///  Exchange rate in case of zero supplies
    initial_exchange_rate: u128,

    /// Total sum of supplied tokens
    total_reserves: TokenAmount,

    /// Total sum of borrowed tokens
    total_borrows: TokenAmount,

    /// Account Id -> Token's amount
    borrows: UnorderedMap<AccountId, TokenAmount>,

    /// Address of underlying token
    underlying_token: AccountId,

    /// Pointer for contract token
    token: FungibleToken,

    /// Contract configuration object
    config: LazyOption<Config>,

    /// BlockHeight of last action user produced
    actions: LookupMap<AccountId, BlockHeight>
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Token contract should be initialized before usage")
    }
}

#[ext_contract(underlying_token)]
trait UnderlineTokenInterface {
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
    fn ft_transfer(&mut self, receiver_id: AccountId, amount: WBalance, memo: Option<String>);
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );
    fn ft_resolve_transfer(&self, account_id: AccountId) -> U128;
}

#[ext_contract(controller)]
trait ControllerInterface {
    fn increase_supplies(&mut self, account: AccountId, token_address: AccountId, tokens_amount: WBalance);
    fn decrease_supplies(&mut self, account_id: AccountId, amount: WBalance);
    fn repay_borrows(&mut self, account_id: AccountId, token_address: AccountId, tokens_amount: WBalance);
    fn withdraw_supplies(&mut self, account_id: AccountId, token_address: AccountId, tokens_amount: WBalance) -> Promise;
    fn make_borrow(&mut self, account_id: AccountId, token_address: AccountId, tokens_amount: WBalance); 
    fn decrease_borrows(&mut self, account: AccountId, token_address: AccountId, tokens_amount: WBalance); 

}

#[ext_contract(ext_self)]
trait InternalTokenInterface {
    fn supply_balance_of_callback(&mut self, token_amount: WBalance);
    fn supply_ft_transfer_call_callback(&mut self, amount: WBalance);
    fn controller_increase_supplies_callback(&mut self, amount: WBalance, dtoken_amount: WBalance) -> PromiseOrValue<U128>;

    fn make_borrow_callback(&mut self, token_amount: WBalance);
    fn borrow_ft_transfer_callback(&mut self, token_amount: WBalance);
    fn controller_repay_borrows_callback(&mut self, amount: WBalance);
    fn controller_decrease_borrows_fail(&mut self);

    fn withdraw_balance_of_callback(&mut self, dtoken_amount: Balance);
    fn withdraw_supplies_callback(&mut self, user_account: AccountId, token_amount: WBalance, dtoken_amount: WBalance);
    fn withdraw_ft_transfer_call_callback(&mut self, token_amount: WBalance, dtoken_amount: WBalance);
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        Self {
            initial_exchange_rate: u128::from(config.initial_exchange_rate.clone()),
            total_reserves: 0,
            total_borrows: 0,
            borrows: UnorderedMap::new(StorageKeys::Borrows),
            underlying_token: config.underlying_token_id.clone(),
            token: FungibleToken::new(b"t".to_vec()),
            config: LazyOption::new(StorageKeys::Config, Some(&config)),
            actions: LookupMap::new(StorageKeys::Actions),
        }
    }
}
