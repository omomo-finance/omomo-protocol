use crate::big_decimal::BigDecimal;
use crate::cancel_order::ext_self;
use crate::ref_finance::{ext_ref_finance, Action, Swap};
use crate::utils::{ext_token, NO_DEPOSIT};
use crate::*;
use near_sdk::env::{block_height, current_account_id};
use near_sdk::{is_promise_success, log, Gas};

#[near_bindgen]
impl Contract {
    pub fn liquidate_order(&mut self, order_id: U128, price_impact: U128) {
        let account_op = self.get_account_by(order_id.0);
        require!(
            account_op.is_some(),
            format!("Not found account for order with id: {}", order_id.0)
        );
        let account = account_op.unwrap();

        let orders = self.orders.get(&account).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", account.clone());
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        require!(
            order.status != OrderStatus::Canceled && order.status != OrderStatus::Executed,
            "Order can't be liquidate."
        );

        //TODO: set real min_amount_x/min_amount_y
        let amount = 1;
        let min_amount_x = order.amount;
        let min_amount_y = 0;

        let (sell_token_decimals, _) =
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
        let min_amount_x =
            self.from_protocol_to_token_decimals(U128::from(min_amount_x), sell_token_decimals);

        if order.status == OrderStatus::Pending {
            ext_ref_finance::ext(self.ref_finance_account.clone())
                .with_static_gas(Gas(10))
                .with_attached_deposit(1)
                .remove_liquidity(
                    order.lpt_id.clone(),
                    U128(amount),
                    min_amount_x,
                    U128(min_amount_y),
                )
                .then(
                    ext_self::ext(current_account_id())
                        .with_static_gas(Gas(5))
                        .with_attached_deposit(NO_DEPOSIT)
                        .remove_liquidity_callback(order_id, order),
                );
        } else {
            self.swap(order_id, order, price_impact, OrderAction::Liquidate);
        }
    }

    #[private]
    pub fn remove_liquidity_callback(&mut self, order_id: U128, order: Order) {
        require!(is_promise_success(), "Some problem with remove liquidity");
        self.final_liquidate(order_id, order, None);
    }

    #[private]
    pub fn final_liquidate(
        &mut self,
        order_id: U128,
        order: Order,
        market_data: Option<MarketData>,
    ) {
        #[allow(clippy::unnecessary_unwrap)]
        let borrow_fee = if market_data.is_some() {
            BigDecimal::from(
                market_data.unwrap().borrow_rate_ratio.0 * (block_height() - order.block) as u128,
            )
        } else {
            BigDecimal::one()
        };

        let buy_token_amount = BigDecimal::from(order.amount)
            * BigDecimal::from(order.sell_token_price.value)
            * order.leverage
            / BigDecimal::from(order.buy_token_price.value);
        let loss = borrow_fee + buy_token_amount * BigDecimal::from(order.buy_token_price.value)
            - BigDecimal::from(order.amount);

        let is_liquidation_possible = loss
            >= BigDecimal::from(order.amount)
                * BigDecimal::from(order.buy_token_price.value)
                * BigDecimal::from(10_u128.pow(24) - self.liquidation_threshold);

        require!(is_liquidation_possible, "This order can't be liquidated");

        let liquidation_incentive = order.amount * self.liquidation_threshold;
        self.increase_balance(
            &env::signer_account_id(),
            &order.buy_token,
            liquidation_incentive,
        );

        let mut order = order;
        order.status = OrderStatus::Liquidated;

        self.add_or_update_order(
            &self.get_account_by(order_id.0).unwrap(),
            order,
            order_id.0 as u64,
        );
    }
}

impl Contract {
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
                    .liquidate_order_swap_callback(order_id, order, price_impact, order_action),
            );
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
            .block_index(103931930)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    //there are questions about the method calculations "final_liquidate"
    #[test]
    #[should_panic]
    fn test_order_was_liquidate() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_id: PairId = (
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        );

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDT".to_string(),
                value: U128::from(1000000000000000000000000),
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "WNEAR".to_string(),
                value: U128::from(4220000000000000000000000),
            },
        );

        contract.set_balance(&alice(), &pair_id.0, 10_u128.pow(30));

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4220000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930900,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#543\",\"history_data\":null}".to_string();
        contract.add_order_from_string(alice(), order1);

        let order_id = U128(1);
        let order = Order {
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: 1000000000000000000000000000,
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
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
            block: 103930900,
            timestamp_ms: 1675423354862,
            lpt_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#238".to_string(),
            history_data: Default::default(),
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

        contract.final_liquidate(order_id, order, Some(market_data));

        let orders = contract.orders.get(&alice()).unwrap();
        let order = orders.get(&1).unwrap();

        let orders_from_pair = contract.orders_per_pair_view.get(&pair_id).unwrap();
        let order_from_pair = orders_from_pair.get(&1).unwrap();

        assert_eq!(order.status, OrderStatus::Liquidated);
        assert_eq!(order_from_pair.status, order.status);
    }
}
