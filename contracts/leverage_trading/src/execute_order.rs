use crate::ref_finance::ext_ref_finance;
use crate::utils::NO_DEPOSIT;
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, Gas, Promise, PromiseResult};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_for_execute_order_callback(&self, order: Order, order_id: U128, take_profit_order: bool);
    fn execute_order_callback(&self, order: Order, order_id: U128, take_profit_order: bool);
}

#[near_bindgen]
impl Contract {
    /// Executes order by inner order_id set on ref finance once the price range was crossed.
    /// Gets pool info, removes liquidity presented by one asset and marks order as executed.
    pub fn execute_order(&self, order_id: U128, take_profit_order: bool) -> PromiseOrValue<U128> {
        let order = if take_profit_order {
            self.take_profit_orders.get(&(order_id.0 as u64))
        } else {
            self.get_order_by(order_id.0)
        };

        require!(order.is_some(), "There is no such order to be executed");

        assert_eq!(
            order.as_ref().unwrap().status.clone(),
            OrderStatus::Pending,
            "Error. Order has to be Pending to be executed"
        );

        let order = order.unwrap();

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 5u64)
            .with_attached_deposit(NO_DEPOSIT)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(99)
                    .with_attached_deposit(NO_DEPOSIT)
                    .execute_order_callback(order, order_id, take_profit_order),
            )
            .into()
    }

    #[private]
    pub fn execute_order_callback(
        &self,
        order: Order,
        order_id: U128,
        take_profit_order: bool,
    ) -> PromiseOrValue<U128> {
        require!(is_promise_success(), "Failed to get_liquidity");

        let position = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                near_sdk::serde_json::from_slice::<ref_finance::LiquidityInfo>(&val).unwrap()
            }
            PromiseResult::Failed => panic!("Ref finance not found pool"),
        };

        let remove_liquidity_amount = position.amount;

        let min_amount_x = 0;
        let min_amount_y =
            BigDecimal::from(order.amount) * order.leverage * order.sell_token_price.value
                / order.buy_token_price.value;

        let (_, buy_token_decimals) =
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
        let min_amount_y =
            self.from_protocol_to_token_decimals(U128::from(min_amount_y), buy_token_decimals);

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 45u64)
            .remove_liquidity(
                order.lpt_id.clone(),
                remove_liquidity_amount,
                U128(min_amount_x),
                min_amount_y,
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(99)
                    .with_attached_deposit(NO_DEPOSIT)
                    .remove_liquidity_for_execute_order_callback(
                        order,
                        order_id,
                        take_profit_order,
                    ),
            )
            .into()
    }

    #[private]
    pub fn remove_liquidity_for_execute_order_callback(
        &mut self,
        order: Order,
        order_id: U128,
        take_profit_order: bool,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            panic!("Some problem with remove liquidity");
        } else {
            self.mark_order_as_executed(order, order_id, take_profit_order);

            if !take_profit_order {
                if let Some(tpo) = self.take_profit_orders.get(&(order_id.0 as u64)){
                    self.add_take_profit_order(order_id,tpo.clone(), WRatio::from(tpo.buy_token_price.value));
                }
            }

            let executor_reward_in_near = env::used_gas().0 as Balance * 2u128;
            Promise::new(env::signer_account_id())
                .transfer(executor_reward_in_near)
                .into()
        }
    }
}

impl Contract {
    pub fn mark_order_as_executed(
        &mut self,
        order: Order,
        order_id: U128,
        take_profit_order: bool,
    ) {
        let mut order = order;
        order.status = OrderStatus::Executed;

        if !take_profit_order {
            self.add_or_update_order(
                &self.get_account_by(order_id.0).unwrap(), // assert there is always some user
                order,
                order_id.0 as u64,
            );
        } else {
            self.take_profit_orders.insert(&(order_id.0 as u64), &order);
        }
    }

    pub fn get_account_by(&self, order_id: u128) -> Option<AccountId> {
        let mut account: Option<AccountId> = None;

        for (account_id, users_order) in self.orders.iter() {
            if users_order.contains_key(&(order_id as u64)) {
                account = Some(account_id);
                break;
            }
        }
        account
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
            .block_index(103930920)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_get_account_by() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let order = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1000000000000000000000000\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#543\"}".to_string();
        contract.add_order_from_string(alice(), order);

        let account_id = contract.orders.get(&alice()).unwrap().contains_key(&1);

        assert!(account_id);
        assert_eq!(contract.get_account_by(1), Some(alice()));
    }

    #[test]
    fn test_order_was_execute() {
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

        let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1000000000000000000000000\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#543\"}".to_string();
        contract.add_order_from_string(alice(), order_as_string.clone());

        let order_id = U128(1);
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();
        contract.mark_order_as_executed(order, order_id, false);

        let orders = contract.orders.get(&alice()).unwrap();
        let order = orders.get(&1).unwrap();

        let orders_from_pair = contract.orders_per_pair_view.get(&pair_id).unwrap();
        let order_from_pair = orders_from_pair.get(&1).unwrap();

        assert_eq!(order.status, OrderStatus::Executed);
        assert_eq!(order_from_pair.status, order.status);
    }
}
