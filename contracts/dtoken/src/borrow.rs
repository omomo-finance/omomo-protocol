use crate::*;
use near_sdk::env::signer_account_id;

const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);

impl Contract {
    pub fn decrease_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let borrows = self.get_account_borrows(account.clone());
        let new_borrows = borrows - Balance::from(token_amount);

        self.set_account_borrows(account, U128(new_borrows))
    }

    pub fn increase_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let borrows: Balance = self.get_account_borrows(account.clone());
        let new_borrows = borrows + Balance::from(token_amount);

        self.set_account_borrows(account, U128(new_borrows))
    }
}

#[near_bindgen]
impl Contract {
    pub fn post_borrow(
        &mut self,
        token_amount: WBalance,
        account_to_borrow: AccountId,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
        }

        if account_to_borrow != self.get_eligible_to_borrow_uncollateralized_account() {
            self.adjust_rewards_by_campaign_type(CampaignType::Borrow);
        }

        underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
        .then(ext_self::borrow_balance_of_callback(
            token_amount,
            account_to_borrow,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(150),
        ))
        .into()
    }
}

#[near_bindgen]
impl Contract {
    pub fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_BORROW,
            "Prepaid gas is not enough for borrow flow"
        );

        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );

        let mut account_to_borrow = env::predecessor_account_id();

        if !self.is_allowed_to_borrow_uncollateralized() {
            account_to_borrow = signer_account_id();
        }

        self.mutex_account_lock(
            Actions::Borrow { account_to_borrow },
            amount,
            self.terra_gas(180),
        )
    }

    #[private]
    pub fn borrow_balance_of_callback(
        &mut self,
        token_amount: WBalance,
        account_to_borrow: AccountId,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::BorrowFailedToGetUnderlyingBalance(
                    account_to_borrow,
                    Balance::from(token_amount),
                    self.get_contract_address(),
                    self.get_underlying_contract_address()
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(token_amount);
        }

        let balance_of: Balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let borrow_rate = self.get_borrow_rate(
            U128(balance_of),
            U128(self.get_total_borrows()),
            U128(self.get_total_reserves()),
        );
        let borrow_accrued_interest = self
            .config
            .get()
            .unwrap()
            .interest_rate_model
            .calculate_accrued_interest(
                borrow_rate,
                self.get_account_borrows(account_to_borrow.clone()),
                self.get_accrued_borrow_interest(account_to_borrow.clone()),
            );
        self.set_accrued_borrow_interest(
            account_to_borrow.clone(),
            borrow_accrued_interest.clone(),
        );

        controller::make_borrow(
            account_to_borrow.clone(),
            self.get_contract_address(),
            token_amount,
            borrow_accrued_interest.last_recalculation_block,
            WRatio::from(borrow_rate),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(15),
        )
        .then(ext_self::make_borrow_callback(
            token_amount,
            account_to_borrow,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(80),
        ))
        .into()
    }

    #[private]
    pub fn make_borrow_callback(
        &mut self,
        token_amount: WBalance,
        account_to_borrow: AccountId,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::BorrowFailedToIncreaseBorrowOnController(
                    account_to_borrow,
                    Balance::from(token_amount)
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(token_amount);
        }

        underlying_token::ft_transfer(
            account_to_borrow.clone(),
            token_amount,
            Some(format!(
                "Borrow with token_amount {}",
                Balance::from(token_amount)
            )),
            self.get_underlying_contract_address(),
            ONE_YOCTO,
            self.terra_gas(10),
        )
        .then(ext_self::borrow_ft_transfer_callback(
            token_amount,
            account_to_borrow,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(40),
        ))
        .into()
    }

    #[private]
    pub fn borrow_ft_transfer_callback(
        &mut self,
        token_amount: WBalance,
        account_to_borrow: AccountId,
    ) -> PromiseOrValue<WBalance> {
        if is_promise_success() {
            self.increase_borrows(account_to_borrow.clone(), token_amount);
            self.update_campaigns_market_total_by_type(CampaignType::Borrow);
            self.mutex_account_unlock();
            log!(
                "{}",
                Events::BorrowSuccess(account_to_borrow, Balance::from(token_amount))
            );
            PromiseOrValue::Value(U128(0))
        } else {
            controller::decrease_borrows(
                account_to_borrow.clone(),
                self.get_contract_address(),
                token_amount,
                self.get_controller_address(),
                NO_DEPOSIT,
                self.terra_gas(5),
            )
            .then(ext_self::controller_decrease_borrows_fail_callback(
                token_amount,
                account_to_borrow,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(20),
            ))
            .into()
        }
    }

    #[private]
    pub fn controller_decrease_borrows_fail_callback(
        &mut self,
        token_amount: WBalance,
        account_to_borrow: AccountId,
    ) {
        if !is_promise_success() {
            self.add_inconsistent_account(env::signer_account_id());
            log!(
                "{}",
                Events::BorrowFailedToFallback(account_to_borrow, Balance::from(token_amount))
            );
        } else {
            self.mutex_account_unlock();
            log!(
                "{}",
                Events::BorrowFallbackSuccess(account_to_borrow, Balance::from(token_amount))
            );
        }
    }

    #[private]
    pub fn set_account_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        let mut user = self.user_profiles.get(&account).unwrap_or_default();
        user.borrows = Balance::from(token_amount);
        self.user_profiles.insert(&account, &user);

        self.get_account_borrows(account)
    }

    pub fn get_account_borrows(&self, account: AccountId) -> Balance {
        self.user_profiles.get(&account).unwrap_or_default().borrows
    }
}
