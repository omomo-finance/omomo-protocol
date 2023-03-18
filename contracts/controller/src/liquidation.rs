use crate::*;
use near_sdk::{env::block_height, PromiseOrValue};

#[near_bindgen]
impl Contract {
    pub fn liquidation(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        require!(
            self.is_dtoken_caller(),
            "This functionality is allowed to be called by admin, contract or dtoken's contract only"
        );

        let half_of_borrower_total_supplies =
            Percentage::from(50).apply_to(self.get_total_supplies(&borrower).0);

        require!(
            liquidation_amount.0 <= half_of_borrower_total_supplies,
            "Liquidation amount must be less than half of the borrower`s total supplies"
        );

        let total_borrows_by_dtoken = self
            .get_total_borrows_by_dtoken(&borrower, &borrowing_dtoken)
            .0;

        require!(
            liquidation_amount.0 <= total_borrows_by_dtoken,
            "Liquidation amount must be less than half of the borrower`s total borrows of dtoken"
        );

        let res = self.calculate_liquidation_revenue(
            borrower,
            borrowing_dtoken,
            liquidator,
            collateral_dtoken,
            liquidation_amount,
        );

        if res.is_err() {
            panic!("Liquidation failed on controller, {:?}", res.unwrap_err());
        }

        let liquidation_revenue_amount = res.unwrap();
        PromiseOrValue::Value(liquidation_revenue_amount)
    }

    pub fn liquidation_repay_and_swap(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
        liquidation_revenue_amount: WBalance,
        borrow_rate: WRatio,
    ) -> PromiseOrValue<U128> {
        require!(
            self.is_dtoken_caller(),
            "This method is allowed to be called by dtoken contract only"
        );
        self.repay_borrows(
            borrower.clone(),
            borrowing_dtoken,
            liquidation_amount,
            block_height(),
            borrow_rate,
        );
        self.decrease_supplies(
            borrower.clone(),
            collateral_dtoken.clone(),
            liquidation_revenue_amount,
        );
        self.increase_supplies(
            liquidator.clone(),
            collateral_dtoken.clone(),
            liquidation_revenue_amount,
        );

        market::swap_supplies(
            borrower,
            liquidator,
            liquidation_revenue_amount,
            collateral_dtoken,
            NO_DEPOSIT,
            near_sdk::Gas::ONE_TERA * 8_u64,
        )
        .into()
    }
}

impl Contract {
    pub fn get_liquidation_revenue(
        &self,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) -> WBalance {
        WBalance::from(
            (self.get_liquidation_incentive()
                * Ratio::from(liquidation_amount.0)
                * Ratio::from(self.prices.get(&borrowing_dtoken).unwrap().value.0)
                / (Ratio::from(self.prices.get(&collateral_dtoken).unwrap().value.0)))
            .round_u128(),
        )
    }

    pub fn calculate_liquidation_revenue(
        &self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) -> Result<WBalance, String> {
        if liquidator == borrower {
            return Err(String::from("cannot liquidate themselves"));
        }

        if self.get_health_factor(borrower) > self.get_liquidation_threshold() {
            return Err(String::from("health factor is above liquidation threshold"));
        }

        let revenue_amount =
            self.get_liquidation_revenue(borrowing_dtoken, collateral_dtoken, liquidation_amount);
        Ok(revenue_amount)
    }
}
