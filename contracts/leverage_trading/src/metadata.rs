use crate::big_decimal::{BigDecimal, WBalance, WRatio};
use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance, BlockHeight, BorshStorageKey, Timestamp};
use std::fmt;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Prices,
    Orders,
    OrdersPerPair,
    SupportedMarkets,
    Balances,
    TokenMarkets,
    ProtocolProfit,
    TakeProfitOrders,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub underlying_token: AccountId,
    /// WARN: should be the same as `underlying_token.ft_metadata.decimals`
    pub underlying_token_decimals: u8,

    /// Total supplies with precision 10^24
    pub total_supplies: WBalance,
    /// Total borrows with precision 10^24
    pub total_borrows: WBalance,
    /// Total reserves with precision 10^24
    pub total_reserves: WBalance,

    pub exchange_rate_ratio: WRatio,
    pub interest_rate_ratio: WRatio,
    pub borrow_rate_ratio: WRatio,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PnLView {
    pub is_profit: bool,
    pub amount: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    pub ticker_id: String,
    pub value: BigDecimal,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderStatus {
    Pending,
    Executed,
    Canceled,
    Liquidated,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub amount: Balance,
    pub sell_token: AccountId,
    pub buy_token: AccountId,
    pub leverage: BigDecimal,
    pub sell_token_price: Price,
    pub buy_token_price: Price,
    pub block: BlockHeight,
    pub lpt_id: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderView {
    pub order_id: U128,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub amount: U128,
    pub sell_token: AccountId,
    pub sell_token_price: WBalance,
    pub buy_token: AccountId,
    pub buy_token_price: WBalance,
    pub leverage: WBigDecimal,
    pub borrow_fee: WBalance,
    pub liquidation_price: WBalance,
    pub lpt_id: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct TakeProfitOrderView {
    pub order_id: U128,
    pub take_profit_price: WBalance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct TradePair {
    pub sell_ticker_id: String,
    pub sell_token: AccountId,
    pub sell_token_decimals: u8,
    pub sell_token_market: AccountId,
    pub buy_ticker_id: String,
    pub buy_token: AccountId,
    pub buy_token_decimals: u8,
    pub pool_id: String,
    pub max_leverage: U128,
    pub swap_fee: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CancelOrderView {
    pub buy_token_amount: WRatio,
    pub sell_token_amount: WRatio,
    pub open_price: WRatio,
    pub close_price: WRatio,
    pub pnl: PnLView,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderAction {
    Create,
    Cancel,
    Liquidate,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInfo {
    pub pool_id: String,
    pub token_x: AccountId,
    pub token_y: AccountId,
    pub fee: u64,
    pub point_delta: u64,
    pub current_point: i64,
    pub liquidity: U128,
    pub liquidity_x: U128,
    pub max_liquidity_per_point: U128,
    pub total_fee_x_charged: U128,
    pub total_fee_y_charged: U128,
    pub volume_x_in: U128,
    pub volume_y_in: U128,
    pub volume_x_out: U128,
    pub volume_y_out: U128,
    pub total_liquidity: U128,
    pub total_order_x: U128,
    pub total_order_y: U128,
    pub total_x: U128,
    pub total_y: U128,
    pub state: PoolState,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub enum PoolState {
    Running,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct Liquidity {
    pub lpt_id: String,
    pub owner_id: AccountId,
    pub pool_id: String,
    pub left_point: i64,
    pub right_point: i64,
    pub amount: U128,
    pub unclaimed_fee_x: U128,
    pub unclaimed_fee_y: U128,
}

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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PendingOrders {
    pub data: Vec<(u64, Order)>,
    pub page: U128,
    pub total: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LimitOrderView {
    pub time_stamp: Timestamp,
    pub pair: String,
    pub order_type: String,
    pub side: OrderType,
    /// (buy_token_price / sell_token_price from order)
    pub price: WBalance,
    pub amount: U128,
    /// (0% if an order is pending, 100% if an order is executed)
    pub filled: u8,
    /// (amount * sell_token_price)
    pub total: WBalance,
}
