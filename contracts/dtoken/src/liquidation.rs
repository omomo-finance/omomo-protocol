use crate::*;

#[near_bindgen]
impl Contract {
    pub fn liquidation(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        liquidator: AccountId,
        collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) {
        assert_eq!(self.get_contract_address(), borrowing_dtoken);
        assert_eq!(self.get_borrows_by_account(borrower.clone()), 0);

        self.repay(liquidation_amount);
        controller::decrease_borrows(
            borrower,
            borrowing_dtoken,
            liquidation_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::liquidation_decrease_borrows_callback(
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(80),
        ));
    }

    #[private]
    pub fn liquidation_decrease_borrows_callback(&mut self) {
        assert_eq!(is_promise_success(), true);
    }
}
