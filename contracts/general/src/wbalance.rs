use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use std::cmp::{max_by, min_by, Ordering};
use std::ops::{Add, Div, Mul, Sub};
use near_sdk::Balance;
use near_sdk::json_types::U128;


#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug, Copy)]
#[serde(crate = "near_sdk::serde")]
pub struct WBalance(pub U128);

impl WBalance {
    pub fn new(balance: U128) -> WBalance {
        WBalance(balance)
    }
}

impl From<Balance> for WBalance {
    fn from(balance: Balance) -> Self {
        WBalance(U128(balance))
    }
}

impl From<WBalance> for Balance {
    fn from(wbalance: WBalance) -> Self {
        wbalance.0.0
    }
}


impl Mul for WBalance {
    type Output = WBalance;

    fn mul(self, rhs: Self) -> Self::Output {
        WBalance(U128(self.0.0 * rhs.0.0))
    }
}

impl Div for WBalance {
    type Output = WBalance;

    fn div(self, rhs: Self) -> Self::Output {
        WBalance(U128(self.0.0 / rhs.0.0))
    }
}

impl Add for WBalance {
    type Output = WBalance;

    fn add(self, rhs: Self) -> Self::Output {
        WBalance(U128(self.0.0 + rhs.0.0))
    }
}

impl Sub for WBalance {
    type Output = WBalance;

    fn sub(self, rhs: Self) -> Self::Output {
        WBalance(U128(self.0.0 - rhs.0.0))
    }
}

impl Eq for WBalance {}

impl PartialEq<Self> for WBalance {
    fn eq(&self, other: &Self) -> bool {
        self.0.0 == other.0.0
    }
}

impl PartialOrd<Self> for WBalance {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Option::from(self.0.0.cmp(&other.0.0))
    }
}

impl Ord for WBalance {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.0.cmp(&other.0.0)
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













