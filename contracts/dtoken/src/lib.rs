use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::require;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, is_promise_success, log, near_bindgen, AccountId, Balance, BlockHeight,
    BorshStorageKey, Gas, PromiseOrValue, PromiseResult,
};
use std::collections::HashMap;

#[allow(unused_imports)]
pub use general::*;

pub use crate::borrow::*;
pub use crate::common::*;
pub use crate::config::*;
pub use crate::ft::*;
pub use crate::interest_model::*;
pub use crate::interest_rate_model::*;
pub use crate::repay::*;
pub use crate::reward::*;
pub use crate::supply::*;
pub use crate::user_profile::*;
pub use crate::withdraw::*;

mod admin;
mod borrow;
mod common;
mod config;
mod ft;
mod interest_model;
mod interest_rate_model;
mod liquidation;
mod repay;
mod reward;
mod supply;
mod user_profile;
mod views;
mod withdraw;

#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKeys {
    Config,
    UserProfiles,
    UserRewards,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    ///  Exchange rate in case of zero supplies
    initial_exchange_rate: u128,

    /// Total sum of supplied tokens
    total_reserves: Balance,

    /// Account Id -> Token's amount
    user_profiles: UnorderedMap<AccountId, UserProfile>,

    /// Address of underlying token
    underlying_token: AccountId,

    /// Pointer for contract token
    token: FungibleToken,

    /// Contract configuration object
    config: LazyOption<Config>,

    model: InterestRateModel,

    /// Contract admin account (dtoken itself by default)
    pub admin: AccountId,

    /// Users rewards
    rewards: UnorderedMap<AccountId, HashMap<String, Reward>>,
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
    fn increase_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    );
    fn decrease_supplies(&mut self, account_id: AccountId, amount: WBalance);
    fn repay_borrows(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    );
    fn withdraw_supplies(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> Promise;
    fn make_borrow(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    );
    fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    );
    fn on_debt_repaying_callback(
        &mut self,
        borrower: AccountId,
        _borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    );
    fn liquidation(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        _liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    );
    fn mutex_lock(&mut self, action: Actions);
    fn mutex_unlock(&mut self);
    fn set_account_consistency(
        &mut self,
        account: AccountId,
        consistency: bool,
        block: BlockHeight,
    );
}

#[ext_contract(ext_self)]
trait InternalTokenInterface {
    fn supply_balance_of_callback(&mut self, token_amount: WBalance);
    fn supply_ft_transfer_call_callback(&mut self, amount: WBalance);
    fn controller_increase_supplies_callback(&mut self, amount: WBalance, dtoken_amount: WBalance);

    fn reward_ft_transfer_callback(&mut self, reward_index: Reward, account_id: AccountId);

    fn borrow_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn make_borrow_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn repay_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn borrow_ft_transfer_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>;
    fn controller_repay_borrows_callback(
        &mut self,
        amount: WBalance,
        borrow_amount: WBalance,
    ) -> PromiseOrValue<WBalance>;
    fn controller_decrease_borrows_fail_callback(
        &mut self,
        token_amount: WBalance,
    ) -> PromiseOrValue<WBalance>;

    fn withdraw_balance_of_callback(&mut self, dtoken_amount: Balance) -> PromiseOrValue<WBalance>;
    fn withdraw_supplies_callback(
        &mut self,
        user_account: AccountId,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<WBalance>;
    fn withdraw_ft_transfer_call_callback(
        &mut self,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<WBalance>;
    fn withdraw_increase_supplies_callback(
        &mut self,
        token_amount: WBalance,
    ) -> PromiseOrValue<WBalance>;

    fn liquidate_callback(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    );
    fn mutex_lock_callback(
        &mut self,
        action: Actions,
        amount: WBalance,
    ) -> PromiseOrValue<WBalance>;
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new_with_config(
        owner_id: AccountId,
        underlying_token_id: AccountId,
        controller_account_id: AccountId,
        initial_exchange_rate: U128,
        interest_rate_model: InterestRateModel,
    ) -> Self {
        Self::new(Config {
            owner_id,
            underlying_token_id,
            controller_account_id,
            initial_exchange_rate,
            interest_rate_model,
        })
    }

    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            initial_exchange_rate: Balance::from(config.initial_exchange_rate),
            total_reserves: 0,
            user_profiles: UnorderedMap::new(StorageKeys::UserProfiles),
            underlying_token: config.underlying_token_id.clone(),
            token: FungibleToken::new(b"t".to_vec()),
            config: LazyOption::new(StorageKeys::Config, Some(&config)),
            model: config.interest_rate_model,
            admin: config.owner_id,
            rewards: UnorderedMap::new(StorageKeys::UserRewards),
        }
    }
}
