use crate::big_decimal::{BigDecimal, WBalance};
use crate::common::Event;
use crate::ref_finance::ext_ref_finance;
use crate::utils::{ext_market, ext_token, NO_DEPOSIT};
use crate::*;

use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, serde_json, Gas, PromiseResult};

const GAS_FOR_BORROW: Gas = Gas(200_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn borrow_callback(&mut self) -> PromiseOrValue<WBalance>;
    fn add_liquidity_callback(&mut self, order: Order) -> PromiseOrValue<Balance>;
}

#[near_bindgen]
impl Contract {
    /// Creates an order with given order_type, left_point, right_point, amount, sell_token, buy_token & leverage.
    ///
    /// As far as we surpassed gas limit for contract call,
    /// borrow call was separated & made within batch of transaction alongside with Deposit & Add_Liquidity function
    ///
    /// Accepts deposit only two times greater than gas for execution in order to cover the execution gas fees and reward an executor
    #[payable]
    pub fn create_order(
        &mut self,
        order_type: OrderType,
        // left point for add_liquidity acquired via getPointByPrice
        left_point: i32,
        // right point for add_liquidity acquired via getPointByPrice
        right_point: i32,
        amount: WBalance,
        sell_token: AccountId,
        buy_token: AccountId,
        leverage: U128,
        open_price: U128,
    ) -> PromiseOrValue<WBalance> {
        require!(
            env::attached_deposit() >= self.view_gas_for_execution() * 2,
            "Create order should accept deposits two times greater than gas for execution"
        );

        let user = env::signer_account_id();
        require!(
            amount.0 <= self.max_order_amount,
            "amount more than allowed value."
        );
        require!(
            self.balance_of(user, sell_token.clone()) >= amount,
            "User doesn't have enough deposit to proceed this action"
        );

        let sell_token_price = self.view_price(sell_token.clone());
        require!(
            sell_token_price.value != BigDecimal::zero(),
            "Sell token price cannot be zero"
        );

        let buy_token_price = self.view_price(buy_token.clone());
        require!(
            buy_token_price.value != BigDecimal::zero(),
            "Buy token price cannot be zero"
        );

        let order = Order {
            status: OrderStatus::Pending,
            order_type,
            amount: Balance::from(amount),
            sell_token,
            buy_token,
            leverage: BigDecimal::from(leverage),
            sell_token_price,
            buy_token_price,
            open_price: BigDecimal::from(open_price),
            block: env::block_height(),
            time_stamp_ms: env::block_timestamp_ms(),
            lpt_id: "".to_string(),
        };

        self.add_liquidity(order, left_point, right_point)
    }

    /// Makes batch of transaction consist of Deposit & Add_Liquidity
    fn add_liquidity(
        &mut self,
        order: Order,
        left_point: i32,
        right_point: i32,
    ) -> PromiseOrValue<WBalance> {
        // calculating the range for the liquidity to be added into
        // consider the smallest gap is point_delta for given pool

        let (amount, amount_x, amount_y, token_to_add_liquidity) = match order.order_type {
            OrderType::Buy => {
                let amount =
                    U128::from(BigDecimal::from(U128::from(order.amount)) * order.leverage);

                let (sell_token_decimals, _) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                let amount = self.from_protocol_to_token_decimals(amount, sell_token_decimals);

                let amount_x = amount;
                let amount_y = U128::from(0);

                let token_to_add_liquidity = order.sell_token.clone();

                (amount, amount_x, amount_y, token_to_add_liquidity)
            }
            OrderType::Sell => {
                let amount =
                    U128::from(BigDecimal::from(U128::from(order.amount)) * order.leverage);

                let (_, buy_token_decimals) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
                let amount = self.from_protocol_to_token_decimals(amount, buy_token_decimals);

                let amount_x = U128::from(0);
                let amount_y = amount;

                let token_to_add_liquidity = order.buy_token.clone();

                (amount, amount_x, amount_y, token_to_add_liquidity)
            }
        };

        let min_amount_x = U128::from(0);
        let min_amount_y = U128::from(0);

        let add_liquidity_promise = ext_token::ext(token_to_add_liquidity)
            .with_static_gas(Gas::ONE_TERA * 35u64)
            .with_attached_deposit(near_sdk::ONE_YOCTO)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                amount,
                None,
                "\"Deposit\"".to_string(),
            )
            .and(
                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_static_gas(Gas::ONE_TERA * 10u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .add_liquidity(
                        self.view_pair(&order.sell_token, &order.buy_token).pool_id,
                        left_point,
                        right_point,
                        amount_x,
                        amount_y,
                        min_amount_x,
                        min_amount_y,
                    ),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 2u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .add_liquidity_callback(order.clone()),
            );
        add_liquidity_promise.into()
    }

    #[private]
    pub fn add_liquidity_callback(&mut self, order: Order) -> PromiseOrValue<WBalance> {
        require!(
            env::promise_results_count() == 2,
            "Contract expected 2 results on the callback"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady | PromiseResult::Failed => {
                panic!("failed to deposit liquidity")
            }
            _ => (),
        };

        self.decrease_balance(&env::signer_account_id(), &order.sell_token, order.amount);

        let lpt_id: String = match env::promise_result(1) {
            PromiseResult::Successful(result) => serde_json::from_slice::<String>(&result).unwrap(),
            _ => panic!("failed to add liquidity"),
        };

        let mut order = order;
        order.lpt_id = lpt_id;

        self.order_nonce += 1;
        let order_id = self.order_nonce;

        self.add_or_update_order(&env::signer_account_id(), order.clone(), order_id);

        Event::CreateOrderEvent {
            order_id,
            sell_token_price: order.sell_token_price,
            buy_token_price: order.buy_token_price,
            pool_id: self.view_pair(&order.sell_token, &order.buy_token).pool_id,
        }
        .emit();

        PromiseOrValue::Value(U128(order_id as u128))
    }

    /// Borrow step made within batch of transaction
    /// Doesn't borrow when leverage is less or equal to 1.0
    pub fn borrow(
        &mut self,
        token: AccountId,
        amount: U128,
        leverage: U128,
    ) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_BORROW,
            "Prepaid gas is not enough for borrow flow"
        );

        require!(
            self.balance_of(env::signer_account_id(), token.clone()) >= amount,
            "User doesn't have enough deposit to proceed this action"
        );

        if BigDecimal::from(leverage) <= BigDecimal::one() {
            return PromiseOrValue::Value(U128(0));
        }

        let token_market = self.get_market_by(&token);
        let borrow_amount =
            U128::from(BigDecimal::from(amount) * (BigDecimal::from(leverage) - BigDecimal::one()));

        ext_market::ext(token_market)
            .with_static_gas(GAS_FOR_BORROW)
            .borrow(borrow_amount)
            .then(
                ext_self::ext(env::current_account_id())
                    .with_unused_gas_weight(100)
                    .borrow_callback(),
            )
            .into()
    }

    #[private]
    pub fn borrow_callback(&mut self) -> PromiseOrValue<WBalance> {
        require!(is_promise_success(), "Contract failed to borrow assets");
        PromiseOrValue::Value(U128(0))
    }

    #[private]
    pub fn add_order_from_string(&mut self, account_id: AccountId, order: String) {
        self.order_nonce += 1;
        let order_id = self.order_nonce;
        let order: Order = serde_json::from_str(order.as_str()).unwrap();
        self.add_or_update_order(&account_id, order, order_id);
    }

    #[payable]
    pub fn add_take_profit_order(
        &mut self,
        order_id: U128,
        new_price: U128,
    ) -> PromiseOrValue<bool> {
        require!(
            Some(signer_account_id()) == self.get_account_by(order_id.0),
            "You do not have permission for this action."
        );

        let mut order = self.get_order_by(order_id.0).unwrap();
        order.status = OrderStatus::Pending;
        order.buy_token_price.value = BigDecimal::from(new_price);
        self.take_profit_orders.insert(&(order_id.0 as u64), &order);

        Event::CreateTakeProfitOrderEvent {
            order_id,
            price: new_price,
            pool_id: self.view_pair(&order.sell_token, &order.buy_token).pool_id,
        }
        .emit();

        PromiseOrValue::Value(true)
    }

    #[payable]
    pub fn set_take_profit_order_price(
        &mut self,
        order_id: U128,
        new_price: U128,
    ) -> PromiseOrValue<bool> {
        require!(
            Some(signer_account_id()) == self.get_account_by(order_id.0),
            "You do not have permission for this action."
        );

        if let Some(mut current_order) = self.take_profit_orders.get(&(order_id.0 as u64)) {
            current_order.buy_token_price.value = BigDecimal::from(new_price);
            self.take_profit_orders
                .insert(&(order_id.0 as u64), &current_order);

            Event::UpdateTakeProfitOrderEvent {
                order_id,
                price: new_price,
                pool_id: self
                    .view_pair(&current_order.sell_token, &current_order.buy_token)
                    .pool_id,
            }
            .emit();

            return PromiseOrValue::Value(true);
        }

        PromiseOrValue::Value(false)
    }
}

impl Contract {
    pub fn add_or_update_order(&mut self, account_id: &AccountId, order: Order, order_id: u64) {
        let pair_id = (order.sell_token.clone(), order.buy_token.clone());

        let mut user_orders_by_id = self.orders.get(account_id).unwrap_or_default();
        user_orders_by_id.insert(order_id, order.clone());
        self.orders.insert(account_id, &user_orders_by_id);

        let mut pair_orders_by_id = self.orders_per_pair_view.get(&pair_id).unwrap_or_default();
        pair_orders_by_id.insert(order_id, order);
        self.orders_per_pair_view
            .insert(&pair_id, &pair_orders_by_id);
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
            .signer_account_id(alice())
            .is_view(is_view)
            .build()
    }

    impl Contract {
        pub fn imitation_add_liquidity_callback(&mut self, order: Order) {
            self.decrease_balance(&env::signer_account_id(), &order.sell_token, order.amount);

            let mut lpt_id = order.sell_token.to_string();
            lpt_id.push('|');
            lpt_id.push_str(order.buy_token.as_str());
            lpt_id.push_str("|0000#0000");

            let mut order = order;
            order.lpt_id = lpt_id;

            self.order_nonce += 1;

            let order_id = self.order_nonce;

            self.add_or_update_order(&env::signer_account_id(), order, order_id);
        }
    }

    #[test]
    fn test_add_order_in_create_order() {
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

        contract.set_balance(&alice(), &pair_id.0, 10_u128.pow(30));

        assert_eq!(
            contract.orders.get(&alice()).unwrap_or_default().len(),
            0_usize
        );
        assert_eq!(
            contract
                .orders_per_pair_view
                .get(&pair_id)
                .unwrap_or_default()
                .len(),
            0_usize
        );

        for _ in 0..5 {
            let order = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"open_price\":\"2.5\",\"block\":103930910, \"time_stamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\"}".to_string();
            contract.imitation_add_liquidity_callback(
                near_sdk::serde_json::from_str(order.as_str()).unwrap(),
            );
        }

        assert_eq!(
            contract.orders.get(&alice()).unwrap_or_default().len(),
            5_usize
        );
        assert_eq!(
            contract
                .orders_per_pair_view
                .get(&pair_id)
                .unwrap_or_default()
                .len(),
            5_usize
        );
    }

    #[test]
    fn test_add_take_profit_order() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 18,
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data.clone());

        let order_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"block\":103930910,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\"}".to_string();
        contract.add_order_from_string(alice(), order_string);

        let new_price = U128(5);
        contract.add_take_profit_order(U128(1), new_price);

        let tpo = contract.take_profit_orders.get(&1).unwrap();
        assert_eq!(tpo.status, OrderStatus::Pending);
        assert_eq!(tpo.buy_token_price.value, BigDecimal::from(new_price));
    }

    #[test]
    #[should_panic(expected = "You do not have permission for this action")]
    fn test_add_take_profit_order_without_order() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let order_id: u128 = 33;
        let new_price = U128(5);
        assert_eq!(contract.get_order_by(order_id), None);

        contract.add_take_profit_order(U128(order_id), new_price);
    }

    #[test]
    fn test_set_take_profit_order_price() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 18,
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data.clone());

        let order_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.05\"},\"block\":103930910,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\"}".to_string();
        contract.add_order_from_string(alice(), order_string);

        let order_id: u128 = 1;
        let new_price = U128(5);
        contract.add_take_profit_order(U128(order_id), new_price);

        let tpo = contract.take_profit_orders.get(&(order_id as u64)).unwrap();
        assert_eq!(tpo.status, OrderStatus::Pending);
        assert_eq!(tpo.buy_token_price.value, BigDecimal::from(new_price));

        let new_price = U128(23);
        contract.set_take_profit_order_price(U128(order_id), new_price);

        let tpo = contract.take_profit_orders.get(&(order_id as u64)).unwrap();
        assert_eq!(tpo.buy_token_price.value, BigDecimal::from(new_price));
    }
}
