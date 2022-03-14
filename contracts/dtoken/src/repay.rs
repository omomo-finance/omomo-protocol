use crate::*;

#[near_bindgen]
impl Contract {

    pub fn repay(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {
        let debt_amount = self.get_borrows_by_account(env::signer_account_id());
        assert!(Balance::from(token_amount) >= debt_amount*self.get_borrow_rate(), "repay amount {} is less than existing borrow {}", Balance::from(token_amount), debt_amount * self.get_borrow_rate());
        return controller::repay_borrows(
            env::signer_account_id(),
            self.get_contract_address(),
            U128(debt_amount),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::controller_repay_borrows_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )).into();
    }

    pub fn controller_repay_borrows_callback(&mut self, amount: WBalance, borrow_amount: WBalance) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!("failed to update user {} balance {}: user is not registered", env::signer_account_id(), Balance::from(amount));
            return PromiseOrValue::Value(amount);
        } 
        let extra_balance = Balance::from(amount) - self.get_borrows_by_account(env::signer_account_id());
        self.decrease_borrows(env::signer_account_id(), U128(self.get_borrows_by_account(env::signer_account_id())));
        return PromiseOrValue::Value(U128(extra_balance));
        
    }

}
