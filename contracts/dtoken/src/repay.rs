use crate::*;

#[near_bindgen]
impl Contract {

    pub fn repay(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {
        assert!(Balance::from(token_amount) >= self.get_borrows_by_account(env::signer_account_id())*self.get_borrow_rate(), "Amount should be more than borrow * borrow rate");
        return controller::repay_borrows(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount,
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

    pub fn controller_repay_borrows_callback(&mut self, amount: WBalance) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!("Call to decrease user borrows ended incorrect");
            return PromiseOrValue::Value(amount);
        } 

        self.decrease_borrows(env::signer_account_id(), amount);
        return PromiseOrValue::Value(U128(0));
        
    }

}
