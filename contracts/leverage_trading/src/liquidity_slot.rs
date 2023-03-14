use near_sdk::{
    env::{current_account_id, signer_account_id},
    ext_contract, is_promise_success, Gas, PromiseResult,
};

use crate::{
    ref_finance::{ext_ref_finance, LptId, ShortLiquidityInfo},
    utils::NO_DEPOSIT,
    *,
};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn get_liquidity_info_callback(
        &self,
        lpt_id: LptId,
        user: Option<AccountId>,
        oldest_pending_order_data: PendingOrderData,
    );
    fn remove_oldest_liquidity_callback(
        &mut self,
        user: Option<AccountId>,
        oldest_pending_order_data: PendingOrderData,
    );
}

#[near_bindgen]
impl Contract {
    pub fn free_up_liquidity_slot(&mut self) {
        require!(
            signer_account_id() == self.config.oracle_account_id
                || signer_account_id() == current_account_id(),
            "You do not have access to call this method."
        );

        let oldest_pending_order_data = self.get_oldest_pending_order_data();

        match oldest_pending_order_data.order_type {
            OrderType::Buy | OrderType::Sell | OrderType::Long | OrderType::Short => {
                if let Some((user, order)) =
                    self.get_order_by_id(oldest_pending_order_data.order_id)
                {
                    if let OrderStatus::Pending = order.status {
                        self.get_liquidity_info(
                            order.lpt_id,
                            Some(user),
                            oldest_pending_order_data,
                        );
                    }
                }
            }
            OrderType::TakeProfit => {
                if let Some(order) =
                    self.get_take_profit_order_by_id(oldest_pending_order_data.order_id)
                {
                    if let OrderStatus::Pending = order.status {
                        self.get_liquidity_info(order.lpt_id, None, oldest_pending_order_data);
                    }
                }
            }
        }
    }

    #[private]
    pub fn get_liquidity_info(
        &self,
        lpt_id: LptId,
        user: Option<AccountId>,
        oldest_pending_order_data: PendingOrderData,
    ) {
        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(2)
            .with_attached_deposit(NO_DEPOSIT)
            .get_liquidity(lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(98)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_liquidity_info_callback(lpt_id, user, oldest_pending_order_data),
            );
    }

    #[private]
    pub fn get_liquidity_info_callback(
        &self,
        lpt_id: LptId,
        user: Option<AccountId>,
        oldest_pending_order_data: PendingOrderData,
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

        let min_amount_x = U128::from(0);
        let min_amount_y = U128::from(0);

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 70)
            .with_attached_deposit(NO_DEPOSIT)
            .remove_liquidity(lpt_id, liquidity_info.amount, min_amount_x, min_amount_y)
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .remove_oldest_liquidity_callback(user, oldest_pending_order_data),
            );
    }

    #[private]
    pub fn remove_oldest_liquidity_callback(
        &mut self,
        user: Option<AccountId>,
        oldest_pending_order_data: PendingOrderData,
    ) {
        require!(
            is_promise_success(),
            "Some problem with removing liquidity."
        );

        match oldest_pending_order_data.order_type {
            OrderType::Buy | OrderType::Sell => {
                self.remove_order_by_ids(user.unwrap(), oldest_pending_order_data.order_id);
            }
            OrderType::Long | OrderType::Short => {
                self.remove_order_by_ids(user.unwrap(), oldest_pending_order_data.order_id);
                self.remove_take_profit_order_by_id(oldest_pending_order_data.order_id);
            }
            OrderType::TakeProfit => {
                self.remove_take_profit_order_by_id(oldest_pending_order_data.order_id);
            }
        }

        if let Some((pair_id, _)) =
            self.get_order_per_pair_view_by_id(oldest_pending_order_data.order_id)
        {
            self.remove_order_per_pair_view_by_ids(pair_id, oldest_pending_order_data.order_id);
        }

        self.pending_orders_data.pop_front();
    }
}

impl Contract {
    pub fn get_order_by_id(&self, order_id: U128) -> Option<(AccountId, Order)> {
        for user in self.orders.keys().collect::<Vec<_>>() {
            if let Some(order) = self
                .orders
                .get(&user)
                .unwrap()
                .get(&(order_id.0 as u64))
                .cloned()
            {
                return Some((user, order));
            }
        }
        None
    }

    pub fn get_take_profit_order_by_id(&self, order_id: U128) -> Option<Order> {
        if let Some((_, order, _)) = self.take_profit_orders.get(&(order_id.0 as u64)) {
            return Some(order);
        }
        None
    }

    pub fn get_order_per_pair_view_by_id(&self, order_id: U128) -> Option<(PairId, Order)> {
        for pair_id in self.orders_per_pair_view.keys().collect::<Vec<_>>() {
            if let Some(order) = self
                .orders_per_pair_view
                .get(&pair_id)
                .unwrap()
                .get(&(order_id.0 as u64))
                .cloned()
            {
                return Some((pair_id, order));
            }
        }
        None
    }

    fn remove_order_by_ids(&mut self, account_id: AccountId, order_id: U128) {
        let mut orders = self.orders.get(&account_id).unwrap();
        orders.remove(&(order_id.0 as u64));
        self.orders.remove(&account_id);
        self.orders.insert(&account_id, &orders);
    }

    fn remove_take_profit_order_by_id(&mut self, order_id: U128) {
        self.take_profit_orders.remove(&(order_id.0 as u64));
    }

    fn remove_order_per_pair_view_by_ids(&mut self, pair_id: PairId, order_id: U128) {
        let mut orders = self.orders_per_pair_view.get(&pair_id).unwrap();
        orders.remove(&(order_id.0 as u64));
        self.orders_per_pair_view.remove(&pair_id);
        self.orders_per_pair_view.insert(&pair_id, &orders);
    }
}
