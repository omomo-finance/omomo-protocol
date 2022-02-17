use crate::*;

impl Contract {

    pub fn repay(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {

        underlying_token::ft_balance_of(
            env::signer_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(40),
        )
        .then(ext_self::repay_balance_of_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(60),
        )).into()
    }

    #[allow(dead_code)]
    pub fn repay_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {

        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        //let borrow_rate: Balance = self.get_borrow_rate(balance_of.into());
        let repay_amount = Balance::from(token_amount)/* * borrow_rate*/;

        log!(
            "Repay from Account {} to Dtoken contract {} with tokens amount {} was successfully done!",
            self.get_signer_address(),
            self.get_contract_address(),
            Balance::from(token_amount)
        );
        
        controller::repay_borrows(
            env::signer_account_id(),
            self.get_contract_address(),
            U128(repay_amount),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(20),
        )
        .then(ext_self::controller_repay_borrows_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )).into()

    }
    
    #[allow(dead_code)]
    pub fn controller_repay_borrows_callback(&mut self, amount: WBalance) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            PromiseOrValue::Value(amount)
            
        } else {
            PromiseOrValue::Value(U128(0))
        }
    }


}
