use crate::metadata::{Order, OrderStatus};
use crate::ref_finance::{ext_ref_finance, ShortLiquidityInfo};
use crate::{common::Event, HistoryData, OrderType, PendingOrderData, PnLView};
use crate::{Contract, ContractExt};

use near_sdk::env::{self, current_account_id, signer_account_id};
use near_sdk::json_types::U128;
use near_sdk::{ext_contract, near_bindgen, require, Gas, PromiseResult};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn get_limit_order_liquidity_callback(&self, order_id: U128, order: Order);
    fn remove_limit_order_liquidity_callback(&mut self, order_id: U128, order: Order);
}

#[near_bindgen]
impl Contract {
    pub fn cancel_limit_order(&mut self, order_id: U128, order: Order) {
        require!(
            order.status == OrderStatus::Pending,
            "To cancel a limit order, its status must be Pending."
        );

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 5_u64)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 270_u64)
                    .with_unused_gas_weight(2_u64)
                    .get_limit_order_liquidity_callback(order_id, order),
            );
    }

    #[private]
    pub fn get_limit_order_liquidity_callback(&self, order_id: U128, order: Order) {
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

        // We need return partial execute amounts for pair tokens => min (0, 0)
        let (min_amount_x, min_amount_y) = (U128::from(0), U128::from(0));

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 90_u64)
            .remove_liquidity(
                order.lpt_id.clone(),
                liquidity_info.amount,
                min_amount_x,
                min_amount_y,
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 170_u64)
                    .with_unused_gas_weight(2_u64)
                    .remove_limit_order_liquidity_callback(order_id, order),
            );
    }

    #[private]
    pub fn remove_limit_order_liquidity_callback(&mut self, order_id: U128, order: Order) {
        let return_amounts = self.get_return_amounts_after_remove_liquidity(order.clone());

        let mut order = order;
        order.status = OrderStatus::Canceled;

        let executed = if order.order_type == OrderType::Buy {
            return_amounts.amount_sell_token
        } else {
            return_amounts.amount_buy_token
        };

        order.history_data = Some(HistoryData {
            fee: U128(0),
            pnl: PnLView {
                is_profit: false,
                amount: U128(0),
            },
            executed,
        });

        self.add_or_update_order(&signer_account_id(), order.clone(), order_id.0 as u64);
        Event::CancelLimitOrderEvent { order_id }.emit();

        self.remove_pending_order_data(PendingOrderData {
            order_id,
            order_type: order.order_type,
        });

        self.increase_balance(
            &signer_account_id(),
            &order.sell_token,
            return_amounts.amount_sell_token.0,
        );

        self.increase_balance(
            &signer_account_id(),
            &order.buy_token,
            return_amounts.amount_buy_token.0,
        );

        self.withdraw(order.sell_token, return_amounts.amount_sell_token, None);
        self.withdraw(order.buy_token, return_amounts.amount_buy_token, None);
    }
}
