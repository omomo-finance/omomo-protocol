use crate::common::Event;
use crate::ref_finance::{ext_ref_finance, Action, LiquidityInfo, Swap};
use crate::utils::ext_token;
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, log, Gas, Promise, PromiseResult};

/// DEX underutilization ratio of the transferred deposit
pub const INACCURACY_RATE: U128 = U128(3300000000000000000000_u128); //0.0033 -> 0.33% -> 33*10^-4

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_for_execute_order_callback(&self, order: Order, order_id: U128);
    fn execute_order_callback(&self, order: Order, order_id: U128);
}

#[near_bindgen]
impl Contract {
    /// Executes order by inner order_id set on ref finance once the price range was crossed.
    /// Gets pool info, removes liquidity presented by one asset and marks order as executed.
    pub fn execute_order(&self, order_id: U128) -> PromiseOrValue<U128> {
        let order = self.get_order_by(order_id.0);
        require!(order.is_some(), "There is no such order to be executed");

        let mut order = order.unwrap();

        if order.status == OrderStatus::Executed {
            if let Some((_, tp_order, _)) = self.take_profit_orders.get(&(order_id.0 as u64)) {
                order = tp_order
            } else {
                panic!(
                    "Order 'order_id:{}' already executed and has no created 'Take Profit' order",
                    order_id.0
                )
            }
        }

        require!(
            order.status == OrderStatus::Pending,
            "Error. Order has to be Pending to be executed"
        );

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 5_u64)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 270_u64)
                    .with_unused_gas_weight(2_u64)
                    .execute_order_callback(order, order_id),
            )
            .into()
    }

    #[private]
    pub fn execute_order_callback(&self, order: Order, order_id: U128) -> PromiseOrValue<U128> {
        require!(is_promise_success(), "Failed to get_liquidity");

        let liquidity_info = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                near_sdk::serde_json::from_slice::<ref_finance::LiquidityInfo>(&val).unwrap()
            }
            PromiseResult::Failed => panic!("Ref finance not found pool"),
        };

        let [amount, min_amount_x, min_amount_y] =
            self.get_amounts_to_execute(order_id, order.clone(), liquidity_info);

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 90_u64)
            .remove_liquidity(order.lpt_id.clone(), amount, min_amount_x, min_amount_y)
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 170_u64)
                    .with_unused_gas_weight(2_u64)
                    .remove_liquidity_for_execute_order_callback(order, order_id),
            )
            .into()
    }

    #[private]
    pub fn remove_liquidity_for_execute_order_callback(&mut self, order: Order, order_id: U128) {
        let return_amounts = self.get_return_amounts_after_remove_liquidity(order.clone());
        let account_id = self.get_account_by(order_id.0).unwrap();

        match order.order_type {
            OrderType::Buy | OrderType::Sell => {
                let (token, amount_increase_balance) = if order.order_type == OrderType::Buy {
                    (
                        order.buy_token.clone(),
                        U128::from(
                            BigDecimal::from(U128(order.amount)) / order.open_or_close_price,
                        ),
                    )
                } else {
                    (
                        order.sell_token.clone(),
                        U128::from(
                            BigDecimal::from(U128(order.amount)) * order.open_or_close_price,
                        ),
                    )
                };

                self.mark_order_as_executed(order.clone(), order_id);

                self.remove_pending_order_data(PendingOrderData {
                    order_id,
                    order_type: order.order_type,
                });

                self.increase_balance(&account_id, &token, amount_increase_balance.0);
                self.withdraw(token, amount_increase_balance, Some(true));
            }
            OrderType::Long | OrderType::Short => {
                self.mark_order_as_executed(order.clone(), order_id);

                self.remove_pending_order_data(PendingOrderData {
                    order_id,
                    order_type: order.order_type.clone(),
                });

                if let Some(tpo) = self.take_profit_orders.get(&(order_id.0 as u64)) {
                    self.set_take_profit_order_pending(order_id, order, tpo, true);
                } else {
                    let executor_reward_in_near = env::used_gas().0 as Balance * 2_u128;
                    Promise::new(env::signer_account_id()).transfer(executor_reward_in_near);
                }
            }
            OrderType::TakeProfit => {
                let parent_order = self.get_order_by(order_id.0).unwrap();
                self.mark_take_profit_order_as_executed(order_id, return_amounts);
                self.close_leverage_position(order_id, parent_order, None, None, true);
            }
        }
    }

    pub fn manual_swap(
        &self,
        pool_id: String,
        sell_token: AccountId,
        buy_token: AccountId,
        buy_token_amount: U128,
    ) {
        let action = Action::SwapAction {
            Swap: Swap {
                pool_ids: vec![pool_id],
                output_token: sell_token,
                min_output_amount: WBalance::from(0),
            },
        };

        log!(
            "Action {}",
            near_sdk::serde_json::to_string(&action).unwrap()
        );

        ext_token::ext(buy_token)
            .with_attached_deposit(1)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                buy_token_amount,
                Some("Swap".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            );
    }
}

impl Contract {
    pub fn get_amounts_to_execute(
        &self,
        order_id: U128,
        order: Order,
        liquidity_info: LiquidityInfo,
    ) -> [U128; 3_usize] {
        match order.order_type {
            OrderType::Long => {
                let min_amount_x = U128::from(0);
                let min_amount_y = U128::from(
                    BigDecimal::from(U128::from(order.amount))
                        * order.leverage
                        * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE))
                        / order.open_or_close_price,
                );

                let (_, buy_token_decimals) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                let min_amount_y =
                    self.from_protocol_to_token_decimals(min_amount_y, buy_token_decimals);

                [liquidity_info.amount, min_amount_x, min_amount_y]
            }
            OrderType::Short => {
                let min_amount_x = U128::from(
                    BigDecimal::from(U128::from(order.amount))
                        * (order.leverage - BigDecimal::one())
                        * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
                );
                let min_amount_y = U128::from(0);

                let (sell_token_decimals, _) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                let min_amount_x =
                    self.from_protocol_to_token_decimals(min_amount_x, sell_token_decimals);

                [liquidity_info.amount, min_amount_x, min_amount_y]
            }
            OrderType::Buy => {
                let min_amount_x = U128::from(0);
                let min_amount_y = U128::from(
                    BigDecimal::from(U128::from(order.amount)) / order.open_or_close_price
                        * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
                );

                let (_, buy_token_decimals) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                let min_amount_y =
                    self.from_protocol_to_token_decimals(min_amount_y, buy_token_decimals);

                [liquidity_info.amount, min_amount_x, min_amount_y]
            }

            OrderType::Sell => {
                let min_amount_x = U128::from(
                    BigDecimal::from(U128::from(order.amount))
                        * order.open_or_close_price
                        * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
                );
                let min_amount_y = U128::from(0);

                let (sell_token_decimals, _) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                let min_amount_x =
                    self.from_protocol_to_token_decimals(min_amount_x, sell_token_decimals);

                [liquidity_info.amount, min_amount_x, min_amount_y]
            }
            OrderType::TakeProfit => {
                let parent_order = self.get_order_by(order_id.0).unwrap();
                if parent_order.order_type == OrderType::Long {
                    let min_amount_x = U128::from(
                        BigDecimal::from(U128::from(order.amount))
                            * order.open_or_close_price
                            * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
                    );

                    let min_amount_y = U128::from(0);

                    let (sell_token_decimals, _) =
                        self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                    let min_amount_x =
                        self.from_protocol_to_token_decimals(min_amount_x, sell_token_decimals);

                    [liquidity_info.amount, min_amount_x, min_amount_y]
                } else {
                    let min_amount_x = U128::from(0);
                    let min_amount_y = U128::from(
                        BigDecimal::from(U128::from(order.amount)) / order.open_or_close_price
                            * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
                    );

                    let (_, buy_token_decimals) =
                        self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                    let min_amount_y =
                        self.from_protocol_to_token_decimals(min_amount_y, buy_token_decimals);

                    [liquidity_info.amount, min_amount_x, min_amount_y]
                }
            }
        }
    }

    pub fn mark_order_as_executed(&mut self, order: Order, order_id: U128) {
        let mut order = order;
        order.status = OrderStatus::Executed;

        self.add_or_update_order(
            &self.get_account_by(order_id.0).unwrap(), // assert there is always some user
            order.clone(),
            order_id.0 as u64,
        );

        Event::ExecuteOrderEvent {
            order_id,
            order_type: order.order_type,
        }
        .emit();
    }

    pub fn mark_take_profit_order_as_executed(
        &mut self,
        order_id: U128,
        return_amounts: ReturnAmounts,
    ) {
        let tpo = self.take_profit_orders.get(&(order_id.0 as u64)).unwrap();
        let mut order = tpo.1;
        order.status = OrderStatus::Executed;
        self.take_profit_orders.insert(
            &(order_id.0 as u64),
            &(tpo.0, order.clone(), return_amounts),
        );

        Event::ExecuteOrderEvent {
            order_id,
            order_type: order.order_type,
        }
        .emit();
    }

    pub fn get_account_by(&self, order_id: u128) -> Option<AccountId> {
        let mut account: Option<AccountId> = None;

        for (account_id, users_order) in self.orders.iter() {
            if users_order.contains_key(&(order_id as u64)) {
                account = Some(account_id);
                break;
            }
        }
        account
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("margin.nearland.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
            .block_index(103930920)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_get_account_by() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let order = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1000000000000000000000000\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4220000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930916, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#543\",\"history_data\":null}".to_string();
        contract.add_order_from_string(alice(), order);

        let account_id = contract.orders.get(&alice()).unwrap().contains_key(&1);

        assert!(account_id);
        assert_eq!(contract.get_account_by(1), Some(alice()));
    }

    #[test]
    fn test_order_was_execute() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_id = PairId {
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        };

        let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1000000000000000000000000\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4220000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930916,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#543\",\"history_data\":null}".to_string();
        contract.add_order_from_string(alice(), order_as_string.clone());

        let order_id = U128(1);
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();
        contract.mark_order_as_executed(order, order_id);

        let orders = contract.orders.get(&alice()).unwrap();
        let order = orders.get(&1).unwrap();

        let orders_from_pair = contract.orders_per_pair_view.get(&pair_id).unwrap();
        let order_from_pair = orders_from_pair.get(&1).unwrap();

        assert_eq!(order.status, OrderStatus::Executed);
        assert_eq!(order_from_pair.status, order.status);
    }

    #[test]
    fn test_get_amounts_to_remove_liquidity_for_long() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
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

        let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":2500000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.3\",\"block\":1, \"timestamp_ms\":86400050,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#100\",\"history_data\":null}".to_string();
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        let liquidity_info = LiquidityInfo {
            lpt_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#100".to_string(),
            owner_id: "owner_id.testnet".parse().unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            left_point: -7040,
            right_point: -7000,
            amount: U128(35 * 10_u128.pow(24)),
            unclaimed_fee_x: U128(0),
            unclaimed_fee_y: U128(3 * 10_u128.pow(24)),
        };

        let expect_amount = U128::from(
            BigDecimal::from(U128::from(order.amount)) * order.leverage / order.open_or_close_price,
        );
        let expect_amount_with_inaccuracy_rate = U128::from(
            BigDecimal::from(expect_amount)
                * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
        );

        let order_id = U128(1);
        let result = contract.get_amounts_to_execute(order_id, order, liquidity_info);

        assert_eq!(
            [
                U128(35 * 10_u128.pow(24)),
                U128(0),
                expect_amount_with_inaccuracy_rate
            ],
            result
        );
    }

    #[test]
    fn test_get_amounts_to_remove_liquidity_for_short() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
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

        let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Short\",\"amount\":2500000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.3\",\"block\":1, \"timestamp_ms\":86400050,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#100\",\"history_data\":null}".to_string();
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        let liquidity_info = LiquidityInfo {
            lpt_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#100".to_string(),
            owner_id: "owner_id.testnet".parse().unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            left_point: -7040,
            right_point: -7000,
            amount: U128(35 * 10_u128.pow(24)),
            unclaimed_fee_x: U128(3 * 10_u128.pow(24)),
            unclaimed_fee_y: U128(0),
        };

        let expect_amount = U128::from(
            BigDecimal::from(U128::from(order.amount)) * (order.leverage - BigDecimal::one()),
        );
        let expect_amount_with_inaccuracy_rate = U128::from(
            BigDecimal::from(expect_amount)
                * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
        );

        let order_id = U128(1);
        let result = contract.get_amounts_to_execute(order_id, order, liquidity_info);

        assert_eq!(
            [
                U128(35 * 10_u128.pow(24)),
                expect_amount_with_inaccuracy_rate,
                U128(0)
            ],
            result
        );
    }
}
