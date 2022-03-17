use crate::*;

#[near_bindgen]
impl Contract {
    pub fn withdraw(&mut self, dtoken_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !self.mutex.try_lock(env::current_account_id()) {
            panic!(
                "failed to acquire action mutex for account {}",
                env::current_account_id()
            );
        }

        underlying_token::ft_balance_of(
            self.get_contract_address(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::withdraw_balance_of_callback(
            Balance::from(dtoken_amount),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(160),
        ))
        .into()
    }

    pub fn withdraw_balance_of_callback(
        &mut self,
        dtoken_amount: Balance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "withdraw_fail", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to get {} balance on {}"}}}}"#,  
                env::signer_account_id(), Balance::from(dtoken_amount), self.get_contract_address(), self.get_underlying_contract_address()
            );
            panic!();

            self.mutex.unlock(env::signer_account_id());
            return PromiseOrValue::Value(WBalance::from(dtoken_amount));
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let exchange_rate: Ratio = self.get_exchange_rate(WBalance::from(balance_of));
        let supply_rate: Ratio = self.get_supply_rate(
            U128(balance_of),
            U128(self.total_borrows),
            U128(self.total_reserves),
            U128(self.model.get_reserve_factor()),
        );
        self.model.calculate_accrued_supply_interest(
            env::signer_account_id(),
            supply_rate,
            self.get_supplies_by_account(env::signer_account_id()),
        );
        let token_amount: Balance = Balance::from(dtoken_amount) * RATIO_DECIMALS / exchange_rate;

        controller::withdraw_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount.into(),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::withdraw_supplies_callback(
            env::signer_account_id(),
            token_amount.into(),
            dtoken_amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(120),
        ))
        .into()
    }

    pub fn withdraw_supplies_callback(
        &mut self,
        user_account: AccountId,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "withdraw_fail", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to decrease {} supply balance of {} on controller"}}}}"#,  
                env::signer_account_id(), Balance::from(token_amount), env::signer_account_id(), self.get_contract_address()
            );
            self.mutex.unlock(env::signer_account_id());
            return PromiseOrValue::Value(WBalance::from(dtoken_amount));
        }

        // Cross-contract call to market token
        underlying_token::ft_transfer(
            user_account,
            token_amount,
            Some(format!(
                "Withdraw with token_amount {}",
                Balance::from(token_amount)
            )),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(40),
        )
        .then(ext_self::withdraw_ft_transfer_call_callback(
            token_amount.into(),
            dtoken_amount.into(),
            env::current_account_id().clone(),
            NO_DEPOSIT,
            self.terra_gas(50),
        ))
        .into()
    }

    pub fn withdraw_ft_transfer_call_callback(
        &mut self,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if is_promise_success() {
            self.burn(&env::signer_account_id(), dtoken_amount);

            self.mutex.unlock(env::signer_account_id());
            return PromiseOrValue::Value(dtoken_amount);
        } else {
            log!(
                "failed to transfer {} tokens from {} to account {}",
                Balance::from(token_amount),
                self.get_contract_address(),
                env::signer_account_id()
            );

            controller::increase_supplies(
                env::signer_account_id(),
                self.get_contract_address(),
                token_amount,
                self.get_controller_address(),
                NO_DEPOSIT,
                self.terra_gas(10),
            )
            .then(ext_self::withdraw_increase_supplies_callback(
                token_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ))
            .into()
        }
    }

    pub fn withdraw_increase_supplies_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance>{
        if !is_promise_success() {
            log!(
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "withdraw_fail", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to increase {} supply balance of {} on controller after ft_transfer fail"}}}}"#,  
                env::signer_account_id(), Balance::from(token_amount), env::signer_account_id(), self.get_contract_address()
            );

            // TODO Account should be marked
            return PromiseOrValue::Value(token_amount);
        }
        
        log!(
            r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "withdraw_success", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,  env::signer_account_id(), Balance::from(token_amount)
        );

        self.mutex.unlock(env::signer_account_id());
        return PromiseOrValue::Value(token_amount);
    }
}
