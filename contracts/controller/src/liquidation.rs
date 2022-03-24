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
        let res = self.is_liquidation_allowed(
            borrower.clone(),
            borrowing_dtoken.clone(),
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
        collateral_dtoken: AccountId,
        amount_for_liquidation: WBalance,
    ) -> Result<WBalance, String> {
        let borrow_amount = self.get_entity_by_token(
            ActionType::Borrow,
            borrower.clone(),
            borrowing_dtoken.clone(),
        );

        if borrow_amount > amount_for_liquidation.0 {
            return Err(String::from("Borrow bigger than liquidation amount"));
        }

        let balance_of_borrower_collateral = self.get_entity_by_token(
            ActionType::Supply,
            borrower.clone(),
            collateral_dtoken.clone(),
        );

        if balance_of_borrower_collateral < amount_for_liquidation.0 {
            return Err(String::from("Borrower collateral balance is too low"));
        }

        Ok(amount_for_liquidation)
    }

    pub fn on_debt_repaying_callback(
        &mut self,
        borrower: AccountId,
        _borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            self.increase_borrows(borrower.clone(), _borrowing_dtoken, liquidation_amount);
            log!("Liquidation failed on borrow_repay call, revert changes...");
            PromiseOrValue::Value(U128(liquidation_amount.0))
        } else {
            self.decrease_supplies(
                borrower.clone(),
                collateral_dtoken.clone(),
                liquidation_amount.clone(),
            );

            // Need to resolve bug with double 'insert' in NEAR map
            self.increase_supplies(
                liquidator.clone(),
                collateral_dtoken.clone(),
                liquidation_amount.clone(),
            );
            
            dtoken::swap_supplies(
                borrower.clone(),
                liquidator.clone(),
                liquidation_amount.clone(),
                collateral_dtoken.clone(),
                NO_DEPOSIT,
                near_sdk::Gas::ONE_TERA * 8 as u64,
            ).into()
        }
    }
}
