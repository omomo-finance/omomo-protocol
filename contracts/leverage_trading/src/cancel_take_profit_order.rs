use crate::common::Event;
use crate::ref_finance::ext_ref_finance;
use crate::ref_finance::ShortLiquidityInfo;
use crate::utils::NO_DEPOSIT;
use crate::*;
use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, Gas, PromiseResult};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn get_take_profit_liquidity_info_callback(
        &self,
        order_id: U128,
        parent_order: Order,
        take_profit_info: (PricePoints, Order),
    );
    fn remove_liquidity_from_take_profit_callback(
        &self,
        order_id: U128,
        parent_order: Order,
        take_profit_info: (PricePoints, Order),
    );
}

#[near_bindgen]
impl Contract {
    pub fn cancel_take_profit_order(&mut self, order_id: U128, parent_order: Order) {
        let take_profit_order = self.take_profit_orders.get(&(order_id.0 as u64));
        require!(take_profit_order.is_some(), "Take profit order not found.");

        let take_profit_order_pair = take_profit_order.unwrap();
        let tpo = take_profit_order_pair.1.clone();
        match tpo.status {
            OrderStatus::PendingOrderExecute => {
                self.take_profit_orders.remove(&(order_id.0 as u64));

                Event::CancelTakeProfitOrderEvent {
                    order_id,
                    order_type: OrderType::TakeProfit,
                    lpt_id: tpo.lpt_id,
                    close_price: WRatio::from(tpo.open_or_close_price),
                    pool_id: self
                        .view_pair(&parent_order.sell_token, &parent_order.buy_token)
                        .pool_id,
                }
                .emit();
            }
            OrderStatus::Pending => {
                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_unused_gas_weight(2)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_liquidity(take_profit_order_pair.1.lpt_id.clone())
                    .then(
                        ext_self::ext(current_account_id())
                            .with_unused_gas_weight(98)
                            .with_attached_deposit(NO_DEPOSIT)
                            .get_take_profit_liquidity_info_callback(
                                order_id,
                                parent_order,
                                take_profit_order_pair,
                            ),
                    );
            }
            _ => {}
        }
    }

    #[private]
    pub fn get_take_profit_liquidity_info_callback(
        &self,
        order_id: U128,
        parent_order: Order,
        take_profit_info: (PricePoints, Order),
    ) {
        require!(is_promise_success(), "Some problem with getting liquidity.");

        let liquidity_info: ShortLiquidityInfo = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(liquidity) = near_sdk::serde_json::from_slice::<ShortLiquidityInfo>(&val)
                {
                    liquidity
                } else {
                    panic!("Some problem with liquidity parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found liquidity."),
        };

        let (min_amount_x, min_amount_y) = if parent_order.order_type == OrderType::Long {
            (U128::from(0), U128::from(liquidity_info.amount.0 - 1000))
        } else {
            (U128::from(liquidity_info.amount.0 - 1000), U128::from(0))
        };

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 70)
            .with_attached_deposit(NO_DEPOSIT)
            .remove_liquidity(
                take_profit_info.1.lpt_id.clone(),
                liquidity_info.amount,
                min_amount_x,
                min_amount_y,
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .remove_liquidity_from_take_profit_callback(
                        order_id,
                        parent_order,
                        take_profit_info,
                    ),
            );
    }

    #[private]
    pub fn remove_liquidity_from_take_profit_callback(
        &mut self,
        order_id: U128,
        parent_order: Order,
        take_profit_info: (PricePoints, Order),
    ) {
        require!(
            is_promise_success(),
            "Some problem with removing liquidity."
        );

        let mut order = take_profit_info.1;

        if parent_order.order_type == OrderType::Long {
            self.increase_balance(&signer_account_id(), &parent_order.buy_token, order.amount);
        } else {
            self.increase_balance(&signer_account_id(), &parent_order.sell_token, order.amount);
        }

        order.status = OrderStatus::Canceled;
        self.take_profit_orders
            .insert(&(order_id.0 as u64), &(take_profit_info.0, order.clone()));

        Event::CancelTakeProfitOrderEvent {
            order_id,
            order_type: OrderType::TakeProfit,
            lpt_id: order.lpt_id,
            close_price: WRatio::from(order.open_or_close_price),
            pool_id: self.view_pair(&order.sell_token, &order.buy_token).pool_id,
        }
        .emit();
    }
}
