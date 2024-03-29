use crate::big_decimal::{BigDecimal, WBalance};
use crate::common::Event;
use crate::ref_finance::ext_ref_finance;
use crate::utils::{ext_market, ext_token, MILLISECONDS_PER_DAY, NO_DEPOSIT};
use crate::*;

use near_sdk::env::{block_height, block_timestamp_ms, current_account_id, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, serde_json, Gas, Promise, PromiseResult};

const GAS_FOR_BORROW: Gas = Gas(200_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn borrow_callback(&mut self, borrow_amount: U128) -> PromiseOrValue<WBalance>;
    fn add_liquidity_callback(&mut self, order: Order, amount: U128) -> PromiseOrValue<Balance>;
    fn take_profit_liquidity_callback(
        &mut self,
        order_id: U128,
        amount: U128,
        parent_order: Order,
        reward_executor: bool,
    );
    fn withdraw_asset_callback(token_id: AccountId, amount: U128);
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
        entry_price: U128,
    ) -> PromiseOrValue<WBalance> {
        require!(
            env::attached_deposit() >= self.view_gas_for_execution() * 2,
            "Create order should accept deposits two times greater than gas for execution"
        );

        let user = env::signer_account_id();
        require!(
            amount.0 <= self.max_order_amount,
            "Amount more than allowed value"
        );

        if order_type == OrderType::Sell {
            require!(
                self.balance_of(user, buy_token.clone()) >= amount,
                "User doesn't have enough deposit to proceed this action"
            )
        } else {
            require!(
                self.balance_of(user, sell_token.clone()) >= amount,
                "User doesn't have enough deposit to proceed this action"
            )
        }

        let sell_token_price = self.view_price(sell_token.clone());
        require!(
            BigDecimal::from(sell_token_price.value) != BigDecimal::zero(),
            "Sell token price cannot be zero"
        );

        let buy_token_price = self.view_price(buy_token.clone());
        require!(
            BigDecimal::from(buy_token_price.value) != BigDecimal::zero(),
            "Buy token price cannot be zero"
        );

        match order_type {
            OrderType::Buy | OrderType::Sell => self.create_limit_order(
                order_type,
                left_point,
                right_point,
                amount,
                sell_token,
                sell_token_price,
                buy_token,
                buy_token_price,
                entry_price,
            ),
            OrderType::Long | OrderType::Short => self.create_leverage_order(
                order_type,
                left_point,
                right_point,
                amount,
                sell_token,
                sell_token_price,
                buy_token,
                buy_token_price,
                leverage,
                entry_price,
            ),
            OrderType::TakeProfit => panic!(
                "Incorrect type of order 'TP'. Expected one of 'Buy', 'Sell', 'Long', 'Short'"
            ),
        }
    }

    /// Makes batch of transaction consist of Deposit & Add_Liquidity
    fn add_liquidity(
        &mut self,
        order: Order,
        token_id: AccountId,
        amount: U128,
        left_point: i32,
        right_point: i32,
        amount_x: U128,
        amount_y: U128,
    ) -> PromiseOrValue<WBalance> {
        let min_amount_x = U128::from(0);
        let min_amount_y = U128::from(0);

        ext_token::ext(token_id)
            .with_static_gas(Gas::ONE_TERA * 35u64)
            .with_attached_deposit(near_sdk::ONE_YOCTO)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                amount,
                None,
                "\"Deposit\"".to_string(),
            )
            .then(
                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_static_gas(Gas::ONE_TERA * 15_u64)
                    .add_liquidity(
                        self.get_trade_pair(&order.sell_token, &order.buy_token)
                            .pool_id,
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
                    .add_liquidity_callback(order.clone(), amount),
            )
            .into()
    }

    #[private]
    pub fn add_liquidity_callback(
        &mut self,
        order: Order,
        amount: U128,
    ) -> PromiseOrValue<WBalance> {
        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let lpt_id = serde_json::from_slice::<String>(&result).unwrap();

                if order.order_type == OrderType::Sell {
                    self.decrease_balance(
                        &env::signer_account_id(),
                        &order.buy_token,
                        order.amount,
                    );
                } else {
                    self.decrease_balance(
                        &env::signer_account_id(),
                        &order.sell_token,
                        order.amount,
                    );
                }

                let mut order = order;
                order.lpt_id = lpt_id.clone();

                self.order_nonce += 1;
                let order_id = self.order_nonce;

                self.add_or_update_order(&env::signer_account_id(), order.clone(), order_id);

                Event::CreateOrderEvent {
                    order_id,
                    order_type: order.order_type.clone(),
                    lpt_id,
                    sell_token_price: order.sell_token_price,
                    buy_token_price: order.buy_token_price,
                    pool_id: self
                        .get_trade_pair(&order.sell_token, &order.buy_token)
                        .pool_id,
                }
                .emit();

                let order_id = U128::from(order_id as u128);

                self.pending_orders_data.push_back(PendingOrderData {
                    order_id,
                    order_type: order.order_type,
                });

                PromiseOrValue::Value(order_id)
            }
            _ => {
                let token_id =
                    if order.order_type == OrderType::Buy || order.order_type == OrderType::Long {
                        order.sell_token
                    } else {
                        order.buy_token
                    };

                near_sdk::log!("No liquidity was added. We return deposits from DEX");

                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_static_gas(Gas::ONE_TERA * 45_u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .withdraw_asset(token_id.clone(), amount)
                    .then(
                        ext_self::ext(current_account_id())
                            .with_static_gas(Gas::ONE_TERA * 2u64)
                            .with_attached_deposit(NO_DEPOSIT)
                            .withdraw_asset_callback(token_id, amount),
                    )
                    .into()
            }
        }
    }

    #[private]
    pub fn withdraw_asset_callback(token_id: AccountId, amount: U128) {
        if is_promise_success() {
            panic!(
                "Failed to add liquidity. The token '{token_id}' in the amount of '{amount}', was returned from the deposit DEX to the protocol balance", amount = amount.0)
        } else {
            panic!(
                "Failed to add liquidity and returned from the deposit DEX to the protocol balance"
            )
        };
    }
    /// Borrow step made within batch of transaction
    /// Doesn't borrow when leverage is less or equal to 1.0
    pub fn borrow(
        &mut self,
        order_type: OrderType,
        sell_token: AccountId,
        buy_token: AccountId,
        amount: U128,
        leverage: U128,
        open_price: U128,
    ) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_BORROW,
            "Prepaid gas is not enough for borrow flow"
        );

        require!(
            self.balance_of(env::signer_account_id(), sell_token.clone()) >= amount,
            "Failed to borrow. User doesn't have enough deposit to proceed this action"
        );

        require!(
            BigDecimal::from(leverage) > BigDecimal::one(),
            "Failed to borrow. Incorrect leverage amount. The leverage should be greater than one"
        );

        let (token_market, borrow_amount) = match order_type {
            OrderType::Long => {
                let borrow_amount = U128::from(
                    BigDecimal::from(amount) * (BigDecimal::from(leverage) - BigDecimal::one()),
                );

                let token_market = self.get_market_by(&sell_token);
                (token_market, borrow_amount)
            }
            OrderType::Short => {
                let borrow_amount = U128::from(
                    BigDecimal::from(amount) * (BigDecimal::from(leverage) - BigDecimal::one())
                        / BigDecimal::from(open_price),
                );

                let token_market = self.get_market_by(&buy_token);
                (token_market, borrow_amount)
            }
            _ => panic!("Borrow amount calculation only for the 'Long' and 'Short' order types"),
        };

        ext_market::ext(token_market)
            .with_static_gas(GAS_FOR_BORROW)
            .borrow(borrow_amount)
            .then(
                ext_self::ext(env::current_account_id())
                    .with_unused_gas_weight(100)
                    .borrow_callback(borrow_amount),
            )
            .into()
    }

    #[private]
    pub fn borrow_callback(&mut self, borrow_amount: U128) -> PromiseOrValue<WBalance> {
        require!(is_promise_success(), "Contract failed to borrow assets");
        PromiseOrValue::Value(borrow_amount)
    }
    #[private]
    pub fn add_order_from_string(&mut self, account_id: AccountId, order: String) {
        self.order_nonce += 1;
        let order_id = self.order_nonce;
        let order: Order = serde_json::from_str(order.as_str()).unwrap();
        self.add_or_update_order(&account_id, order, order_id);
    }

    #[payable]
    pub fn create_take_profit_order(
        &mut self,
        order_id: U128,
        close_price: U128,
        left_point: i32,
        right_point: i32,
        market_data: MarketData,
    ) -> PromiseOrValue<bool> {
        require!(
            Some(signer_account_id()) == self.get_account_by(order_id.0),
            "You do not have permission for this action."
        );

        let parent_order = self.get_order_by(order_id.0).unwrap();
        require!(
            parent_order.order_type == OrderType::Long
                || parent_order.order_type == OrderType::Short,
            "Invalid parent order type."
        );

        let sell_token_price = self.view_price(parent_order.sell_token.clone());
        require!(
            BigDecimal::from(sell_token_price.value) != BigDecimal::zero(),
            "Sell token price cannot be zero"
        );

        let buy_token_price = self.view_price(parent_order.buy_token.clone());
        require!(
            BigDecimal::from(buy_token_price.value) != BigDecimal::zero(),
            "Buy token price cannot be zero"
        );

        let price_points = (left_point, right_point);
        match parent_order.status {
            OrderStatus::Pending => self.create_take_profit_order_when_parent_pending(
                order_id,
                price_points,
                parent_order,
                close_price,
                sell_token_price,
                buy_token_price,
                market_data,
            ),
            OrderStatus::Executed => self.create_take_profit_order_when_parent_executed(
                order_id,
                price_points,
                parent_order,
                close_price,
                sell_token_price,
                buy_token_price,
                market_data,
            ),
            _ => {
                panic!("Take profit order can't be created at the current moment.");
            }
        }
    }

    #[private]
    pub fn take_profit_liquidity_callback(
        &mut self,
        order_id: U128,
        amount: U128,
        parent_order: Order,
        reward_executor: bool,
    ) {
        require!(is_promise_success(), "Some problems with liquidity adding.");

        match env::promise_result(0) {
            PromiseResult::Successful(result) => {
                let lpt_id = serde_json::from_slice::<String>(&result).unwrap();
                if let Some(current_tpo) = self.take_profit_orders.get(&(order_id.0 as u64)) {
                    let mut order = current_tpo.1;
                    order.lpt_id = lpt_id.clone();
                    order.status = OrderStatus::Pending;

                    let return_amounts = ReturnAmounts {
                        amount_buy_token: U128(0_u128),
                        amount_sell_token: U128(0_u128),
                    };

                    self.take_profit_orders.insert(
                        &(order_id.0 as u64),
                        &(current_tpo.0, order.clone(), return_amounts),
                    );

                    Event::UpdateTakeProfitOrderEvent {
                        order_id,
                        parent_order_type: parent_order.order_type,
                        order_type: order.order_type.clone(),
                        order_status: order.status,
                        lpt_id,
                        close_price: WRatio::from(order.open_or_close_price),
                        sell_token: order.sell_token.to_string(),
                        buy_token: order.sell_token.to_string(),
                        sell_token_price: order.sell_token_price.value,
                        buy_token_price: order.buy_token_price.value,
                        pool_id: self
                            .get_trade_pair(&order.sell_token, &order.buy_token)
                            .pool_id,
                    }
                    .emit();

                    self.pending_orders_data.push_back(PendingOrderData {
                        order_id,
                        order_type: order.order_type,
                    });
                }

                if reward_executor {
                    let executor_reward_in_near = env::used_gas().0 as Balance * 2_u128;
                    Promise::new(env::signer_account_id()).transfer(executor_reward_in_near);
                }
            }
            _ => {
                let order = self.get_order_by(order_id.0).unwrap();
                let token_id = if order.order_type == OrderType::Long {
                    order.buy_token
                } else {
                    order.sell_token
                };

                near_sdk::log!("No liquidity was added. We return deposits from DEX");

                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_static_gas(Gas::ONE_TERA * 45_u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .withdraw_asset(token_id.clone(), amount)
                    .then(
                        ext_self::ext(current_account_id())
                            .with_static_gas(Gas::ONE_TERA * 2u64)
                            .with_attached_deposit(NO_DEPOSIT)
                            .withdraw_asset_callback(token_id, amount),
                    );
            }
        };
    }
}

impl Contract {
    pub fn create_limit_order(
        &mut self,
        order_type: OrderType,
        // left point for add_liquidity acquired via getPointByPrice
        left_point: i32,
        // right point for add_liquidity acquired via getPointByPrice
        right_point: i32,
        amount: WBalance,
        sell_token: AccountId,
        sell_token_price: Price,
        buy_token: AccountId,
        buy_token_price: Price,
        buy_or_sell_price: U128,
    ) -> PromiseOrValue<WBalance> {
        let order = Order {
            status: OrderStatus::Pending,
            order_type,
            amount: Balance::from(amount),
            sell_token,
            buy_token,
            leverage: BigDecimal::one(),
            sell_token_price,
            buy_token_price,
            open_or_close_price: BigDecimal::from(buy_or_sell_price),
            block: env::block_height(),
            timestamp_ms: env::block_timestamp_ms(),
            lpt_id: "".to_string(),
            history_data: Default::default(),
        };
        // calculating the range for the liquidity to be added into
        // consider the smallest gap is point_delta for given pool
        let (amount_x, amount_y, token_to_add_liquidity) = if order.order_type == OrderType::Buy {
            let (sell_token_decimals, _) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);

            let amount_x = self.from_protocol_to_token_decimals(amount, sell_token_decimals);
            // (amount_x, amount_y, token_id)
            (amount_x, U128::from(0), order.sell_token.clone())
        } else {
            let (_, buy_token_decimals) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);

            let amount_y = self.from_protocol_to_token_decimals(amount, buy_token_decimals);
            // (amount_x, amount_y, token_id)
            (U128::from(0), amount_y, order.buy_token.clone())
        };

        self.add_liquidity(
            order,
            token_to_add_liquidity,
            amount,
            left_point,
            right_point,
            amount_x,
            amount_y,
        )
    }

    pub fn create_leverage_order(
        &mut self,
        order_type: OrderType,
        // left point for add_liquidity acquired via getPointByPrice
        left_point: i32,
        // right point for add_liquidity acquired via getPointByPrice
        right_point: i32,
        amount: WBalance,
        sell_token: AccountId,
        sell_token_price: Price,
        buy_token: AccountId,
        buy_token_price: Price,
        leverage: U128,
        open_price: U128,
    ) -> PromiseOrValue<WBalance> {
        require!(
            BigDecimal::from(leverage) > BigDecimal::one(),
            "Incorrect leverage for order type 'Long' and 'Short'"
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
            open_or_close_price: BigDecimal::from(open_price),
            block: env::block_height(),
            timestamp_ms: env::block_timestamp_ms(),
            lpt_id: "".to_string(),
            history_data: Default::default(),
        };
        // calculating the range for the liquidity to be added into
        // consider the smallest gap is point_delta for given pool
        let (deposit_amount, amount_x, amount_y, token_to_add_liquidity) = if order.order_type
            == OrderType::Long
        {
            let total_amount =
                U128::from(BigDecimal::from(U128::from(order.amount)) * order.leverage);

            let (sell_token_decimals, _) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);

            let amount_x = self.from_protocol_to_token_decimals(total_amount, sell_token_decimals);
            // (deposit_amount, amount_x, amount_y, token_id)
            (amount_x, amount_x, U128::from(0), order.sell_token.clone())
        } else {
            let total_amount = U128::from(
                BigDecimal::from(U128::from(order.amount)) * (order.leverage - BigDecimal::one())
                    / order.open_or_close_price,
            );

            let (_, buy_token_decimals) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);

            let amount_y = self.from_protocol_to_token_decimals(total_amount, buy_token_decimals);
            // (deposit_amount, amount_x, amount_y, token_id)
            (amount_y, U128::from(0), amount_y, order.buy_token.clone())
        };

        self.add_liquidity(
            order,
            token_to_add_liquidity,
            deposit_amount,
            left_point,
            right_point,
            amount_x,
            amount_y,
        )
    }

    pub fn add_or_update_order(&mut self, account_id: &AccountId, order: Order, order_id: u64) {
        let pair_id = PairId {
            sell_token: order.sell_token.clone(),
            buy_token: order.buy_token.clone(),
        };

        let mut user_orders_by_id = self.orders.get(account_id).unwrap_or_default();
        user_orders_by_id.insert(order_id, order.clone());
        self.orders.insert(account_id, &user_orders_by_id);

        let mut pair_orders_by_id = self.orders_per_pair_view.get(&pair_id).unwrap_or_default();
        pair_orders_by_id.insert(order_id, order);
        self.orders_per_pair_view
            .insert(&pair_id, &pair_orders_by_id);
    }

    pub fn create_take_profit_order_when_parent_pending(
        &mut self,
        order_id: U128,
        price_points: PricePoints,
        parent_order: Order,
        price: U128,
        sell_token_price: Price,
        buy_token_price: Price,
        market_data: MarketData,
    ) -> PromiseOrValue<bool> {
        let take_profit_order_amount = if parent_order.order_type == OrderType::Long {
            // full amount (buy token) open 'long' position
            U128::from(
                BigDecimal::from(U128::from(parent_order.amount)) * parent_order.leverage
                    / parent_order.open_or_close_price,
            )
        } else {
            // part amount (sell token) open "short" position
            let borrow_amount = BigDecimal::from(U128::from(parent_order.amount))
                * (parent_order.leverage - BigDecimal::one())
                / parent_order.open_or_close_price;

            let borrow_fee_amount =
                BigDecimal::from(self.get_borrow_fee_amount(parent_order.clone(), market_data));

            U128::from((borrow_amount + borrow_fee_amount) * BigDecimal::from(price))
        };

        let order = Order {
            status: OrderStatus::PendingOrderExecute,
            order_type: OrderType::TakeProfit,
            amount: take_profit_order_amount.0,
            sell_token: parent_order.sell_token.clone(),
            buy_token: parent_order.buy_token.clone(),
            leverage: BigDecimal::one(),
            sell_token_price,
            buy_token_price,
            open_or_close_price: BigDecimal::from(price),
            block: block_height(),
            timestamp_ms: block_timestamp_ms(),
            lpt_id: "".to_string(),
            history_data: Default::default(),
        };

        let return_amounts = ReturnAmounts {
            amount_buy_token: U128(0_u128),
            amount_sell_token: U128(0_u128),
        };

        self.take_profit_orders
            .insert(&(order_id.0 as u64), &(price_points, order, return_amounts));

        Event::CreateTakeProfitOrderEvent {
            order_id,
            order_type: OrderType::TakeProfit,
            lpt_id: "".to_string(),
            close_price: price,
            parent_order_type: parent_order.order_type,
            pool_id: self
                .get_trade_pair(&parent_order.sell_token, &parent_order.buy_token)
                .pool_id,
        }
        .emit();
        PromiseOrValue::Value(true)
    }

    pub fn create_take_profit_order_when_parent_executed(
        &mut self,
        order_id: U128,
        price_points: PricePoints,
        parent_order: Order,
        price: U128,
        sell_token_price: Price,
        buy_token_price: Price,
        market_data: MarketData,
    ) -> PromiseOrValue<bool> {
        if self.take_profit_orders.get(&(order_id.0 as u64)).is_some() {
            panic!(
                "The 'Take Profit' order has already been created. You cannot modify an existing 'Take Profit' order if the position is already open"
            )
        } else {
            let pv = self.create_take_profit_order_when_parent_pending(
                order_id,
                price_points,
                parent_order.clone(),
                price,
                sell_token_price,
                buy_token_price,
                market_data,
            );
            let tpo_info = self.take_profit_orders.get(&(order_id.0 as u64)).unwrap();
            self.set_take_profit_order_pending(order_id, parent_order, tpo_info, false);
            pv
        }
    }

    pub fn set_take_profit_order_pending(
        &mut self,
        order_id: U128,
        parent_order: Order,
        take_profit_order: (PricePoints, Order, ReturnAmounts),
        reward_executor: bool,
    ) {
        // if the 'Take Profit' order is created when the parental order has not yet been executed
        // and if the parent order type is 'Short' we are updating the amount in 'Take Profit' order
        let order = if reward_executor && parent_order.order_type == OrderType::Short {
            let current_timestamp_ms = env::block_timestamp_ms();

            let borrow_amount = BigDecimal::from(U128::from(parent_order.amount))
                * (parent_order.leverage - BigDecimal::one())
                / parent_order.open_or_close_price;

            let previous_borrow_fee_amount = BigDecimal::from(U128(take_profit_order.1.amount))
                / take_profit_order.1.open_or_close_price
                - borrow_amount;

            let previous_borrow_period = ((take_profit_order.1.timestamp_ms
                - parent_order.timestamp_ms) as f64
                / MILLISECONDS_PER_DAY as f64)
                .ceil();

            let current_borrow_period = ((current_timestamp_ms - parent_order.timestamp_ms) as f64
                / MILLISECONDS_PER_DAY as f64)
                .ceil();

            let current_borrow_fee_amount = previous_borrow_fee_amount
                / BigDecimal::from(U128(previous_borrow_period as u128))
                * BigDecimal::from(U128(current_borrow_period as u128));

            Order {
                amount: U128::from(
                    (borrow_amount + current_borrow_fee_amount)
                        * take_profit_order.1.open_or_close_price,
                )
                .0,
                ..take_profit_order.1
            }
        } else {
            take_profit_order.1
        };

        let (amount_x, amount_y, token_to_add_liquidity) =
            if parent_order.order_type == OrderType::Long {
                let (_, buy_token_decimals) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);

                let amount_y =
                    self.from_protocol_to_token_decimals(U128(order.amount), buy_token_decimals);
                // (amount_x, amount_y, token_id)
                (U128::from(0), amount_y, order.buy_token.clone())
            } else {
                let (sell_token_decimals, _) =
                    self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);

                let amount_x =
                    self.from_protocol_to_token_decimals(U128(order.amount), sell_token_decimals);
                // (amount_x, amount_y, token_id)
                (amount_x, U128::from(0), order.sell_token.clone())
            };

        let min_amount_x = U128::from(0);
        let min_amount_y = U128::from(0);

        ext_token::ext(token_to_add_liquidity)
            .with_static_gas(Gas::ONE_TERA * 35u64)
            .with_attached_deposit(near_sdk::ONE_YOCTO)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                U128(order.amount),
                None,
                "\"Deposit\"".to_string(),
            )
            .then(
                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_static_gas(Gas::ONE_TERA * 50_u64)
                    .with_unused_gas_weight(2_u64)
                    .add_liquidity(
                        self.get_trade_pair(&order.sell_token, &order.buy_token)
                            .pool_id,
                        take_profit_order.0 .0,
                        take_profit_order.0 .1,
                        amount_x,
                        amount_y,
                        min_amount_x,
                        min_amount_y,
                    ),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 50_u64)
                    .take_profit_liquidity_callback(
                        order_id,
                        U128(order.amount),
                        parent_order,
                        reward_executor,
                    ),
            );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool, block_timestamp: Option<u64>) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("margin.nearland.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
            .block_index(103930916)
            .block_timestamp(block_timestamp.unwrap_or(1))
            .is_view(is_view)
            .build()
    }
    // lend land
    fn get_current_day_in_nanoseconds(day: u64) -> Option<u64> {
        let nanoseconds_in_one_millisecond = 1_000_000;
        Some(MILLISECONDS_PER_DAY * day * nanoseconds_in_one_millisecond)
    }

    fn get_market_data() -> MarketData {
        MarketData {
            underlying_token: AccountId::new_unchecked("usdt.qa.v1.nearlend.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        }
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
        let context = get_context(false, None);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_id = PairId {
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        };

        contract.set_balance(&alice(), &pair_id.sell_token, 10_u128.pow(30));

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
            let order = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930910, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\",\"history_data\":null}".to_string();
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
    fn test_create_take_profit_order() {
        let context = get_context(false, None);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let market_data = get_market_data();

        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 18,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128(3 * 10_u128.pow(24)), // current price token
            },
        );

        let order_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2500000000000000000000000\",\"block\":103930910, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\",\"history_data\":null}".to_string();
        contract.add_order_from_string(alice(), order_string);

        let new_price = U128(24500000000000000000000000);
        let left_point = -9860;
        let right_point = -9820;
        contract.create_take_profit_order(U128(1), new_price, left_point, right_point, market_data);

        let tpo = contract.take_profit_orders.get(&1).unwrap();
        assert_eq!(tpo.1.status, OrderStatus::PendingOrderExecute);
        assert_eq!(WBigDecimal::from(tpo.1.open_or_close_price), new_price);
    }

    #[test]
    #[should_panic(expected = "You do not have permission for this action")]
    fn test_create_take_profit_order_without_order() {
        let context = get_context(false, None);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let market_data = get_market_data();

        let order_id: u128 = 33;
        assert_eq!(contract.get_order_by(order_id), None);

        let new_price = U128(30500000000000000000000000);
        let left_point = -9860;
        let right_point = -9820;
        contract.create_take_profit_order(
            U128(order_id),
            new_price,
            left_point,
            right_point,
            market_data,
        );
    }

    #[test]
    fn test_update_take_profit_order_price_when_parent_pending() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let market_data = get_market_data();

        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 18,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128(3 * 10_u128.pow(24)), // current price token
            },
        );

        let order_string = "{\"status\":\"Pending\",\"order_type\":\"Short\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2500000000000000000000000\",\"block\":103930910, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\",\"history_data\":null}".to_string();
        contract.add_order_from_string(alice(), order_string);

        let order_id: u128 = 1;
        let new_price = U128(2450000000000000000000000);
        let left_point = -9860;
        let right_point = -9820;
        contract.create_take_profit_order(
            U128(order_id),
            new_price,
            left_point,
            right_point,
            market_data.clone(),
        );

        let tpo = contract.take_profit_orders.get(&(order_id as u64)).unwrap();
        assert_eq!(tpo.1.status, OrderStatus::PendingOrderExecute);
        assert_eq!(WBigDecimal::from(tpo.1.open_or_close_price), new_price);

        let new_price = U128(2350000000000000000000000);
        let left_point = -8040;
        let right_point = -8000;

        contract.create_take_profit_order(
            U128(order_id),
            new_price,
            left_point,
            right_point,
            market_data,
        );

        let tpo = contract.take_profit_orders.get(&(order_id as u64)).unwrap();
        assert_eq!(WBigDecimal::from(tpo.1.open_or_close_price), new_price);
        assert_eq!(tpo.0 .0, left_point);
        assert_eq!(tpo.0 .1, right_point);
    }

    #[test]
    #[should_panic(
        expected = "The 'Take Profit' order has already been created. You cannot modify an existing 'Take Profit' order if the position is already open"
    )]
    fn test_update_take_profit_order_price_when_parent_executed() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let market_data = get_market_data();

        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 18,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128(3 * 10_u128.pow(24)), // current price token
            },
        );

        let order_string = "{\"status\":\"Executed\",\"order_type\":\"Short\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2500000000000000000000000\",\"block\":103930910, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\",\"history_data\":null}".to_string();
        contract.add_order_from_string(alice(), order_string);

        let order_id: u128 = 1;
        let new_price = U128(2450000000000000000000000);
        let left_point = -9860;
        let right_point = -9820;

        contract.create_take_profit_order(
            U128(order_id),
            new_price,
            left_point,
            right_point,
            market_data.clone(),
        );

        let (price_points, mut tp_order, return_amounts) =
            contract.take_profit_orders.get(&(order_id as u64)).unwrap();

        tp_order.status = OrderStatus::Pending;

        contract.take_profit_orders.insert(
            &(order_id as u64),
            &(price_points, tp_order, return_amounts),
        );

        let (_, tp_order, _) = contract.take_profit_orders.get(&(order_id as u64)).unwrap();

        assert_eq!(tp_order.status, OrderStatus::Pending);

        let new_price = U128(2350000000000000000000000);
        let left_point = -8040;
        let right_point = -8000;

        contract.create_take_profit_order(
            U128(order_id),
            new_price,
            left_point,
            right_point,
            market_data,
        );
    }
}
