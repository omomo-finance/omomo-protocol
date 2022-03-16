use crate::*;

#[near_bindgen]
impl Contract {
    pub fn repay(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {
        if !self.mutex.try_lock(env::current_account_id()) {
            panic!("failed to acquire action mutex for account {}", env::current_account_id());
        }

        return underlying_token::ft_balance_of(
            self.get_contract_address(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(40),
        )
            .then(ext_self::repay_balance_of_callback(
                token_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(60),
            )).into();
    }

    pub fn repay_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!("failed to get {} balance on {}", self.get_contract_address(), self.get_underlying_contract_address());
            return PromiseOrValue::Value(token_amount);
        }

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

        return controller::repay_borrows(
            env::signer_account_id(),
            self.get_contract_address(),
            U128(borrow_amount),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
            .then(ext_self::controller_repay_borrows_callback(
                token_amount,
                U128(borrow_with_rate_amount),
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
        let extra_balance = Balance::from(amount) - Balance::from(borrow_amount);
        self.decrease_borrows(env::signer_account_id(), U128(self.get_borrows_by_account(env::signer_account_id())));
        self.mutex.unlock(env::signer_account_id());
        return PromiseOrValue::Value(U128(extra_balance));
    }
}
