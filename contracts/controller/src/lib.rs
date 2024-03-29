use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
use near_sdk::{env, ext_contract, near_bindgen, require, AccountId, Balance, BorshStorageKey};

#[allow(unused_imports)]
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use percentage::Percentage;

use general::ratio::{BigBalance, Ratio};
use general::*;
use std::str::FromStr;

pub use crate::borrows_supplies::*;
pub use crate::config::*;
pub use crate::healthfactor::*;
pub use crate::liquidation::*;
pub use crate::oraclehook::*;
pub use crate::prices::*;
pub use crate::repay::*;
pub use crate::user_flow_protection::*;
pub use crate::user_profile::*;
pub use crate::views::*;

mod admin;
pub mod borrows_supplies;
#[allow(unused_imports)]
mod config;
mod healthfactor;
mod liquidation;
mod oraclehook;
mod prices;
pub mod repay;
mod upgrade;
pub mod user_flow_protection;
pub mod user_profile;
mod views;

pub fn get_default_liquidation_incentive() -> Ratio {
    Ratio::from_str("0.05").unwrap()
}

pub fn get_default_liquidation_health_factor_threshold() -> Ratio {
    Ratio::from_str("1.0").unwrap()
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Supplies,
    Prices,
    Config,
    Borrows,
    UserProfiles,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Utoken Id [Underlying asset name] -> Dtoken address
    /// Utoken Id [Underlying asset name] -> Ticker Id
    /// Utoken Id [Underlying asset name] -> LTV
    /// Utoken Id [Underlying asset name] -> LTH
    pub markets: UnorderedMap<AccountId, MarketProfile>,

    /// User Account ID -> Dtoken address -> Supplies balance
    /// User Account ID -> Dtoken address -> Borrow balance
    user_profiles: UnorderedMap<AccountId, UserProfile>,

    /// Dtoken ID -> Price
    pub prices: LookupMap<AccountId, Price>,

    /// Contract configuration object
    pub config: LazyOption<Config>,

    /// Contract admin account (controller itself by default)
    pub admin: AccountId,

    /// Contracts that are allowed to do uncollateralized borrow from market
    /// contract itself by default, can be set to different account
    eligible_to_borrow_uncollateralized: AccountId,

    /// Configuration for pausing/proceeding controller processes (false by default)
    pub is_action_paused: ActionStatus,

    /// Liquidation Incentive
    pub liquidation_incentive: Ratio,

    /// Liquidation Health Factor
    pub liquidation_health_factor_threshold: Ratio,

    ///User action protection
    mutex: ActionMutex,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Controller contract should be initialized before usage")
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PriceJsonList {
    /// Block number
    pub block_height: u64,

    /// Vector of asset prices
    pub price_list: Vec<Price>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketProfile {
    /// Dtoken address
    pub dtoken: AccountId,

    /// Ticker name
    pub ticker_id: String,

    /// Loan to value for the market
    pub ltv: Ratio,

    /// Liquidation threshold for the market
    pub lth: Ratio,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ActionStatus {
    supply: bool,
    withdraw: bool,
    borrow: bool,
    repay: bool,
    liquidate: bool,
    deposit: bool,
}

pub trait OraclePriceHandlerHook {
    fn oracle_on_data(&mut self, price_data: PriceJsonList);
}

#[ext_contract(market)]
trait MarketInterface {
    fn swap_supplies(
        &mut self,
        borrower: AccountId,
        liquidator: AccountId,
        liquidation_revenue_amount: WBalance,
    ) -> PromiseOrValue<U128>;

    fn increase_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance;
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new_with_config(owner_id: AccountId, oracle_account_id: AccountId) -> Self {
        Self::new(Config {
            owner_id,
            oracle_account_id,
        })
    }

    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            markets: UnorderedMap::new(StorageKeys::Markets),
            user_profiles: UnorderedMap::new(StorageKeys::UserProfiles),
            prices: LookupMap::new(StorageKeys::Prices),
            config: LazyOption::new(StorageKeys::Config, Some(&config)),
            admin: config.owner_id.clone(),
            eligible_to_borrow_uncollateralized: config.owner_id.clone(),
            is_action_paused: ActionStatus {
                withdraw: false,
                repay: false,
                supply: false,
                liquidate: false,
                borrow: false,
                deposit: false,
            },
            liquidation_incentive: get_default_liquidation_incentive(),
            liquidation_health_factor_threshold: get_default_liquidation_health_factor_threshold(),
            mutex: ActionMutex::default(),
        }
    }
}
