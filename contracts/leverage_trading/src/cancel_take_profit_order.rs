use crate::common::Event;

use crate::ref_finance::ext_ref_finance;
use crate::utils::ext_market;
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, Gas, PromiseResult};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn get_take_profit_liquidity_info_callback(
        &self,
        order_id: U128,
        take_profit_info: (PricePoints, Order),
        position_action: Option<OrderAction>,
    );
    fn remove_liquidity_from_take_profit_callback(
        &self,
        order_id: U128,
        position_action: Option<OrderAction>,
    );
    fn market_data_callback(
        &self,
        order_id: U128,
        order: Order,
        amount_x: Option<U128>,
        amount_y: Option<U128>,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    );
}

#[near_bindgen]
impl Contract {
    pub fn cancel_take_profit_order(
        &mut self,
        order_id: U128,
        position_action: Option<OrderAction>,
    ) {
        let take_profit_order = self.take_profit_orders.get(&(order_id.0 as u64));
        require!(take_profit_order.is_some(), "Take profit order not found.");

        let take_profit_order_pair = take_profit_order.unwrap();
        let tpo = take_profit_order_pair.1.clone();
        match tpo.status {
            OrderStatus::PendingOrderExecute => {
                self.take_profit_orders.remove(&(order_id.0 as u64));

                Event::CancelTakeProfitOrderEvent { order_id }.emit();
            }
            OrderStatus::Pending => {
                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_static_gas(Gas::ONE_TERA * 5_u64)
                    .get_liquidity(take_profit_order_pair.1.lpt_id.clone())
                    .then(
                        ext_self::ext(current_account_id())
                            .with_static_gas(Gas::ONE_TERA * 245_u64)
                            .get_take_profit_liquidity_info_callback(
                                order_id,
                                take_profit_order_pair,
                                position_action,
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
        take_profit_info: (PricePoints, Order),
        position_action: Option<OrderAction>,
    ) {
        require!(is_promise_success(), "Some problem with getting liquidity.");

        let liquidity_info: Liquidity = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(liquidity) = near_sdk::serde_json::from_slice::<Liquidity>(&val) {
                    liquidity
                } else {
                    panic!("Some problem with liquidity parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found liquidity."),
        };

        let (min_amount_x, min_amount_y) = (U128::from(0), U128::from(0));

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 90_u64)
            .remove_liquidity(
                take_profit_info.1.lpt_id,
                liquidity_info.amount,
                min_amount_x,
                min_amount_y,
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 150_u64)
                    .remove_liquidity_from_take_profit_callback(order_id, position_action),
            );
    }

    #[private]
    pub fn remove_liquidity_from_take_profit_callback(
        &mut self,
        order_id: U128,
        position_action: Option<OrderAction>,
    ) {
        require!(
            is_promise_success(),
            "Some problem with return amount from Dex"
        );

        self.take_profit_orders.remove(&(order_id.0 as u64));

        Event::CancelTakeProfitOrderEvent { order_id }.emit();

        self.remove_pending_order_data(PendingOrderData {
            order_id,
            order_type: OrderType::TakeProfit,
        });

        if let Some(action) = position_action {
            match action {
                OrderAction::Close {
                    order_id,
                    order,
                    current_buy_token_price,
                    slippage_price_impact,
                } => {
                    let token_market = if order.order_type == OrderType::Long {
                        self.get_market_by(&order.sell_token)
                    } else {
                        self.get_market_by(&order.buy_token)
                    };

                    ext_market::ext(token_market)
                        .with_static_gas(Gas::ONE_TERA * 10_u64)
                        .view_market_data()
                        .then(
                            ext_self::ext(current_account_id())
                                .with_static_gas(Gas::ONE_TERA * 135_u64)
                                .with_unused_gas_weight(4_u64)
                                .market_data_callback(
                                    order_id,
                                    order,
                                    None,
                                    None,
                                    current_buy_token_price,
                                    slippage_price_impact,
                                ),
                        );
                }
                _ => {
                    panic!("Incorrect action for close position with take profit")
                }
            };
        }
    }
}
