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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct PnLView {
    pub is_profit: bool,
    pub amount: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    pub ticker_id: String,
    pub value: U128,
}

/// left і right points for work with liquidity from ref-finance
pub type PricePoints = (i32, i32);

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderStatus {
    Pending,
    Executed,
    Canceled,
    Closed,
    Liquidated,
    PendingOrderExecute,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderType {
    Buy,
    Sell,
    Long,
    Short,
    TakeProfit,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
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
    /// position opening or closing price (xrate)
    pub open_or_close_price: BigDecimal,
    pub block: BlockHeight,
    pub timestamp_ms: Timestamp,
    pub lpt_id: String,
    /// data after closed position for trade history -> (Fee, PnL)
    pub history_data: Option<HistoryData>,
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
    pub buy_token_market: AccountId,
    pub pool_id: String,
    pub max_leverage: U128,
    pub swap_fee: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(crate = "near_sdk::serde")]
pub struct TradePairView {
    pub pair_id: PairId,
    pub pair_tickers_id: String,
    pub sell_ticker_id: String,
    pub sell_token: AccountId,
    pub sell_token_decimals: u8,
    pub sell_token_market: AccountId,
    pub buy_ticker_id: String,
    pub buy_token: AccountId,
    pub buy_token_decimals: u8,
    pub buy_token_market: AccountId,
    pub pool_id: String,
    pub max_leverage: U128,
    pub swap_fee: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CancelOrderView {
    pub buy_token_amount: WRatio,
    pub sell_token_amount: WRatio,
    pub open_or_close_price: WRatio,
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

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LimitOrderView {
    pub order_id: U128,
    pub timestamp: Timestamp,
    pub pair: String,
    pub order_type: String,
    pub side: OrderType,
    /// position opening price (xrate)
    pub price: WBalance,
    pub amount: U128,
    /// (0% if an order is pending, 100% if an order is executed)
    pub filled: u8,
    /// (amount * sell_token_price)
    pub total: WBalance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LimitOrders {
    pub data: Vec<LimitOrderView>,
    pub page: U128,
    pub total_orders: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LeveragedPositionView {
    pub order_id: U128,
    pub timestamp: Timestamp,
    pub pair: String,
    pub order_type: OrderType,
    /// (buy_token_price / sell_token_price from order)
    pub price: WBalance,
    pub leverage: U128,
    pub amount: U128,
    /// (0% if an order is pending, 100% if an order is executed)
    pub filled: u8,
    /// (amount * sell_token_price)
    pub total: WBalance,
    pub pnl: PnLView,
    /// Optional field with Take profit order related to the position
    pub take_profit_order: Option<TakeProfitOrderView>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LeveragedPositions {
    pub data: Vec<LeveragedPositionView>,
    pub page: U128,
    pub total_positions: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct TakeProfitOrderView {
    pub timestamp: Timestamp,
    pub pair: String,
    pub order_type: OrderType,
    /// position opening price (xrate)
    pub price: WBalance,
    pub amount: U128,
    /// (0% if an order is pending, 100% if an order is executed)
    pub filled: u8,
    /// (amount * sell_token_price)
    pub total: WBalance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Eq, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct LimitOrderTradeHistory {
    pub date: Timestamp,
    pub pair: String,
    pub side: OrderType,
    pub status: OrderStatus,
    pub price: U128,
    pub executed: U128,
    pub fee: U128,
    pub total: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct LimitTradeHistory {
    pub data: Vec<LimitOrderTradeHistory>,
    pub page: U128,
    pub total_orders: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Eq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct HistoryData {
    pub fee: U128,
    pub pnl: PnLView,
}