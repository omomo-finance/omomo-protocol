use crate::*;

#[near_bindgen]
impl Contract {
    pub fn view_non_pending_margin_orders_by_user(
        &self,
        account_id: AccountId,
        orders_per_page: U128,
        page: U128,
    ) -> MarginTradeHistory {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut non_pending_margin_order = orders
            .iter()
            .filter_map(|(order_id, order)| {
                if order.leverage != BigDecimal::one() {
                    self.get_non_pending_margin_order(U128::from(*order_id as u128), order)
                } else {
                    None
                }
            })
            .collect::<Vec<MarginOrderTradeHistory>>();

        non_pending_margin_order.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_orders = U128::from(non_pending_margin_order.len() as u128);

        let sorted_non_pending_margin_order = non_pending_margin_order
            .into_iter()
            .skip((orders_per_page.0 * page.0 - orders_per_page.0) as usize)
            .take(orders_per_page.0 as usize)
            .collect();

        MarginTradeHistory {
            data: sorted_non_pending_margin_order,
            page,
            total_orders,
        }
    }

    pub fn view_non_pending_margin_orders_by_user_by_pair(
        &self,
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        orders_per_page: U128,
        page: U128,
    ) -> MarginTradeHistory {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut non_pending_margin_order = orders
            .iter()
            .filter_map(|(order_id, order)| {
                if order.leverage != BigDecimal::one()
                    && order.sell_token == sell_token
                    && order.buy_token == buy_token
                {
                    self.get_non_pending_margin_order(U128::from(*order_id as u128), order)
                } else {
                    None
                }
            })
            .collect::<Vec<MarginOrderTradeHistory>>();

        non_pending_margin_order.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_orders = U128::from(non_pending_margin_order.len() as u128);

        let sorted_non_pending_margin_order = non_pending_margin_order
            .into_iter()
            .skip((orders_per_page.0 * page.0 - orders_per_page.0) as usize)
            .take(orders_per_page.0 as usize)
            .collect();

        MarginTradeHistory {
            data: sorted_non_pending_margin_order,
            page,
            total_orders,
        }
    }
}

impl Contract {
    pub fn get_non_pending_margin_order(
        &self,
        order_id: U128,
        order: &Order,
    ) -> Option<MarginOrderTradeHistory> {
        match order.status {
            OrderStatus::Canceled | OrderStatus::Closed | OrderStatus::Liquidated => {
                let timestamp = order.timestamp_ms;

                let pair = self.view_pair(&order.sell_token, &order.buy_token);
                let pair = format!("{}/{}", pair.sell_ticker_id, pair.buy_ticker_id);

                let side = order.order_type.clone();

                let status = order.status.clone();

                let leverage = U128::from(order.leverage);

                let price = U128::from(order.open_or_close_price);

                let executed = match order.order_type {
                    OrderType::Long | OrderType::Short => U128::from(
                        BigDecimal::from(U128::from(order.amount)) * order.leverage
                            / order.open_or_close_price,
                    ),
                    _ => unreachable!(),
                };

                let (fee, pnl) = if let Some(history_data) = &order.history_data {
                    (history_data.fee, history_data.pnl.clone())
                } else {
                    (
                        U128::from(0),
                        PnLView {
                            is_profit: false,
                            amount: U128::from(0),
                        },
                    )
                };

                let total = match order.order_type {
                    OrderType::Long | OrderType::Short => {
                        U128::from(BigDecimal::from(U128::from(order.amount)) * order.leverage)
                    }
                    _ => unreachable!(),
                };

                Some(MarginOrderTradeHistory {
                    order_id,
                    timestamp,
                    pair,
                    side,
                    status,
                    leverage,
                    price,
                    executed,
                    fee,
                    pnl,
                    total,
                })
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::utils::MILLISECONDS_PER_DAY;
    use near_sdk::{
        test_utils::{test_env::alice, VMContextBuilder},
        testing_env, VMContext,
    };

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

    fn get_current_day_in_nanoseconds(day: u64) -> Option<u64> {
        let nanoseconds_in_one_millisecond = 1_000_000;
        Some(MILLISECONDS_PER_DAY * day * nanoseconds_in_one_millisecond)
    }

    #[test]
    fn view_non_pending_margin_orders_by_user_test() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data1 = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        // pair data for "WNEAR/USDT"
        let pair_data2 = TradePair {
            sell_ticker_id: "WNEAR".to_string(),
            sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "USDT".to_string(),
            buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 6,
            buy_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            pool_id: "wnear.qa.v1.nearlend.testnet|usdt.qa.v1.nearlend.testnet|2001".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data1);
        contract.add_pair(pair_data2);

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDT".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "WNEAR".to_string(),
                value: U128::from(3 * 10_u128.pow(24)), // current price token
            },
        );

        for count in 0..6 {
            if count < 1 {
                // order with status of "Pending" on leverage "3.0" and with timestamp "86400000"
                let order_as_string = "{\"status\":\"Closed\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Executed" on leverage "3.0" and with timestamp "86400001"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Short\",\"amount\":2000000000000000000000000000,\"sell_token\":\"wnear.qa.v1.nearlend.testnet\",\"buy_token\":\"usdt.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Pending" on leverage "1.0"
                let order_as_string = "{\"status\":\"Liquidated\",\"order_type\":\"Short\",\"amount\":3000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400002,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":4000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400003,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let non_pending_margin_orders_by_user =
            contract.view_non_pending_margin_orders_by_user(alice(), U128(10), U128(1));

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(non_pending_margin_orders_by_user.data.len(), 2_usize);
        assert_eq!(non_pending_margin_orders_by_user.total_orders, U128(2));
    }

    #[test]
    fn view_non_pending_margin_orders_by_user_by_pair_test() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_id: PairId = (
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data1 = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        // pair data for "WNEAR/USDT"
        let pair_data2 = TradePair {
            sell_ticker_id: "WNEAR".to_string(),
            sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "USDT".to_string(),
            buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 6,
            buy_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            pool_id: "wnear.qa.v1.nearlend.testnet|usdt.qa.v1.nearlend.testnet|2001".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data1);
        contract.add_pair(pair_data2);

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDT".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "WNEAR".to_string(),
                value: U128::from(3 * 10_u128.pow(24)), // current price token
            },
        );

        for count in 0..6 {
            if count < 1 {
                // order with status of "Pending" on leverage "3.0" and with timestamp "86400000"
                let order_as_string = "{\"status\":\"Closed\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Executed" on leverage "3.0" and with timestamp "86400001"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Short\",\"amount\":2000000000000000000000000000,\"sell_token\":\"wnear.qa.v1.nearlend.testnet\",\"buy_token\":\"usdt.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Pending" on leverage "1.0"
                let order_as_string = "{\"status\":\"Liquidated\",\"order_type\":\"Short\",\"amount\":3000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400002,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":4000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400003,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\",\"history_data\":null}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let non_pending_margin_orders_by_user = contract
            .view_non_pending_margin_orders_by_user_by_pair(
                alice(),
                pair_id.0,
                pair_id.1,
                U128(10),
                U128(1),
            );

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(non_pending_margin_orders_by_user.data.len(), 1_usize);
        assert_eq!(non_pending_margin_orders_by_user.total_orders, U128(1));
    }
}
