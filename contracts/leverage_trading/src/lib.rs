extern crate core;

#[allow(
    clippy::manual_range_contains,
    clippy::assign_op_pattern,
    clippy::ptr_offset_with_cast
)]
mod big_decimal;
mod cancel_order;
mod common;
mod config;
#[allow(clippy::too_many_arguments)]
mod create_order;
mod deposit;
mod execute_order;
mod ft;
mod liquidate_order;
mod market;
mod metadata;
mod oraclehook;
mod pnl;
mod price;
#[allow(clippy::too_many_arguments)]
mod ref_finance;
mod utils;
mod view;
mod withdraw;

pub use crate::metadata::*;

use crate::big_decimal::*;
use crate::common::PairId;
use crate::config::Config;
use crate::ref_finance::ShortLiquidityInfo;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::env::current_account_id;
use near_sdk::json_types::U128;
use near_sdk::{
    env, ext_contract, is_promise_success, log, near_bindgen, require, AccountId, Balance, Gas,
    PromiseOrValue, PromiseResult,
};
use ref_finance::ext_ref_finance;
use std::collections::HashMap;
use utils::{NO_DEPOSIT, PROTOCOL_DECIMALS};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn list_liquidities_callback(&self);
    fn remove_liquidity_from_list_liquidities_callback(&self, liquidity: ShortLiquidityInfo);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Protocol fee
    protocol_fee: u128,

    /// token ➝ Price
    prices: UnorderedMap<AccountId, Price>,

    /// total orders created on contract
    order_nonce: u64,

    /// user ➝ order_id ➝ Order
    orders: UnorderedMap<AccountId, HashMap<u64, Order>>,

    /// take profit orders, order_id ➝ Order
    take_profit_orders: LookupMap<u64, (PricePoints, Order)>,

    /// (sell token, buy token) ➝ TradePair
    supported_markets: UnorderedMap<PairId, TradePair>,

    /// User ➝ Token ➝ Balance
    balances: UnorderedMap<AccountId, HashMap<AccountId, Balance>>,

    config: Config,

    /// token id -> market id
    tokens_markets: LookupMap<AccountId, AccountId>,

    /// Protocol profit token_id -> amount
    protocol_profit: LookupMap<AccountId, BigDecimal>,

    /// Ref finance accountId [ as default "dclv2-dev.ref-dev.testnet" ]
    ref_finance_account: AccountId,

    /// Liquidation threshold
    liquidation_threshold: u128,

    /// Volatility rate
    volatility_rate: BigDecimal,

    /// Max value for order amount
    max_order_amount: u128,

    /// PairId ➝ order id ➝ Order
    orders_per_pair_view: UnorderedMap<PairId, HashMap<u64, Order>>,

    counter: u8,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new_with_config(owner_id: AccountId, oracle_account_id: AccountId) -> Self {
        Self::new(Config {
            owner_id,
            oracle_account_id,
        })
    }

    #[init]
    #[private]
    pub fn new(config: Config) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            protocol_fee: 10u128.pow(23),
            prices: UnorderedMap::new(StorageKeys::Prices),
            order_nonce: 0,
            orders: UnorderedMap::new(StorageKeys::Orders),
            take_profit_orders: LookupMap::new(StorageKeys::TakeProfitOrders),
            supported_markets: UnorderedMap::new(StorageKeys::SupportedMarkets),
            config,
            balances: UnorderedMap::new(StorageKeys::Balances),
            tokens_markets: LookupMap::new(StorageKeys::TokenMarkets),
            protocol_profit: LookupMap::new(StorageKeys::ProtocolProfit),
            ref_finance_account: "dclv2-dev.ref-dev.testnet".parse().unwrap(),
            liquidation_threshold: 10_u128.pow(23),
            volatility_rate: BigDecimal::from(U128(95 * 10_u128.pow(22))),
            max_order_amount: 10_u128.pow(30),
            orders_per_pair_view: UnorderedMap::new(StorageKeys::OrdersPerPair),
            counter: Default::default(),
        }
    }

    #[private]
    pub fn set_protocol_fee(&mut self, fee: U128) {
        self.protocol_fee = fee.0
    }

    #[private]
    pub fn add_token_market(&mut self, token_id: AccountId, market_id: AccountId) {
        self.tokens_markets.insert(&token_id, &market_id);
    }

    #[private]
    pub fn set_liquidation_threshold(&mut self, threshold: U128) {
        self.liquidation_threshold = threshold.0;
    }

    #[private]
    pub fn set_volatility_rate(&mut self, rate: U128) {
        self.volatility_rate = BigDecimal::from(rate)
    }

    #[private]
    pub fn set_max_order_amount(&mut self, value: U128) {
        self.max_order_amount = value.0
    }

    #[private]
    pub fn set_max_leverage(&mut self, pair: &PairId, leverage: U128) {
        let mut traid_pair = self
            .supported_markets
            .get(pair)
            .unwrap_or_else(|| panic!("Max leverage for pair {} | {} not found", pair.0, pair.1));

        traid_pair.max_leverage = leverage;
        self.supported_markets.insert(pair, &traid_pair);
    }

    pub fn get_max_leverage(&self, pair: &PairId) -> U128 {
        self.supported_markets
            .get(pair)
            .unwrap_or_else(|| panic!("Max leverage for pair {} | {} not found", pair.0, pair.1))
            .max_leverage
    }

    #[payable]
    pub fn simulate_create_order(&mut self) {
        if self.counter == 3 {
            self.counter = 0;
            self.clear_state();
        }
        self.counter += 1;
    }

    #[private]
    pub fn clear_state(&mut self) {
        self.orders.clear();

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(2)
            .with_attached_deposit(NO_DEPOSIT)
            .list_liquidities(current_account_id())
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(98)
                    .with_attached_deposit(NO_DEPOSIT)
                    .list_liquidities_callback(),
            );
    }

    #[private]
    pub fn list_liquidities_callback(&self) {
        require!(
            is_promise_success(),
            "Some problem with liquidity on ref finance"
        );

        let list_liquidities: Vec<ShortLiquidityInfo> = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(list_liquidities) =
                    near_sdk::serde_json::from_slice::<Vec<ShortLiquidityInfo>>(&val)
                {
                    list_liquidities
                } else {
                    panic!("Some problem with list liquidities parsing.")
                }
            }
            PromiseResult::Failed => panic!(
                "Ref finance not found list liquidities for {}",
                current_account_id()
            ),
        };

        let min_amount_x = U128::from(0);
        let min_amount_y = U128::from(0);

        for liquidity in list_liquidities {
            ext_ref_finance::ext(self.ref_finance_account.clone())
                .with_static_gas(Gas::ONE_TERA * 45u64)
                .remove_liquidity(
                    liquidity.lpt_id.clone(),
                    liquidity.amount,
                    min_amount_x,
                    min_amount_y,
                )
                .then(
                    ext_self::ext(current_account_id())
                        .with_unused_gas_weight(99)
                        .with_attached_deposit(NO_DEPOSIT)
                        .remove_liquidity_from_list_liquidities_callback(liquidity),
                );
        }
    }

    #[private]
    pub fn remove_liquidity_from_list_liquidities_callback(&self, liquidity: ShortLiquidityInfo) {
        require!(
            is_promise_success(),
            "Some problem with removing liquidity."
        );

        log!(
            "Successfully removed {} liquidity from {}",
            liquidity.amount.0,
            liquidity.lpt_id
        );
    }
}

impl Contract {
    pub fn get_swap_fee(&self, order: &Order) -> U128 {
        let pair = (order.sell_token.clone(), order.buy_token.clone());
        self.supported_markets
            .get(&pair)
            .unwrap_or_else(|| panic!("Swap fee for pair {} | {} not found", pair.0, pair.1))
            .swap_fee
    }

    pub fn from_token_to_protocol_decimals(&self, token_amount: u128, token_decimals: u8) -> U128 {
        if token_decimals != PROTOCOL_DECIMALS {
            U128::from(
                BigDecimal::from(token_amount)
                    / BigDecimal::from(10u128.pow(token_decimals.into())),
            )
        } else {
            U128::from(token_amount)
        }
    }

    pub fn from_protocol_to_token_decimals(&self, token_amount: U128, token_decimals: u8) -> U128 {
        U128::from(
            BigDecimal::from(token_amount)
                * BigDecimal::from(U128::from(10u128.pow(token_decimals.into()))),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::ref_finance::ShortLiquidityInfo;

    use super::*;
    use std::str::FromStr;

    #[test]
    fn test() {
        let bytes: Vec<u8> = [
            91, 123, 34, 108, 112, 116, 95, 105, 100, 34, 58, 34, 117, 115, 100, 116, 46, 100, 101,
            118, 101, 108, 111, 112, 46, 118, 49, 46, 111, 109, 111, 109, 111, 45, 102, 105, 110,
            97, 110, 99, 101, 46, 116, 101, 115, 116, 110, 101, 116, 124, 119, 110, 101, 97, 114,
            46, 100, 101, 118, 101, 108, 111, 112, 46, 118, 49, 46, 111, 109, 111, 109, 111, 45,
            102, 105, 110, 97, 110, 99, 101, 46, 116, 101, 115, 116, 110, 101, 116, 124, 50, 48,
            48, 48, 35, 49, 54, 48, 34, 44, 34, 111, 119, 110, 101, 114, 95, 105, 100, 34, 58, 34,
            108, 101, 118, 101, 114, 97, 103, 101, 46, 100, 101, 118, 101, 108, 111, 112, 46, 118,
            49, 46, 111, 109, 111, 109, 111, 45, 102, 105, 110, 97, 110, 99, 101, 46, 116, 101,
            115, 116, 110, 101, 116, 34, 44, 34, 112, 111, 111, 108, 95, 105, 100, 34, 58, 34, 117,
            115, 100, 116, 46, 100, 101, 118, 101, 108, 111, 112, 46, 118, 49, 46, 111, 109, 111,
            109, 111, 45, 102, 105, 110, 97, 110, 99, 101, 46, 116, 101, 115, 116, 110, 101, 116,
            124, 119, 110, 101, 97, 114, 46, 100, 101, 118, 101, 108, 111, 112, 46, 118, 49, 46,
            111, 109, 111, 109, 111, 45, 102, 105, 110, 97, 110, 99, 101, 46, 116, 101, 115, 116,
            110, 101, 116, 124, 50, 48, 48, 48, 34, 44, 34, 108, 101, 102, 116, 95, 112, 111, 105,
            110, 116, 34, 58, 55, 56, 56, 48, 44, 34, 114, 105, 103, 104, 116, 95, 112, 111, 105,
            110, 116, 34, 58, 55, 57, 50, 48, 44, 34, 97, 109, 111, 117, 110, 116, 34, 58, 34, 49,
            54, 51, 50, 55, 52, 57, 50, 55, 53, 48, 57, 56, 57, 48, 50, 53, 48, 52, 48, 49, 54, 55,
            50, 34, 44, 34, 109, 102, 116, 95, 105, 100, 34, 58, 34, 34, 44, 34, 118, 95, 108, 105,
            113, 117, 105, 100, 105, 116, 121, 34, 58, 34, 48, 34, 44, 34, 117, 110, 99, 108, 97,
            105, 109, 101, 100, 95, 102, 101, 101, 95, 120, 34, 58, 110, 117, 108, 108, 44, 34,
            117, 110, 99, 108, 97, 105, 109, 101, 100, 95, 102, 101, 101, 95, 121, 34, 58, 110,
            117, 108, 108, 125, 44, 123, 34, 108, 112, 116, 95, 105, 100, 34, 58, 34, 117, 115,
            100, 116, 46, 100, 101, 118, 101, 108, 111, 112, 46, 118, 49, 46, 111, 109, 111, 109,
            111, 45, 102, 105, 110, 97, 110, 99, 101, 46, 116, 101, 115, 116, 110, 101, 116, 124,
            119, 110, 101, 97, 114, 46, 100, 101, 118, 101, 108, 111, 112, 46, 118, 49, 46, 111,
            109, 111, 109, 111, 45, 102, 105, 110, 97, 110, 99, 101, 46, 116, 101, 115, 116, 110,
            101, 116, 124, 50, 48, 48, 48, 35, 49, 54, 49, 34, 44, 34, 111, 119, 110, 101, 114, 95,
            105, 100, 34, 58, 34, 108, 101, 118, 101, 114, 97, 103, 101, 46, 100, 101, 118, 101,
            108, 111, 112, 46, 118, 49, 46, 111, 109, 111, 109, 111, 45, 102, 105, 110, 97, 110,
            99, 101, 46, 116, 101, 115, 116, 110, 101, 116, 34, 44, 34, 112, 111, 111, 108, 95,
            105, 100, 34, 58, 34, 117, 115, 100, 116, 46, 100, 101, 118, 101, 108, 111, 112, 46,
            118, 49, 46, 111, 109, 111, 109, 111, 45, 102, 105, 110, 97, 110, 99, 101, 46, 116,
            101, 115, 116, 110, 101, 116, 124, 119, 110, 101, 97, 114, 46, 100, 101, 118, 101, 108,
            111, 112, 46, 118, 49, 46, 111, 109, 111, 109, 111, 45, 102, 105, 110, 97, 110, 99,
            101, 46, 116, 101, 115, 116, 110, 101, 116, 124, 50, 48, 48, 48, 34, 44, 34, 108, 101,
            102, 116, 95, 112, 111, 105, 110, 116, 34, 58, 56, 51, 50, 48, 44, 34, 114, 105, 103,
            104, 116, 95, 112, 111, 105, 110, 116, 34, 58, 56, 51, 54, 48, 44, 34, 97, 109, 111,
            117, 110, 116, 34, 58, 34, 51, 52, 56, 57, 56, 54, 53, 49, 57, 51, 55, 50, 55, 48, 53,
            54, 48, 49, 52, 49, 53, 54, 55, 49, 34, 44, 34, 109, 102, 116, 95, 105, 100, 34, 58,
            34, 34, 44, 34, 118, 95, 108, 105, 113, 117, 105, 100, 105, 116, 121, 34, 58, 34, 48,
            34, 44, 34, 117, 110, 99, 108, 97, 105, 109, 101, 100, 95, 102, 101, 101, 95, 120, 34,
            58, 110, 117, 108, 108, 44, 34, 117, 110, 99, 108, 97, 105, 109, 101, 100, 95, 102,
            101, 101, 95, 121, 34, 58, 110, 117, 108, 108, 125, 93,
        ]
        .to_vec();
        let list_liquidities: Vec<ShortLiquidityInfo> =
            near_sdk::serde_json::from_slice::<Vec<ShortLiquidityInfo>>(&bytes).unwrap();
        println!("{:#?}", list_liquidities);
    }

    #[test]
    fn test_get_max_leverage() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );
        let pair = (
            AccountId::from_str("usdt.fakes.testnet").unwrap(),
            AccountId::from_str("wrap.testnet").unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(10u128.pow(20)),
        };
        contract.add_pair(pair_data.clone());

        let result = pair_data.max_leverage;
        let max_leverage = contract.get_max_leverage(&pair);
        assert_eq!(max_leverage, result);
    }

    #[test]
    fn test_set_max_leverage() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );
        let pair = (
            AccountId::from_str("usdt.fakes.testnet").unwrap(),
            AccountId::from_str("wrap.testnet").unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(10u128.pow(20)),
        };
        contract.add_pair(pair_data);

        contract.set_max_leverage(&pair, U128(10 * 10_u128.pow(24)));
        let max_leverage = contract.get_max_leverage(&pair);
        assert_eq!(max_leverage, U128(10 * 10_u128.pow(24)));
    }

    #[test]
    fn test_get_swap_fee() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10u128.pow(20)),
        };
        contract.add_pair(pair_data.clone());

        let order = Order {
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: 1000000000000000000000000000,
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            buy_token: "wrap.testnet".parse().unwrap(),
            leverage: BigDecimal::from(1.0),
            sell_token_price: Price {
                ticker_id: "USDT".to_string(),
                value: U128::from(1010000000000000000000000),
            },
            buy_token_price: Price {
                ticker_id: "near".to_string(),
                value: U128::from(3070000000000000000000000),
            },
            open_or_close_price: BigDecimal::from(U128(1)),
            block: 105210654,
            timestamp_ms: 1675423354862,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#238".to_string(),
        };

        let swap_fee = contract.get_swap_fee(&order);

        assert_eq!(swap_fee, pair_data.swap_fee);
    }

    #[test]
    fn from_token_to_protocol_decimals_test() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        let token: AccountId = "usdt.fakes.testnet".parse().unwrap();
        let token_amount = U128::from(1_000_000_000);
        let token_decimals = contract.view_token_decimals(&token);

        let result = contract.from_token_to_protocol_decimals(token_amount.0, token_decimals);
        let expected_result = U128::from(1_000_000_000_000_000_000_000_000_000);

        assert_eq!(result, expected_result);

        assert_eq!(
            BigDecimal::from(result),          // 1000.0
            BigDecimal::from(expected_result)  // 1000.0
        );
    }

    #[test]
    fn from_protocol_to_token_decimals_test() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        let token: AccountId = "usdt.fakes.testnet".parse().unwrap();
        let token_amount_with_protocol_decimals = U128::from(1_000_000_000_000_000_000_000_000_000);
        let token_decimals = contract.view_token_decimals(&token);

        let result = contract
            .from_protocol_to_token_decimals(token_amount_with_protocol_decimals, token_decimals);
        let expected_result = U128::from(1_000_000_000);

        assert_eq!(result, expected_result);

        assert_eq!(
            BigDecimal::from(result),          // 1000.0
            BigDecimal::from(expected_result)  // 1000.0
        );
    }
}
