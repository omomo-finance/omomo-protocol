use crate::big_decimal::{BigDecimal, WBalance};
use crate::ref_finance::ext_ref_finance;
use crate::utils::{ext_market, ext_token, NO_DEPOSIT};
use crate::*;
use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, serde_json, Gas, PromiseResult};

const GAS_FOR_BORROW: Gas = Gas(200_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn get_pool_info_callback(&mut self, order: Order) -> PromiseOrValue<WBalance>;
    fn borrow_callback(&mut self) -> PromiseOrValue<WBalance>;
    fn add_liquidity_callback(&mut self, order: Order) -> PromiseOrValue<Balance>;
}

#[near_bindgen]
impl Contract {
    /// Creates an order with given order_type, amount, sell_token, buy_token & leverage.
    ///
    /// Checks ref finance pool information for current price & borrow if leverage > 1.
    ///
    /// As far as we surpassed gas limit for contract call,
    /// borrow call was separated & made within batch of transaction alongside with Deposit & Add_Liquidity function
    ///
    /// Accepts deposit only two times greater than gas for execution in order to cover the execution gas fees and reward an executor
    #[payable]
    pub fn create_order(
        &mut self,
        order_type: OrderType,
        amount: WBalance,
        sell_token: AccountId,
        buy_token: AccountId,
        leverage: U128,
        take_profit_order: Option<Price>
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

        let order = Order {
            status: OrderStatus::Pending,
            order_type,
            amount: Balance::from(amount),
            sell_token: sell_token.clone(),
            buy_token: buy_token.clone(),
            leverage: BigDecimal::from(leverage),
            sell_token_price: self.view_price(sell_token),
            buy_token_price: self.view_price(buy_token),
            block: env::block_height(),
            lpt_id: "".to_string(),
        };

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_attached_deposit(NO_DEPOSIT)
            .with_static_gas(Gas::ONE_TERA * 5u64)
            .get_pool(self.view_pair(&order.sell_token, &order.buy_token).pool_id)
            .then(
                ext_self::ext(current_account_id())
                    .with_attached_deposit(NO_DEPOSIT)
                    .with_static_gas(Gas::ONE_TERA * 200u64 + Gas::ONE_TERA * 50u64)
                    .get_pool_info_callback(order, take_profit_order),
            )
            .into()
    }

    #[private]
    pub fn get_pool_info_callback(&mut self, order: Order, take_profit_order: Option<Price>) -> PromiseOrValue<WBalance> {
        require!(
            is_promise_success(),
            "Problem with pool on ref finance has occurred"
        );

        let pool_info = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pool) = serde_json::from_slice::<PoolInfo>(&val) {
                    pool
                } else {
                    panic!("Some problem with pool parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found pool"),
        };

        require!(
            pool_info.state == PoolState::Running,
            "Some problem with pool, please contact with ref finance to support."
        );

        self.add_liquidity(pool_info, order, take_profit_order)
    }

    /// Makes batch of transaction consist of Deposit & Add_Liquidity
    fn add_liquidity(&mut self, pool_info: PoolInfo, order: Order, take_profit_order: Option<Price>) -> PromiseOrValue<WBalance> {
        // calculating the range for the liquidity to be added into
        // consider the smallest gap is point_delta for given pool

        let (left_point, right_point, amount, amount_x, amount_y, token_to_add_liquidity) =
            match order.order_type {
                OrderType::Buy => {
                    let mut left_point = pool_info.current_point as i32;

                    while left_point % pool_info.point_delta as i32 != 0 {
                        left_point += 1;
                    }

                    let right_point = left_point + pool_info.point_delta as i32;

                    let amount =
                        U128::from(BigDecimal::from(U128::from(order.amount)) * order.leverage);

                    let amount_x = amount;
                    let amount_y = U128::from(0);

                    let token_to_add_liquidity = order.sell_token.clone();

                    (
                        left_point,
                        right_point,
                        amount,
                        amount_x,
                        amount_y,
                        token_to_add_liquidity,
                    )
                }
                OrderType::Sell => {
                    let mut right_point = pool_info.current_point as i32;

                    while right_point % pool_info.point_delta as i32 != 0 {
                        right_point -= 1;
                    }

                    let left_point = right_point - pool_info.point_delta as i32;

                    let amount =
                        U128::from(BigDecimal::from(U128::from(order.amount)) * order.leverage);

                    let amount_x = U128::from(0);
                    let amount_y = amount;

                    let token_to_add_liquidity = order.buy_token.clone();

                    (
                        left_point,
                        right_point,
                        amount,
                        amount_x,
                        amount_y,
                        token_to_add_liquidity,
                    )
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
                    .add_liquidity_callback(order.clone(), take_profit_order),
            );
        add_liquidity_promise.into()
    }

    #[private]
    pub fn add_liquidity_callback(&mut self, order: Order, take_profit_order: Option<Price>) -> PromiseOrValue<WBalance> {
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
        self.insert_order_for_user(&env::signer_account_id(), order, order_id, take_profit_order);

        PromiseOrValue::Value(U128(0))
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
    pub fn add_order(&mut self, account_id: AccountId, order: String) {
        self.order_nonce += 1;
        let order_id = self.order_nonce;
        let order = serde_json::from_str(order.as_str()).unwrap();
        self.insert_order_for_user(&account_id, order, order_id, None);
    }

    pub fn insert_order_for_user(&mut self, account_id: &AccountId, order: Order, order_id: u64, take_profit_order: Option<Price>) {
        let mut user_orders_by_id = self.orders.get(account_id).unwrap_or_default();
        user_orders_by_id.insert(order_id, order);
        self.orders.insert(account_id, &user_orders_by_id);
        if take_profit_order.is_some() {
            self.insert_or_update_tpo(order_id, take_profit_order.unwrap());
        }
    }

    // tpo => take profit order
    pub fn insert_or_update_tpo(&mut self, order_id: u64, price: Price) {
       if signer_account_id() == self.get_account_by(order_id) {
           self.take_profit_orders.insert(&order_id, &price);
       }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::test_env::{alice, bob};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("margin.nearland.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
            .block_index(103930916)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn insert_tpo_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "near".to_string(),
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(10u128.pow(20)),
        };
        contract.add_pair(pair_data.clone());

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: BigDecimal::from(2.0),
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: BigDecimal::from(4.22),
            },
        );

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1000000000000000000000000\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#543\"}".to_string();
        contract.add_order(alice(), order1);

        let tpo_price = Price {
            ticker_id: "Near".to_string(),
            value: BigDecimal::from(4.22)
        };

        contract.insert_or_update_tpo(1, tpo_price);

        assert_eq!();
    }
}
