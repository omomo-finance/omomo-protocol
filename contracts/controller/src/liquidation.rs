use crate::*;
use near_sdk::PromiseOrValue;

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
            panic!("{}", res.unwrap_err());
        }

        self.decrease_borrows(borrower, borrowing_dtoken, liquidation_amount);
    }

    pub fn is_liquidation_allowed(
        &self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        amount_for_liquidation: WBalance,
    ) -> Result<WBalance, String> {
        let borrow_amount = WBalance::from(
            self.account_borrows
                .get(&borrower)
                .unwrap()
                .get(&borrowing_dtoken)
                .unwrap(),
        );

        if borrow_amount.0 > amount_for_liquidation.0 {
            return Err(String::from("Borrow bigger than liquidation amount"));
        }

        let balance_of_borrower_collateral = WBalance::from(
            self.account_supplies
                .get(&borrower)
                .unwrap()
                .get(&collateral_dtoken)
                .unwrap(),
        );

        if balance_of_borrower_collateral.0 < amount_for_liquidation.0 {
            return Err(String::from("Borrower collateral balance is too low"));
        }

        Ok(amount_for_liquidation)
    }

    pub fn on_debt_repaying(
        &mut self,
        borrower: AccountId,
        _borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        self.decrease_supplies(
            borrower.clone(),
            collateral_dtoken.clone(),
            liquidation_amount.clone(),
        );
        self.increase_supplies(
            liquidator.clone(),
            collateral_dtoken.clone(),
            liquidation_amount.clone(),
        );

        dtoken::swap_supplies(
            borrower,
            liquidator,
            liquidation_amount,
            collateral_dtoken,
            NO_DEPOSIT,
            near_sdk::Gas::ONE_TERA * 10 as u64,
        )
        .into()
    }
}
