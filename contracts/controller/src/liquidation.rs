use crate::*;
use near_sdk::{is_promise_success, log, PromiseOrValue};

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
            panic!("Liquidation failed on controller, {}", res.unwrap_err());
        }
    }

    pub fn is_liquidation_allowed(
        &self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        amount_for_liquidation: WBalance,
    ) -> Result<WBalance, String> {
        if self.get_health_factor(borrower.clone()) > self.get_health_factor_threshold() {
            Err(String::from(
                "User can't be liquidated as he has normal value of health factor",
            ))
        } else {
            if liquidator == borrower {
                return Err(String::from("Liquidation cannot liquidate his on borrow"));
            }

            let borrow_amount = self.get_entity_by_token(
                ActionType::Borrow,
                borrower.clone(),
                borrowing_dtoken,
            );

            if borrow_amount > amount_for_liquidation.0 {
                return Err(String::from("Borrow bigger than liquidation amount"));
            }

            let balance_of_borrower_collateral = self.get_entity_by_token(
                ActionType::Supply,
                borrower,
                collateral_dtoken,
            );

            if balance_of_borrower_collateral < amount_for_liquidation.0 {
                return Err(String::from("Borrower collateral balance is too low"));
            }

            Ok(amount_for_liquidation)
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
