use crate::*;
use near_sdk::{env::block_height, PromiseOrValue};
use partial_min_max::min;

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

        dtoken::swap_supplies(
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

        let borrow_price = self.prices.get(&borrowing_dtoken).unwrap().value.0;

        let max_unhealth_repay =
            unhealth_factor * Ratio::from(borrow_amount) * Ratio::from(borrow_price) / Ratio::one();

        let supply_amount =
            self.get_entity_by_token(ActionType::Supply, borrower, collateral_dtoken.clone());
        let collateral_price = self.prices.get(&collateral_dtoken).unwrap().value.0;

        let max_possible_liquidation_amount = min(
            max_unhealth_repay,
            (Ratio::one() - self.liquidation_incentive)
                * Ratio::from(supply_amount)
                * Ratio::from(collateral_price),
        ) / Ratio::from(borrow_price);

        WBalance::from(max_possible_liquidation_amount.round_u128())
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

        if self.get_health_factor(borrower.clone()) > self.get_liquidation_threshold() {
            return Err(String::from("health factor is above liquidation threshold"));
        }

        let max_possible_liquidation_amount = self.maximum_possible_liquidation_amount(
            borrower.clone(),
            borrowing_dtoken.clone(),
            collateral_dtoken.clone(),
        );

        if liquidation_amount.0 > max_possible_liquidation_amount.0 {
            return Err(String::from(
                "liquidation amount exceeds maximum possible liquidation amount",
            ));
        }

        let revenue_amount =
            self.get_liquidation_revenue(borrowing_dtoken, collateral_dtoken, liquidation_amount);
        Ok(revenue_amount)
    }
}
