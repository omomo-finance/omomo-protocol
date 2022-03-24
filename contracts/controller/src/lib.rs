use near_sdk::{AccountId, Balance, BorshStorageKey, env, near_bindgen, ext_contract, require};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap};
use std::collections::HashMap;
#[allow(unused_imports)]
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use percentage::Percentage;

#[allow(unused_imports)]
use general::*;

pub use crate::borrows_supplies::*;
pub use crate::config::*;
pub use crate::oraclehook::*;
pub use crate::prices::*;
pub use crate::repay::*;
pub use crate::liquidation::*;

#[allow(unused_imports)]
mod config;
mod oraclehook;
mod prices;
pub mod borrows_supplies;
pub mod repay;
mod healthfactor;
mod admin;
mod liquidation;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Supplies,
    Prices,
    Config,
    Borrows,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Market name [Underlying asset name] -> Dtoken contract address
    pub markets: LookupMap<AccountId, AccountId>,

    /// User Account ID -> Dtoken address -> Supplies balance
    pub account_supplies: LookupMap<AccountId, HashMap<AccountId, Balance>>,

    /// User Account ID -> Dtoken address -> Borrow balance
    pub account_borrows: LookupMap<AccountId, HashMap<AccountId, Balance>>,

    /// Asset ID -> Price value
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

#[ext_contract(dtoken)]
trait DtokenInterface {
    fn swap_supplies(
        &mut self,
        borrower: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128>;
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
            markets: LookupMap::new(StorageKeys::Markets),
            account_supplies: LookupMap::new(StorageKeys::Supplies),
            account_borrows: LookupMap::new(StorageKeys::Borrows),
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
