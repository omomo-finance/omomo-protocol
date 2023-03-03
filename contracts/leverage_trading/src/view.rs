use crate::big_decimal::{BigDecimal, WRatio};
use crate::utils::{DAYS_PER_YEAR, MILLISECONDS_PER_DAY};
use crate::*;
use near_sdk::env::signer_account_id;
use near_sdk::Gas;

#[near_bindgen]
impl Contract {
    pub fn view_order(
        &self,
        account_id: AccountId,
        order_id: U128,
        borrow_rate_ratio: WRatio,
    ) -> OrderView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {account_id} not found");
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        let swap_fee = self.get_swap_fee(&order);

        let borrow_fee = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - order.block as u128)),
        );

        OrderView {
            order_id,
            status: order.status,
            order_type: order.order_type,
            amount: U128(order.amount),
            sell_token: order.sell_token,
            sell_token_price: order.sell_token_price.value,
            buy_token: order.buy_token,
            buy_token_price: order.buy_token_price.value,
            leverage: WBigDecimal::from(order.leverage),
            borrow_fee,
            liquidation_price: self.calculate_liquidation_price(
                U128(order.amount),
                order.sell_token_price.value,
                order.buy_token_price.value,
                WBigDecimal::from(order.leverage),
                borrow_fee,
                swap_fee,
            ),
            lpt_id: order.lpt_id,
        }
    }

    pub fn calculate_pnl(
        &self,
        account_id: AccountId,
        order_id: U128,
        data: Option<MarketData>,
    ) -> PnLView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {account_id} not found");
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        match order.order_type {
            OrderType::Long => self.calculate_pnl_long_order(order, data),
            OrderType::Short => self.calculate_pnl_short_order(order, data),
            _ => panic!("PnL calculation only for 'Long' and 'Short' order types"),
        }
    }

    pub fn view_orders(
        &self,
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        borrow_rate_ratio: WRatio,
    ) -> Vec<OrderView> {
        let orders = self.orders.get(&account_id).unwrap_or_default();
        let result = orders
            .iter()
            .filter_map(|(id, order)| {
                match order.sell_token == sell_token && order.buy_token == buy_token {
                    true => {
                        let swap_fee = self.get_swap_fee(order);

                        let borrow_fee = WBigDecimal::from(
                            BigDecimal::from(borrow_rate_ratio)
                                * BigDecimal::from(U128(
                                    env::block_height() as u128 - order.block as u128,
                                )),
                        );

                        Some(OrderView {
                            order_id: U128(*id as u128),
                            status: order.status.clone(),
                            order_type: order.order_type.clone(),
                            amount: U128(order.amount),
                            sell_token: order.sell_token.clone(),
                            sell_token_price: order.sell_token_price.value,
                            buy_token: order.buy_token.clone(),
                            buy_token_price: order.buy_token_price.value,
                            leverage: WBigDecimal::from(order.leverage),
                            borrow_fee,
                            liquidation_price: self.calculate_liquidation_price(
                                U128(order.amount),
                                order.sell_token_price.value,
                                order.buy_token_price.value,
                                WBigDecimal::from(order.leverage),
                                borrow_fee,
                                swap_fee,
                            ),
                            lpt_id: order.lpt_id.clone(),
                        })
                    }
                    false => None,
                }
            })
            .collect::<Vec<OrderView>>();
        result
    }

    pub fn view_pair(&self, sell_token: &AccountId, buy_token: &AccountId) -> TradePair {
        self.supported_markets
            .get(&(sell_token.clone(), buy_token.clone()))
            .unwrap_or_else(|| panic!("Pair {sell_token}|{buy_token} not found"))
    }

    pub fn view_supported_pairs(&self) -> Vec<TradePairView> {
        let pairs = self
            .supported_markets
            .iter()
            .map(|(pair_id, trade_pair)| TradePairView {
                pair_id,
                pair_tickers_id: format!(
                    "{}-{}",
                    trade_pair.sell_ticker_id, trade_pair.buy_ticker_id
                ),
                sell_ticker_id: trade_pair.sell_ticker_id,
                sell_token: trade_pair.sell_token,
                sell_token_decimals: trade_pair.sell_token_decimals,
                sell_token_market: trade_pair.sell_token_market,
                buy_ticker_id: trade_pair.buy_ticker_id,
                buy_token: trade_pair.buy_token,
                buy_token_decimals: trade_pair.buy_token_decimals,
                buy_token_market: trade_pair.buy_token_market,
                pool_id: trade_pair.pool_id,
                max_leverage: trade_pair.max_leverage,
                swap_fee: trade_pair.swap_fee,
            })
            .collect::<Vec<TradePairView>>();

        pairs
    }

    /// Returns the balance of the given account on certain token. If the account doesn't exist will return `"0"`.
    pub fn balance_of(&self, account_id: AccountId, token: AccountId) -> WBalance {
        match self.balances.get(&account_id) {
            None => WBalance::from(0_u128),
            Some(user_balance_per_token) => {
                WBalance::from(*user_balance_per_token.get(&token).unwrap_or(&0_u128))
            }
        }
    }

    /// Returns price of the given token. If the token is not registered, will return `"0"`.
    pub fn view_price(&self, token_id: AccountId) -> Price {
        self.prices.get(&token_id).unwrap_or_else(|| {
            panic!("Price for token: {token_id} not found");
        })
    }

    pub fn cancel_order_view(
        &self,
        account_id: AccountId,
        order_id: U128,
        market_data: MarketData,
    ) -> CancelOrderView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {account_id} not found");
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        let buy_token = BigDecimal::from(U128(order.amount))
            * order.leverage
            * BigDecimal::from(order.sell_token_price.value)
            / BigDecimal::from(order.buy_token_price.value);

        let sell_token = BigDecimal::from(U128(order.amount)) * order.leverage;

        let open_or_close_price = order.buy_token_price.clone();

        let close_price = self.get_price(order.buy_token.clone());

        let calc_pnl = self.calculate_pnl(account_id, order_id, Some(market_data));

        CancelOrderView {
            buy_token_amount: WRatio::from(buy_token),
            sell_token_amount: WRatio::from(sell_token),
            open_or_close_price: open_or_close_price.value,
            close_price: WRatio::from(close_price),
            pnl: calc_pnl,
        }
    }

    pub fn view_liquidation_threshold(&self) -> U128 {
        U128(self.liquidation_threshold)
    }

    pub fn calculate_liquidation_price(
        &self,
        sell_token_amount: U128,
        sell_token_price: U128,
        buy_token_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> WBigDecimal {
        require!(
            sell_token_price != U128::from(0),
            "Sell token price cannot be zero"
        );

        require!(
            buy_token_price != U128::from(0),
            "Buy token price cannot be zero"
        );

        let collateral_usd =
            BigDecimal::from(sell_token_amount) * BigDecimal::from(sell_token_price);
        let position_amount_usd = collateral_usd * BigDecimal::from(leverage);
        let borrow_amount = collateral_usd * (BigDecimal::from(leverage) - BigDecimal::one());
        let buy_amount = position_amount_usd / BigDecimal::from(buy_token_price);

        let liquidation_price = (position_amount_usd - self.volatility_rate * collateral_usd
            + borrow_amount * BigDecimal::from(borrow_fee)
            + position_amount_usd * BigDecimal::from(swap_fee))
            / buy_amount;

        liquidation_price.into()
    }

    /// returns const gas amount required for executing orders: 50 TGas
    pub fn view_gas_for_execution(&self) -> Balance {
        Gas::ONE_TERA.0 as Balance * 50u128
    }

    pub fn view_max_position_amount(&self) -> U128 {
        U128(self.max_order_amount)
    }

    pub fn get_total_pending_orders_per_pair(&self, pair_id: &PairId) -> U128 {
        self.view_pair(&pair_id.0, &pair_id.1);

        let orders = self.orders_per_pair_view.get(pair_id).unwrap_or_default();

        let pending_orders = orders
            .iter()
            .filter_map(|(id, order)| match order.status == OrderStatus::Pending {
                true => Some((*id, order.clone())),
                false => None,
            })
            .collect::<HashMap<u64, Order>>();

        let total = pending_orders.len();
        U128(total as u128)
    }

    pub fn get_pending_orders(
        &self,
        pair_id: &PairId,
        orders_per_page: U128,
        page: U128,
    ) -> PendingOrders {
        self.view_pair(&pair_id.0, &pair_id.1);

        let orders = self.orders_per_pair_view.get(pair_id).unwrap_or_default();
        let mut pending_orders = orders
            .iter()
            .filter_map(|(id, order)| match order.status == OrderStatus::Pending {
                true => Some((*id, order.clone())),
                false => None,
            })
            .collect::<Vec<(u64, Order)>>();

        pending_orders.sort_by(|a, b| a.0.cmp(&b.0));

        let total = U128(pending_orders.len() as u128);

        let sort_pending_orders = pending_orders
            .into_iter()
            .skip((orders_per_page.0 * page.0 - orders_per_page.0) as usize)
            .take(orders_per_page.0 as usize)
            .collect();

        PendingOrders {
            data: sort_pending_orders,
            page,
            total,
        }
    }

    pub fn view_pair_tokens_decimals(
        &self,
        sell_token: &AccountId,
        buy_token: &AccountId,
    ) -> (u8, u8) {
        let pair_id = &(sell_token.clone(), buy_token.clone());
        let pair = self.supported_markets.get(pair_id).unwrap_or_else(|| {
            panic!(
                "Sell and Buy token decimals for pair {} | {} not found",
                pair_id.0, pair_id.1
            )
        });
        (pair.sell_token_decimals, pair.buy_token_decimals)
    }

    pub fn view_token_decimals(&self, token: &AccountId) -> u8 {
        let pair_id = self
            .supported_markets
            .keys()
            .find(|pair| pair.0 == token.clone() || pair.1 == token.clone());
        if let Some((sell_token, buy_token)) = pair_id {
            let (sell_token_decimals, buy_token_decimals) =
                self.view_pair_tokens_decimals(&sell_token, &buy_token);
            if token == &sell_token {
                sell_token_decimals
            } else {
                buy_token_decimals
            }
        } else {
            panic!("Token is not supported");
        }
    }

    pub fn view_pending_limit_orders_by_user(
        &self,
        account_id: AccountId,
        orders_per_page: U128,
        page: U128,
    ) -> LimitOrders {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut pending_limit_orders = orders
            .iter()
            .filter_map(|(id, order)| {
                match order.status == OrderStatus::Pending && order.leverage == BigDecimal::one() {
                    true => self.get_pending_limit_order(id, order),
                    false => None,
                }
            })
            .collect::<Vec<LimitOrderView>>();

        pending_limit_orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_orders = U128(pending_limit_orders.len() as u128);

        let sort_pending_orders = pending_limit_orders
            .into_iter()
            .skip((orders_per_page.0 * page.0 - orders_per_page.0) as usize)
            .take(orders_per_page.0 as usize)
            .collect();

        LimitOrders {
            data: sort_pending_orders,
            page,
            total_orders,
        }
    }

    pub fn view_pending_limit_orders_by_user_by_pair(
        &self,
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        orders_per_page: U128,
        page: U128,
    ) -> LimitOrders {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut pending_limit_orders = orders
            .iter()
            .filter_map(|(id, order)| {
                match order.status == OrderStatus::Pending
                    && order.leverage == BigDecimal::one()
                    && order.sell_token == sell_token
                    && order.buy_token == buy_token
                {
                    true => self.get_pending_limit_order(id, order),
                    false => None,
                }
            })
            .collect::<Vec<LimitOrderView>>();

        pending_limit_orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_orders = U128(pending_limit_orders.len() as u128);

        let sort_pending_limit_orders = pending_limit_orders
            .into_iter()
            .skip((orders_per_page.0 * page.0 - orders_per_page.0) as usize)
            .take(orders_per_page.0 as usize)
            .collect();

        LimitOrders {
            data: sort_pending_limit_orders,
            page,
            total_orders,
        }
    }

    pub fn view_opened_leverage_positions_by_user(
        &self,
        account_id: AccountId,
        market_data: Option<MarketData>,
        positions_per_page: U128,
        page: U128,
    ) -> LeveragedPositions {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut pending_limit_orders = orders
            .iter()
            .filter_map(|(id, order)| {
                match order.status == OrderStatus::Pending && order.leverage != BigDecimal::one()
                    || order.status == OrderStatus::Executed && order.leverage != BigDecimal::one()
                {
                    true => self.get_opened_leverage_position(
                        account_id.clone(),
                        id,
                        order,
                        market_data.clone(),
                    ),
                    false => None,
                }
            })
            .collect::<Vec<LeveragedPositionView>>();

        pending_limit_orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_positions = U128(pending_limit_orders.len() as u128);

        let sort_pending_orders = pending_limit_orders
            .into_iter()
            .skip((positions_per_page.0 * page.0 - positions_per_page.0) as usize)
            .take(positions_per_page.0 as usize)
            .collect();

        LeveragedPositions {
            data: sort_pending_orders,
            page,
            total_positions,
        }
    }

    pub fn view_opened_leverage_positions_by_user_by_pair(
        &self,
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        market_data: Option<MarketData>,
        positions_per_page: U128,
        page: U128,
    ) -> LeveragedPositions {
        let orders = self.orders.get(&account_id).unwrap_or_default();

        let mut pending_limit_orders = orders
            .iter()
            .filter_map(|(id, order)| {
                match order.status == OrderStatus::Pending
                    && order.leverage != BigDecimal::one()
                    && order.sell_token == sell_token
                    && order.buy_token == buy_token
                    || order.status == OrderStatus::Executed
                        && order.leverage != BigDecimal::one()
                        && order.sell_token == sell_token
                        && order.buy_token == buy_token
                {
                    true => self.get_opened_leverage_position(
                        account_id.clone(),
                        id,
                        order,
                        market_data.clone(),
                    ),
                    false => None,
                }
            })
            .collect::<Vec<LeveragedPositionView>>();

        pending_limit_orders.sort_by(|a, b| a.timestamp.cmp(&b.timestamp));

        let total_positions = U128(pending_limit_orders.len() as u128);

        let sort_pending_orders = pending_limit_orders
            .into_iter()
            .skip((positions_per_page.0 * page.0 - positions_per_page.0) as usize)
            .take(positions_per_page.0 as usize)
            .collect();

        LeveragedPositions {
            data: sort_pending_orders,
            page,
            total_positions,
        }
    }

    pub fn take_profit_order_view(&self, order_id: U128) -> Option<TakeProfitOrderView> {
        require!(
            Some(signer_account_id()) == self.get_account_by(order_id.0),
            "You have no access for this order."
        );

        if let Some((_, order)) = self.take_profit_orders.get(&(order_id.0 as u64)) {
            let trade_pair = self.view_pair(&order.sell_token, &order.buy_token);

            let pair = format!("{}/{}", trade_pair.sell_ticker_id, trade_pair.buy_ticker_id);

            let leverage_positions = self
                .orders_per_pair_view
                .get(&(trade_pair.sell_token, trade_pair.buy_token))
                .unwrap();

            let leverage_position = leverage_positions.get(&(order_id.0 as u64)).unwrap();

            let total = if order.order_type == OrderType::Long {
                BigDecimal::from(U128(leverage_position.amount)) * leverage_position.leverage
            } else {
                BigDecimal::from(U128(leverage_position.amount))
                    * (leverage_position.leverage - BigDecimal::one())
            };

            let filled = (order.status == OrderStatus::Executed).into();

            Some(TakeProfitOrderView {
                timestamp: order.timestamp_ms,
                pair,
                order_type: order.order_type.clone(),
                price: WBigDecimal::from(order.open_or_close_price),
                amount: U128(order.amount),
                filled,
                total: LowU128::from(total),
            })
        } else {
            None
        }
    }

    pub fn calculate_short_liquidation_price(
        &self,
        sell_token_amount: U128,
        buy_token_amount: U128,
        open_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> U128 {
        let sell_token_amount = BigDecimal::from(sell_token_amount);
        let buy_token_amount = BigDecimal::from(buy_token_amount);
        let open_price = BigDecimal::from(open_price);
        let leverage = BigDecimal::from(leverage);
        let borrow_fee = BigDecimal::from(borrow_fee);
        let swap_fee = BigDecimal::from(swap_fee);

        let borrow_amount = sell_token_amount * (leverage - BigDecimal::one()) / open_price;
        let borrow_period = BigDecimal::one();

        let liquidation_price = (sell_token_amount
            + self.volatility_rate * buy_token_amount * open_price
            - borrow_amount * borrow_period * borrow_fee
            - borrow_amount * swap_fee)
            / buy_token_amount;

        U128::from(liquidation_price)
    }

    pub fn calculate_long_liquidation_price(
        &self,
        sell_token_amount: U128,
        open_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> U128 {
        let sell_token_amount = BigDecimal::from(sell_token_amount);
        let open_price = BigDecimal::from(open_price);
        let leverage = BigDecimal::from(leverage);
        let borrow_fee = BigDecimal::from(borrow_fee);
        let swap_fee = BigDecimal::from(swap_fee);

        let borrow_amount = sell_token_amount * (leverage - BigDecimal::one());
        let borrow_period = BigDecimal::one();
        let days_per_year = BigDecimal::from(U128::from(
            DAYS_PER_YEAR as u128 * 10u128.pow(PROTOCOL_DECIMALS.into()),
        ));
        let buy_token_amount = (sell_token_amount + borrow_amount) / open_price;

        let liquidation_price = open_price
            - self.volatility_rate
                * (sell_token_amount
                    - borrow_amount * (borrow_period * borrow_fee / days_per_year)
                    - borrow_amount * swap_fee)
                / buy_token_amount;

        U128::from(liquidation_price)
    }

    pub fn view_order_by_id(&self, order_id: U128, borrow_rate_ratio: U128) -> Option<OrderView> {
        if let Some((_, order)) = self.get_order_by_id(order_id) {
            return Some(self.get_order_view(order_id, order, borrow_rate_ratio));
        }
        None
    }

    pub fn view_take_profit_order_by_id(
        &self,
        order_id: U128,
        borrow_rate_ratio: U128,
    ) -> Option<OrderView> {
        if let Some(order) = self.get_take_profit_order_by_id(order_id) {
            return Some(self.get_order_view(order_id, order, borrow_rate_ratio));
        }
        None
    }
}

impl Contract {
    pub fn get_order_view(
        &self,
        order_id: U128,
        order: Order,
        borrow_rate_ratio: U128,
    ) -> OrderView {
        let borrow_fee = self.calculate_borrow_fee(order.timestamp_ms, borrow_rate_ratio);
        let swap_fee = self.get_swap_fee(&order);

        let liquidation_price = match order.order_type {
            OrderType::Long => self.calculate_long_liquidation_price(
                U128::from(order.amount),
                U128::from(order.open_or_close_price),
                U128::from(order.leverage),
                borrow_fee,
                swap_fee,
            ),
            OrderType::Short => {
                let borrow_amount = BigDecimal::from(U128::from(order.amount))
                    * (order.leverage - BigDecimal::one())
                    / order.open_or_close_price;
                let buy_token_amount = BigDecimal::from(U128::from(order.amount)) * borrow_amount;

                self.calculate_short_liquidation_price(
                    U128::from(order.amount),
                    U128::from(buy_token_amount),
                    U128::from(order.open_or_close_price),
                    U128::from(order.leverage),
                    borrow_fee,
                    swap_fee,
                )
            }
            _ => U128::from(0),
        };

        OrderView {
            order_id,
            status: order.status,
            order_type: order.order_type,
            amount: U128::from(order.amount),
            sell_token: order.sell_token,
            sell_token_price: order.sell_token_price.value,
            buy_token: order.buy_token,
            buy_token_price: order.buy_token_price.value,
            leverage: U128::from(order.leverage),
            borrow_fee,
            liquidation_price,
            lpt_id: order.lpt_id,
        }
    }

    pub fn calculate_borrow_fee(&self, order_timestamp_ms: u64, borrow_rate_ratio: U128) -> U128 {
        let current_timestamp_ms = env::block_timestamp_ms();

        let borrow_period = ((current_timestamp_ms - order_timestamp_ms) as f64
            / MILLISECONDS_PER_DAY as f64)
            .ceil();

        U128::from(
            BigDecimal::from(borrow_rate_ratio)
                / BigDecimal::from(U128::from(DAYS_PER_YEAR as u128))
                * BigDecimal::from(U128::from(borrow_period as u128)),
        )
    }

    pub fn get_pending_limit_order(&self, order_id: &u64, order: &Order) -> Option<LimitOrderView> {
        let trade_pair = self.view_pair(&order.sell_token, &order.buy_token);

        let pair = format!("{}/{}", trade_pair.sell_ticker_id, trade_pair.buy_ticker_id);

        let total = if order.order_type == OrderType::Buy {
            BigDecimal::from(U128(order.amount))
        } else {
            BigDecimal::from(U128(order.amount)) * order.open_or_close_price
        };

        Some(LimitOrderView {
            order_id: U128(*order_id as u128),
            timestamp: order.timestamp_ms,
            pair,
            order_type: "Limit".to_string(),
            side: order.order_type.clone(),
            price: WBigDecimal::from(order.open_or_close_price),
            amount: U128(order.amount),
            filled: 0,
            total: LowU128::from(total),
        })
    }

    pub fn get_opened_leverage_position(
        &self,
        account_id: AccountId,
        order_id: &u64,
        order: &Order,
        market_data: Option<MarketData>,
    ) -> Option<LeveragedPositionView> {
        let trade_pair = self.view_pair(&order.sell_token, &order.buy_token);

        let pair = format!("{}/{}", trade_pair.sell_ticker_id, trade_pair.buy_ticker_id);

        let total = if order.order_type == OrderType::Long {
            BigDecimal::from(U128(order.amount)) * order.leverage
        } else {
            BigDecimal::from(U128(order.amount)) * (order.leverage - BigDecimal::one())
        };

        let filled = (order.status != OrderStatus::Pending).into();

        let pnl = self.calculate_pnl(account_id, U128(*order_id as u128), market_data);

        let take_profit_order = self.get_take_profit_order(order_id, order);

        Some(LeveragedPositionView {
            order_id: U128(*order_id as u128),
            timestamp: order.timestamp_ms,
            pair,
            order_type: order.order_type.clone(),
            price: WBigDecimal::from(order.open_or_close_price),
            leverage: WBigDecimal::from(order.leverage),
            amount: U128(order.amount),
            filled,
            total: LowU128::from(total),
            pnl,
            take_profit_order,
        })
    }

    pub fn get_take_profit_order(
        &self,
        order_id: &u64,
        leverage_position: &Order,
    ) -> Option<TakeProfitOrderView> {
        match self.take_profit_orders.get(order_id) {
            Some((_, order)) => {
                if order.status == OrderStatus::Pending
                    || order.status == OrderStatus::PendingOrderExecute
                {
                    let trade_pair = self.view_pair(&order.sell_token, &order.buy_token);

                    let pair =
                        format!("{}/{}", trade_pair.sell_ticker_id, trade_pair.buy_ticker_id);

                    let total = if leverage_position.order_type == OrderType::Long {
                        BigDecimal::from(U128(leverage_position.amount))
                            * leverage_position.leverage
                    } else {
                        BigDecimal::from(U128(leverage_position.amount))
                            * (leverage_position.leverage - BigDecimal::one())
                    };

                    Some(TakeProfitOrderView {
                        timestamp: order.timestamp_ms,
                        pair,
                        order_type: order.order_type.clone(),
                        price: WBigDecimal::from(order.open_or_close_price),
                        amount: U128(order.amount),
                        filled: 0,
                        total: LowU128::from(total),
                    })
                } else {
                    None
                }
            }
            None => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::utils::MILLISECONDS_PER_DAY;

    use super::*;

    use near_sdk::test_utils::test_env::{alice, bob};
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

    fn get_current_day_in_nanoseconds(day: u64) -> Option<u64> {
        let nanoseconds_in_one_millisecond = 1_000_000;
        Some(MILLISECONDS_PER_DAY * day * nanoseconds_in_one_millisecond)
    }

    #[test]
    fn test_get_pending_orders() {
        let current_day = get_current_day_in_nanoseconds(6); // borrow period 5 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_id: PairId = (
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        let market_data = MarketData {
            underlying_token: AccountId::new_unchecked("usdt.fakes.testnet".to_string()),
            underlying_token_decimals: 24,
            total_supplies: U128(60000000000000000000000000000),
            total_borrows: U128(25010000000000000000000000000),
            total_reserves: U128(1000176731435219096024128768),
            exchange_rate_ratio: U128(1000277139994639276176632),
            interest_rate_ratio: U128(261670051778601),
            borrow_rate_ratio: U128(5 * 10_u128.pow(24)),
        };

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDT".to_string(),
                value: U128::from(1010000000000000000000000),
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "WNEAR".to_string(),
                value: U128::from(3050000000000000000000000),
            },
        );

        contract.add_pair(pair_data);

        contract.set_balance(&alice(), &pair_id.0, 10_u128.pow(30));

        let amount = U128::from(
            BigDecimal::from(U128(2 * 10_u128.pow(27)))
                * (BigDecimal::one() - BigDecimal::from(execute_order::INACCURACY_RATE)),
        );
        let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930910,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\"}".to_string();
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        for count in 0..9 {
            if count < 6 {
                contract.imitation_add_liquidity_callback(order.clone());
            } else {
                contract.final_cancel_order(
                    U128(count as u128 - 5),
                    order.clone(),
                    amount,
                    market_data.clone(),
                );
            }
        }

        let pending_orders_par_1st_page = contract.get_pending_orders(&pair_id, U128(10), U128(1));
        let order_id_with_pending_status = pending_orders_par_1st_page
            .data
            .iter()
            .map(|(order_id, _)| *order_id)
            .collect::<Vec<u64>>();

        assert_eq!(
            contract.orders_per_pair_view.get(&pair_id).unwrap().len(),
            6_usize
        );
        assert_eq!(pending_orders_par_1st_page.data.len(), 3_usize);
        assert_eq!(
            pending_orders_par_1st_page.data.get(0).unwrap().1.status,
            OrderStatus::Pending
        );
        assert_eq!(pending_orders_par_1st_page.total, U128(3));
        assert_eq!(order_id_with_pending_status, vec![4, 5, 6]);

        let pending_orders_par_2nd_page = contract.get_pending_orders(&pair_id, U128(10), U128(2));

        assert_eq!(pending_orders_par_2nd_page.data.len(), 0_usize);
    }

    #[test]
    fn view_supported_pairs_test() {
        let context = get_context(false, None);
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
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        let pair_data2 = TradePair {
            sell_ticker_id: "near".to_string(),
            sell_token: "wrap.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "USDt".to_string(),
            buy_token: "usdt.fakes.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data2.clone());

        let pair_data_view = TradePairView {
            pair_id: (
                "usdt.fakes.testnet".parse().unwrap(),
                "wrap.testnet".parse().unwrap(),
            ),
            pair_tickers_id: "USDt-near".to_string(),
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
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        let pair_data2_view = TradePairView {
            pair_id: (
                "wrap.testnet".parse().unwrap(),
                "usdt.fakes.testnet".parse().unwrap(),
            ),
            pair_tickers_id: "near-USDt".to_string(),
            sell_ticker_id: "near".to_string(),
            sell_token: "wrap.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            buy_ticker_id: "USDt".to_string(),
            buy_token: "usdt.fakes.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "usdt_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.fakes.testnet|wrap.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data2);

        let result = vec![pair_data_view, pair_data2_view];
        let pairs = contract.view_supported_pairs();
        assert_eq!(result, pairs);
    }

    #[test]
    fn test_calculate_pnl() {
        let current_day = get_current_day_in_nanoseconds(121); // borrow period 120 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
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
            swap_fee: U128(2 * 10_u128.pow(21)),
        };
        contract.add_pair(pair_data);

        let order = "{\"status\":\"Executed\",\"order_type\":\"Long\",\"amount\":1500000000000000000000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        contract.add_order_from_string(alice(), order);

        contract.update_or_insert_price(
            "usdt.fakes.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDt".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wrap.testnet".parse().unwrap(),
            Price {
                ticker_id: "near".to_string(),
                value: U128::from(3 * 10_u128.pow(24)), // current price token
            },
        );

        let market_data = MarketData {
            underlying_token: AccountId::new_unchecked("usdt.fakes.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        };

        let pnl = contract.calculate_pnl(alice(), U128(1), Some(market_data));
        assert!(pnl.is_profit);
        assert_eq!(pnl.amount, U128(8392 * 10_u128.pow(23)));
    }

    #[test]
    fn test_calculate_liquidation_leverage_3() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
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
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        let result = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(10_u128.pow(24)),
            U128(10_u128.pow(25)),
            U128(3 * 10_u128.pow(24)),
            U128(5 * 10_u128.pow(22)),
            U128(3 * 10_u128.pow(20)),
        );

        assert_eq!(result, U128(7169666666666666666666666));
    }

    #[test]
    fn test_calculate_liquidation_leverage_1_5() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
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
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data);

        let result = contract.calculate_liquidation_price(
            U128(10_u128.pow(27)),
            U128(10_u128.pow(24)),
            U128(10_u128.pow(25)),
            U128(15 * 10_u128.pow(23)),
            U128(5 * 10_u128.pow(22)),
            U128(3 * 10_u128.pow(20)),
        );

        assert_eq!(result, U128(3836333333333333333333333));
    }

    #[test]
    fn test_view_order() {
        let context = get_context(false, None);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
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
            swap_fee: U128(10u128.pow(23)),
        };
        contract.add_pair(pair_data.clone());

        let borrow_rate_ratio = U128(634273735391536);

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930910,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#540\"}".to_string();
        contract.add_order_from_string(alice(), order1);

        let order2 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930910,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#541\"}".to_string();
        contract.add_order_from_string(alice(), order2);

        let order_id = U128(1); //order_id for order1

        let block_order = 103930910_u64;

        let borrow_fee = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order as u128)),
        );

        let liquidation_price = contract.calculate_liquidation_price(
            U128(10_u128.pow(9)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee,
            pair_data.swap_fee, // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let result_view_order1 = OrderView {
            order_id: U128(1),
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: U128(10_u128.pow(9)),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
            leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
            borrow_fee,
            liquidation_price,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#540".to_string(),
        };

        assert_eq!(
            contract.view_order(alice(), order_id, borrow_rate_ratio),
            result_view_order1
        );
    }

    #[test]
    fn test_view_orders() {
        let context = get_context(false, None);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let pair_data = TradePair {
            sell_ticker_id: "USDt".to_string(),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_decimals: 6,
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
            swap_fee: U128(10u128.pow(23)),
        };
        contract.add_pair(pair_data.clone());

        let borrow_rate_ratio = U128(634273735391536);

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930910,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#540\"}".to_string();
        contract.add_order_from_string(alice(), order1);

        let order2 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930911,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#541\"}".to_string();
        contract.add_order_from_string(alice(), order2);

        let order3 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000,\"sell_token\":\"usdt.fakes.testnet\",\"buy_token\":\"wrap.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3050000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930912,\"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#542\"}".to_string();
        contract.add_order_from_string(bob(), order3);

        let block_order1 = 103930910_u64;
        let block_order2 = 103930911_u64;
        let block_order3 = 103930912_u64;

        let borrow_fee_order1 = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order1 as u128)),
        );

        let borrow_fee_order2 = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order2 as u128)),
        );

        let borrow_fee_order3 = WBigDecimal::from(
            BigDecimal::from(borrow_rate_ratio)
                * BigDecimal::from(U128(env::block_height() as u128 - block_order3 as u128)),
        );

        let liquidation_price_order1 = contract.calculate_liquidation_price(
            U128(10_u128.pow(9)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee_order1,
            pair_data.swap_fee, // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let liquidation_price_order2 = contract.calculate_liquidation_price(
            U128(2 * 10_u128.pow(9)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee_order2,
            pair_data.swap_fee, // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let liquidation_price_order3 = contract.calculate_liquidation_price(
            U128(2 * 10_u128.pow(9)),
            U128(101 * 10_u128.pow(22)),
            U128(305 * 10_u128.pow(22)),
            U128(10_u128.pow(24)),
            borrow_fee_order3,
            pair_data.swap_fee, // hardcore of swap_fee 0.1 % with 10^24 precision
        );

        let result_view_orders_alice = vec![
            OrderView {
                order_id: U128(1),
                status: OrderStatus::Pending,
                order_type: OrderType::Buy,
                amount: U128(10_u128.pow(9)),
                sell_token: "usdt.fakes.testnet".parse().unwrap(),
                sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
                buy_token: "wrap.testnet".parse().unwrap(),
                buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
                leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
                borrow_fee: borrow_fee_order1,
                liquidation_price: liquidation_price_order1,
                lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#540".to_string(),
            },
            OrderView {
                order_id: U128(2),
                status: OrderStatus::Pending,
                order_type: OrderType::Buy,
                amount: U128(2 * 10_u128.pow(9)),
                sell_token: "usdt.fakes.testnet".parse().unwrap(),
                sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
                buy_token: "wrap.testnet".parse().unwrap(),
                buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
                leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
                borrow_fee: borrow_fee_order2,
                liquidation_price: liquidation_price_order2,
                lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#541".to_string(),
            },
        ];

        let result_view_orders_bob = vec![OrderView {
            order_id: U128(3),
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: U128(2 * 10_u128.pow(9)),
            sell_token: "usdt.fakes.testnet".parse().unwrap(),
            sell_token_price: U128(101 * 10_u128.pow(22)), // 1.01 with 10^24 precision
            buy_token: "wrap.testnet".parse().unwrap(),
            buy_token_price: U128(305 * 10_u128.pow(22)), // 3.05 with 10^24 precision
            leverage: U128(10_u128.pow(24)),              // 1 with 10^24 precision
            borrow_fee: borrow_fee_order3,
            liquidation_price: liquidation_price_order3,
            lpt_id: "usdt.fakes.testnet|wrap.testnet|2000#542".to_string(),
        }];

        let mut view_orders_alice = contract.view_orders(
            alice(),
            pair_data.sell_token.clone(),
            pair_data.buy_token.clone(),
            borrow_rate_ratio,
        );
        view_orders_alice.sort_by(|a, b| a.order_id.cmp(&b.order_id));

        let view_orders_bob = contract.view_orders(
            bob(),
            pair_data.sell_token,
            pair_data.buy_token,
            borrow_rate_ratio,
        );

        assert_eq!(view_orders_alice, result_view_orders_alice);
        assert_eq!(view_orders_bob, result_view_orders_bob);
    }

    #[test]
    fn test_view_pair_tokens_decimals() {
        let context = get_context(false, None);
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
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data.clone());

        let sell_and_buy_tokens_decimals =
            contract.view_pair_tokens_decimals(&pair_data.sell_token, &pair_data.buy_token);

        assert_eq!(
            sell_and_buy_tokens_decimals,
            (pair_data.sell_token_decimals, pair_data.buy_token_decimals)
        );
    }

    #[test]
    fn view_token_decimals_test() {
        let context = get_context(false, None);
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
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };
        contract.add_pair(pair_data.clone());

        let sell_token_decimals = contract.view_token_decimals(&pair_data.sell_token);
        let buy_token_decimals = contract.view_token_decimals(&pair_data.buy_token);

        assert_eq!(sell_token_decimals, 24);
        assert_eq!(buy_token_decimals, 18)
    }

    #[test]
    fn test_view_pending_limit_orders_by_user() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data);

        for count in 0..6 {
            if count < 1 {
                // order with status of "Pending" on leverage "1.0" and with timestamp "86400000"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Pending" on leverage "1.0"and with timestamp "86400001"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Pending" on leverage "2.0"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let true_2nd_limit_order = LimitOrderView {
            order_id: U128(2),
            timestamp: 86400001,
            pair: "USDT/WNEAR".to_string(),
            order_type: "Limit".to_string(),
            side: OrderType::Buy,
            price: U128(25 * 10_u128.pow(23)),
            amount: U128(2 * 10_u128.pow(27)),
            filled: 0,
            total: U128(2 * 10_u128.pow(27)),
        };

        let limit_orders = contract.view_pending_limit_orders_by_user(alice(), U128(10), U128(1));
        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(limit_orders.data.len(), 2_usize);
        assert_eq!(limit_orders.total_orders, U128(2));
        assert_eq!(limit_orders.data.get(1).unwrap(), &true_2nd_limit_order);
    }

    #[test]
    fn test_view_pending_limit_orders_by_user_by_pair() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair id "USDT/WNEAR"
        let pair_id: PairId = (
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data1 = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        // pair data for "WNEAR/USDT"
        let pair_data2 = TradePair {
            sell_ticker_id: "WNEAR".to_string(),
            sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "USDT".to_string(),
            buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 6,
            buy_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            pool_id: "wnear.qa.v1.nearlend.testnet|usdt.qa.v1.nearlend.testnet|2001".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data1);
        contract.add_pair(pair_data2);

        for count in 0..6 {
            if count < 1 {
                // order with status of "Pending" on leverage "1.0" and in pair "USDT/WNEAR" with timestamp "86400000"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Pending" on leverage "1.0" and in pair "USDT/WNEAR" with timestamp "86400001"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Sell\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 3 {
                // order with status of "Pending" on leverage "1.0" and in pair "WNEAR/USDT"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"wnear.qa.v1.nearlend.testnet\",\"buy_token\":\"usdt.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Pending" on leverage "2.0" and in pair "USDT/WNEAR"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0" and in pair "USDT/WNEAR"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let true_2nd_limit_order = LimitOrderView {
            order_id: U128(2),
            timestamp: 86400001,
            pair: "USDT/WNEAR".to_string(),
            order_type: "Limit".to_string(),
            side: OrderType::Sell,
            price: U128(25 * 10_u128.pow(23)),
            amount: U128(2 * 10_u128.pow(27)),
            filled: 0,
            total: U128(5 * 10_u128.pow(27)),
        };

        // view pending limit orders by pair "USDT/WNEAR"
        let limit_orders = contract.view_pending_limit_orders_by_user_by_pair(
            alice(),
            pair_id.0,
            pair_id.1,
            U128(10),
            U128(1),
        );

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(
            contract
                .view_pending_limit_orders_by_user(alice(), U128(10), U128(1))
                .total_orders,
            U128(3)
        );
        assert_eq!(limit_orders.data.len(), 2_usize);
        assert_eq!(limit_orders.total_orders, U128(2));
        assert_eq!(limit_orders.data.get(1).unwrap(), &true_2nd_limit_order);
    }

    #[test]
    fn test_view_pending_limit_orders_when_user_has_no_pending_orders() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        for count in 0..3 {
            if count < 1 {
                // order with status of "Liquidated" on leverage "1.0"
                let order_as_string = "{\"status\":\"Liquidated\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Canceled" on leverage "1.0"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400002,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let limit_orders = contract.view_pending_limit_orders_by_user(alice(), U128(10), U128(1));
        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 3_usize);
        assert_eq!(limit_orders.data.len(), 0_usize);
        assert_eq!(limit_orders.total_orders, U128(0));
    }

    #[test]
    fn test_view_pending_limit_orders_when_user_has_no_pending_orders_by_pair() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair id "WNEAR/USDT"
        let pair_id: PairId = (
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data);

        for count in 0..4 {
            if count < 1 {
                // order with status of "Liquidated" on leverage "1.0"
                let order_as_string = "{\"status\":\"Liquidated\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Canceled" on leverage "1.0"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 3 {
                // order with status of "Executed" on leverage "1.0"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400002,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Pending" on leverage "1.0" and in pair "USDT/WNEAR"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        // view pending limit orders by pair "WNEAR/USDT"
        let limit_orders = contract.view_pending_limit_orders_by_user_by_pair(
            alice(),
            pair_id.0,
            pair_id.1,
            U128(10),
            U128(1),
        );

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 4_usize);
        assert_eq!(
            contract
                .view_pending_limit_orders_by_user(alice(), U128(10), U128(1))
                .total_orders,
            U128(1)
        );
        assert_eq!(limit_orders.data.len(), 0_usize);
        assert_eq!(limit_orders.total_orders, U128(0));
    }

    #[test]
    fn test_view_opened_leverage_positions_by_user() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data1 = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        // pair data for "WNEAR/USDT"
        let pair_data2 = TradePair {
            sell_ticker_id: "WNEAR".to_string(),
            sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "USDT".to_string(),
            buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 6,
            buy_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            pool_id: "wnear.qa.v1.nearlend.testnet|usdt.qa.v1.nearlend.testnet|2001".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data1);
        contract.add_pair(pair_data2);

        let market_data = Some(MarketData {
            underlying_token: AccountId::new_unchecked("usdt.qa.v1.nearlend.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        });

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDT".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "WNEAR".to_string(),
                value: U128::from(3 * 10_u128.pow(24)), // current price token
            },
        );

        for count in 0..6 {
            if count < 1 {
                // order with status of "Pending" on leverage "3.0" and with timestamp "86400000"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Executed" on leverage "3.0" and with timestamp "86400001"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Short\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Pending" on leverage "1.0"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":3000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400002,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":4000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400003,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }
        // take-profit order for order with timestamp "86400001"
        let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"TakeProfit\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400050,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        contract.take_profit_orders.insert(&2, &((0, 40), order));

        // opened position without take-profit order
        let true_1st_opened_position = LeveragedPositionView {
            order_id: U128(1),
            timestamp: 86400000,
            pair: "USDT/WNEAR".to_string(),
            order_type: OrderType::Long,
            price: U128(25 * 10_u128.pow(23)),
            leverage: U128(3 * 10_u128.pow(24)),
            amount: U128(2 * 10_u128.pow(27)),
            filled: 0,
            total: U128(6 * 10_u128.pow(27)),
            pnl: PnLView {
                is_profit: true,
                amount: U128(114784 * 10_u128.pow(22)),
            },
            take_profit_order: None,
        };

        // opened position with take-profit order
        let true_2nd_opened_position = LeveragedPositionView {
            order_id: U128(2),
            timestamp: 86400001,
            pair: "USDT/WNEAR".to_string(),
            order_type: OrderType::Short,
            price: U128(25 * 10_u128.pow(23)),
            leverage: U128(3 * 10_u128.pow(24)),
            amount: U128(2 * 10_u128.pow(27)),
            filled: 1,
            total: U128(4 * 10_u128.pow(27)),
            pnl: PnLView {
                is_profit: false,
                amount: U128(12918 * 10_u128.pow(23)),
            },
            take_profit_order: Some(TakeProfitOrderView {
                timestamp: 86400050,
                pair: "USDT/WNEAR".to_string(),
                order_type: OrderType::TakeProfit,
                price: U128(25 * 10_u128.pow(23)),
                amount: U128(2 * 10_u128.pow(27)),
                filled: 0,
                total: U128(4 * 10_u128.pow(27)),
            }),
        };

        let opened_positions = contract.view_opened_leverage_positions_by_user(
            alice(),
            market_data,
            U128(10),
            U128(1),
        );

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(opened_positions.data.len(), 2_usize);
        assert_eq!(opened_positions.total_positions, U128(2));
        assert_eq!(
            opened_positions.data.get(0).unwrap(),
            &true_1st_opened_position
        );
        assert_eq!(
            opened_positions.data.get(1).unwrap(),
            &true_2nd_opened_position
        );
    }

    #[test]
    fn test_view_opened_leverage_positions_by_user_by_pair() {
        let current_day = get_current_day_in_nanoseconds(91); // borrow period 90 days
        let context = get_context(false, current_day);
        testing_env!(context);

        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair id "WNEAR/USDT"
        let pair_id: PairId = (
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data1 = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        // pair data for "WNEAR/USDT"
        let pair_data2 = TradePair {
            sell_ticker_id: "WNEAR".to_string(),
            sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 24,
            sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "USDT".to_string(),
            buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 6,
            buy_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            pool_id: "wnear.qa.v1.nearlend.testnet|usdt.qa.v1.nearlend.testnet|2001".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data1);
        contract.add_pair(pair_data2);

        let market_data = Some(MarketData {
            underlying_token: AccountId::new_unchecked("usdt.qa.v1.nearlend.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        });

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDT".to_string(),
                value: U128::from(10_u128.pow(24)), // current price token
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "WNEAR".to_string(),
                value: U128::from(3 * 10_u128.pow(24)), // current price token
            },
        );

        for count in 0..6 {
            if count < 1 {
                // order with status of "Pending" on leverage "3.0" and in pair "USDT/WNEAR" with timestamp "86400000"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Executed" on leverage "2.0" and in pair "WNEAR/USDT" with timestamp "86400001"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Short\",\"amount\":2000000000000000000000000000,\"sell_token\":\"wnear.qa.v1.nearlend.testnet\",\"buy_token\":\"usdt.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 3 {
                // order with status of "Pending" on leverage "4.0" and in pair "WNEAR/USDT" with timestamp "86400002"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Short\",\"amount\":2000000000000000000000000000,\"sell_token\":\"wnear.qa.v1.nearlend.testnet\",\"buy_token\":\"usdt.qa.v1.nearlend.testnet\",\"leverage\":\"4.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400002,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 4 {
                // order with status of "Pending" on leverage "1.0"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":3000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400003,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Executed" on leverage "1.0"
                let order_as_string = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":4000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400004,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }
        // take-profit order for order with timestamp "86400001"
        let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"TakeProfit\",\"amount\":2000000000000000000000000000,\"sell_token\":\"wnear.qa.v1.nearlend.testnet\",\"buy_token\":\"usdt.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1500000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400050,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
        let order: Order = near_sdk::serde_json::from_str(order_as_string.as_str()).unwrap();

        contract.take_profit_orders.insert(&2, &((0, 40), order));

        // opened position with take-profit order in pair "WNEAR/USDT"
        let true_1st_opened_position_by_pair = LeveragedPositionView {
            order_id: U128(2),
            timestamp: 86400001,
            pair: "WNEAR/USDT".to_string(),
            order_type: OrderType::Short,
            price: U128(25 * 10_u128.pow(23)),
            leverage: U128(2 * 10_u128.pow(24)),
            amount: U128(2 * 10_u128.pow(27)),
            filled: 1,
            total: U128(2 * 10_u128.pow(27)),
            pnl: PnLView {
                is_profit: false,
                amount: U128(2153 * 10_u128.pow(23)),
            },
            take_profit_order: Some(TakeProfitOrderView {
                timestamp: 86400050,
                pair: "WNEAR/USDT".to_string(),
                order_type: OrderType::TakeProfit,
                price: U128(25 * 10_u128.pow(23)),
                amount: U128(2 * 10_u128.pow(27)),
                filled: 0,
                total: U128(2 * 10_u128.pow(27)),
            }),
        };

        // opened position without take-profit order in pair "USDT/WNEAR"
        let true_2nd_opened_position_by_pair = LeveragedPositionView {
            order_id: U128(3),
            timestamp: 86400002,
            pair: "WNEAR/USDT".to_string(),
            order_type: OrderType::Short,
            price: U128(25 * 10_u128.pow(23)),
            leverage: U128(4 * 10_u128.pow(24)),
            amount: U128(2 * 10_u128.pow(27)),
            filled: 0,
            total: U128(6 * 10_u128.pow(27)),
            pnl: PnLView {
                is_profit: false,
                amount: U128(6459 * 10_u128.pow(23)),
            },
            take_profit_order: None,
        };

        let opened_positions = contract.view_opened_leverage_positions_by_user_by_pair(
            alice(),
            pair_id.0,
            pair_id.1,
            market_data.clone(),
            U128(10),
            U128(1),
        );

        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 6_usize);
        assert_eq!(
            contract
                .view_opened_leverage_positions_by_user(alice(), market_data, U128(10), U128(1))
                .total_positions,
            U128(3)
        );
        assert_eq!(opened_positions.data.len(), 2_usize);
        assert_eq!(opened_positions.total_positions, U128(2));
        assert_eq!(
            opened_positions.data.get(0).unwrap(),
            &true_1st_opened_position_by_pair
        );
        assert_eq!(
            opened_positions.data.get(1).unwrap(),
            &true_2nd_opened_position_by_pair
        );
    }

    #[test]
    fn test_view_view_opened_leverage_positions_when_user_has_no_opened_positions() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data);

        let market_data = Some(MarketData {
            underlying_token: AccountId::new_unchecked("usdt.qa.v1.nearlend.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        });

        for count in 0..3 {
            if count < 1 {
                // order with status of "Pending" on leverage "1.0"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Liquidated" on leverage "2.0"
                let order_as_string = "{\"status\":\"Liquidated\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Canceled" on leverage "3.0"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        let limit_orders = contract.view_opened_leverage_positions_by_user(
            alice(),
            market_data,
            U128(10),
            U128(1),
        );
        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 3_usize);
        assert_eq!(limit_orders.data.len(), 0_usize);
        assert_eq!(limit_orders.total_positions, U128(0));
    }

    #[test]
    fn test_view_view_opened_leverage_positions_when_user_has_no_view_opened_positions_by_pair() {
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // pair id "WNEAR/USDT"
        let pair_id: PairId = (
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
        );

        // pair data for "USDT/WNEAR"
        let pair_data = TradePair {
            sell_ticker_id: "USDT".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_decimals: 6,
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "WNEAR".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token_decimals: 24,
            buy_token_market: "wnear_market.develop.v1.omomo-finance.testnet"
                .parse()
                .unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
            swap_fee: U128(3 * 10_u128.pow(20)),
        };

        contract.add_pair(pair_data);

        let market_data = Some(MarketData {
            underlying_token: AccountId::new_unchecked("usdt.qa.v1.nearlend.testnet".to_string()),
            underlying_token_decimals: 6,
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(5 * 10_u128.pow(22)),
        });

        for count in 0..4 {
            if count < 1 {
                // order with status of "Pending" on leverage "1.0"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 2 {
                // order with status of "Liquidated" on leverage "2.0"
                let order_as_string = "{\"status\":\"Liquidated\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else if count < 3 {
                // order with status of "Canceled" on leverage "3.0"
                let order_as_string = "{\"status\":\"Canceled\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            } else {
                // order with status of "Pending" on leverage "3.0" in pair "USDT/WNEAR"
                let order_as_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":2000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"3.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1000000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"2500000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":1, \"timestamp_ms\":86400001,\"lpt_id\":\"usdt.fakes.testnet|wrap.testnet|2000#132\"}".to_string();
                contract.add_order_from_string(alice(), order_as_string);
            }
        }

        // view opened leverage positions in pair "WNEAR/USDT"
        let limit_orders = contract.view_opened_leverage_positions_by_user_by_pair(
            alice(),
            pair_id.0,
            pair_id.1,
            market_data,
            U128(10),
            U128(1),
        );
        assert_eq!(contract.orders.get(&alice()).unwrap().len(), 4_usize);
        assert_eq!(limit_orders.data.len(), 0_usize);
        assert_eq!(limit_orders.total_positions, U128(0));
    }

    #[test]
    fn test_take_profit_order_view() {
        let context = get_context(false, None);
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

        let order_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3040000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930910, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\"}".to_string();
        contract.add_order_from_string(alice(), order_string);

        let order_id: u128 = 1;
        let new_price = U128(305 * 10_u128.pow(22));
        let left_point = -9860;
        let right_point = -9820;
        contract.create_take_profit_order(U128(order_id), new_price, left_point, right_point);

        let tpo = contract.take_profit_orders.get(&(order_id as u64)).unwrap();
        assert_eq!(tpo.1.status, OrderStatus::PendingOrderExecute);
        assert_eq!(tpo.1.open_or_close_price, BigDecimal::from(new_price));

        let tpo_view = contract.take_profit_order_view(U128(order_id)).unwrap();
        assert_eq!(tpo_view.price, new_price);
    }

    #[test]
    fn test_take_profit_order_view_if_not_exist() {
        let context = get_context(false, None);
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

        let order_string = "{\"status\":\"Pending\",\"order_type\":\"Long\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1010000000000000000000000\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3040000000000000000000000\"},\"open_or_close_price\":\"2.5\",\"block\":103930910, \"timestamp_ms\":86400000,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#540\"}".to_string();
        contract.add_order_from_string(alice(), order_string);

        let order_id: u128 = 1;

        let tpo_view = contract.take_profit_order_view(U128(order_id));
        assert_eq!(tpo_view, None);
    }

    #[test]
    fn calculate_short_liquidation_price_test() {
        let contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // 3000.00 USDT
        let sell_token_amount = U128::from(3000000000000000000000000000);
        // 12000.00 NEAR
        let buy_token_amount = U128::from(12000000000000000000000000000);
        // 2.50$
        let open_price = U128::from(2500000000000000000000000);
        // 5.0
        let leverage = U128::from(5000000000000000000000000);
        // 5.00%
        let borrow_fee = U128::from(50000000000000000000000);
        // 0.20%
        let swap_fee = U128::from(2000000000000000000000);

        let short_liquidation_price = contract.calculate_short_liquidation_price(
            sell_token_amount,
            buy_token_amount,
            open_price,
            leverage,
            borrow_fee,
            swap_fee,
        );

        // 2.6042$
        let expected_result = U128::from(2604200000000000000000000);

        assert_eq!(short_liquidation_price, expected_result);
    }

    #[test]
    fn calculate_long_liquidation_price_test() {
        let contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        // 2000.00 USDT
        let sell_token_amount = U128::from(2000000000000000000000000000);
        // 2.50$
        let open_price = U128::from(2500000000000000000000000);
        // 5.0
        let leverage = U128::from(5000000000000000000000000);
        // 5.00%
        let borrow_fee = U128::from(50000000000000000000000);
        // 0.20%
        let swap_fee = U128::from(2000000000000000000000);

        let long_liquidation_price = contract.calculate_long_liquidation_price(
            sell_token_amount,
            open_price,
            leverage,
            borrow_fee,
            swap_fee,
        );

        // 2.0221$
        let expected_result = U128::from(2029063888888888888888888);

        assert_eq!(long_liquidation_price, expected_result);
    }
}
