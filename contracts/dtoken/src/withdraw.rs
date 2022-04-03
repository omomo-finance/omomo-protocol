use crate::*;

const GAS_FOR_WITHDRAW: Gas = Gas(130_000_000_000_000);

#[near_bindgen]
impl Contract {
    pub fn withdraw(&mut self, dtoken_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_WITHDRAW,
            "Prepaid gas is not enough for withdraw flow"
        );
        self.mutex_account_lock(String::from("withdraw"));

        underlying_token::ft_balance_of(
            self.get_contract_address(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
        .then(ext_self::withdraw_balance_of_callback(
            Balance::from(dtoken_amount),
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(100),
        ))
        .into()
    }

    #[private]
    pub fn withdraw_balance_of_callback(
        &mut self,
        dtoken_amount: Balance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::WithdrawFailedToGetUnderlyingBalance(
                    env::signer_account_id(),
                    dtoken_amount,
                    self.get_contract_address(),
                    self.get_underlying_contract_address()
                )
            );
            self.mutex_account_unlock();
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
            U128(self.get_total_borrows()),
            U128(self.total_reserves),
            U128(self.model.get_reserve_factor()),
        );
        let accrued_supply_interest = self.model.calculate_accrued_interest(
            supply_rate,
            self.get_supplies_by_account(env::signer_account_id()),
            self.get_accrued_supply_interest(env::signer_account_id()),
        );
        self.set_accrued_supply_interest(env::signer_account_id(), accrued_supply_interest);

        let token_amount: Balance = dtoken_amount * RATIO_DECIMALS / exchange_rate;

        controller::withdraw_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            token_amount.into(),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        )
        .then(ext_self::withdraw_supplies_callback(
            env::signer_account_id(),
            token_amount.into(),
            dtoken_amount.into(),
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(70),
        ))
        .into()
    }

    #[private]
    pub fn withdraw_supplies_callback(
        &mut self,
        user_account: AccountId,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::WithdrawFailedToDecreaseSupplyOnController(
                    env::signer_account_id(),
                    Balance::from(dtoken_amount),
                    self.get_contract_address()
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(dtoken_amount);
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
            self.terra_gas(10),
        )
        .then(ext_self::withdraw_ft_transfer_call_callback(
            token_amount,
            dtoken_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(40),
        ))
        .into()
    }

    #[private]
    pub fn withdraw_ft_transfer_call_callback(
        &mut self,
        token_amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if is_promise_success() {
            self.burn(&env::signer_account_id(), dtoken_amount);
            self.mutex_account_unlock();
            log!(
                "{}",
                Events::WithdrawSuccess(env::signer_account_id(), Balance::from(dtoken_amount))
            );
            PromiseOrValue::Value(dtoken_amount)
        } else {
            controller::increase_supplies(
                env::signer_account_id(),
                self.get_contract_address(),
                token_amount,
                self.get_controller_address(),
                NO_DEPOSIT,
                self.terra_gas(5),
            )
            .then(ext_self::withdraw_increase_supplies_callback(
                token_amount,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(5),
            ))
            .into()
        }
    }

    #[private]
    pub fn withdraw_increase_supplies_callback(
        &mut self,
        token_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            self.add_inconsistent_account(env::signer_account_id());
            log!(
                "{}",
                Events::WithdrawFailedToFallback(
                    env::signer_account_id(),
                    Balance::from(token_amount)
                )
            );
            return PromiseOrValue::Value(token_amount);
        }
        self.mutex_account_unlock();
        log!(
            "{}",
            Events::WithdrawFallbackSuccess(env::signer_account_id(), Balance::from(token_amount))
        );
        PromiseOrValue::Value(token_amount)
    }
}
