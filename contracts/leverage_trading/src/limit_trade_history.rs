use crate::*;

#[near_bindgen]
impl Contract {
    pub fn view_non_pending_limit_orders_by_user(
        &self,
        account_id: AccountId,
        orders_per_page: U128,
        page: U128,
    ) -> LimitTradeHistory {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut non_pending_limit_orders = orders
            .iter()
            .filter_map(|(order_id, order)| {
                if order.leverage == BigDecimal::one() {
                    self.get_non_pending_limit_order(U128::from(*order_id as u128), order)
                } else {
                    None
                }
            })
            .collect::<Vec<LimitOrderTradeHistory>>();

        non_pending_limit_orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_orders = U128::from(non_pending_limit_orders.len() as u128);

        let sorted_non_pending_limit_orders = non_pending_limit_orders
            .into_iter()
            .skip((orders_per_page.0 * page.0 - orders_per_page.0) as usize)
            .take(orders_per_page.0 as usize)
            .collect();

        LimitTradeHistory {
            data: sorted_non_pending_limit_orders,
            page,
            total_orders,
        }
    }

    pub fn view_non_pending_limit_orders_by_user_by_pair(
        &self,
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        orders_per_page: U128,
        page: U128,
    ) -> LimitTradeHistory {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut non_pending_limit_orders = orders
            .iter()
            .filter_map(|(order_id, order)| {
                if order.leverage == BigDecimal::one()
                    && order.sell_token == sell_token
                    && order.buy_token == buy_token
                {
                    self.get_non_pending_limit_order(U128::from(*order_id as u128), order)
                } else {
                    None
                }
            })
            .collect::<Vec<LimitOrderTradeHistory>>();

        non_pending_limit_orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_orders = U128::from(non_pending_limit_orders.len() as u128);

        let sorted_non_pending_limit_orders = non_pending_limit_orders
            .into_iter()
            .skip((orders_per_page.0 * page.0 - orders_per_page.0) as usize)
            .take(orders_per_page.0 as usize)
            .collect();

        LimitTradeHistory {
            data: sorted_non_pending_limit_orders,
            page,
            total_orders,
        }
    }
}

impl Contract {
    pub fn get_non_pending_limit_order(
        &self,
        order_id: U128,
        order: &Order,
    ) -> Option<LimitOrderTradeHistory> {
        match order.status {
            OrderStatus::Canceled | OrderStatus::Executed => {
                let timestamp = order.timestamp_ms;

                let pair = self.get_trade_pair(&order.sell_token, &order.buy_token);
                let pair = format!("{}/{}", pair.sell_ticker_id, pair.buy_ticker_id);

                let side = order.order_type.clone();

                let status = order.status.clone();

                let price = U128::from(order.open_or_close_price);

                let (fee, executed) = if let Some(history_data) = &order.history_data {
                    (history_data.fee, history_data.executed)
                } else {
                    (
                        U128::from(0),
                        match order.order_type {
                            OrderType::Buy => U128::from(
                                BigDecimal::from(U128::from(order.amount))
                                    / order.open_or_close_price,
                            ),
                            OrderType::Sell => U128::from(order.amount),
                            _ => U128::from(0),
                        },
                    )
                };

                let total = match order.order_type {
                    OrderType::Buy => U128::from(order.amount),
                    OrderType::Sell => U128::from(
                        BigDecimal::from(U128::from(order.amount)) * order.open_or_close_price,
                    ),
                    _ => U128::from(0),
                };

                Some(LimitOrderTradeHistory {
                    order_id,
                    timestamp,
                    pair,
                    side,
                    status,
                    price,
                    executed,
                    fee,
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
    use near_sdk::test_utils::test_env::alice;

    #[test]
    fn view_non_pending_limit_orders_by_user_test() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
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

        contract.add_pair(pair_data);

        for count in 0..6 {
            if count < 1 {
                // order with status of "Canceled" on leverage "1.0" and with timestamp "86400000"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Pending" on leverage "1.0"and with timestamp "86400001"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Pending" on leverage "2.0"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0" and with timestamp "86400003"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400003,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let first_limit_order_trade_history = LimitOrderTradeHistory {
            order_id: U128::from(1),
            timestamp: 86400000,
            pair: "USDT/WNEAR".to_string(),
            side: OrderType::Buy,
            status: OrderStatus::Canceled,
            price: U128(25 * 10_u128.pow(23)),
            executed: U128::from(
                BigDecimal::from(U128::from(2000000000000000000000000000))
                    / BigDecimal::from(U128::from(2500000000000000000000000)),
            ),
            fee: U128::from(0),
            total: U128(2 * 10_u128.pow(27)),
        };

        let limit_trade_history_by_user =
            contract.view_non_pending_limit_orders_by_user(alice(), U128(10), U128(1));

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(limit_trade_history_by_user.data.len(), 3_usize);
        assert_eq!(limit_trade_history_by_user.total_orders, U128(3));
        assert_eq!(
            limit_trade_history_by_user.data.get(0).unwrap(),
            &first_limit_order_trade_history
        );
    }

    #[test]
    fn view_non_pending_limit_orders_by_user_by_pair_test() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_id = PairId {
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        };

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

        for count in 0..6 {
            if count < 1 {
                // order with status of "Executed" on leverage "1.0" and in pair "USDT/WNEAR" with timestamp "86400000"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Pending" on leverage "1.0" and in pair "USDT/WNEAR" with timestamp "86400001"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Sell\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 3 {
                // order with status of "Executed" on leverage "1.0" and in pair "WNEAR/USDT"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"wnear.qa.v1.nearlend.testnet\",\"buy_token\":\"usdt.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Canceled" on leverage "2.0" and in pair "USDT/WNEAR"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Canceled" on leverage "1.0" and in pair "USDT/WNEAR" with timestamp "86400003"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400003,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let first_limit_order_trade_history = LimitOrderTradeHistory {
            order_id: U128::from(1),
            timestamp: 86400000,
            pair: "USDT/WNEAR".to_string(),
            side: OrderType::Buy,
            status: OrderStatus::Executed,
            price: U128(25 * 10_u128.pow(23)),
            executed: U128::from(
                BigDecimal::from(U128::from(2000000000000000000000000000))
                    / BigDecimal::from(U128::from(2500000000000000000000000)),
            ),
            fee: U128::from(0),
            total: U128(2 * 10_u128.pow(27)),
        };

        let limit_trade_history_by_user_by_pair = contract
            .view_non_pending_limit_orders_by_user_by_pair(
                alice(),
                pair_id.sell_token,
                pair_id.buy_token,
                U128(10),
                U128(1),
            );

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(
            contract
                .view_non_pending_limit_orders_by_user(alice(), U128(10), U128(1))
                .total_orders,
            U128(4)
        );
        assert_eq!(limit_trade_history_by_user_by_pair.data.len(), 3_usize);
        assert_eq!(limit_trade_history_by_user_by_pair.total_orders, U128(3));
        assert_eq!(
            limit_trade_history_by_user_by_pair.data.get(0).unwrap(),
            &first_limit_order_trade_history
        );
    }
}
