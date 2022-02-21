use crate::*;

#[near_bindgen]
impl Contract {

    pub fn repay(&mut self, dtoken_amount: WBalance) -> PromiseOrValue<U128> {
        return controller::repay_borrows(
            env::signer_account_id(),
            self.get_contract_address(),
            dtoken_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::controller_repay_borrows_callback(
            dtoken_amount,
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
