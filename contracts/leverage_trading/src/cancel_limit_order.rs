use near_sdk::{
    env::{current_account_id, signer_account_id},
    ext_contract, is_promise_success, Gas, PromiseResult,
};

use crate::{
    common::Event,
    ref_finance::{ext_ref_finance, ShortLiquidityInfo},
    utils::NO_DEPOSIT,
    *,
};

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
            .with_unused_gas_weight(2)
            .with_attached_deposit(NO_DEPOSIT)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(98)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_limit_order_liquidity_callback(order_id, order),
            );
    }

    #[private]
    pub fn get_limit_order_liquidity_callback(&self, order_id: U128, order: Order) {
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

        // We need return partial execute amounts for pair tokens => min (0, 0)
        let (min_amount_x, min_amount_y) = (U128::from(0), U128::from(0));

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 70)
            .with_attached_deposit(NO_DEPOSIT)
            .remove_liquidity(
                order.lpt_id.clone(),
                liquidity_info.amount,
                min_amount_x,
                min_amount_y,
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .remove_limit_order_liquidity_callback(order_id, order),
            );
    }

    #[private]
    pub fn remove_limit_order_liquidity_callback(&mut self, order_id: U128, order: Order) {
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

        let token_decimals = self.view_token_decimals(&order.sell_token);
        let amount_x = self.from_token_to_protocol_decimals(
            return_liquidity_amounts.get(0).unwrap().0,
            token_decimals,
        );
        let token_decimals = self.view_token_decimals(&order.buy_token);
        let amount_y = self.from_token_to_protocol_decimals(
            return_liquidity_amounts.get(1).unwrap().0,
            token_decimals,
        );

        self.increase_balance(&signer_account_id(), &order.sell_token, amount_x.0);
        self.increase_balance(&signer_account_id(), &order.buy_token, amount_y.0);

        let mut order = order;
        order.status = OrderStatus::Canceled;

        let mut executed = *return_liquidity_amounts.get(0).unwrap();
        if order.order_type == OrderType::Sell {
            executed = *return_liquidity_amounts.get(1).unwrap();
        }
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
    }
}
