use crate::*;

#[near_bindgen]
impl Contract {
    pub fn liquidate(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        assert_eq!(self.get_contract_address(), borrowing_dtoken);

        controller::liquidation(
            borrower.clone(),
            borrowing_dtoken.clone(),
            liquidator.clone(),
            collateral_dtoken.clone(),
            liquidation_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(30),
        )
        .then(ext_self::liquidate_callback(
            borrower,
            borrowing_dtoken,
            collateral_dtoken,
            liquidator,
            liquidation_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(80),
        ))
        .into()
    }

    #[private]
    pub fn liquidate_callback(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            self.increase_borrows(borrower.clone(), liquidation_amount);
            log!(
                "{}",
                Events::LiquidationFailed(liquidator, borrower, Balance::from(liquidation_amount))
            );
            env::panic_str("Revenue amount is not available!");
        }

        let liquidation_revenue_amount: WBalance = match env::promise_result(0) {
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
            _ => env::panic_str("Revenue amount is not available!"),
        };

        self.decrease_borrows(borrower.clone(), liquidation_amount);

        controller::liquidation_repay_and_swap(
            borrower.clone(),
            borrowing_dtoken,
            collateral_dtoken,
            liquidator.clone(),
            liquidation_amount,
            liquidation_revenue_amount.clone(),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(40),
        )
        .then(ext_self::liquidation_repay_and_swap_callback(
            borrower,
            liquidator,
            liquidation_revenue_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(10),
        ))
        .into()
    }

    #[private]
    pub fn liquidation_repay_and_swap_callback(
        &mut self,
        borrower: AccountId,
        liquidator: AccountId,
        liquidation_revenue_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        let amount = liquidation_revenue_amount.0;

        if !self.token.accounts.contains_key(&liquidator) {
            self.token.internal_register_account(&liquidator);
        }

        self.token
            .internal_transfer(&borrower, &liquidator, amount, None);
        log!(
            "{}",
            Events::LiquidationSuccess(
                liquidator,
                borrower,
                Balance::from(liquidation_revenue_amount)
            )
        );
        PromiseOrValue::Value(U128(0))
    }
}
