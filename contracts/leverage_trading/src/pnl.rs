use crate::big_decimal::{BigDecimal, LowU128};
use crate::metadata::{MarketData, Order, PnLView};
use crate::Contract;

use near_sdk::{env, json_types::U128};

pub const MILLISECONDS_PER_DAY: u64 = 86400000;
const DAYS_PER_YEAR: u16 = 360;

impl Contract {
    pub fn calculate_pnl_long_order(&self, order: Order, data: Option<MarketData>) -> PnLView {
        let current_timestamp_ms = env::block_timestamp_ms();

        let borrow_period = ((current_timestamp_ms - order.timestamp_ms) as f64
            / MILLISECONDS_PER_DAY as f64)
            .ceil();

        let swap_fee = BigDecimal::from(self.get_swap_fee(&order));

        let borrow_amount =
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one());

        let buy_amount =
            order.leverage * BigDecimal::from(U128(order.amount)) / order.open_or_close_price;

        let mut borrow_fee_amount = BigDecimal::zero();
        #[allow(clippy::unnecessary_unwrap)]
        if data.is_some() && (order.leverage > BigDecimal::one()) {
            borrow_fee_amount = borrow_amount * BigDecimal::from(data.unwrap().borrow_rate_ratio)
                / BigDecimal::from(U128(DAYS_PER_YEAR as u128))
                * BigDecimal::from(U128(borrow_period as u128));
        }

        let current_buy_token_price = BigDecimal::from(self.view_price(order.buy_token).value);
        let current_sell_token_price = BigDecimal::from(self.view_price(order.sell_token).value);

        let swap_fee_amount =
            buy_amount * current_buy_token_price / current_sell_token_price * swap_fee;

        let expect_amount = buy_amount * current_buy_token_price / current_sell_token_price
            - borrow_amount
            - borrow_fee_amount
            - swap_fee_amount;

        let pnlv: PnLView = if LowU128::from(expect_amount).0 > order.amount {
            let lenpnl = LowU128::from(
                (expect_amount - BigDecimal::from(U128(order.amount))) * current_sell_token_price,
            );

            PnLView {
                is_profit: true,
                amount: lenpnl,
            }
        } else {
            let lenpnl = LowU128::from(
                (BigDecimal::from(U128(order.amount)) - expect_amount) * current_sell_token_price,
            );

            PnLView {
                is_profit: false,
                amount: lenpnl,
            }
        };

        pnlv
    }

    pub fn calculate_pnl_short_order(&self, order: Order, data: Option<MarketData>) -> PnLView {
        let current_timestamp_ms = env::block_timestamp_ms();

        let borrow_period = ((current_timestamp_ms - order.timestamp_ms) as f64
            / MILLISECONDS_PER_DAY as f64)
            .ceil();

        let swap_fee = BigDecimal::from(self.get_swap_fee(&order));

        let borrow_amount = BigDecimal::from(U128(order.amount))
            * (order.leverage - BigDecimal::one())
            * BigDecimal::from(order.sell_token_price.value)
            / BigDecimal::from(order.buy_token_price.value);

        let buy_amount = borrow_amount * order.open_or_close_price;

        let mut borrow_fee_amount = BigDecimal::zero();
        #[allow(clippy::unnecessary_unwrap)]
        if data.is_some() && (order.leverage > BigDecimal::one()) {
            borrow_fee_amount = borrow_amount * BigDecimal::from(data.unwrap().borrow_rate_ratio)
                / BigDecimal::from(U128(DAYS_PER_YEAR as u128))
                * BigDecimal::from(U128(borrow_period as u128));
        }

        let current_buy_token_price = BigDecimal::from(self.view_price(order.buy_token).value);
        let current_sell_token_price = BigDecimal::from(self.view_price(order.sell_token).value);

        let swap_fee_amount =
            buy_amount / current_buy_token_price / current_sell_token_price * swap_fee;

        let expect_amount = buy_amount / current_buy_token_price / current_sell_token_price
            - borrow_fee_amount
            - swap_fee_amount;

        let pnlv: PnLView = if expect_amount > borrow_amount {
            let lenpnl = LowU128::from((expect_amount - borrow_amount) * current_buy_token_price);

            PnLView {
                is_profit: true,
                amount: lenpnl,
            }
        } else {
            let lenpnl = LowU128::from((borrow_amount - expect_amount) * current_buy_token_price);

            PnLView {
                is_profit: false,
                amount: lenpnl,
            }
        };

        pnlv
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::metadata::{Price, TradePair};

    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, VMContext};

    fn get_context(is_view: bool, block_timestamp: Option<u64>) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("margin.nearland.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
            .block_index(103930916)
            .block_timestamp(block_timestamp.unwrap_or(1))
            .is_view(is_view)
            .build()
    }

    fn get_market_data() -> MarketData {
        MarketData {
            underlying_token: AccountId::new_unchecked("usdt.fakes.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        }
    }

    fn get_pair_data() -> TradePair {
        TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(2 * 10_u128.pow(21)),
        }
    }

    fn get_current_day_in_nanoseconds(day: u64) -> Option<u64> {
        let nanoseconds_in_one_millisecond = 1_000_000;
        Some(MILLISECONDS_PER_DAY * day * nanoseconds_in_one_millisecond)
    }

    #[test]
    fn test_calculate_pnl_long_position_with_profit() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = get_pair_data();
        contract.add_pair(pair_data);

        let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        contract.add_order_from_string(alice(), order_as_string.clone());
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128::from(3 * 10_u128.pow(24)), // current price token
            },
        );

        let market_data = get_market_data();
        let pnl = contract.calculate_pnl_long_order(order, Some(market_data));
        assert!(pnl.is_profit);
        assert_eq!(pnl.amount, U128(7654 * 10_u128.pow(23)));
    }

    #[test]
    fn test_calculate_pnl_short_position_with_profit() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = get_pair_data();
        contract.add_pair(pair_data);

        let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Short\",\"amount\":3000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        contract.add_order_from_string(alice(), order_as_string.clone());
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128::from(2 * 10_u128.pow(24)), // current price token
            },
        );

        let market_data = get_market_data();
        let pnl = contract.calculate_pnl_short_order(order, Some(market_data));
        assert!(pnl.is_profit);
        assert_eq!(pnl.amount, U128(1128 * 10_u128.pow(24)));
    }

    #[test]
    fn test_calculate_pnl_long_position_without_profit() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = get_pair_data();
        contract.add_pair(pair_data);

        let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        contract.add_order_from_string(alice(), order_as_string.clone());
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128::from(2 * 10_u128.pow(24)), // current price token
            },
        );

        let market_data = get_market_data();
        let pnl = contract.calculate_pnl_long_order(order, Some(market_data));
        assert!(!pnl.is_profit);
        assert_eq!(pnl.amount, U128(8314 * 10_u128.pow(23)));
    }

    #[test]
    fn test_calculate_pnl_short_position_without_profit() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = get_pair_data();
        contract.add_pair(pair_data);

        let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Short\",\"amount\":3000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        contract.add_order_from_string(alice(), order_as_string.clone());
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128::from(3 * 10_u128.pow(24)), // current price token
            },
        );

        let market_data = get_market_data();
        let pnl = contract.calculate_pnl_short_order(order, Some(market_data));
        assert!(!pnl.is_profit);
        assert_eq!(pnl.amount, U128(651 * 10_u128.pow(24)));
    }
}
