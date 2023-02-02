use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::AccountId;
use std::fmt;

use crate::metadata::Price;

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

pub enum Events {
    CreateOrderSuccess(u64, Price, Price, String),
}

impl fmt::Display for Events {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Events::CreateOrderSuccess(order_id, sell_token_price, buy_token_price, pool_id) => {
                write!(
                    f,
                    r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "CreateOrderSuccess", "data": {{"order_id": "{order_id}", "sell_token_price": "{sell_token_price:?}", "buy_token_price": "{buy_token_price:?}", "pool_id": "{pool_id}"}}}}"#
                )
            }
        }
    }
}
