use crate::metadata::Price;
use crate::{OrderStatus, OrderType};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{log, serde_json, AccountId};
use std::fmt;

pub const STANDARD: &str = "nep297";
pub const VERSION: &str = "1.0.0";
pub const EVENT_JSON_STR: &str = "EVENT_JSON:";

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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
#[serde(tag = "event", content = "data")]
#[serde(rename_all = "snake_case")]
#[allow(clippy::enum_variant_names)]
#[allow(dead_code)]
pub enum Event {
    CreateOrderEvent {
        order_id: u64,
        order_type: OrderType,
        lpt_id: String,
        sell_token_price: Price,
        buy_token_price: Price,
        pool_id: String,
    },
    CreateTakeProfitOrderEvent {
        order_id: U128,
        order_type: OrderType,
        lpt_id: String,
        close_price: U128,
        parent_order_type: OrderType,
        pool_id: String,
    },
    UpdateTakeProfitOrderEvent {
        order_id: U128,
        parent_order_type: OrderType,
        order_type: OrderType,
        order_status: OrderStatus,
        lpt_id: String,
        close_price: U128,
        sell_token: String,
        buy_token: String,
        sell_token_price: U128,
        buy_token_price: U128,
        pool_id: String,
    },
    CancelTakeProfitOrderEvent {
        order_id: U128,
    },
    CancelLimitOrderEvent {
        order_id: U128,
    },
    CancelLeverageOrderEvent {
        order_id: U128,
    },
    CloseLeveragePositionEvent {
        order_id: U128,
    },
    ExecuteOrderEvent {
        order_id: U128,
        order_type: OrderType,
    },
    RepayEvent {
        token_borrow: AccountId,
        token_market: AccountId,
        repay_amount: U128,
    },
    WithdrawEvent {
        token: AccountId,
        amount: U128,
    },
}

impl Event {
    #[allow(dead_code)]
    pub fn emit(&self) {
        emit_event(&self);
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct EventMessage {
    pub standard: String,
    pub version: String,
    pub event: serde_json::Value,
    pub data: serde_json::Value,
}

#[allow(dead_code)]
pub(crate) fn emit_event<T: ?Sized + Serialize>(data: &T) {
    let result = json!(data);
    let event_json = json!(EventMessage {
        standard: STANDARD.to_string(),
        version: VERSION.to_string(),
        event: result["event"].clone(),
        data: result["data"].clone()
    })
    .to_string();
    log!(format!("{EVENT_JSON_STR}{event_json}"));
}
