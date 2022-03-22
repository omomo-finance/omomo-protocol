use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::{U128};
use near_sdk::{AccountId, Balance, Gas};
use near_sdk::near_bindgen;

pub const NO_DEPOSIT: Balance = 0;
pub const ONE_YOCTO: Balance = 1;
pub const TGAS: Gas = near_sdk::Gas::ONE_TERA;
pub const RATIO_DECIMALS: u128 = 10u128.pow(4);

pub type WBalance = U128;

pub type Ratio = u128;
pub type WRatio = U128;

pub type Percent = u128;
pub type WPercent = U128;

pub type Digits = u32;


#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Price {
    /// Asset Id
    pub asset_id: AccountId,

    /// Asset price value
    pub value: WBalance,

    /// Asset volatility value
    pub volatility: WPercent, // 0..100%

    /// Asset precision digits number
    pub fraction_digits: Digits
}
