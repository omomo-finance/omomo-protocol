use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn supply(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
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
            .then(ext_self::supply_balance_of_callback(
                token_amount,
                env::current_account_id().clone(),
                NO_DEPOSIT,
                self.terra_gas(60),
            ))
            .into()
    }

    #[allow(dead_code)]
    pub fn supply_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "failed to get {} balance on {}",
                self.get_contract_address(),
                self.get_underlying_contract_address()
            );

            self.mutex.unlock(env::signer_account_id());
            return PromiseOrValue::Value(token_amount);
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<WBalance>(&result)
                .unwrap()
                .into(),
        };

        let exchange_rate: Balance =
            self.get_exchange_rate((balance_of - Balance::from(token_amount)).into());
        let dtoken_amount = Balance::from(token_amount) * exchange_rate / RATIO_DECIMALS;
        let supply_rate: Ratio = self.get_supply_rate(
            U128(balance_of - Balance::from(token_amount)),
            U128(self.total_borrows),
            U128(self.total_reserves),
            U128(self.model.get_reserve_factor()),
        );
        self.model.calculate_accrued_supply_interest(
            env::signer_account_id(),
            supply_rate,
            self.get_supplies_by_account(env::signer_account_id()),
        );

        // Dtokens minting and adding them to the user account
        self.mint(&self.get_signer_address(), dtoken_amount.into());
        log!(
            "Supply from Account {} to Dtoken contract {} with tokens amount {} was successfully done!",
            self.get_signer_address(),
            self.get_contract_address(),
            Balance::from(token_amount)
        );

        controller::increase_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(20),
        )
            .then(ext_self::controller_increase_supplies_callback(
                token_amount,
                U128(dtoken_amount),
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ))
            .into()
    }

    #[allow(dead_code)]
    pub fn controller_increase_supplies_callback(
        &mut self,
        amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!(
                "failed to increase supply {} balance of {} on controller",
                env::signer_account_id(),
                self.get_contract_address()
            );
            self.burn(&self.get_signer_address(), dtoken_amount);
        }

        if !self.user_profile.is_exist(env::signer_account_id()) {
            self.user_profile.set_supply_amount(env::signer_account_id(), amount);
        }

        self.mutex.unlock(env::signer_account_id());
        PromiseOrValue::Value(U128(0))
    }

    pub fn get_supplies_by_account(&self, account: AccountId) -> Balance {
        self.token.accounts.get(&account).unwrap_or(0).into()
    }
}
