use crate::*;

#[near_bindgen]
impl Contract {
    // account_id: value for cases, when the one who pays the debt is another person (account). If leave this 'None', payer will be signer of the call
    pub fn repay(&mut self, token_amount: WBalance, account_id: Option<AccountId>) -> PromiseOrValue<U128> {
        log!("dtoken::repay start");

        underlying_token::ft_balance_of(
            self.get_contract_address(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(15),//20
        )
        .then(ext_self::repay_balance_of_callback(
            token_amount,
            account_id,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(10),//30 // 5
        )).into()
    }

    pub fn repay_balance_of_callback(&mut self, token_amount: WBalance, account_id: Option<AccountId>) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!("failed to get {} balance on {}", self.get_contract_address(), self.get_underlying_contract_address());
            return PromiseOrValue::Value(token_amount);
        }
        log!("dtoken::repay_balance_of_callback start");
        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let borrow_rate: Balance = self.get_borrow_rate(U128(balance_of - Balance::from(token_amount)), U128(self.total_borrows), U128(self.total_reserves));
        let borrow_amount = self.get_borrows_by_account(env::signer_account_id());
        let borrow_with_rate_amount = borrow_amount * borrow_rate / RATIO_DECIMALS;
        assert!(Balance::from(token_amount) >= borrow_with_rate_amount);

        let payer_account = match account_id {
            None => { env::signer_account_id() }
            Some(id) => { id }
        };

        log!("repay_balance_of_callback used: {:?}", env::used_gas());

        return controller::repay_borrows(
            payer_account,
            self.get_contract_address(),
            U128(borrow_amount),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(2),
        ).into()
        /*.then(ext_self::controller_repay_borrows_callback(
            token_amount,
            U128(borrow_with_rate_amount),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(1),
        )).into();*/
    }

    pub fn controller_repay_borrows_callback(&mut self, amount: WBalance, borrow_amount: WBalance) -> PromiseOrValue<U128> {
        log!("dtoken::controller_repay_borrows_callback start");
        if !is_promise_success() {
            log!("failed to update user {} balance {}: user is not registered", env::signer_account_id(), Balance::from(amount));
            return PromiseOrValue::Value(amount);
        } 
        let extra_balance = Balance::from(amount) - Balance::from(borrow_amount);
        self.decrease_borrows(env::signer_account_id(), U128(self.get_borrows_by_account(env::signer_account_id())));
        return PromiseOrValue::Value(U128(extra_balance));
    }
}
