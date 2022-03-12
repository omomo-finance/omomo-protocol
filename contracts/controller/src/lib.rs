use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{env, near_bindgen, AccountId, Balance, BorshStorageKey, ext_contract};

use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap};
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

pub mod borrows_supplies;
#[allow(unused_imports)]
mod config;
mod healthfactor;
pub mod liquidation;
mod oraclehook;
mod prices;
pub mod repay;
mod test_utils;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Supplies,
    SuppliesToken,
    BorrowsToken,
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
    pub account_supplies: LookupMap<AccountId, UnorderedMap<AccountId, Balance>>,

    /// User Account ID -> Dtoken address -> Borrow balance
    pub account_borrows: LookupMap<AccountId, UnorderedMap<AccountId, Balance>>,

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

#[ext_contract(dtoken)]
trait DtokenInterface {
    fn swap_supplies(
        &mut self,
        borrower: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    );
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
