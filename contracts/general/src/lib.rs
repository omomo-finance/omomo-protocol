pub mod ratio;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId};
use near_sdk::{Balance, Gas};
use std::cmp::{max_by, min_by, Ordering};
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

pub const NO_DEPOSIT: Balance = 0;
pub const ONE_YOCTO: Balance = 1;
pub const TGAS: Gas = near_sdk::Gas::ONE_TERA;
pub const RATIO_DECIMALS: Ratio = Ratio(10u128.pow(4));

pub const ONE_TOKEN: u128 = 10u128.pow(24);

pub type WBalance = U128;

pub type USD = U128;
pub type WRatio = U128;
pub type Percent = u128;
pub type WPercent = U128;

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
pub enum Actions {
    Supply,
    Withdraw,
    Borrow,
    Repay,
    Liquidate {
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    },
}

impl fmt::Display for Actions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct Ratio(pub u128);

impl Ratio {
    pub fn new(ratio: u128) -> Ratio {
        Ratio(ratio)
    }
}

impl Mul for Ratio {
    type Output = Ratio;

    fn mul(self, rhs: Self) -> Self::Output {
        Ratio(self.0 * rhs.0)
    }
}

impl Div for Ratio {
    type Output = Ratio;

    fn div(self, rhs: Self) -> Self::Output {
        Ratio(self.0 / rhs.0)
    }
}

impl Add for Ratio {
    type Output = Ratio;

    fn add(self, rhs: Self) -> Self::Output {
        Ratio(self.0 + rhs.0)
    }
}

impl Sub for Ratio {
    type Output = Ratio;

    fn sub(self, rhs: Self) -> Self::Output {
        Ratio(self.0 - rhs.0)
    }
}

impl Eq for Ratio {}

impl PartialEq<Self> for Ratio {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialOrd<Self> for Ratio {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Option::from(self.0.cmp(&other.0))
    }
}

impl Ord for Ratio {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }

    fn max(self, other: Self) -> Self
    where
        Self: Sized,
    {
        max_by(self, other, Ord::cmp)
    }

    fn min(self, other: Self) -> Self
    where
        Self: Sized,
    {
        min_by(self, other, Ord::cmp)
    }

    fn clamp(self, min: Self, max: Self) -> Self
    where
        Self: Sized,
    {
        assert!(min <= max);
        if self < min {
            min
        } else if self > max {
            max
        } else {
            self
        }
    }
}

impl fmt::Display for Ratio {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}
