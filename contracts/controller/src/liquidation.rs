use crate::*;
use general::ratio::RATIO_DECIMALS;
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
        let res = self.is_liquidation_allowed(
            borrower,
            borrowing_dtoken,
            liquidator,
            collateral_dtoken,
            liquidation_amount,
        );
        if res.is_err() {
            panic!("Liquidation failed on controller, {:?}", res.unwrap_err());
        }
        let (_, liquidation_revenue_amount) = res.unwrap();
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
            self.get_liquidation_incentive().0
                * liquidation_amount.0
                * self.prices.get(&borrowing_dtoken).unwrap().value.0
                / (self.prices.get(&collateral_dtoken).unwrap().value.0 * RATIO_DECIMALS.0),
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
            unhealth_factor.0 * borrow_amount * borrow_price / RATIO_DECIMALS.0;

        let supply_amount =
            self.get_entity_by_token(ActionType::Supply, borrower, collateral_dtoken.clone());
        let collateral_price = self.prices.get(&collateral_dtoken).unwrap().value.0;

        let max_possible_liquidation_amount = min(
            max_unhealth_repay,
            (RATIO_DECIMALS - self.liquidation_incentive).0 * supply_amount * collateral_price,
        ) / borrow_price;

        WBalance::from(max_possible_liquidation_amount)
    }

    pub fn is_liquidation_allowed(
        &self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) -> Result<(WBalance, WBalance), (WBalance, WBalance, String)> {
        if self.get_health_factor(borrower.clone()) > self.get_health_threshold() {
            Err((
                WBalance::from(liquidation_amount.0),
                WBalance::from(0),
                String::from("User can't be liquidated as he has normal value of health factor"),
            ))
        } else {
            let max_possible_liquidation_amount = self.maximum_possible_liquidation_amount(
                borrower.clone(),
                borrowing_dtoken.clone(),
                collateral_dtoken.clone(),
            );

            if max_possible_liquidation_amount.0 <= liquidation_amount.0 {
                return Err((
                    WBalance::from(liquidation_amount.0),
                    WBalance::from(max_possible_liquidation_amount.0),
                    String::from(
                        "Max possible liquidation amount cannot be less than liquidation amount",
                    ),
                ));
            }

            if liquidator == borrower {
                return Err((
                    WBalance::from(liquidation_amount.0),
                    WBalance::from(max_possible_liquidation_amount.0),
                    String::from("Liquidation cannot liquidate his on borrow"),
                ));
            }

            let revenue_amount = self.get_liquidation_revenue(
                borrowing_dtoken,
                collateral_dtoken,
                liquidation_amount,
            );
            Ok((liquidation_amount, revenue_amount))
        }
    }
}
