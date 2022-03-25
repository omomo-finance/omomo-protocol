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
            liquidator,
            collateral_dtoken,
            liquidation_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(60),
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
            Contract::custom_fail_log(
                String::from("withdraw_fail"),
                env::signer_account_id(),
                Balance::from(liquidation_amount.0),
                format!(
                    "Liquidation failed, while tried to call controller ({:?})",
                    self.get_controller_address()
                ),
            );
            env::panic_str("");
        }

        controller::repay_borrows(
            borrower.clone(),
            self.get_contract_address(),
            U128(liquidation_amount.0),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(20),
        )
        .then(controller::on_debt_repaying_callback(
            borrower,
            borrowing_dtoken,
            collateral_dtoken,
            liquidator,
            liquidation_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(20),
        ))
        .into()
    }

    #[private]
    pub fn liquidation_repay_borrows_callback(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        controller::on_debt_repaying_callback(
            borrower,
            borrowing_dtoken,
            collateral_dtoken,
            liquidator,
            liquidation_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(15),
        )
        .into()
    }

    pub fn swap_supplies(
        &mut self,
        borrower: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        assert_eq!(env::predecessor_account_id(), self.get_controller_address());

        let amount = Balance::from(liquidation_amount.0);

        if !self.token.accounts.contains_key(&liquidator) {
            self.token.internal_register_account(&liquidator);
        }

        self.token
            .internal_transfer(&borrower, &liquidator, amount, None);
        Contract::custom_success_log(
            String::from("liquidation_success"),
            env::signer_account_id(),
            Balance::from(liquidation_amount.0),
        );
        PromiseOrValue::Value(U128(0))
    }
}
