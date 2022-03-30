use near_sdk::{AccountId, Balance, BorshStorageKey, env, near_bindgen};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};

#[allow(unused_imports)]
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use percentage::Percentage;
use near_sdk::require;

use general::*;

pub use crate::borrows_supplies::*;
pub use crate::config::*;
pub use crate::oraclehook::*;
pub use crate::prices::*;
pub use crate::repay::*;
pub use crate::liquidation::*;
pub use crate::user_profile::*;
pub use crate::views::*;

#[allow(unused_imports)]
mod config;
mod oraclehook;
mod prices;
pub mod borrows_supplies;
pub mod repay;
mod healthfactor;
mod admin;
mod liquidation;
pub mod user_profile;
mod views;

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
    pub markets: UnorderedMap<AccountId, MarketProfile>,

    /// User Account ID -> Dtoken address -> Supplies balance
    /// User Account ID -> Dtoken address -> Borrow balance
    user_profiles: LookupMap<AccountId, UserProfile>,

    /// Dtoken ID -> Price
    pub prices: LookupMap<AccountId, Price>,

    /// Contract configuration object
    pub config: LazyOption<Config>,

    /// Contract admin account (controller itself by default)
    pub admin: AccountId,

    /// Configuration for pausing/proceeding controller processes (false by default)
    pub is_action_paused: ActionStatus,

    /// Health Factor
    pub health_factor_threshold: Ratio,

    /// Liquidation Incentive
    pub liquidation_incentive: Ratio,

    /// Reserve Factor
    pub reserve_factor: Percent,

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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketProfile {
    /// Dtoken address
    pub dtoken: AccountId,

    /// Ticker name
    pub ticker_id: String,
}


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct ActionStatus {
    supply: bool,
    withdraw: bool,
    borrow: bool,
    repay: bool,
    liquidate: bool,
}

pub trait OraclePriceHandlerHook {
    fn oracle_on_data(&mut self, price_data: PriceJsonList);
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new_with_config(owner_id: AccountId, oracle_account_id: AccountId) -> Self {
        Self::new(
            Config{
                owner_id: owner_id,
                oracle_account_id: oracle_account_id
            }
        )
    }

    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            markets: UnorderedMap::new(StorageKeys::Markets),
            user_profiles: LookupMap::new(StorageKeys::UserProfiles),
            prices: LookupMap::new(StorageKeys::Prices),
            config: LazyOption::new(StorageKeys::Config, Some(&config)),
            admin: config.owner_id,
            is_action_paused: ActionStatus {
                withdraw: false,
                repay: false,
                supply: false,
                liquidate: false,
                borrow: false,
            },
            health_factor_threshold: 0,
            liquidation_incentive: 0,
            reserve_factor: 0,
        }
    }
}
