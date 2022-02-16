mod config;
mod oraclehook;
mod prices;
mod supplies;
mod borrows;

pub use crate::config::*;
pub use crate::oraclehook::*;
pub use crate::prices::*;
pub use crate::supplies::*;
pub use crate::borrows::*;

#[allow(unused_imports)]
use general::*;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LazyOption};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, AccountId, BorshStorageKey, Balance};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Supplies,
    SuppliesToken,
    Prices,
    Config,
    Borrows
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Market name -> Dtoken contract address
    pub markets: LookupMap<AccountId, AccountId>,

    /// User Account ID -> Dtoken address -> Supplies balance
    pub account_supplies: LookupMap<AccountId, LookupMap<AccountId, Balance>>,

    /// User Account ID -> Dtoken address -> Borrow balance
    pub account_borrows: LookupMap<AccountId, LookupMap<AccountId, Balance>>,

    /// Asset ID -> Price value
    pub prices: LookupMap<AccountId, Price>,

    /// Contract configuration object
    pub config: LazyOption<Config>,
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
            account_borrows: LookupMap::new(StorageKeys::Borrows),
            prices: LookupMap::new(StorageKeys::Prices),
            config: LazyOption::new(StorageKeys::Config, Some(&config)),
        }
    }
}