use crate::big_decimal::{BigDecimal, WRatio};
use crate::*;
use near_sdk::Gas;

#[near_bindgen]
impl Contract {
    pub fn view_order(
        &self,
        account_id: AccountId,
        order_id: U128,
        borrow_rate_ratio: WRatio,
    ) -> OrderView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", account_id);
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        let borrow_fee = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - order.block as u128)),
        );

        OrderView {
            order_id,
            status: order.status,
            order_type: order.order_type,
            amount: U128(order.amount),
            sell_token: order.sell_token,
            sell_token_price: WBalance::from(order.sell_token_price.value),
            buy_token: order.buy_token,
            buy_token_price: WBalance::from(order.buy_token_price.value),
            leverage: WBigDecimal::from(order.leverage),
            borrow_fee,
            liquidation_price: self.calculate_liquidation_price(
                U128(order.amount),
                WBigDecimal::from(order.sell_token_price.value),
                WBigDecimal::from(order.buy_token_price.value),
                WBigDecimal::from(order.leverage),
                borrow_fee,
                U128(10u128.pow(23)), // hardcore of swap_fee 0.1 % with 10^24 precision
            ),
            lpt_id: order.lpt_id,
        }
    }

    pub fn calculate_pnl(
        &self,
        account_id: AccountId,
        order_id: U128,
        data: Option<MarketData>,
    ) -> PnLView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", account_id);
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        let buy_amount = order.leverage / BigDecimal::from(10_u128.pow(24))
            * BigDecimal::from(order.amount)
            / order.buy_token_price.value;

        let borrow_amount = BigDecimal::from(U128(order.amount))
            * (order.leverage - BigDecimal::from(10u128.pow(24)));

        let mut borrow_fee = BigDecimal::zero();
        #[allow(clippy::unnecessary_unwrap)]
        if data.is_some() && (order.leverage > BigDecimal::one()) {
            borrow_fee = borrow_amount * BigDecimal::from(data.unwrap().borrow_rate_ratio);
        } // fee by blocks count
          //* BigDecimal::from(block_height() - order.block);

        //swap_fee 0.0003
        let expect_amount = buy_amount * order.sell_token_price.value
            - borrow_amount
            - borrow_fee
            - borrow_amount * BigDecimal::from(0.0003);

        let pnlv: PnLView = if expect_amount.round_u128() > order.amount {
            let lenpnl = (expect_amount
                - BigDecimal::from(order.amount)
                - (BigDecimal::from(order.amount)
                    * BigDecimal::from(self.protocol_fee / 10_u128.pow(24))))
            .round_u128();

            PnLView {
                is_profit: true,
                amount: U128(lenpnl),
            }
        } else {
            let lenpnl = (BigDecimal::from(order.amount) - expect_amount).round_u128();

            PnLView {
                is_profit: false,
                amount: U128(lenpnl),
            }
        };

        pnlv
    }

    pub fn view_orders(
        &self,
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        borrow_rate_ratio: WRatio,
    ) -> Vec<OrderView> {
        let orders = self.orders.get(&account_id).unwrap_or_default();
        let result = orders
            .iter()
            .filter_map(|(id, order)| {
                match order.sell_token == sell_token && order.buy_token == buy_token {
                    true => {
                        let borrow_fee = WBigDecimal::from(
                            BigDecimal::from(borrow_rate_ratio)
                                * BigDecimal::from(U128(
                                    env::block_height() as u128 - order.block as u128,
                                )),
                        );

                        Some(OrderView {
                            order_id: U128(*id as u128),
                            status: order.status.clone(),
                            order_type: order.order_type.clone(),
                            amount: U128(order.amount),
                            sell_token: order.sell_token.clone(),
                            sell_token_price: WBigDecimal::from(order.sell_token_price.value),
                            buy_token: order.buy_token.clone(),
                            buy_token_price: WBigDecimal::from(order.buy_token_price.value),
                            leverage: WBigDecimal::from(order.leverage),
                            borrow_fee,
                            liquidation_price: self.calculate_liquidation_price(
                                U128(order.amount),
                                WBigDecimal::from(order.sell_token_price.value),
                                WBigDecimal::from(order.buy_token_price.value),
                                WBigDecimal::from(order.leverage),
                                borrow_fee,
                                U128(10u128.pow(23)), // hardcore of swap_fee 0.1 % with 10^24 precision
                            ),
                            lpt_id: order.lpt_id.clone(),
                        })
                    }
                    false => None,
                }
            })
            .collect::<Vec<OrderView>>();
        result
    }

    pub fn view_pair(&self, sell_token: &AccountId, buy_token: &AccountId) -> TradePair {
        self.supported_markets
            .get(&(sell_token.clone(), buy_token.clone()))
            .unwrap()
    }

    pub fn view_supported_pairs(&self) -> Vec<TradePair> {
        let pairs = self
            .supported_markets
            .iter()
            .map(|(_, trade_pair)| trade_pair)
            .collect::<Vec<TradePair>>();

        pairs
    }

    /// Returns the balance of the given account on certain token. If the account doesn't exist will return `"0"`.
    pub fn balance_of(&self, account_id: AccountId, token: AccountId) -> WBalance {
        match self.balances.get(&account_id) {
            None => WBalance::from(0_u128),
            Some(user_balance_per_token) => {
                WBalance::from(*user_balance_per_token.get(&token).unwrap_or(&0_u128))
            }
        }
    }

    /// Returns price of the given token. If the token is not registered, will return `"0"`.
    pub fn view_price(&self, token_id: AccountId) -> Price {
        self.prices.get(&token_id).unwrap_or_else(|| {
            panic!("Price for token: {} not found", token_id);
        })
    }

    pub fn cancel_order_view(
        &self,
        account_id: AccountId,
        order_id: U128,
        market_data: MarketData,
    ) -> CancelOrderView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", account_id);
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        let buy_token =
            BigDecimal::from(U128(order.amount)) * order.leverage * order.sell_token_price.value
                / order.buy_token_price.value;

        let sell_token = BigDecimal::from(U128(order.amount)) * order.leverage;

        let open_price = order.buy_token_price.clone();

        let close_price = self.get_price(order.buy_token.clone());

        let calc_pnl = self.calculate_pnl(account_id, order_id, Some(market_data));

        CancelOrderView {
            buy_token_amount: WRatio::from(buy_token),
            sell_token_amount: WRatio::from(sell_token),
            open_price: WRatio::from(open_price.value),
            close_price: WRatio::from(close_price),
            pnl: calc_pnl,
        }
    }

    pub fn view_liquidation_threshold(&self) -> U128 {
        U128(self.liquidation_threshold)
    }

    pub fn calculate_liquidation_price(
        &self,
        sell_token_amount: U128,
        sell_token_price: U128,
        buy_token_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> WBigDecimal {
        let collateral_usd =
            BigDecimal::from(sell_token_amount) * BigDecimal::from(sell_token_price);
        let position_amount_usd = collateral_usd * BigDecimal::from(leverage);
        let borrow_amount = collateral_usd * (BigDecimal::from(leverage) - BigDecimal::one());
        let buy_amount = position_amount_usd / BigDecimal::from(buy_token_price);

        let liquidation_price = (position_amount_usd - self.volatility_rate * collateral_usd
            + borrow_amount * BigDecimal::from(borrow_fee)
            + position_amount_usd * BigDecimal::from(swap_fee))
            / buy_amount;

        liquidation_price.into()
    }

    /// returns const gas amount required for executing orders: 50 TGas
    pub fn view_gas_for_execution(&self) -> Balance {
        Gas::ONE_TERA.0 as Balance * 50u128
    }

    pub fn view_max_position_amount(&self) -> U128 {
        U128(self.max_order_amount)
    }

    /// this method uses the mock data
    #[allow(unused_variables)]
    pub fn get_total_pending_orders_per_pair(&self, pair_id: PairId) -> U128 {
        U128(3)
    }

    /// his method uses the mock data
    #[allow(unused_variables)]
    pub fn get_pending_orders(
        &self,
        pair_id: PairId,
        user_per_page: U128,
        page: U128,
    ) -> PendingOrders {
        let sell_token: AccountId = "usdt.fakes.testnet".parse().unwrap();
        let buy_token: AccountId = "wrap.testnet".parse().unwrap();

        let pending_orders = vec![
            (
                1_u64,
                Order {
                    status: OrderStatus::Pending,
                    order_type: OrderType::Buy,
                    amount: 1000000000000000000000000000,
                    sell_token: sell_token.clone(),
                    buy_token: buy_token.clone(),
                    leverage: BigDecimal::from(U128(10_u128.pow(24))),
                    sell_token_price: Price {
                        ticker_id: "USDt".to_string(),
                        value: BigDecimal::from(U128(101 * 10_u128.pow(22))),
                    },
                    buy_token_price: Price {
                        ticker_id: "near".to_string(),
                        value: BigDecimal::from(U128(305 * 10_u128.pow(22))),
                    },
                    block: 103930910,
                    lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#540"
                        .to_string(),
                },
            ),
            (
                2_u64,
                Order {
                    status: OrderStatus::Pending,
                    order_type: OrderType::Buy,
                    amount: 1500000000000000000000000000,
                    sell_token: sell_token.clone(),
                    buy_token: buy_token.clone(),
                    leverage: BigDecimal::from(U128(15 * 10_u128.pow(23))),
                    sell_token_price: Price {
                        ticker_id: "USDt".to_string(),
                        value: BigDecimal::from(U128(101 * 10_u128.pow(22))),
                    },
                    buy_token_price: Price {
                        ticker_id: "near".to_string(),
                        value: BigDecimal::from(U128(305 * 10_u128.pow(22))),
                    },
                    block: 103930910,
                    lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#541"
                        .to_string(),
                },
            ),
            (
                3_u64,
                Order {
                    status: OrderStatus::Pending,
                    order_type: OrderType::Buy,
                    amount: 2000000000000000000000000000,
                    sell_token,
                    buy_token,
                    leverage: BigDecimal::from(U128(2 * 10_u128.pow(24))),
                    sell_token_price: Price {
                        ticker_id: "USDt".to_string(),
                        value: BigDecimal::from(U128(101 * 10_u128.pow(22))),
                    },
                    buy_token_price: Price {
                        ticker_id: "near".to_string(),
                        value: BigDecimal::from(U128(305 * 10_u128.pow(22))),
                    },
                    block: 103930910,
                    lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#542"
                        .to_string(),
                },
            ),
        ];

        PendingOrders {
            data: pending_orders,
            page: U128(1),
            total: U128(3),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::test_env::{alice, bob};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("margin.nearland.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
            .block_index(103930916)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn view_supported_pairs_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );
        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet".parse().unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
        };
        contract.add_pair(pair_data.clone());

        let pair_data2 = TradePair {
            sell_ticker_id: "near".to_string(),
            sell_token: "wrap.testnet".parse().unwrap(),
            sell_token_market: "wnear_market.develop.v1.omomo-finance.testnet".parse().unwrap(),
            buy_ticker_id: "USDt".to_string(),
            buy_token: "usdt.fakes.testnet".parse().unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
        };

        contract.add_pair(pair_data2.clone());

        let result = vec![pair_data, pair_data2];
        let pairs = contract.view_supported_pairs();
        assert_eq!(result, pairs);
    }

    #[test]
    fn calculate_pnl_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: BigDecimal::from(2.0),
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: BigDecimal::from(4.22),
            },
        );
        let order1 = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1500000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"2000000000000000000000000\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"3.3\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.59\"},\"block\":1,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        contract.add_order(alice(), order1);
        let market_data = MarketData {
            underlying_token: AccountId::new_unchecked("usdt.fakes.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        };
        let pnl = contract.calculate_pnl(alice(), U128(1), Some(market_data));
        assert!(!pnl.is_profit);
        assert_eq!(pnl.amount, U128(918587254901960784313725490));
    }

    #[test]
    fn calculate_pnl_leverage_3_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: BigDecimal::from(2.0),
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: BigDecimal::from(4.22),
            },
        );
        let order1 = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1500000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"3000000000000000000000000\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"3.3\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.59\"},\"block\":1,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        contract.add_order(alice(), order1);
        let market_data = MarketData {
            underlying_token:  AccountId::new_unchecked("alice.testnet".to_string()),
            underlying_token_decimals: 24,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        };
        let pnl = contract.calculate_pnl(alice(), U128(1), Some(market_data));
        assert!(!pnl.is_profit);
        assert_eq!(pnl.amount, U128(1415605882352941176470588235));
    }

    #[test]
    fn test_calculate_liquidation_leverage_3() {
        let contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let result = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(10_u128.pow(24)),
            U128(10_u128.pow(25)),
            U128(3 * 10_u128.pow(24)),
            U128(5 * 10_u128.pow(22)),
            U128(3 * 10_u128.pow(20)),
        );

        assert_eq!(result, U128(7169666666666666666666666));
    }

    #[test]
    fn test_calculate_liquidation_leverage_1_5() {
        let contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let result = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(10_u128.pow(24)),
            U128(10_u128.pow(25)),
            U128(15 * 10_u128.pow(23)),
            U128(5 * 10_u128.pow(22)),
            U128(3 * 10_u128.pow(20)),
        );

        assert_eq!(result, U128(3836333333333333333333333));
    }

    #[test]
    fn test_view_order() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let borrow_rate_ratio = U128(634273735391536);

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"block\":103930910,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#540\"}".to_string();
        contract.add_order(alice(), order1);

        let order2 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"block\":103930910,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#541\"}".to_string();
        contract.add_order(alice(), order2);

        let order_id = U128(1); //order_id for order1

        let block_order = 103930910_u64;

        let borrow_fee = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order as u128)),
        );

        let liquidation_price = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee,
            U128(10u128.pow(23)), // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let result_view_order1 = OrderView {
            order_id: U128(1),
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: U128(10_u128.pow(27)),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
            leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
            borrow_fee,
            liquidation_price,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#540".to_string(),
        };

        assert_eq!(
            contract.view_order(alice(), order_id, borrow_rate_ratio),
            result_view_order1
        );
    }

    #[test]
    fn test_view_orders() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let sell_token: AccountId = "usdt.fakes.testnet".parse().unwrap();
        let buy_token: AccountId = "wrap.testnet".parse().unwrap();

        let borrow_rate_ratio = U128(634273735391536);

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"block\":103930910,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#540\"}".to_string();
        contract.add_order(alice(), order1);

        let order2 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"block\":103930911,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#541\"}".to_string();
        contract.add_order(alice(), order2);

        let order3 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"block\":103930912,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#542\"}".to_string();
        contract.add_order(bob(), order3);

        let block_order1 = 103930910_u64;
        let block_order2 = 103930911_u64;
        let block_order3 = 103930912_u64;

        let borrow_fee_order1 = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order1 as u128)),
        );

        let borrow_fee_order2 = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order2 as u128)),
        );

        let borrow_fee_order3 = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order3 as u128)),
        );

        let liquidation_price_order1 = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee_order1,
            U128(10u128.pow(23)), // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let liquidation_price_order2 = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee_order2,
            U128(10u128.pow(23)), // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let liquidation_price_order3 = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee_order3,
            U128(10u128.pow(23)), // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let result_view_orders_alice = vec![
            OrderView {
                order_id: U128(1),
                status: OrderStatus::Pending,
                order_type: OrderType::Buy,
                amount: U128(10_u128.pow(27)),
                sell_token: "usdt.fakes.testnet".parse().unwrap(),
                sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
                buy_token: "wrap.testnet".parse().unwrap(),
                buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
                leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
                borrow_fee: borrow_fee_order1,
                liquidation_price: liquidation_price_order1,
                lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#540"
                    .to_string(),
            },
            OrderView {
                order_id: U128(2),
                status: OrderStatus::Pending,
                order_type: OrderType::Buy,
                amount: U128(2 * 10_u128.pow(27)),
                sell_token: "usdt.fakes.testnet".parse().unwrap(),
                sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
                buy_token: "wrap.testnet".parse().unwrap(),
                buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
                leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
                borrow_fee: borrow_fee_order2,
                liquidation_price: liquidation_price_order2,
                lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#541"
                    .to_string(),
            },
        ];

        let result_view_orders_bob = vec![OrderView {
            order_id: U128(3),
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: U128(2 * 10_u128.pow(27)),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
            leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
            borrow_fee: borrow_fee_order3,
            liquidation_price: liquidation_price_order3,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#542".to_string(),
        }];

        let mut view_orders_alice = contract.view_orders(
            alice(),
            sell_token.clone(),
            buy_token.clone(),
            borrow_rate_ratio,
        );
        view_orders_alice.sort_by(|a, b| a.order_id.cmp(&b.order_id));

        let view_orders_bob = contract.view_orders(bob(), sell_token, buy_token, borrow_rate_ratio);

        assert_eq!(view_orders_alice, result_view_orders_alice);
        assert_eq!(view_orders_bob, result_view_orders_bob);
    }
}
