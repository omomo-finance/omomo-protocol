use crate::common::Event;

use crate::ref_finance::ext_ref_finance;
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
    fn remove_liquidity_from_take_profit_callback(&self, order_id: U128, parent_order: Order);
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
                    .remove_liquidity_from_take_profit_callback(order_id, parent_order),
            );
    }

    #[private]
    pub fn remove_liquidity_from_take_profit_callback(
        &mut self,
        order_id: U128,
        parent_order: Order,
    ) {
        require!(
            is_promise_success(),
            "Some problem with removing liquidity."
        );

        let return_liquidity_amounts = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(amounts) = near_sdk::serde_json::from_slice::<Vec<U128>>(&val) {
                    amounts
                } else {
                    panic!("Some problem with return amount from Dex.")
                }
            }
            _ => panic!("DEX not found liquidity amounts."),
        };

        let token_decimals = self.view_token_decimals(&parent_order.sell_token);
        let amount_x = self.from_token_to_protocol_decimals(
            return_liquidity_amounts.get(0).unwrap().0,
            token_decimals,
        );
        let token_decimals = self.view_token_decimals(&parent_order.buy_token);
        let amount_y = self.from_token_to_protocol_decimals(
            return_liquidity_amounts.get(1).unwrap().0,
            token_decimals,
        );

        self.increase_balance(&signer_account_id(), &parent_order.sell_token, amount_x.0);
        self.increase_balance(&signer_account_id(), &parent_order.buy_token, amount_y.0);

        self.take_profit_orders.remove(&(order_id.0 as u64));

        Event::CancelTakeProfitOrderEvent { order_id }.emit();

        self.remove_pending_order_data(PendingOrderData {
            order_id,
            order_type: OrderType::TakeProfit,
        });
    }
}
