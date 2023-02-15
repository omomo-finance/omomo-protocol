use crate::big_decimal::BigDecimal;
use crate::ref_finance::ext_ref_finance;
use crate::ref_finance::{Action, Swap};
use crate::utils::NO_DEPOSIT;
use crate::utils::{ext_market, ext_token};
use crate::*;
use near_sdk::env::{block_height, current_account_id, prepaid_gas, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, log, Gas, PromiseResult, ONE_YOCTO};

const CANCEL_ORDER_GAS: Gas = Gas(160_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_callback(
        &self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn order_cancel_swap_callback(
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
        price_impact: U128,
        order_action: OrderAction,
    );
    fn get_pool_callback(
        &self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn get_liquidity_callback(
        &self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
        pool_info: PoolInfo,
    );
    fn repay_callback(&self) -> PromiseOrValue<U128>;
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

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(1)
            .with_attached_deposit(NO_DEPOSIT)
            .get_pool(self.view_pair(&order.sell_token, &order.buy_token).pool_id)
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(29)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_pool_callback(order_id, order, price_impact, OrderAction::Cancel),
            );
    }

    #[private]
    pub fn get_pool_callback(
        &mut self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        require!(
            is_promise_success(),
            "Some problem with pool on ref finance"
        );
        let pool_info = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<PoolInfo>(&val) {
                    pool
                } else {
                    panic!("Some problem with pool parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found pool"),
        };

        require!(
            pool_info.state == PoolState::Running,
            "Some problem with pool, please contact with ref finance to support."
        );

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(2)
            .with_attached_deposit(NO_DEPOSIT)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(98)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_liquidity_callback(order_id, order, price_impact, order_action, pool_info),
            );
    }

    #[private]
    pub fn get_liquidity_callback(
        &mut self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
        pool_info: PoolInfo,
    ) {
        require!(
            is_promise_success(),
            "Some problem with liquidity on ref finance"
        );
        let liquidity: Liquidity = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<Liquidity>(&val) {
                    pool
                } else {
                    panic!("Some problem with liquidity parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found liquidity"),
        };

        let remove_liquidity_amount = liquidity.amount.0;

        let (min_amount_x, min_amount_y) = match order.order_type {
            OrderType::Buy => (liquidity.amount.0 - 1000, 0),
            OrderType::Sell => (0, liquidity.amount.0 - 1000),
            _ => todo!(
                "Currently, the functionality is developed only for 'Buy' and 'Sell' order types"
            ),
        };

        require!(
            pool_info.total_x.0 > remove_liquidity_amount,
            "Pool not have enough liquidity"
        );

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 70)
            .with_attached_deposit(NO_DEPOSIT)
            .remove_liquidity(
                order.lpt_id.to_string(),
                U128(remove_liquidity_amount),
                U128(min_amount_x),
                U128(min_amount_y),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .remove_liquidity_callback(order_id, order, price_impact, order_action),
            );
    }

    #[private]
    pub fn remove_liquidity_callback(
        &mut self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        require!(is_promise_success(), "Some problem with remove liquidity");
        self.order_cancel_swap_callback(order_id, order, price_impact, order_action);
    }

    pub fn swap(
        &self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        let buy_amount = BigDecimal::from(U128::from(order.amount))
            * order.leverage
            * BigDecimal::from(order.sell_token_price.value)
            * self.get_price(order.buy_token.clone())
            / BigDecimal::from(order.buy_token_price.value);

        let (_, buy_token_decimals) =
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
        let buy_amount =
            self.from_protocol_to_token_decimals(U128::from(buy_amount), buy_token_decimals);

        let action = Action::SwapAction {
            Swap: Swap {
                pool_ids: vec![self.view_pair(&order.sell_token, &order.buy_token).pool_id],
                output_token: order.sell_token.clone(),
                min_output_amount: WBalance::from(0),
            },
        };

        log!(
            "action {}",
            near_sdk::serde_json::to_string(&action).unwrap()
        );

        ext_token::ext(order.buy_token.clone())
            .with_attached_deposit(1)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                buy_amount,
                Some("Swap".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .order_cancel_swap_callback(order_id, order, price_impact, order_action),
            );
    }

    #[private]
    pub fn order_cancel_swap_callback(
        &mut self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        log!(
            "Order cancel swap callback attached gas: {}",
            env::prepaid_gas().0
        );

        if order.leverage > BigDecimal::one() {
            let market_id = self.tokens_markets.get(&order.sell_token).unwrap();
            ext_market::ext(market_id)
                .with_attached_deposit(NO_DEPOSIT)
                .view_market_data()
                .then(
                    ext_self::ext(current_account_id())
                        .with_attached_deposit(NO_DEPOSIT)
                        .market_data_callback(order_id, order, price_impact, order_action),
                );
        } else {
            #[allow(clippy::collapsible_else_if)]
            if order_action == OrderAction::Cancel {
                self.final_order_cancel(order_id, order, price_impact, None);
            } else {
                self.final_liquidate(order_id, order, None);
            }
        }
    }

    #[private]
    pub fn market_data_callback(
        &mut self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        log!(
            "Market data callback attached gas: {}",
            env::prepaid_gas().0
        );
        require!(is_promise_success(), "failed to get market data.");
        let market_data = match env::promise_result(0) {
            PromiseResult::NotReady => panic!("failed to get market data"),
            PromiseResult::Successful(val) => {
                if let Ok(data) = near_sdk::serde_json::from_slice::<MarketData>(&val) {
                    data
                } else {
                    panic!("failed parse market data")
                }
            }
            PromiseResult::Failed => panic!("failed to get market data"),
        };

        if order_action == OrderAction::Cancel {
            self.final_order_cancel(order_id, order, price_impact, Some(market_data));
        } else {
            self.final_liquidate(order_id, order, Some(market_data));
        }
    }

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
        let market_id = self.tokens_markets.get(&order.sell_token).unwrap();
        let borrow_fee = BigDecimal::from(market_data.borrow_rate_ratio.0)
            * BigDecimal::from((block_height() - order.block) as u128);

        ext_token::ext(order.sell_token)
            .with_static_gas(Gas::ONE_TERA * 35u64)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer_call(
                market_id,
                U128(borrow_fee.round_u128()),
                None,
                "\"Repay\"".to_string(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 3u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .repay_callback(),
            );
    }

    #[private]
    pub fn repay_callback(&self) -> PromiseOrValue<U128> {
        require!(is_promise_success(), "failed to repay assets");
        //TODO: add repay success event
        PromiseOrValue::Value(U128(0))
    }
}

impl Contract {
    pub fn final_order_cancel(
        &mut self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        market_data: Option<MarketData>,
    ) {
        log!("Final order cancel attached gas: {}", env::prepaid_gas().0);

        let mut order = order;
        let sell_amount = BigDecimal::from(order.sell_token_price.value)
            * BigDecimal::from(U128::from(order.amount))
            * order.leverage;

        let pnl = self.calculate_pnl(signer_account_id(), order_id, market_data);

        let swap_fee = self.get_swap_fee(&order);

        let expect_amount = self.get_price(order.buy_token.clone())
            * sell_amount
            * (BigDecimal::one() - BigDecimal::from(swap_fee))
            * (BigDecimal::one() - BigDecimal::from(price_impact))
            / BigDecimal::from(order.buy_token_price.value);

        self.increase_balance(&signer_account_id(), &order.sell_token, order.amount);

        if pnl.is_profit && expect_amount > sell_amount + BigDecimal::from(pnl.amount) {
            let protocol_profit = expect_amount - sell_amount - BigDecimal::from(pnl.amount);

            let token_profit = self
                .protocol_profit
                .get(&order.sell_token)
                .unwrap_or_default();
            self.protocol_profit
                .insert(&order.sell_token, &(token_profit + protocol_profit));
        }
        order.status = OrderStatus::Canceled;

        self.add_or_update_order(&signer_account_id(), order, order_id.0 as u64);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    use crate::pnl::MILLISECONDS_PER_DAY;

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

    #[test]
    fn test_order_was_canceled() {
        let borrow_period = Some(MILLISECONDS_PER_DAY * 91 * 1_000_000); //90 days
        let context = get_context(false, borrow_period);
        testing_env!(context);

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
            swap_fee: U128(10u128.pow(20)),
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

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":100000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3070000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930916,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#543\"}".to_string();
        contract.add_order_from_string(alice(), order1);

        let order_id = U128(1);
        let order = Order {
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: 100000000000000000000000000,
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
            timestamp_ms: 1675423354862,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#238".to_string(),
        };

        let market_data = MarketData {
            underlying_token: AccountId::new_unchecked("usdt.fakes.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(60000000000000000000000000000),
            total_borrows: U128(25010000000000000000000000000),
            total_reserves: U128(1000176731435219096024128768),
            exchange_rate_ratio: U128(1000277139994639276176632),
            interest_rate_ratio: U128(261670051778601),
            borrow_rate_ratio: U128(634273735391536),
        };

        let pair_id = (
            "usdt.fakes.testnet".parse().unwrap(),
            "wrap.testnet".parse().unwrap(),
        );

        let price_impact = U128(1);
        contract.final_order_cancel(order_id, order, price_impact, Some(market_data));

        let orders = contract.orders.get(&alice()).unwrap();
        let order = orders.get(&1).unwrap();

        let orders_from_pair = contract.orders_per_pair_view.get(&pair_id).unwrap();
        let order_from_pair = orders_from_pair.get(&1).unwrap();

        assert_eq!(order.status, OrderStatus::Canceled);
        assert_eq!(order_from_pair.status, order.status);
    }
}
