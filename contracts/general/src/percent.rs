use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::json_types::U128;

pub type WPercent = U128;


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct Percent(pub u128);

impl Percent {
    pub fn new(percent: u128) -> Percent {
        Percent(percent)
    }
}