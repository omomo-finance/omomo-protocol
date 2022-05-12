use crate::*;
use general::ratio::{Ratio, RATIO_DECIMALS};
use near_contract_standards::fungible_token::core::FungibleTokenCore;
use std::fmt;

const BLOCK_PER_DAY: BlockHeight = 72000;
const BLOCK_PER_WEEK: BlockHeight = 1048896;

pub enum Events {
    BorrowFailedToGetUnderlyingBalance(AccountId, Balance, AccountId, AccountId),
    BorrowFailedToIncreaseBorrowOnController(AccountId, Balance),
    BorrowSuccess(AccountId, Balance),
    BorrowFailedToFallback(AccountId, Balance),
    BorrowFallbackSuccess(AccountId, Balance),

    RepayFailedToGetUnderlyingBalance(AccountId, Balance, AccountId, AccountId),
    RepayFailedToUpdateUserBalance(AccountId, Balance),
    RepaySuccess(AccountId, Balance),

    SupplyFailedToGetUnderlyingBalance(AccountId, Balance, AccountId, AccountId),
    SupplyFailedToIncreaseSupplyOnController(AccountId, Balance),
    SupplySuccess(AccountId, Balance),

    WithdrawFailedToGetUnderlyingBalance(AccountId, Balance, AccountId, AccountId),
    WithdrawFailedToDecreaseSupplyOnController(AccountId, Balance, AccountId),
    WithdrawSuccess(AccountId, Balance),
    WithdrawFailedToFallback(AccountId, Balance),
    WithdrawFallbackSuccess(AccountId, Balance),

    LiquidationSuccess(AccountId, AccountId, Balance),
    LiquidationFailed(AccountId, AccountId, Balance),
}

impl Contract {
    pub fn get_controller_address(&self) -> AccountId {
        let config: Config = self.get_contract_config();

        AccountId::new_unchecked(config.controller_account_id.to_string())
    }

    pub fn get_contract_address(&self) -> AccountId {
        env::current_account_id()
    }

    pub fn get_signer_address(&self) -> AccountId {
        env::signer_account_id()
    }

    pub fn get_underlying_contract_address(&self) -> AccountId {
        self.underlying_token.clone()
    }

    pub fn get_exchange_rate(&self, underlying_balance: WBalance) -> Ratio {
        let total_borrows = self.get_total_borrows();
        let total_reserves = self.get_total_reserves();
        let total_supplies = self.get_total_supplies();

        self.calculate_exchange_rate(
            underlying_balance,
            total_borrows,
            total_reserves,
            total_supplies,
        )
    }

    fn calculate_exchange_rate(
        &self,
        underlying_balance: WBalance,
        total_borrows: Balance,
        total_reserves: Balance,
        total_supplies: Balance,
    ) -> Ratio {
        if total_supplies == 0 {
            return Ratio(self.initial_exchange_rate);
        }
        Ratio(
            (Balance::from(underlying_balance) + total_borrows - total_reserves) * RATIO_DECIMALS.0
                / total_supplies,
        )
    }

    pub fn terra_gas(&self, gas: u64) -> Gas {
        TGAS * gas
    }

    pub fn mutex_account_lock(
        &mut self,
        action: Actions,
        amount: WBalance,
        gas: Gas,
    ) -> PromiseOrValue<U128> {
        controller::mutex_lock(
            action.clone(),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        )
            .then(ext_self::mutex_lock_callback(
                action,
                amount,
                env::current_account_id(),
                NO_DEPOSIT,
                gas,
            ))
            .into()
    }

    pub fn mutex_account_unlock(&mut self) {
        controller::mutex_unlock(self.get_controller_address(), NO_DEPOSIT, self.terra_gas(5));
    }

    pub fn get_total_supplies(&self) -> Balance {
        self.token.total_supply
    }

    pub fn get_total_borrows(&self) -> Balance {
        self.user_profiles
            .iter()
            .map(|(_, value)| value.borrows)
            .sum()
    }

    pub fn get_total_reserves(&self) -> Balance {
        self.total_reserves
    }

    pub fn get_repay_info(&self, user_id: AccountId, underlying_balance: WBalance) -> RepayInfo {
        let borrow_rate = self.get_borrow_rate(
            underlying_balance,
            WBalance::from(self.get_total_borrows()),
            WBalance::from(self.total_reserves),
        );
        let user_borrows = self.get_account_borrows(user_id.clone());

        let borrow_accrued_interest = self
            .config
            .get()
            .unwrap()
            .interest_rate_model
            .calculate_accrued_interest(
                borrow_rate,
                user_borrows,
                self.get_accrued_borrow_interest(user_id),
            );
        let accumulated_interest = borrow_accrued_interest.accumulated_interest;
        let accrued_interest_per_block = user_borrows * borrow_rate.0 / RATIO_DECIMALS.0;

        RepayInfo {
            accrued_interest_per_block: WBalance::from(accrued_interest_per_block),
            total_amount: WBalance::from(accumulated_interest + user_borrows),
            borrow_amount: WBalance::from(user_borrows),
            accumulated_interest: WBalance::from(accumulated_interest),
        }
    }

    pub fn get_withdraw_info(
        &self,
        user_id: AccountId,
        underlying_balance: WBalance,
    ) -> WithdrawInfo {
        let exchange_rate = U128::from(self.get_exchange_rate(underlying_balance).0);
        let interest_rate_model = self.config.get().unwrap().interest_rate_model;
        let supply_rate: Ratio = self.get_supply_rate(
            underlying_balance,
            WBalance::from(self.get_total_borrows()),
            WBalance::from(self.total_reserves),
            WBalance::from(interest_rate_model.get_reserve_factor().0),
        );
        let accrued_supply_interest = interest_rate_model.calculate_accrued_interest(
            supply_rate,
            self.get_supplies_by_account(user_id.clone()),
            self.get_accrued_supply_interest(user_id.clone()),
        );
        let total_interest = U128::from(self
            .get_accrued_supply_interest(user_id)
            .accumulated_interest
            + accrued_supply_interest.accumulated_interest);

        WithdrawInfo {
            exchange_rate,
            total_interest,
        }
    }

    pub fn calculate_reward_amount(
        &self,
        account_id: AccountId,
        reward_setting: &RewardSetting,
        current_block: BlockHeight,
        last_recalculation_block: BlockHeight,
    ) -> Balance {
        let blocks_per_period = match reward_setting.reward_per_period.period {
            RewardPeriod::Day => BLOCK_PER_DAY,
            RewardPeriod::Week => BLOCK_PER_WEEK,
        };
        Balance::from(reward_setting.reward_per_period.amount)
            * (self.token.accounts.get(&account_id).unwrap_or(0) * 10u128.pow(8)
            / self.get_total_supplies())
            * ((current_block - last_recalculation_block) * 10u64.pow(8) / blocks_per_period)
            as u128
            / 10u128.pow(16)
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn mint(&mut self, account_id: AccountId, amount: WBalance) {
        if self.token.accounts.get(&account_id).is_none() {
            self.token.internal_register_account(&account_id);
        };
        self.token.internal_deposit(&account_id, amount.into());
    }

    #[private]
    pub fn burn(&mut self, account_id: &AccountId, amount: WBalance) {
        if !self.token.accounts.contains_key(&account_id.clone()) {
            panic!("User with account {} wasn't found", account_id.clone());
        }
        self.token.internal_withdraw(account_id, amount.into());
    }

    #[private]
    pub fn mutex_lock_callback(
        &mut self,
        action: Actions,
        amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        match action {
            Actions::Repay => self.post_repay(amount),
            Actions::Withdraw => self.post_withdraw(amount),
            Actions::Supply => self.post_supply(amount),
            Actions::Borrow => self.post_borrow(amount),
            _ => {
                panic!("Incorrect action at mutex lock callback")
            }
        }
    }

    pub fn ft_total_supply(&self) -> U128 {
        self.token.ft_total_supply()
    }

    pub fn ft_balance_of(&self, account_id: AccountId) -> U128 {
        self.token.ft_balance_of(account_id)
    }
}

impl fmt::Display for Events {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Events::BorrowFailedToGetUnderlyingBalance(
                account,
                balance,
                contract_id,
                underlying_token_id,
            ) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "BorrowFailedToGetUnderlyingBalance", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to get {} balance on {}"}}}}"#,
                account, balance, contract_id, underlying_token_id
            ),
            Events::BorrowFailedToIncreaseBorrowOnController(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "BorrowFailedToInceaseBorrowOnController", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to make borrow for {} on {} token amount"}}}}"#,
                account, balance, account, balance
            ),
            Events::BorrowSuccess(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "BorrowSuccess", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,
                account, balance
            ),
            Events::BorrowFailedToFallback(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "BorrowFailedToMakeFallback", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to revert state for {}"}}}}"#,
                account, balance, account
            ),
            Events::BorrowFallbackSuccess(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "BorrowFallbackSuccess", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,
                account, balance
            ),
            Events::RepayFailedToGetUnderlyingBalance(
                account,
                balance,
                contract_id,
                underlying_token_id,
            ) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "RepayFailedToGetUnderlyingBalance", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to get {} balance on {}"}}}}"#,
                account, balance, contract_id, underlying_token_id
            ),
            Events::RepayFailedToUpdateUserBalance(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "RepayFailedToUpdateUserBalance", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to update user {} balance {}: user is not registered"}}}}"#,
                account, balance, account, balance
            ),
            Events::RepaySuccess(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "RepaySuccess", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,
                account, balance
            ),
            Events::SupplyFailedToGetUnderlyingBalance(
                account,
                balance,
                contract_id,
                underlying_token_id,
            ) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "SupplyFailedToGetUnderlyingBalance", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to get {} balance on {}"}}}}"#,
                account, balance, contract_id, underlying_token_id
            ),
            Events::SupplyFailedToIncreaseSupplyOnController(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "SupplyFailedToInceaseSupplyOnController", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to increase {} supply balance of {} on controller"}}}}"#,
                account, balance, account, balance
            ),
            Events::SupplySuccess(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "SupplySuccess", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,
                account, balance
            ),
            Events::WithdrawFailedToGetUnderlyingBalance(
                account,
                balance,
                contract_id,
                underlying_token_id,
            ) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "WithdrawFailedToGetUnderlyingBalance", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to get {} balance on {}"}}}}"#,
                account, balance, contract_id, underlying_token_id
            ),
            Events::WithdrawFailedToDecreaseSupplyOnController(account, balance, contract_id) => {
                write!(
                    f,
                    r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "WithdrawFailedToDecreaseSupplyOnController", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to decrease {} supply balance of {} on controller"}}}}"#,
                    account, balance, account, contract_id
                )
            }
            Events::WithdrawSuccess(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "WithdrawSuccess", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,
                account, balance
            ),
            Events::WithdrawFailedToFallback(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "WithdrawFailedToMakeFallback", "data": {{"account_id": "{}", "amount": "{}", "reason": "failed to revert state for {}"}}}}"#,
                account, balance, account
            ),
            Events::WithdrawFallbackSuccess(account, balance) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "WithdrawFallbackSuccess", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,
                account, balance
            ),
            Events::LiquidationSuccess(liquidator, borrower, amount_liquidate) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "LiquidationSuccess", "data": {{"liquidator_account_id": "{}", "borrower_account_id": {},"amount": "{}"}}}}"#,
                liquidator, borrower, amount_liquidate
            ),
            Events::LiquidationFailed(liquidator, borrower, amount_liquidate) => write!(
                f,
                r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "LiquidationFailed", "data": {{"liquidator_account_id": "{}", "borrower_account_id": {},"amount": "{}"}}}}"#,
                liquidator, borrower, amount_liquidate
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::RewardAmount;
    use crate::RewardPeriod::Day;
    use crate::{Config, Contract};
    use crate::{InterestRateModel, RewardSetting, VestingPlans};
    use general::ratio::Ratio;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use general::wbalance::WBalance;

    pub fn init_env() -> Contract {
        let (dtoken_account, underlying_token_account, controller_account) =
            (alice(), bob(), carol());

        Contract::new(Config {
            initial_exchange_rate: U128(10000),
            underlying_token_id: underlying_token_account,
            owner_id: dtoken_account,
            controller_account_id: controller_account,
            interest_rate_model: InterestRateModel::default(),
        })
    }

    #[test]
    fn test_calculate_reward_amount() {
        let mut contract = init_env();

        contract.mint(bob(), WBalance::from(100));
        contract.mint(carol(), WBalance::from(1000));

        let reward_setting = RewardSetting {
            token: alice(),
            reward_per_period: RewardAmount {
                period: Day,
                amount: WBalance::from(1000000),
            },
            lock_time: 20000,
            penalty: Ratio(1000),
            vesting: VestingPlans::None,
        };

        let reward_amount = contract.calculate_reward_amount(bob(), &reward_setting, 300, 100);

        assert_eq!(reward_amount, 252);
    }

    #[test]
    fn test_exchange_rate_initial_state() {
        let contract = init_env();

        let total_reserves = 10_000;
        let total_borrows = 0;
        let total_supplies = 0;

        // Ratio that represents xrate = 1
        assert_eq!(
            Ratio(10000),
            contract.calculate_exchange_rate(
                WBalance::from(10_000),
                total_borrows,
                total_reserves,
                total_supplies,
            )
        );
    }

    #[test]
    fn test_exchange_rate_after_supply() {
        let contract = init_env();

        // supply 1000 tokens
        let total_reserves = 10_000;
        let total_borrows = 0;
        let total_supplies = 1000;

        // Ratio that represents xrate = 1
        assert_eq!(
            Ratio(10000),
            contract.calculate_exchange_rate(
                WBalance::from(11_000),
                total_borrows,
                total_reserves,
                total_supplies,
            )
        );
    }

    #[test]
    fn test_exchange_rate_after_borrow() {
        let contract = init_env();

        // borrow 1000 tokens
        let total_reserves = 10_000;
        let total_borrows = 1000;
        let total_supplies = 1000;

        // Ratio that represents xrate = 1
        assert_eq!(
            Ratio(10000),
            contract.calculate_exchange_rate(
                WBalance::from(10_000),
                total_borrows,
                total_reserves,
                total_supplies,
            )
        );
    }

    #[test]
    fn test_exchange_rate_after_repay() {
        let contract = init_env();

        // repay 1000 borrows and 50 debt
        let total_reserves = 10_000;
        let total_borrows = 0;
        let total_supplies = 1000;

        // Ratio that represents xrate = 1.05
        assert_eq!(
            Ratio(10500),
            contract.calculate_exchange_rate(
                WBalance::from(11_050),
                total_borrows,
                total_reserves,
                total_supplies,
            )
        );
    }

    #[test]
    fn test_exchange_rate_after_withdraw() {
        let contract = init_env();

        // withdraw all supplies
        let total_reserves = 10_002.5;
        let total_borrows = 0;
        let total_supplies = 0;

        // Ratio that represents xrate = 1
        assert_eq!(
            Ratio(10000),
            contract.calculate_exchange_rate(
                WBalance::from(10_002.5 as u128),
                total_borrows,
                total_reserves as u128,
                total_supplies,
            )
        );
    }
}
