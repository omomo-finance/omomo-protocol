use near_sdk::AccountId;
use std::fmt;

use near_sdk::serde::{Deserialize, Serialize};

/// (sell token, buy token)
pub type PairId = (AccountId, AccountId);

#[derive(Clone, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub enum Actions {
    Deposit { token: AccountId },
}

impl fmt::Display for Actions {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
