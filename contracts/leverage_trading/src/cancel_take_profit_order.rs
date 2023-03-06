use crate::common::Event;
use crate::execute_order::INACCURACY_RATE;
use crate::ref_finance::ext_ref_finance;
use crate::utils::NO_DEPOSIT;
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, Gas, PromiseResult};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn get_take_profit_liquidity_info_callback(
        &self,
        order_id: U128,
        parent_order: Order,
        take_profit_info: (PricePoints, Order),
    );
    fn remove_liquidity_from_take_profit_callback(&self, order_id: U128);
}

#[near_bindgen]
impl Contract {
    pub fn cancel_take_profit_order(&mut self, order_id: U128) {
        let take_profit_order = self.take_profit_orders.get(&(order_id.0 as u64));
        require!(take_profit_order.is_some(), "Take profit order not found.");

        let parent_order = self.get_order_by_id(order_id).unwrap().1;
        let take_profit_order_pair = take_profit_order.unwrap();
        let tpo = take_profit_order_pair.1.clone();
        match tpo.status {
            OrderStatus::PendingOrderExecute => {
                self.take_profit_orders.remove(&(order_id.0 as u64));

                Event::CancelTakeProfitOrderEvent { order_id }.emit();
            }
            OrderStatus::Pending => {
                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_unused_gas_weight(2)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_liquidity(take_profit_order_pair.1.lpt_id.clone())
                    .then(
                        ext_ref_finance::ext(self.ref_finance_account.clone())
                            .with_unused_gas_weight(1_u64)
                            .with_attached_deposit(NO_DEPOSIT)
                            .get_pool(
                                self.get_trade_pair(
                                    &parent_order.sell_token,
                                    &parent_order.buy_token,
                                )
                                .pool_id,
                            )
                            .then(
                                ext_self::ext(current_account_id())
                                    .with_unused_gas_weight(98)
                                    .with_attached_deposit(NO_DEPOSIT)
                                    .get_take_profit_liquidity_info_callback(
                                        order_id,
                                        parent_order,
                                        take_profit_order_pair,
                                    ),
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

        let liquidity_info: Liquidity = match env::promise_result(1) {
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

        if parent_order.order_type == OrderType::Long {
            require!(
                pool_info.current_point > liquidity_info.right_point,
                "You cannot cancel the opening of a position. Liquidity is already used by DEX"
            );
        } else {
            require!(
                pool_info.current_point < liquidity_info.left_point,
                "You cannot cancel the opening of a position. Liquidity is already used by DEX"
            );
        }

        let (min_amount_x, min_amount_y) = if parent_order.order_type == OrderType::Long {
            (
                U128::from(0),
                U128::from(
                    BigDecimal::from(U128(take_profit_info.1.amount))
                        * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
                ),
            )
        } else {
            (
                U128::from(
                    BigDecimal::from(U128(take_profit_info.1.amount))
                        * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
                ),
                U128::from(0),
            )
        };

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 70)
            .with_attached_deposit(NO_DEPOSIT)
            .remove_liquidity(
                take_profit_info.1.lpt_id,
                liquidity_info.amount,
                min_amount_x,
                min_amount_y,
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .remove_liquidity_from_take_profit_callback(order_id),
            );
    }

    #[private]
    pub fn remove_liquidity_from_take_profit_callback(&mut self, order_id: U128) {
        require!(
            is_promise_success(),
            "Some problem with removing liquidity."
        );

        self.take_profit_orders.remove(&(order_id.0 as u64));

        Event::CancelTakeProfitOrderEvent { order_id }.emit();

        self.remove_pending_order_data(PendingOrderData {
            order_id,
            order_type: OrderType::TakeProfit,
        });
    }
}
