use crate::big_decimal::BigDecimal;
use crate::cancel_order::ext_self;
use crate::ref_finance::ext_ref_finance;
use crate::utils::NO_DEPOSIT;
use crate::*;
use near_sdk::env::{block_height, current_account_id, signer_account_id};
use near_sdk::Gas;

#[near_bindgen]
impl Contract {
    pub fn liquidate_order(
        //+-
        &mut self,
        order_id: U128,
        price_impact: U128,
    ) {
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

        let (sell_token_decimals, _) =
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
        let order_amount = self.convert_token_amount_to_10_24(order.amount, sell_token_decimals);

        //TODO: set real min_amount_x/min_amount_y
        let amount = 1;
        let min_amount_x = order_amount.0; // Тут order_amount это количество Sell или Buy токена?
        let min_amount_y = 0;

        if order.status == OrderStatus::Pending {
            ext_ref_finance::ext(self.ref_finance_account.clone())
                .with_static_gas(Gas(10))
                .with_attached_deposit(1)
                .remove_liquidity(
                    order.lpt_id.clone(),
                    U128(amount),
                    U128(min_amount_x),
                    U128(min_amount_y),
                )
                .then(
                    ext_self::ext(current_account_id())
                        .with_static_gas(Gas(5))
                        .with_attached_deposit(NO_DEPOSIT)
                        .remove_liquidity_callback(
                            order_id,
                            order,
                            price_impact,
                            OrderAction::Liquidate,
                        ),
                );
        } else {
            self.swap(order_id, order, price_impact, OrderAction::Liquidate);
        }
    }

    #[private]
    pub fn final_liquidate(
        //+-
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

        let (sell_token_decimals, _) =
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
        let order_amount = self.convert_token_amount_to_10_24(order.amount, sell_token_decimals);

        // В формулах ниже order_amount это количество Sell или Buy токена?
        let buy_token_amount =
            BigDecimal::from(order_amount.0) * order.sell_token_price.value * order.leverage
                / order.buy_token_price.value;
        let loss = borrow_fee + buy_token_amount * order.buy_token_price.value
            - BigDecimal::from(order_amount.0);

        let is_liquidation_possible = loss
            >= BigDecimal::from(order_amount.0)
                * order.buy_token_price.value
                * BigDecimal::from(10_u128.pow(24) - self.liquidation_threshold);

        require!(is_liquidation_possible, "This order can't be liquidated");

        let liquidation_incentive = order_amount.0 * self.liquidation_threshold;
        self.increase_balance(
            &env::signer_account_id(),
            &order.buy_token,
            liquidation_incentive,
        );
        let account = self.get_account_by(order_id.0).unwrap();
        let mut orders = self.orders.get(&account).unwrap();
        let mut order = order;
        order.status = OrderStatus::Liquidated;
        orders.insert(order_id.0 as u64, order);
        self.orders.insert(&signer_account_id(), &orders);
    }
}
