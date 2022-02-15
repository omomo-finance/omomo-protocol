mod constants;
mod oraclehook;
mod config;
mod prices;
mod supplies;

pub use crate::constants::*;
pub use crate::oraclehook::*;
pub use crate::config::*;
pub use crate::prices::*;
pub use crate::supplies::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{UnorderedMap, LookupMap, LazyOption};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, Balance, Gas};
use near_sdk::json_types::U128;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Supplies,
    SuppliesToken,
    Prices,
    Config
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Market name -> Dtoken contract address
    pub markets: LookupMap<AccountId, AccountId>,

    /// User Account ID -> Dtoken address -> Supplies balance
    pub account_supplies: LookupMap<AccountId, LookupMap<AccountId, Balance>>,

    /// Asset ID -> Price value
    pub prices: UnorderedMap<AccountId, u128>,

    /// Contract configuration object
    pub config: LazyOption<Config>
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Controller contract should be initialized before usage")
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    /// Asset Id
    pub asset_id: AccountId,

    /// Asset price value
    pub value: u128
}


#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PriceJsonList {
    /// Timestamp in milliseconds
    pub timestamp: u64,

    // pub blockheight: ...

    /// Vector of asset prices
    pub price_list: Vec<Price>
}

pub trait OraclePriceHandlerHook {

    fn oracle_on_data(&mut self, price_data: PriceJsonList);

}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new(config: Config) -> Self {
        Self {
            markets: LookupMap::new(StorageKeys::Markets),
            account_supplies: LookupMap::new(StorageKeys::Supplies),
            prices: UnorderedMap::new(StorageKeys::Prices),
            config: LazyOption::new(StorageKeys::Config, Some(&config))
        }
    }
}