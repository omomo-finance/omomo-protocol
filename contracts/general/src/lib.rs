pub mod percent;
pub mod ratio;

use crate::percent::WPercent;

use crate::ratio::{BigBalance, Ratio};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId};
use near_sdk::{Balance, Gas};
use std::fmt;

pub const NO_DEPOSIT: Balance = 0;
pub const ONE_YOCTO: Balance = 1;
pub const TGAS: Gas = near_sdk::Gas::ONE_TERA;
pub const ONE_TOKEN: u128 = 10u128.pow(24);

pub type WBalance = U128;

pub type USD = U128;

pub type WRatio = U128;

pub type Digits = u32;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Price {
    /// Ticker Id
    pub ticker_id: String,

    /// Ticker price value
    pub value: WBalance,

    /// Ticker volatility value
    pub volatility: WPercent, // 0..100%

    /// Ticker precision digits number
    pub fraction_digits: Digits,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub enum CampaignTypeArgs {
    Supply,
    Borrow,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct VestingArgs {
    /// Campaign vesting start time, seconds
    pub start_time: u64,
    /// Campaign vesting end time, seconds
    pub end_time: u64,
    /// Penalty amount which will be arrested in case of early withdraw
    pub penalty: Ratio,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub enum Actions {
    Supply,
    Withdraw,
    Borrow,
    Repay,
    Liquidate {
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
    },
    Reserve,
    CreateAndFundReward {
        /// Reward campaign type
        campaign_type_args: CampaignTypeArgs,
        /// Campaign start time seconds
        start_time: u64,
        /// Campaign end time seconds
        end_time: u64,
        /// Reward token address
        token_id: AccountId,
        /// Token ticker id
        ticker_id: String,
        /// Reward tokens total amount
        reward_amount: WBalance,
        /// Last time when rewardPerToken was recomputed/updated
        last_update_time: u64,
        /// Represent the token rewards amount which contract should pay for 1 token putted into liquidity
        rewards_per_token: BigBalance,
        /// Last market total by campaign type value
        last_market_total: WBalance,
        /// Vesting configuration
        vesting: VestingArgs,
    },
}

impl fmt::Display for Actions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
