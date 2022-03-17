mod borrow;
mod common;
mod config;
mod ft;
mod repay;
mod supply;
mod withdraw;
mod interest_model;
mod interest_rate_model;
mod user_flow_protection;

pub use crate::borrow::*;
pub use crate::common::*;
pub use crate::config::*;
pub use crate::ft::*;
pub use crate::repay::*;
pub use crate::supply::*;
pub use crate::withdraw::*;
pub use crate::interest_model::*;
pub use crate::interest_rate_model::*;
pub use crate::user_flow_protection::*;

#[allow(unused_imports)]
use general::*;

use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, ext_contract, is_promise_success, log, near_bindgen, AccountId, Balance,
               BorshStorageKey, Gas, PromiseOrValue, PromiseResult, BlockHeight};
use near_sdk::require;

pub type TokenAmount = u128;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    Borrows,
    Config,
    Actions,
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
    actions: LookupMap<AccountId, BlockHeight>,

    model: InterestRateModel,

    ///User action protection
    mutex: ActionMutex,
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
    fn increase_supplies(&mut self, account: AccountId, token_address: AccountId, token_amount: WBalance);
    fn decrease_supplies(&mut self, account_id: AccountId, amount: WBalance);
    fn repay_borrows(&mut self, account_id: AccountId, token_address: AccountId, token_amount: WBalance);
    fn withdraw_supplies(&mut self, account_id: AccountId, token_address: AccountId, token_amount: WBalance) -> Promise;
    fn make_borrow(&mut self, account_id: AccountId, token_address: AccountId, token_amount: WBalance);
    fn decrease_borrows(&mut self, account: AccountId, token_address: AccountId, token_amount: WBalance);
}

#[ext_contract(ext_self)]
trait InternalTokenInterface {
    fn supply_balance_of_callback(&mut self, token_amount: WBalance);
    fn supply_ft_transfer_call_callback(&mut self, amount: WBalance);
    fn controller_increase_supplies_callback(&mut self, amount: WBalance, dtoken_amount: WBalance);

    fn borrow_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn make_borrow_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn repay_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn borrow_ft_transfer_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn controller_repay_borrows_callback(&mut self, amount: WBalance, borrow_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn controller_decrease_borrows_fail(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;

    fn withdraw_balance_of_callback(&mut self, dtoken_amount: Balance) -> PromiseOrValue<WBalance>;
    fn withdraw_supplies_callback(&mut self, user_account: AccountId, token_amount: WBalance, dtoken_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn withdraw_ft_transfer_call_callback(&mut self, token_amount: WBalance, dtoken_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn withdraw_increase_supplies_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;

}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new_with_config(owner_id: AccountId, underlying_token_id: AccountId, controller_account_id: AccountId, initial_exchange_rate: U128) -> Self {
        Self::new(
            Config {
                owner_id: owner_id,
                underlying_token_id: underlying_token_id,
                controller_account_id: controller_account_id,
                initial_exchange_rate: initial_exchange_rate,
            }
        )
    }

    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            initial_exchange_rate: Balance::from(config.initial_exchange_rate.clone()),
            total_reserves: 0,
            total_borrows: 0,
            borrows: UnorderedMap::new(StorageKeys::Borrows),
            underlying_token: config.underlying_token_id.clone(),
            token: FungibleToken::new(b"t".to_vec()),
            config: LazyOption::new(StorageKeys::Config, Some(&config)),
            actions: LookupMap::new(StorageKeys::Actions),
            model: InterestRateModel::default(),
            mutex: ActionMutex::default(),
        }
    }
}
