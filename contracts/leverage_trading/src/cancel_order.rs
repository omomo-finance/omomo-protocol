use crate::big_decimal::BigDecimal;
use crate::common::Event;
use crate::execute_order::INACCURACY_RATE;
use crate::ref_finance::ext_ref_finance;
use crate::ref_finance::{Action, Swap};
use crate::utils::{ext_market, ext_token};
use crate::utils::{DAYS_PER_YEAR, MILLISECONDS_PER_DAY};
use crate::*;
use near_sdk::env::{current_account_id, prepaid_gas, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, serde_json, Gas, PromiseResult, ONE_YOCTO};

const CANCEL_ORDER_GAS: Gas = Gas(160_000_000_000_000);
const GAS_FOR_BORROW: Gas = Gas(200_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_callback(&self, order_id: U128, order: Order);
    fn remove_liquidity_for_cancel_leverage_order_callback(
        &mut self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    );
    fn close_order_swap_callback(
        &self,
        order_id: U128,
        order: Order,
        token_amount: U128,
        protocol_profit_amount: Option<BigDecimal>,
        history_data: Option<HistoryData>,
    );
    fn cancel_order_swap_callback(
        &self,
        order_id: U128,
        order: Order,
        token_amount: U128,
        history_data: Option<HistoryData>,
    );
    fn liquidate_order_swap_callback(
        &self,
        order_id: U128,
        order: Order,
        price_impact: U128,
        order_action: OrderAction,
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
    fn get_pool_callback(
        &self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    );
    fn get_liquidity_callback(
        &self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    );
    fn repay_callback(
        &self,
        token_borrow: AccountId,
        token_market: AccountId,
        repay_amount: U128,
    ) -> PromiseOrValue<U128>;
    fn withdraw_callback(&mut self, account_id: AccountId, token: AccountId, amount: U128);
}

#[near_bindgen]
impl Contract {
    pub fn cancel_order(
        &mut self,
        order_id: U128,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    ) {
        require!(
            prepaid_gas() >= CANCEL_ORDER_GAS,
            "Not enough gas for method: 'Cancel order'"
        );

        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        match order.order_type {
            OrderType::Buy | OrderType::Sell => {
                self.cancel_limit_order(order_id, order);
            }
            OrderType::Long | OrderType::Short => {
                self.cancel_leverage_order_or_close_leverage_position(
                    order_id,
                    order,
                    current_buy_token_price,
                    slippage_price_impact,
                );
            }
            OrderType::TakeProfit => {}
        }
    }

    #[private]
    pub fn get_liquidity_callback(
        &mut self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    ) {
        let liquidity_info: Liquidity = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<Liquidity>(&val) {
                    pool
                } else {
                    panic!("Some problem with liquidity parsing")
                }
            }
            _ => panic!("DEX not found liquidity or some problem with pool, please contact with DEX to support"),
        };

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 90_u64)
            .remove_liquidity(
                order.lpt_id.to_string(),
                liquidity_info.amount,
                U128(0_u128),
                U128(0_u128),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 165_u64)
                    .with_unused_gas_weight(2_u64)
                    .remove_liquidity_for_cancel_leverage_order_callback(
                        order_id,
                        order,
                        current_buy_token_price,
                        slippage_price_impact,
                    ),
            );
    }

    #[private]
    pub fn remove_liquidity_for_cancel_leverage_order_callback(
        &mut self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    ) {
        let (amount_x, amount_y) = match env::promise_result(0) {
            PromiseResult::Successful(amounts) => {
                if let Ok((amount_x, amount_y)) = serde_json::from_slice::<(U128, U128)>(&amounts) {
                    let (sell_token_decimals, buy_token_decimals) =
                        self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);

                    let amount_x =
                        self.from_token_to_protocol_decimals(amount_x.0, sell_token_decimals);

                    let amount_y =
                        self.from_token_to_protocol_decimals(amount_y.0, buy_token_decimals);

                    (amount_x, amount_y)
                } else {
                    panic!("Some problems with the parsing result remove liquidity")
                }
            }
            _ => panic!("Some problem with remove liquidity"),
        };

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
                        Some(amount_x),
                        Some(amount_y),
                        current_buy_token_price,
                        slippage_price_impact,
                    ),
            );
    }

    #[private]
    pub fn close_order_swap_callback(
        &mut self,
        order_id: U128,
        order: Order,
        token_amount: U128,
        protocol_profit_amount: Option<BigDecimal>,
        history_data: Option<HistoryData>,
    ) {
        require!(is_promise_success(), "Some problem with swap");

        self.final_close_order(
            order_id,
            order,
            token_amount,
            protocol_profit_amount,
            history_data,
        );
    }

    #[private]
    pub fn cancel_order_swap_callback(
        &mut self,
        order_id: U128,
        order: Order,
        token_amount: U128,
        history_data: Option<HistoryData>,
    ) {
        require!(is_promise_success(), "Some problem with swap");
        self.final_cancel_order(order_id, order, token_amount, history_data);
    }

    #[private]
    pub fn market_data_callback(
        &mut self,
        order_id: U128,
        order: Order,
        amount_x: Option<U128>,
        amount_y: Option<U128>,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    ) {
        let market_data = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(data) = near_sdk::serde_json::from_slice::<MarketData>(&val) {
                    data
                } else {
                    panic!("Failed parse market data")
                }
            }
            _ => panic!("Failed to get market data"),
        };

        if order.status == OrderStatus::Pending {
            if order.order_type == OrderType::Long && amount_y.unwrap() == U128(0_u128) {
                let borrow_fee_amount = BigDecimal::from(
                    self.get_borrow_fee_amount(order.clone(), market_data.clone()),
                );

                let amount_increase_balance = if order.order_type == OrderType::Long {
                    U128::from(BigDecimal::from(U128(order.amount)) - borrow_fee_amount)
                } else {
                    U128::from(
                        BigDecimal::from(U128(order.amount))
                            - (borrow_fee_amount
                                * BigDecimal::from(current_buy_token_price)
                                * (BigDecimal::one() + BigDecimal::from(slippage_price_impact))),
                    )
                };

                let history_data = Some(HistoryData {
                    fee: U128::from(self.get_borrow_fee(order.clone(), market_data.clone())),
                    pnl: PnLView {
                        is_profit: false,
                        amount: U128::from(
                            BigDecimal::from(U128(order.amount))
                                - BigDecimal::from(
                                    self.get_borrow_fee_amount(order.clone(), market_data),
                                ),
                        ),
                    },
                    executed: U128(0_u128),
                });

                self.final_cancel_order(order_id, order, amount_increase_balance, history_data);
            } else {
                self.swap_to_cancel_leverage_order(
                    order_id,
                    order,
                    amount_x,
                    amount_y,
                    current_buy_token_price,
                    slippage_price_impact,
                    market_data,
                );
            };
        } else {
            self.swap_to_close_leverage_order(
                order_id,
                order,
                amount_x,
                amount_y,
                current_buy_token_price,
                slippage_price_impact,
                market_data,
            )
        }
    }

    /// Called by a separate transaction with UI
    pub fn repay(&self, order_id: U128, market_data: MarketData) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        require!(
            order.status == OrderStatus::Canceled || order.status == OrderStatus::Closed,
            "Error. The order must be in the status 'Canceled' or 'Closed'"
        );

        let (token_borrow, token_market) = if order.order_type == OrderType::Long {
            (
                order.sell_token.clone(),
                self.get_market_by(&order.sell_token),
            )
        } else {
            (
                order.buy_token.clone(),
                self.get_market_by(&order.buy_token),
            )
        };

        let repay_amount = self.get_amount_to_repay(order, market_data);

        ext_token::ext(token_borrow.clone())
            .with_static_gas(GAS_FOR_BORROW)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer_call(
                token_market.clone(),
                repay_amount,
                None,
                "\"Repay\"".to_string(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 3_u64)
                    .repay_callback(token_borrow, token_market, repay_amount),
            );
    }

    #[private]
    pub fn repay_callback(
        &self,
        token_borrow: AccountId,
        token_market: AccountId,
        repay_amount: U128,
    ) -> PromiseOrValue<U128> {
        require!(is_promise_success(), "Failed to repay assets");

        Event::RepayEvent {
            token_borrow,
            token_market,
            repay_amount,
        }
        .emit();

        PromiseOrValue::Value(repay_amount)
    }
}

impl Contract {
    pub fn cancel_leverage_order_or_close_leverage_position(
        &mut self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    ) {
        match order.status {
            OrderStatus::Pending => self.cancel_leverage_order(
                order_id,
                order,
                current_buy_token_price,
                slippage_price_impact,
            ),
            OrderStatus::Executed => self.close_leverage_position(
                order_id,
                order,
                current_buy_token_price,
                slippage_price_impact,
            ),
            _ => panic!("Error. Order status has to be 'Pending' or 'Executed'"),
        }
    }

    pub fn cancel_leverage_order(
        &mut self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    ) {
        if self.take_profit_orders.get(&(order_id.0 as u64)).is_some() {
            self.take_profit_orders.remove(&(order_id.0 as u64));
            Event::CancelTakeProfitOrderEvent { order_id }.emit();
        };

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas::ONE_TERA * 5_u64)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 275_u64)
                    .with_unused_gas_weight(2_u64)
                    .get_liquidity_callback(
                        order_id,
                        order,
                        current_buy_token_price,
                        slippage_price_impact,
                    ),
            );
    }

    pub fn close_leverage_position(
        &mut self,
        order_id: U128,
        order: Order,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
    ) {
        if self.take_profit_orders.get(&(order_id.0 as u64)).is_some() {
            self.cancel_take_profit_order(
                order_id,
                Some(OrderAction::Close {
                    order_id,
                    order: Box::new(order),
                    current_buy_token_price,
                    slippage_price_impact,
                }),
            );
        } else {
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
    }

    pub fn swap_to_close_leverage_order(
        &mut self,
        order_id: U128,
        order: Order,
        amount_x: Option<U128>,
        amount_y: Option<U128>,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
        market_data: MarketData,
    ) {
        let (swap_amount, input_token, output_token) = if order.order_type == OrderType::Long {
            self.get_data_to_swap_for_long(order.clone(), amount_y)
        } else {
            self.get_data_to_swap_for_short(
                order.clone(),
                amount_x,
                current_buy_token_price,
                slippage_price_impact,
                market_data.clone(),
            )
        };

        let borrow_fee_amount =
            BigDecimal::from(self.get_borrow_fee_amount(order.clone(), market_data.clone()));

        let (amount_increase_balance, protocol_profit_amount, history_data) = if order.order_type
            == OrderType::Long
        // flow for 'Long'
        {
            let borrow_amount =
                BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one());

            let amount_after_swap = BigDecimal::from(swap_amount)
                * BigDecimal::from(current_buy_token_price)
                * (BigDecimal::one() - BigDecimal::from(slippage_price_impact));

            let close_amount = amount_after_swap - borrow_amount - borrow_fee_amount;
            let profit = close_amount > BigDecimal::from(U128(order.amount));

            // close 'Long' position with profit
            if profit {
                let protocol_fee = BigDecimal::from(self.get_protocol_fee());
                let protocol_profit_amount =
                    (close_amount - BigDecimal::from(U128(order.amount))) * protocol_fee;

                let amount_increase_balance = U128::from(close_amount - protocol_profit_amount);

                let history_data = Some(HistoryData {
                    fee: U128::from(
                        protocol_fee
                            + self.get_borrow_fee(order.clone(), market_data)
                            + BigDecimal::from(self.get_swap_fee(&order)),
                    ),
                    pnl: PnLView {
                        is_profit: true,
                        amount: U128::from(
                            BigDecimal::from(amount_increase_balance)
                                - BigDecimal::from(U128(order.amount)),
                        ),
                    },
                    executed: U128(0_u128),
                });

                (
                    amount_increase_balance,
                    Some(protocol_profit_amount),
                    history_data,
                )
            // close 'Long' position with loss
            } else {
                let amount_increase_balance = U128::from(close_amount);

                let history_data = Some(HistoryData {
                    fee: U128::from(
                        self.get_borrow_fee(order.clone(), market_data)
                            + BigDecimal::from(self.get_swap_fee(&order)),
                    ),
                    pnl: PnLView {
                        is_profit: false,
                        amount: U128::from(
                            BigDecimal::from(U128(order.amount))
                                - BigDecimal::from(amount_increase_balance),
                        ),
                    },
                    executed: U128(0_u128),
                });

                (amount_increase_balance, None, history_data)
            }
        // flow for 'Short'
        } else {
            let borrow_amount = BigDecimal::from(U128(order.amount))
                * (order.leverage - BigDecimal::one())
                / order.open_or_close_price;

            let position_amount = borrow_amount * order.open_or_close_price;

            let close_amount = (BigDecimal::from(U128(order.amount)) + position_amount)
                - BigDecimal::from(swap_amount);

            let profit = close_amount > BigDecimal::from(U128(order.amount));
            // close 'Short' position with profit
            if profit {
                let protocol_fee = BigDecimal::from(self.get_protocol_fee());
                let protocol_profit_amount =
                    (close_amount - BigDecimal::from(U128(order.amount))) * protocol_fee;

                let amount_increase_balance = U128::from(
                    (BigDecimal::from(U128(order.amount)) + position_amount)
                        - BigDecimal::from(swap_amount)
                        - protocol_profit_amount,
                );

                let history_data = Some(HistoryData {
                    fee: U128::from(
                        protocol_fee
                            + self.get_borrow_fee(order.clone(), market_data)
                            + BigDecimal::from(self.get_swap_fee(&order)),
                    ),
                    pnl: PnLView {
                        is_profit: true,
                        amount: U128::from(
                            BigDecimal::from(amount_increase_balance)
                                - BigDecimal::from(U128(order.amount)),
                        ),
                    },
                    executed: U128(0_u128),
                });

                (
                    amount_increase_balance,
                    Some(protocol_profit_amount),
                    history_data,
                )
            // close 'Short' position with loss
            } else {
                let amount_increase_balance = U128::from(
                    (BigDecimal::from(U128(order.amount)) + position_amount)
                        - BigDecimal::from(swap_amount),
                );

                let history_data = Some(HistoryData {
                    fee: U128::from(
                        self.get_borrow_fee(order.clone(), market_data)
                            + BigDecimal::from(self.get_swap_fee(&order)),
                    ),
                    pnl: PnLView {
                        is_profit: false,
                        amount: U128::from(
                            BigDecimal::from(U128(order.amount))
                                - BigDecimal::from(amount_increase_balance),
                        ),
                    },
                    executed: U128(0_u128),
                });

                (amount_increase_balance, None, history_data)
            }
        };

        let action = Action::SwapAction {
            Swap: Swap {
                pool_ids: vec![
                    self.get_trade_pair(&order.sell_token, &order.buy_token)
                        .pool_id,
                ],
                output_token,
                min_output_amount: WBalance::from(0),
            },
        };

        let token_decimals = if input_token == order.sell_token {
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token)
                .0
        } else {
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token)
                .1
        };

        let swap_amount = self.from_protocol_to_token_decimals(swap_amount, token_decimals);

        ext_token::ext(input_token)
            .with_static_gas(Gas::ONE_TERA * 90_u64)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                swap_amount,
                Some("Swap".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 30_u64)
                    .with_unused_gas_weight(4_u64)
                    .close_order_swap_callback(
                        order_id,
                        order,
                        amount_increase_balance,
                        protocol_profit_amount,
                        history_data,
                    ),
            );
    }

    pub fn swap_to_cancel_leverage_order(
        &mut self,
        order_id: U128,
        order: Order,
        amount_x: Option<U128>,
        amount_y: Option<U128>,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
        market_data: MarketData,
    ) {
        let (swap_amount, input_token, output_token) = if order.order_type == OrderType::Long {
            self.get_data_to_swap_for_long(order.clone(), amount_y)
        } else {
            self.get_data_to_swap_for_short(
                order.clone(),
                amount_x,
                current_buy_token_price,
                slippage_price_impact,
                market_data.clone(),
            )
        };

        let borrow_fee_amount =
            BigDecimal::from(self.get_borrow_fee_amount(order.clone(), market_data.clone()));

        let amount_increase_balance = if order.order_type == OrderType::Long {
            // flow for 'Long'
            U128::from(
                BigDecimal::from(U128(order.amount))
                    - borrow_fee_amount
                    - (BigDecimal::from(swap_amount)
                        * BigDecimal::from(current_buy_token_price)
                        * BigDecimal::from(slippage_price_impact)),
            )
        // flow for 'Short'
        } else {
            U128::from(
                BigDecimal::from(U128(order.amount))
                    - (borrow_fee_amount
                        * BigDecimal::from(current_buy_token_price)
                        * (BigDecimal::one() + BigDecimal::from(slippage_price_impact))
                        + BigDecimal::from(amount_x.unwrap())
                            * BigDecimal::from(slippage_price_impact)),
            )
        };

        let history_data = Some(HistoryData {
            fee: U128::from(
                self.get_borrow_fee(order.clone(), market_data)
                    + BigDecimal::from(self.get_swap_fee(&order)),
            ),
            pnl: PnLView {
                is_profit: false,
                amount: U128::from(
                    BigDecimal::from(U128(order.amount))
                        - BigDecimal::from(amount_increase_balance),
                ),
            },
            executed: U128(0_u128),
        });

        let action = Action::SwapAction {
            Swap: Swap {
                pool_ids: vec![
                    self.get_trade_pair(&order.sell_token, &order.buy_token)
                        .pool_id,
                ],
                output_token,
                min_output_amount: WBalance::from(0),
            },
        };

        let token_decimals = if input_token == order.sell_token {
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token)
                .0
        } else {
            self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token)
                .1
        };

        let swap_amount = self.from_protocol_to_token_decimals(swap_amount, token_decimals);

        ext_token::ext(input_token)
            .with_static_gas(Gas::ONE_TERA * 90_u64)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                swap_amount,
                Some("Swap".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 30_u64)
                    .with_unused_gas_weight(4_u64)
                    .cancel_order_swap_callback(
                        order_id,
                        order,
                        amount_increase_balance,
                        history_data,
                    ),
            );
    }

    pub fn get_amounts_to_cancel(
        &self,
        order: Order,
        liquidity_info: Liquidity,
    ) -> [U128; 3_usize] {
        if order.order_type == OrderType::Long {
            let min_amount_x = U128::from(
                BigDecimal::from(U128::from(order.amount))
                    * order.leverage
                    * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
            );
            let min_amount_y = U128::from(0);

            let (sell_token_decimals, _) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
            let min_amount_x =
                self.from_protocol_to_token_decimals(min_amount_x, sell_token_decimals);

            [liquidity_info.amount, min_amount_x, min_amount_y]
        } else {
            let min_amount_x = U128::from(0);
            let min_amount_y = U128::from(
                BigDecimal::from(U128::from(order.amount))
                    * (order.leverage - BigDecimal::one())
                    * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE))
                    / order.open_or_close_price,
            );

            let (_, buy_token_decimals) =
                self.view_pair_tokens_decimals(&order.sell_token, &order.buy_token);
            let min_amount_y =
                self.from_protocol_to_token_decimals(min_amount_y, buy_token_decimals);

            [liquidity_info.amount, min_amount_x, min_amount_y]
        }
    }

    pub fn get_data_to_swap_for_long(
        &self,
        order: Order,
        amount_y: Option<U128>,
    ) -> (U128, AccountId, AccountId) {
        require!(
            order.order_type == OrderType::Long,
            "Only for order type`Long`"
        );

        let swap_amount = if order.status == OrderStatus::Pending {
            amount_y.unwrap()
        } else {
            U128::from(
                BigDecimal::from(U128::from(order.amount)) * order.leverage
                    / order.open_or_close_price,
            )
        };
        // amount, input_token, output_token
        (swap_amount, order.buy_token, order.sell_token)
    }

    pub fn get_data_to_swap_for_short(
        &self,
        order: Order,
        amount_x: Option<U128>,
        current_buy_token_price: U128,
        slippage_price_impact: U128,
        market_data: MarketData,
    ) -> (U128, AccountId, AccountId) {
        require!(
            order.order_type == OrderType::Short,
            "Only for order type`Short`"
        );

        let require_amount = if order.status == OrderStatus::Pending {
            let borrow_fee_amount =
                BigDecimal::from(self.get_borrow_fee_amount(order.clone(), market_data));

            (borrow_fee_amount * BigDecimal::from(current_buy_token_price)
                + BigDecimal::from(amount_x.unwrap()))
                * (BigDecimal::one() + BigDecimal::from(slippage_price_impact))
        } else {
            let borrow_amount = BigDecimal::from(U128(order.amount))
                * (order.leverage - BigDecimal::one())
                / order.open_or_close_price;

            let borrow_fee_amount =
                BigDecimal::from(self.get_borrow_fee_amount(order.clone(), market_data));

            (borrow_fee_amount + borrow_amount)
                * BigDecimal::from(current_buy_token_price)
                * (BigDecimal::one() + BigDecimal::from(slippage_price_impact))
        };

        let swap_fee = BigDecimal::from(self.get_swap_fee(&order));
        let swap_fee_amount = require_amount * swap_fee;
        let swap_amount = U128::from(require_amount + swap_fee_amount);
        // amount, input_token, output_token
        (swap_amount, order.sell_token, order.buy_token)
    }

    pub fn final_cancel_order(
        &mut self,
        order_id: U128,
        order: Order,
        token_amount: U128,
        history_data: Option<HistoryData>,
    ) {
        let order = Order {
            status: OrderStatus::Canceled,
            history_data,
            ..order
        };

        self.add_or_update_order(&signer_account_id(), order.clone(), order_id.0 as u64);

        self.remove_pending_order_data(PendingOrderData {
            order_id,
            order_type: order.order_type,
        });

        Event::CancelLeverageOrderEvent { order_id }.emit();

        self.increase_balance(&signer_account_id(), &order.sell_token, token_amount.0);

        self.withdraw(order.sell_token, token_amount);
    }

    pub fn final_close_order(
        &mut self,
        order_id: U128,
        order: Order,
        token_amount: U128,
        protocol_profit_amount: Option<BigDecimal>,
        history_data: Option<HistoryData>,
    ) {
        let order = Order {
            status: OrderStatus::Closed,
            history_data,
            ..order
        };

        self.add_or_update_order(&signer_account_id(), order.clone(), order_id.0 as u64);

        self.remove_pending_order_data(PendingOrderData {
            order_id,
            order_type: order.order_type,
        });

        if let Some(amount) = protocol_profit_amount {
            let current_profit = self
                .protocol_profit
                .get(&order.sell_token)
                .unwrap_or_default();

            self.protocol_profit
                .insert(&order.sell_token, &(current_profit + amount));
        }

        Event::CloseLeveragePositionEvent { order_id }.emit();

        self.increase_balance(&signer_account_id(), &order.sell_token, token_amount.0);

        self.withdraw(order.sell_token, token_amount);
    }

    pub fn get_borrow_fee(&self, order: Order, market_data: MarketData) -> BigDecimal {
        let current_timestamp_ms = env::block_timestamp_ms();

        let borrow_period = ((current_timestamp_ms - order.timestamp_ms) as f64
            / MILLISECONDS_PER_DAY as f64)
            .ceil();

        BigDecimal::from(market_data.borrow_rate_ratio)
            / BigDecimal::from(U128(DAYS_PER_YEAR as u128))
            * BigDecimal::from(U128(borrow_period as u128))
    }

    pub fn get_borrow_fee_amount(&self, order: Order, market_data: MarketData) -> U128 {
        let borrow_fee = self.get_borrow_fee(order.clone(), market_data);

        let borrow_amount = if order.order_type == OrderType::Long {
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one())
        } else {
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one())
                / order.open_or_close_price
        };

        let borrow_fee_amount = borrow_amount * borrow_fee;

        U128::from(borrow_fee_amount)
    }

    fn get_amount_to_repay(&self, order: Order, market_data: MarketData) -> U128 {
        let borrow_amount = if order.order_type == OrderType::Long {
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one())
        } else {
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one())
                / order.open_or_close_price
        };

        let borrow_fee_amount = BigDecimal::from(self.get_borrow_fee_amount(order, market_data));
        let repay_amount = borrow_amount + borrow_fee_amount;
        U128::from(repay_amount)
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::MILLISECONDS_PER_DAY;

    use super::*;

    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool, block_timestamp: Option<u64>) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("leverage.develop.v1.omomo-finance.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("alice.testnet".parse().unwrap())
            .block_index(103930920)
            .block_timestamp(block_timestamp.unwrap_or(1))
            .is_view(is_view)
            .build()
    }

    fn get_current_day_in_nanoseconds(day: u64) -> Option<u64> {
        let nanoseconds_in_one_millisecond = 1_000_000;
        Some(MILLISECONDS_PER_DAY * day * nanoseconds_in_one_millisecond)
    }

    #[test]
    fn test_order_was_canceled() {
        let current_day = get_current_day_in_nanoseconds(2); // borrow period 1 days
        let context = get_context(false, current_day);
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
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(2000000000000000000000000),
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128::from(4220000000000000000000000),
            },
        );

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":10000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3070000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930916,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#543\",\"history_data\":null}".to_string();
        contract.add_order_from_string(alice(), order1);

        let order_id = U128(1);
        let order = Order {
            status: OrderStatus::Pending,
            order_type: OrderType::Long,
            amount: 10_u128.pow(25),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            buy_token: "wrap.testnet".parse().unwrap(),
            leverage: BigDecimal::from(U128(2 * 10_u128.pow(24))),
            sell_token_price: Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(101 * 10_u128.pow(22)),
            },
            buy_token_price: Price {
                ticker_id: "near".to_string(),
                value: U128::from(307 * 10_u128.pow(22)),
            },
            open_or_close_price: BigDecimal::from(U128(1)),
            block: 105210654,
            timestamp_ms: 86400000,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#238".to_string(),
            history_data: Default::default(),
        };

        let pair_id = PairId {
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            buy_token: "wrap.testnet".parse().unwrap(),
        };

        let amount = U128::from(
            BigDecimal::from(U128(2 * 10_u128.pow(25)))
                * (BigDecimal::one() - BigDecimal::from(INACCURACY_RATE)),
        );

        contract.final_cancel_order(order_id, order, amount, None);

        let orders = contract.orders.get(&alice()).unwrap();
        let order = orders.get(&1).unwrap();

        let orders_from_pair = contract.orders_per_pair_view.get(&pair_id).unwrap();
        let order_from_pair = orders_from_pair.get(&1).unwrap();

        assert_eq!(order.status, OrderStatus::Canceled);
        assert_eq!(order_from_pair.status, order.status);
    }
}
