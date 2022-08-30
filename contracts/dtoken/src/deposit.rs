use crate::*;
use std::convert::TryFrom;

const GAS_FOR_DEPOSIT: Gas = Gas(120_000_000_000_000);
const MARGIN_TRADING_CONTRACT: &str = "mtrading.omomo-finance.testnet";

impl Contract {
    pub fn post_deposit(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
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
            self.terra_gas(50),
        ))
        .into()
    }
}

#[near_bindgen]
impl Contract {
    pub fn deposit(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_DEPOSIT,
            "Prepaid gas is not enough for deposit flow"
        );

        self.mutex_account_lock(Actions::Deposit, token_amount, self.terra_gas(120))
    }

    #[private]
    pub fn deposit_balance_of_callback(
        &mut self,
        token_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::DepositFailedToGetUnderlyingBalance(
                    env::signer_account_id(),
                    Balance::from(token_amount),
                    self.get_contract_address(),
                    self.get_underlying_contract_address()
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(token_amount);
        }

        mtrading::increase_user_deposit(
            env::current_account_id(),
            env::signer_account_id(),
            token_amount,
            AccountId::try_from(MARGIN_TRADING_CONTRACT.to_string()).unwrap(),
            NO_DEPOSIT,
            self.terra_gas(5),
        )
        .then(ext_self::mtrading_increase_user_deposit_callback(
            env::current_account_id(),
            env::signer_account_id(),
            token_amount,
            self.get_contract_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        ))
        .into()
    }

    #[private]
    pub fn mtrading_increase_user_deposit_callback(
        &mut self,
        amount: WBalance,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::MarginTradingFailedToIncreaseUserDeposit(
                    env::signer_account_id(),
                    Balance::from(amount)
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(amount);
        }

        underlying_token::ft_transfer(
            AccountId::try_from(MARGIN_TRADING_CONTRACT.to_string()).unwrap(),
            amount,
            Some(format!(
                "Deposit form {} with token_amount {}",
                env::signer_account_id(),
                Balance::from(amount)
            )),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(10),
        )
        .then(ext_self::deposit_ft_transfer_callback(
            amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(40),
        ))
        .into()
    }

    #[private]
    pub fn deposit_ft_transfer_callback(&mut self, amount: WBalance) -> PromiseOrValue<U128> {
        if is_promise_success() {
            self.mutex_account_unlock();
            log!(
                "{}",
                Events::MarginTradingDepositSuccess(
                    env::signer_account_id(),
                    Balance::from(amount)
                )
            );
            PromiseOrValue::Value(U128(0))
        } else {
            mtrading::decrease_user_deposit(
                env::current_account_id(),
                env::signer_account_id(),
                amount,
                AccountId::try_from(MARGIN_TRADING_CONTRACT.to_string()).unwrap(),
                NO_DEPOSIT,
                self.terra_gas(5),
            )
            .then(ext_self::mtrading_decrease_user_deposit_fail_callback(
                amount,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(20),
            ))
            .into()
        }
    }

    #[private]
    pub fn mtrading_decrease_user_deposit_fail_callback(&mut self, token_amount: WBalance) {
        if !is_promise_success() {
            log!(
                "{}",
                Events::MarginTradingFailedToDecreaseUserDeposit(
                    env::signer_account_id(),
                    Balance::from(token_amount)
                )
            );
        } else {
            self.mutex_account_unlock();
            log!(
                "{}",
                Events::MarginTradingRevertDepositSuccess(
                    env::signer_account_id(),
                    Balance::from(token_amount)
                )
            );
        }
    }
}
