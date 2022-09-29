use crate::*;

const GAS_FOR_REPAY: Gas = Gas(120_000_000_000_000);

impl Contract {
    pub fn repay(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_REPAY,
            "Prepaid gas is not enough for repay flow"
        );
        self.mutex_account_lock(Actions::Repay, token_amount, self.terra_gas(140))
    }

    pub fn post_repay(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
        }
        self.adjust_rewards_by_campaign_type(CampaignType::Borrow);
        underlying_token::ft_balance_of(
            self.get_contract_address(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
            .then(ext_self::repay_balance_of_callback(
                token_amount,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(60),
            ))
            .into()
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn repay_balance_of_callback(&mut self, token_amount: WBalance) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::RepayFailedToGetUnderlyingBalance(
                    env::signer_account_id(),
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
            U128(balance_of - Balance::from(token_amount)),
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
                self.get_account_borrows(env::signer_account_id()),
                self.get_accrued_borrow_interest(env::signer_account_id()),
            );

        // let borrow_amount = self.get_account_borrows(env::signer_account_id());
        //
        // let borrow_with_rate_amount = borrow_amount + borrow_accrued_interest.accumulated_interest;
        self.set_accrued_borrow_interest(env::signer_account_id(), borrow_accrued_interest.clone());

        let borrow_amount = self.get_account_borrows(env::signer_account_id());

        // if borrow is less then accrued interest then we do not decerease borrows on controller
        // if borrow is up to borrows + accrued then we repay accrued and decrease remaining tokens on controller
        // if borrow is over borrows + accrued then we decrease full borrows on controller
        let borrow_decrease_amount = if token_amount.0
            <= borrow_accrued_interest.accumulated_interest
        {
            0u128
        } else if token_amount.0 <= borrow_amount + borrow_accrued_interest.accumulated_interest {
            token_amount.0 - borrow_accrued_interest.accumulated_interest
        } else {
            borrow_amount
        };

        controller::repay_borrows(
            env::signer_account_id(),
            self.get_contract_address(),
            U128(borrow_decrease_amount),
            borrow_accrued_interest.last_recalculation_block,
            U128::from(borrow_rate),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        )
            .then(ext_self::controller_repay_borrows_callback(
                token_amount,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(20),
            ))
            .into()
    }

    #[private]
    pub fn controller_repay_borrows_callback(
        &mut self,
        token_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::RepayFailedToUpdateUserBalance(
                    env::signer_account_id(),
                    Balance::from(token_amount)
                )
            );
            self.mutex_account_unlock();
            return PromiseOrValue::Value(token_amount);
        }

        let mut borrow_interest = self.get_accrued_borrow_interest(env::signer_account_id());
        // update total reserves only after successful repay
        let new_total_reserve = self.get_total_reserves()
            + (Ratio::from(borrow_interest.accumulated_interest) * self.model.get_reserve_factor())
            .round_u128();
        self.set_total_reserves(new_total_reserve);

        let dust_balance = token_amount
            .0
            .saturating_sub(self.get_account_borrows(env::signer_account_id()))
            .saturating_sub(borrow_interest.accumulated_interest);

        let borrow_accrued_interest = self.get_accrued_borrow_interest(env::signer_account_id());
        let borrow_amount = self.get_account_borrows(env::signer_account_id());

        // if borrow is less then accrued interest then we decerease only accrued interest
        // if borrow is up to borrows + accrued then we repay accrued and decrease borrows by remaining tokens
        // if borrow is over borrows + accrued then we decrease full borrows
        if token_amount.0 <= borrow_accrued_interest.accumulated_interest {
            borrow_interest.accumulated_interest -= token_amount.0;
            self.set_accrued_borrow_interest(env::signer_account_id(), borrow_interest);
            self.increase_contract_balance(token_amount)
        } else if token_amount.0 <= borrow_amount + borrow_accrued_interest.accumulated_interest {
            self.decrease_borrows(
                env::signer_account_id(),
                WBalance::from(token_amount.0 - borrow_interest.accumulated_interest),
            );
            self.set_accrued_borrow_interest(env::signer_account_id(), AccruedInterest::default());
            self.increase_contract_balance(token_amount)
        } else {
            self.decrease_borrows(env::signer_account_id(), U128(borrow_amount));
            self.set_accrued_borrow_interest(env::signer_account_id(), AccruedInterest::default());
            self.increase_contract_balance(U128::from(borrow_amount + borrow_accrued_interest.accumulated_interest))
        };

        self.mutex_account_unlock();
        self.update_campaigns_market_total_by_type(CampaignType::Borrow);
        log!(
            "{}",
            Events::RepaySuccess(env::signer_account_id(), Balance::from(token_amount))
        );


        PromiseOrValue::Value(U128(dust_balance))
    }
}
