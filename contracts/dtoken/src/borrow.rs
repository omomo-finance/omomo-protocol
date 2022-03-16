use crate::*;


#[near_bindgen]
impl Contract {
    pub fn borrow(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !self.mutex.try_lock(env::current_account_id()) {
            panic!(
                "failed to acquire action mutex for account {}",
                env::current_account_id()
            );
        }

        underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(40),
        )
        .then(ext_self::borrow_balance_of_callback(
            token_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(200),
        ))
        .into()
    }

    pub fn borrow_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!("failed to get {} balance on {}", self.get_contract_address(), self.get_underlying_contract_address());

            self.mutex.unlock(env::signer_account_id());
            return PromiseOrValue::Value(WBalance::from(token_amount));
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let borrow_rate: Balance = self.get_borrow_rate(U128(balance_of), U128(self.total_borrows), U128(self.total_reserves));
        self.model.calculate_accrued_borrow_interest(env::signer_account_id(), borrow_rate, self.get_borrows_by_account(env::signer_account_id()));

        return controller::make_borrow(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::make_borrow_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(150),
        ))
        .into()
    }

    pub fn make_borrow_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "failed to borrow {} for account {}",
                token_amount.0,
                env::signer_account_id()
            );

            self.mutex.unlock(env::signer_account_id());
            return PromiseOrValue::Value(WBalance::from(token_amount));
        }

        underlying_token::ft_transfer(
            env::signer_account_id(),
            token_amount,
            Some(format!(
                "Borrow with token_amount {}",
                Balance::from(token_amount)
            )),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(40),
        )
        .then(ext_self::borrow_ft_transfer_callback(
            token_amount,
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(80),
        ))
        .into()
    }

    pub fn borrow_ft_transfer_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if is_promise_success() {
            self.increase_borrows(env::signer_account_id(), token_amount);

            self.mutex.unlock(env::signer_account_id());
            return PromiseOrValue::Value(WBalance::from(token_amount));
        } else {
            log!(
                "failed to transfer {} tokens from {} to account {}",
                Balance::from(token_amount),
                self.get_contract_address(),
                env::signer_account_id()
            );

            controller::decrease_borrows(
                env::signer_account_id(),
                self.get_contract_address(),
                token_amount,
                self.get_controller_address(),
                NO_DEPOSIT,
                self.terra_gas(10),
            )
            .then(ext_self::controller_decrease_borrows_fail(
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ))
            .into()
        }
    }

    pub fn controller_decrease_borrows_fail(&mut self) {
        if !is_promise_success() {
            log!("failed to revert state for {}", env::signer_account_id());

            // TODO Account should be marked
        }

        self.mutex.unlock(env::signer_account_id());
    }

    pub fn decrease_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());

        assert!(
            existing_borrows >= Balance::from(token_amount),
            "Repay amount is more than existing borrows"
        );
        let decreased_borrows: Balance = existing_borrows - Balance::from(token_amount);

        let new_total_borrows = self.total_borrows.checked_sub(Balance::from(token_amount));
        assert!(
            new_total_borrows.is_some(),
            "Overflow occurs while decreasing total borrow"
        );
        self.total_borrows = new_total_borrows.unwrap();
        return self.set_borrows(account.clone(), U128(decreased_borrows));
    }

    pub fn increase_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(token_amount);

        let new_total_borrows = self.total_borrows.checked_add(Balance::from(token_amount));
        assert!(
            new_total_borrows.is_some(),
            "Overflow occurs while incresing total borrow"
        );
        self.total_borrows = new_total_borrows.unwrap();
        return self.set_borrows(account.clone(), U128(increased_borrows));
    }

    #[private]
    pub fn set_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        self.borrows.insert(&account, &Balance::from(token_amount));
        return Balance::from(token_amount);
    }

    pub fn get_borrows_by_account(&self, account: AccountId) -> Balance {
        if self.borrows.get(&account).is_none() {
            return 0;
        }
        return self.borrows.get(&account).unwrap();
    }
}
