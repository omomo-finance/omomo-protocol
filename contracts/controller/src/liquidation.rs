use crate::*;
use near_sdk::{is_promise_success, log, PromiseOrValue};
use partial_min_max::min;

#[near_bindgen]
impl Contract {
    pub fn liquidation(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        _liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) {
        // TODO: Add check that this function was called by real Dtoken that we store somewhere in self.markets
        let res = self.is_liquidation_allowed(
            borrower,
            borrowing_dtoken,
            _liquidator,
            collateral_dtoken,
            liquidation_amount,
        );
        if res.is_err() {
            panic!("Liquidation failed on controller, {:?}", res.unwrap_err());
        }
    }

    pub fn get_liquidation_revenue(
        &self,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        amount_for_liquidation: WBalance,
    ) -> WBalance {
        WBalance::from(
            self.get_liquidation_incentive()
                * amount_for_liquidation.0
                * self.prices.get(&borrowing_dtoken).unwrap().value.0
                / self.prices.get(&collateral_dtoken).unwrap().value.0,
        )
    }

    pub fn maximum_possible_liquidation_amount(
        &self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
    ) -> WBalance {
        let unhealth_factor =
            self.liquidation_health_factor_threshold - self.get_health_factor(borrower.clone());

        let borrow_amount = self.get_entity_by_token(
            ActionType::Borrow,
            borrower.clone(),
            borrowing_dtoken.clone(),
        );

        let borrow_price = self.prices.get(&borrowing_dtoken.clone()).unwrap().value.0;

        let max_unhealth_repay = unhealth_factor * borrow_amount * borrow_price / RATIO_DECIMALS;

        let supply_amount = self.get_entity_by_token(
            ActionType::Supply,
            borrower.clone(),
            borrowing_dtoken.clone(),
        );
        let collateral_price = self.prices.get(&collateral_dtoken.clone()).unwrap().value.0;

        let max_possible_liquidation_amount = min(
            max_unhealth_repay,
            (RATIO_DECIMALS - self.liquidation_incentive) * supply_amount * collateral_price,
        ) / borrow_price;

        WBalance::from(max_possible_liquidation_amount as u128)
    }

    pub fn is_liquidation_allowed(
        &self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        amount_for_liquidation: WBalance,
    ) -> Result<(WBalance, WBalance), (WBalance, WBalance, String)> {
        if self.get_health_factor(borrower.clone()) > self.get_health_threshold() {
            return Err((
                WBalance::from(amount_for_liquidation.0),
                WBalance::from(0),
                String::from("User can't be liquidated as he has normal value of health factor"),
            ));
        } else {
            let max_possible_liquidation_amount = self.maximum_possible_liquidation_amount(
                borrower.clone(),
                borrowing_dtoken.clone(),
                collateral_dtoken.clone(),
            );

            if max_possible_liquidation_amount.0 < amount_for_liquidation.0 {
                return Err((
                    WBalance::from(amount_for_liquidation.0),
                    WBalance::from(max_possible_liquidation_amount.0),
                    String::from(
                        "Max possible liquidation amount cannot be less than liquidation amount",
                    ),
                ));
            }

            let borrower_supply_amount = self.get_entity_by_token(
                ActionType::Supply,
                borrower.clone(),
                collateral_dtoken.clone(),
            );



            if borrower_supply_amount < amount_for_liquidation.0 {
                return Err((
                    WBalance::from(amount_for_liquidation.0),
                    WBalance::from(borrower_supply_amount),
                    String::from(
                        "Borrower collateral amount is not enough to pay it to liquidator",
                    ),
                ));
            }

            if liquidator == borrower {
                return Err((
                    WBalance::from(amount_for_liquidation.0),
                    WBalance::from(max_possible_liquidation_amount.0),
                    String::from("Liquidation cannot liquidate his on borrow"),
                ));
            }

            let revenue_amount = self.get_liquidation_revenue(
                borrowing_dtoken.clone(),
                collateral_dtoken.clone(),
                amount_for_liquidation,
            );
            Ok((amount_for_liquidation, revenue_amount))
        }
    }

    pub fn on_debt_repaying_callback(
        &mut self,
        borrower: AccountId,
        _borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        // TODO: Add check that only real Dtoken address can call this
        if !is_promise_success() {
            self.increase_borrows(borrower, _borrowing_dtoken, liquidation_amount);
            log!("Liquidation failed on borrow_repay call, revert changes...");
            PromiseOrValue::Value(U128(liquidation_amount.0))
        } else {
            self.decrease_supplies(
                borrower.clone(),
                collateral_dtoken.clone(),
                liquidation_amount,
            );

            self.increase_supplies(
                liquidator.clone(),
                collateral_dtoken.clone(),
                liquidation_amount,
            );

            dtoken::swap_supplies(
                borrower,
                liquidator,
                liquidation_amount,
                collateral_dtoken,
                NO_DEPOSIT,
                near_sdk::Gas::ONE_TERA * 8_u64,
            )
            .into()
        }
    }
}
