use crate::*;
use near_sdk::promise_result_as_success;

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
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn liquidate_callback(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        collateral_dtoken: AccountId,
        liquidator: AccountId,
        liquidation_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        let err_message = format!(
            "Revenue amount is not available! liquidator_account_id: {}, borrower_account_id: {}, amount: {}",
            liquidator,
            borrower,
            Balance::from(liquidation_amount)
        );
        require!(is_promise_success(), &err_message);

        let result = promise_result_as_success();
        require!(result.is_some(), err_message);

        let liquidation_revenue_amount: WBalance =
            WBalance(near_sdk::serde_json::from_slice::<U128>(&result.unwrap()).unwrap());

        self.decrease_borrows(borrower.clone(), liquidation_amount);

        controller::liquidation_repay_and_swap(
            borrower,
            borrowing_dtoken,
            collateral_dtoken,
            liquidator,
            liquidation_amount,
            liquidation_revenue_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(40),
        )
            .into()
    }

    pub fn swap_supplies(
        &mut self,
        borrower: AccountId,
        liquidator: AccountId,
        liquidation_revenue_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            env::predecessor_account_id(),
            self.get_controller_address(),
            "This method can be called only from controller contract"
        );
        let amount: Balance = liquidation_revenue_amount.into();

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