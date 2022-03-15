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
        assert_eq!(self.get_borrows_by_account(borrower.clone()), 0);

        controller::liquidation(
            borrower,
            borrowing_dtoken,
            liquidator.clone(),
            collateral_dtoken,
            liquidation_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(40),
        )
        .then(ext_self::liquidate_callback(
            liquidator,
            liquidation_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(40),
        ))
        .into()
    }

    #[private]
    pub fn liquidate_callback(
        &mut self,
        liquidator: AccountId,
        amount: WBalance,
    ) -> PromiseOrValue<U128> {
        assert_eq!(is_promise_success(), true);

        ext_self::repay(
            amount,
            Some(liquidator),
            self.get_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(120),
        )
        .then(controller::on_debt_repaying(
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        ))
        .into()
    }

    pub fn swap_supplies(
        &mut self,
        borrower: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        let amount = Balance::from(liquidation_amount.0);

        if !self.token.accounts.contains_key(&liquidator) {
            self.token.internal_register_account(&liquidator);
        }

        self.token
            .internal_transfer(&borrower, &liquidator, amount, None);
        PromiseOrValue::Value(U128(0))
    }
}
