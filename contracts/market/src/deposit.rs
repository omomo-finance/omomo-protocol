use crate::*;

const GAS_FOR_DEPOSIT: Gas = Gas(120_000_000_000_000);

impl Contract {
    pub fn deposit(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_DEPOSIT,
            "Prepaid gas is not enough for deposit flow"
        );

        self.mutex_account_lock(Actions::Deposit, token_amount, self.terra_gas(120))
    }

    pub fn post_deposit(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(self.to_decimals_token(token_amount));
        }
        underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
        .then(ext_self::deposit_balance_of_callback(
            token_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(100),
        ))
        .into()
        // TODO better gas amount and create corresponding task in margin trading scope
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn deposit_balance_of_callback(&mut self, amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::DepositFailedToGetUnderlyingBalance(
                    env::signer_account_id(),
                    Balance::from(amount),
                    self.get_contract_address(),
                    self.get_underlying_contract_address()
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(self.to_decimals_token(amount));
        }
        log!(
            "prepaid {:?}; burn{:?}",
            env::prepaid_gas(),
            env::used_gas()
        );

        mtrading::increase_user_deposit(
            self.get_underlying_contract_address(),
            env::signer_account_id(),
            amount,
            self.eligible_to_borrow_uncollateralized.clone(),
            NO_DEPOSIT,
            self.terra_gas(16),
        )
        .then(ext_self::mtrading_increase_user_deposit_callback(
            self.get_underlying_contract_address(),
            env::signer_account_id(),
            amount,
            self.get_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(26),
        ))
        .into()
    }

    #[private]
    pub fn mtrading_increase_user_deposit_callback(
        &mut self,
        amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if is_promise_success() {
            log!(
                "{}",
                Events::MarginTradingDepositSuccess(
                    self.get_underlying_contract_address(),
                    Balance::from(amount)
                )
            );
            self.mutex_account_unlock();

            PromiseOrValue::Value(U128(0))
        } else {
            log!(
                "{}",
                Events::MarginTradingFailedToIncreaseUserDeposit(
                    self.get_underlying_contract_address(),
                    Balance::from(amount)
                )
            );

            mtrading::decrease_user_deposit(
                self.get_underlying_contract_address(),
                env::signer_account_id(),
                amount,
                self.eligible_to_borrow_uncollateralized.clone(),
                NO_DEPOSIT,
                self.terra_gas(15),
            )
            .then(ext_self::mtrading_decrease_user_deposit_fail_callback(
                amount,
                self.get_contract_address(),
                NO_DEPOSIT,
                self.terra_gas(10),
            ))
            .into()
        }
    }

    #[private]
    pub fn mtrading_decrease_user_deposit_fail_callback(
        &mut self,
        amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::MarginTradingFailedToDecreaseUserDeposit(
                    self.get_underlying_contract_address(),
                    Balance::from(amount)
                )
            );
            self.mutex_account_unlock();

            PromiseOrValue::Value(self.to_decimals_token(amount))
        } else {
            log!(
                "{}",
                Events::MarginTradingRevertDepositSuccess(
                    self.get_underlying_contract_address(),
                    Balance::from(amount)
                )
            );
            self.mutex_account_unlock();

            PromiseOrValue::Value(U128(0))
        }
    }
}
