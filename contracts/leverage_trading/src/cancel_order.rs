use crate::big_decimal::BigDecimal;
use crate::execute_order::INACCURACY_RATE;
use crate::ref_finance::ext_ref_finance;
use crate::ref_finance::{Action, Swap};
use crate::utils::NO_DEPOSIT;
use crate::utils::{ext_market, ext_token};
use crate::utils::{DAYS_PER_YEAR, MILLISECONDS_PER_DAY};
use crate::*;
use near_sdk::env::{current_account_id, prepaid_gas, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, log, Gas, PromiseResult, ONE_YOCTO};

const CANCEL_ORDER_GAS: Gas = Gas(160_000_000_000_000);
const GAS_FOR_BORROW: Gas = Gas(200_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_callback(&self, order_id: U128, order: Order);
    fn remove_liquidity_for_cancel_leverage_order_callback(
        &mut self,
        order_id: U128,
        order: Order,
        amount_x: U128,
        amount_y: U128,
    );
    fn close_order_swap_callback(
        &self,
        order_id: U128,
        order: Order,
        amount: U128,
        price_impact: U128,
    );
    fn liquidate_order_swap_callback(
        &self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn market_data_callback(
        &self,
        order_id: U128,
        order: Order,
        amount: U128,
        price_impact: Option<U128>,
    );
    fn get_pool_callback(&self, order_id: U128, order: Order);
    fn get_liquidity_callback(&self, order_id: U128, order: Order, pool_info: PoolInfo);
    fn repay_callback(&self, repay_amount: U128) -> PromiseOrValue<U128>;
    fn withdraw_callback(&mut self, account_id: AccountId, token: AccountId, amount: U128);
}

#[near_bindgen]
impl Contract {
    pub fn cancel_order(&mut self, order_id: U128, price_impact: U128) {
        require!(
            prepaid_gas() >= CANCEL_ORDER_GAS,
            "Not enough gas for method: 'Cancel order'"
        );

        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        match order.order_type {
            OrderType::Buy | OrderType::Sell => self.cancel_limit_order(order_id, order),
            OrderType::Long | OrderType::Short => {
                self.cancel_or_close_leverage_order(order_id, order, price_impact)
            }
            OrderType::TP => panic!(
                "Incorrect type of order 'TP'. Expected one of 'Buy', 'Sell', 'Long', 'Short'"
            ),
        }
    }

    #[private]
    pub fn get_pool_callback(&mut self, order_id: U128, order: Order) {
        let pool_info = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<PoolInfo>(&val) {
                    pool
                } else {
                    panic!("Some problem with pool parsing")
                }
            }
            _ => panic!("Some problem with pool on DEX"),
        };

        require!(
            pool_info.state == PoolState::Running,
            "Some problem with pool, please contact with DEX to support"
        );

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(2_u64)
            .with_attached_deposit(NO_DEPOSIT)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(30_u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_liquidity_callback(order_id, order, pool_info),
            );
    }

    #[private]
    pub fn get_liquidity_callback(&mut self, order_id: U128, order: Order, pool_info: PoolInfo) {
        let liquidity_info: Liquidity = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<Liquidity>(&val) {
                    pool
                } else {
                    panic!("Some problem with liquidity parsing.")
                }
            }
            _ => panic!("DEX not found liquidity"),
        };

        if order.order_type == OrderType::Long {
            require!(
                pool_info.current_point < liquidity_info.left_point,
                "You cannot cancel the opening of a position. Liquidity is already used by DEX"
            );
        } else {
            require!(
                pool_info.current_point > liquidity_info.right_point,
                "You cannot cancel the opening of a position. Liquidity is already used by DEX"
            );
        }

        let [amount, min_amount_x, min_amount_y] =
            self.get_amounts_to_cancel(order.clone(), liquidity_info);

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 50_u64)
            .with_attached_deposit(NO_DEPOSIT)
            .remove_liquidity(order.lpt_id.to_string(), amount, min_amount_x, min_amount_y)
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .remove_liquidity_for_cancel_leverage_order_callback(
                        order_id,
                        order,
                        min_amount_x,
                        min_amount_y,
                    ),
            );
    }

    #[private]
    pub fn remove_liquidity_for_cancel_leverage_order_callback(
        &mut self,
        order_id: U128,
        order: Order,
        amount_x: U128,
        amount_y: U128,
    ) {
        require!(is_promise_success(), "Some problem with remove liquidity");

        let (amount, token_market) = if order.order_type == OrderType::Long {
            (amount_x, self.get_market_by(&order.sell_token))
        } else {
            (amount_y, self.get_market_by(&order.buy_token))
        };

        ext_market::ext(token_market)
            .with_attached_deposit(NO_DEPOSIT)
            .view_market_data()
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .market_data_callback(order_id, order, amount, None),
            );
    }

    #[private]
    pub fn close_order_swap_callback(
        &mut self,
        order_id: U128,
        order: Order,
        amount: U128,
        price_impact: U128,
    ) {
        log!(
            "Order cancel swap callback attached gas: {}",
            env::prepaid_gas().0
        );

        require!(is_promise_success(), "Some problem with swap");

        let token_market = if order.order_type == OrderType::Long {
            self.get_market_by(&order.sell_token)
        } else {
            self.get_market_by(&order.buy_token)
        };

        ext_market::ext(token_market)
            .with_attached_deposit(NO_DEPOSIT)
            .view_market_data()
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .market_data_callback(order_id, order, amount, Some(price_impact)),
            );
    }

    #[private]
    pub fn market_data_callback(
        &mut self,
        order_id: U128,
        order: Order,
        amount: U128,
        price_impact: Option<U128>,
    ) {
        log!(
            "Market data callback attached gas: {}",
            env::prepaid_gas().0
        );

        let market_data = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(data) = near_sdk::serde_json::from_slice::<MarketData>(&val) {
                    data
                } else {
                    panic!("Failed parse market data")
                }
            }
            _ => panic!("Failed to get market data"),
        };

        if order.status == OrderStatus::Pending {
            self.final_cancel_order(order_id, order, amount, market_data);
        } else {
            self.final_close_order(order_id, order, amount, price_impact, market_data)
        }
    }

    /// Called by a separate transaction with UI
    pub fn repay(&self, order_id: U128, market_data: MarketData) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        require!(order.status == OrderStatus::Canceled, "Error. The order must be in the status 'Cancel'");

        let (token_borrow, token_market) = if order.order_type == OrderType::Long {
            (
                order.sell_token.clone(),
                self.get_market_by(&order.sell_token),
            )
        } else {
            (
                order.buy_token.clone(),
                self.get_market_by(&order.buy_token),
            )
        };

        let borrow_fee_amount = self.get_borrow_fee_amount(order, market_data);

        ext_token::ext(token_borrow)
            .with_static_gas(GAS_FOR_BORROW)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer_call(
                token_market,
                borrow_fee_amount,
                None,
                "\"Repay\"".to_string(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 3_u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .repay_callback(borrow_fee_amount),
            );
    }

    #[private]
    pub fn repay_callback(&self, repay_amount: U128) -> PromiseOrValue<U128> {
        require!(is_promise_success(), "Failed to repay assets");
        //TODO: add repay success event
        PromiseOrValue::Value(repay_amount)
    }
}

impl Contract {
    pub fn cancel_or_close_leverage_order(&self, order_id: U128, order: Order, price_impact: U128) {
        match order.status {
            OrderStatus::Pending => self.cancel_leverage_order(order_id, order),
            OrderStatus::Executed => self.close_leverage_order(order_id, order, price_impact),
            _ => panic!("Error. Order status has to be 'Pending' or 'Executed'"),
        }
    }

    pub fn cancel_leverage_order(&self, order_id: U128, order: Order) {
        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(1_u64)
            .with_attached_deposit(NO_DEPOSIT)
            .get_pool(self.view_pair(&order.sell_token, &order.buy_token).pool_id)
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(30_u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_pool_callback(order_id, order),
            );
    }

    pub fn close_leverage_order(&self, order_id: U128, order: Order, price_impact: U128) {
        let (amount, input_token, output_token) = self.get_data_to_swap(order.clone());

        let action = Action::SwapAction {
            Swap: Swap {
                pool_ids: vec![self.view_pair(&order.sell_token, &order.buy_token).pool_id],
                output_token,
                min_output_amount: WBalance::from(0),
            },
        };

        log!(
            "action {}",
            near_sdk::serde_json::to_string(&action).unwrap()
        );

        ext_token::ext(input_token)
            .with_attached_deposit(1_u128)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                amount,
                Some("Swap".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .close_order_swap_callback(order_id, order, amount, price_impact),
            );
    }

    pub fn get_amounts_to_cancel(
        &self,
        order: Order,
        liquidity_info: Liquidity,
    ) -> [U128; 3_usize] {
        if order.order_type == OrderType::Long {
            let min_amount_x = U128::from(
                BigDecimal::from(U128::from(order.amount))
                    * order.leverage
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
                BigDecimal::from(U128::from(order.amount))
                    * (order.leverage - BigDecimal::one())
                    * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE))
                    / order.open_or_close_price,
            );

            let (_, buy_token_decimals) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
            let min_amount_y =
                self.from_protocol_to_token_decimals(min_amount_y, buy_token_decimals);

            [liquidity_info.amount, min_amount_x, min_amount_y]
        }
    }

    pub fn get_data_to_swap(&self, order: Order) -> (U128, AccountId, AccountId) {
        if order.order_type == OrderType::Long {
            let amount = U128::from(
                BigDecimal::from(U128::from(order.amount)) * order.leverage
                    / order.open_or_close_price,
            );

            let (_, buy_token_decimals) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
            let swap_amount = self.from_protocol_to_token_decimals(amount, buy_token_decimals);
            // amount, input_token, output_token
            (swap_amount, order.buy_token, order.sell_token)
        } else {
            let amount = U128::from(
                BigDecimal::from(U128::from(order.amount)) * (order.leverage - BigDecimal::one()),
            );

            let (sell_token_decimals, _) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
            let swap_amount = self.from_protocol_to_token_decimals(amount, sell_token_decimals);
            // amount, input_token, output_token
            (swap_amount, order.sell_token, order.buy_token)
        }
    }

    pub fn final_cancel_order(
        &mut self,
        order_id: U128,
        order: Order,
        amount: U128,
        market_data: MarketData,
    ) {
        log!("Final order cancel attached gas: {}", env::prepaid_gas().0);

        let return_amount =
            BigDecimal::from(amount) / (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE));

        let borrow_fee_amount =
            BigDecimal::from(self.get_borrow_fee_amount(order.clone(), market_data));

        let total_amount = if order.order_type == OrderType::Long {
            U128::from(return_amount - borrow_fee_amount)
        } else {
            todo!()
        };

        self.increase_balance(&signer_account_id(), &order.sell_token, total_amount.0);

        let order = Order {
            status: OrderStatus::Canceled,
            ..order
        };

        self.add_or_update_order(&signer_account_id(), order, order_id.0 as u64);
    }

    pub fn final_close_order(
        &mut self,
        order_id: U128,
        order: Order,
        amount: U128,
        price_impact: Option<U128>,
        market_data: MarketData,
    ) {
        let total_amount = if order.order_type == OrderType::Long {
            let open_amount = BigDecimal::from(U128::from(order.amount)) * order.leverage;

            let sell_token_prise = self.view_price(order.sell_token.clone()).value;
            let buy_token_prise = self.view_price(order.buy_token.clone()).value;

            let expect_amount_after_swap = BigDecimal::from(amount)
                * BigDecimal::from(buy_token_prise)
                / BigDecimal::from(sell_token_prise)
                * (BigDecimal::one()
                    - BigDecimal::from(price_impact.unwrap_or_else(|| U128(10_u128.pow(22)))));

            let borrow_fee_amount =
                BigDecimal::from(self.get_borrow_fee_amount(order.clone(), market_data));

            let swap_fee = BigDecimal::from(self.get_swap_fee(&order));

            let swap_fee_amount = expect_amount_after_swap * swap_fee;

            let mut total_amount = expect_amount_after_swap - borrow_fee_amount - swap_fee_amount;

            if open_amount < total_amount {
                let protocol_fee = BigDecimal::from(self.get_protocol_fee());
                let protocol_profit_amount = total_amount * protocol_fee;

                let current_profit = self
                    .protocol_profit
                    .get(&order.sell_token)
                    .unwrap_or_default();

                self.protocol_profit.insert(
                    &order.sell_token,
                    &(current_profit + protocol_profit_amount),
                );

                total_amount = total_amount - protocol_profit_amount;
            }

            U128::from(total_amount)
        } else {
            todo!()
        };

        self.increase_balance(&signer_account_id(), &order.sell_token, total_amount.0);

        let order = Order {
            status: OrderStatus::Canceled,
            ..order
        };

        self.add_or_update_order(&signer_account_id(), order, order_id.0 as u64);
    }

    fn get_borrow_fee_amount(&self, order: Order, market_data: MarketData) -> U128 {
        let current_timestamp_ms = env::block_timestamp_ms();

        let borrow_period = ((current_timestamp_ms - order.timestamp_ms) as f64
            / MILLISECONDS_PER_DAY as f64)
            .ceil();

        let borrow_amount = if order.order_type == OrderType::Long {
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one())
        } else {
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one())
                / order.open_or_close_price
        };

        let borrow_fee_amount = borrow_amount * BigDecimal::from(market_data.borrow_rate_ratio)
            / BigDecimal::from(U128(DAYS_PER_YEAR as u128))
            * BigDecimal::from(U128(borrow_period as u128));

        U128::from(borrow_fee_amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::MILLISECONDS_PER_DAY;

    use super::*;

    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool, block_timestamp: Option<u64>) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("leverage.develop.v1.omomo-finance.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("alice.testnet".parse().unwrap())
            .block_index(103930920)
            .block_timestamp(block_timestamp.unwrap_or(1))
            .is_view(is_view)
            .build()
    }

    fn get_current_day_in_nanoseconds(day: u64) -> Option<u64> {
        let nanoseconds_in_one_millisecond = 1_000_000;
        Some(MILLISECONDS_PER_DAY * day * nanoseconds_in_one_millisecond)
    }

    #[test]
    fn test_order_was_canceled() {
        let current_day = get_current_day_in_nanoseconds(2); // borrow period 1 days
        let context = get_context(false, current_day);
        testing_env!(context);

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
            swap_fee: U128(10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(2000000000000000000000000),
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128::from(4220000000000000000000000),
            },
        );

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":10000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3070000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930916,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#543\"}".to_string();
        contract.add_order_from_string(alice(), order1);

        let order_id = U128(1);
        let order = Order {
            status: OrderStatus::Pending,
            order_type: OrderType::Long,
            amount: 10_u128.pow(25),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            buy_token: "wrap.testnet".parse().unwrap(),
            leverage: BigDecimal::from(U128(2 * 10_u128.pow(24))),
            sell_token_price: Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(101 * 10_u128.pow(22)),
            },
            buy_token_price: Price {
                ticker_id: "near".to_string(),
                value: U128::from(307 * 10_u128.pow(22)),
            },
            open_or_close_price: BigDecimal::from(U128(1)),
            block: 105210654,
            timestamp_ms: 86400000,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#238".to_string(),
        };

        let market_data = MarketData {
            underlying_token: AccountId::new_unchecked("usdt.fakes.testnet".to_string()),
            underlying_token_decimals: 24,
            total_supplies: U128(60000000000000000000000000000),
            total_borrows: U128(25010000000000000000000000000),
            total_reserves: U128(1000176731435219096024128768),
            exchange_rate_ratio: U128(1000277139994639276176632),
            interest_rate_ratio: U128(261670051778601),
            borrow_rate_ratio: U128(5 * 10_u128.pow(24)),
        };

        let pair_id = (
            "usdt.fakes.testnet".parse().unwrap(),
            "wrap.testnet".parse().unwrap(),
        );

        let amount = U128::from(
            BigDecimal::from(U128(2 * 10_u128.pow(25)))
                * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
        );

        contract.final_cancel_order(order_id, order, amount, market_data);

        let orders = contract.orders.get(&alice()).unwrap();
        let order = orders.get(&1).unwrap();

        let orders_from_pair = contract.orders_per_pair_view.get(&pair_id).unwrap();
        let order_from_pair = orders_from_pair.get(&1).unwrap();

        assert_eq!(order.status, OrderStatus::Canceled);
        assert_eq!(order_from_pair.status, order.status);
    }
}
