use crate::*;
use general::ratio::Ratio;

const GAS_FOR_WITHDRAW: Gas = Gas(180_000_000_000_000);

impl Contract {
    pub fn post_withdraw(&mut self, dtoken_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(dtoken_amount);
        }
        self.adjust_rewards_by_campaign_type(CampaignType::Supply);

        let balance_of = self.view_contract_balance();

        let exchange_rate: Ratio = self.get_exchange_rate(balance_of);

        let interest_rate_model = self.config.get().unwrap().interest_rate_model;

        let supply_rate: Ratio = self.get_supply_rate(
            balance_of,
            U128(self.get_total_borrows()),
            U128(self.get_total_reserves()),
            interest_rate_model.get_reserve_factor(),
        );

        let accrued_supply_interest = interest_rate_model.calculate_accrued_interest(
            supply_rate,
            self.get_account_supplies(env::signer_account_id()),
            self.get_accrued_supply_interest(env::signer_account_id()),
        );

        let token_amount: Balance =
            (Ratio::from(Balance::from(dtoken_amount)) * exchange_rate).round_u128();

        let whole_amount: Balance = token_amount + accrued_supply_interest.accumulated_interest;

        self.set_accrued_supply_interest(env::signer_account_id(), accrued_supply_interest);

        controller::withdraw_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            dtoken_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(10),
        )
        .then(ext_self::withdraw_supplies_callback(
            env::signer_account_id(),
            token_amount.into(),
            dtoken_amount,
            whole_amount.into(),
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(80),
        ))
        .into()
    }
}

#[near_bindgen]
impl Contract {
    pub fn withdraw(&mut self, amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_WITHDRAW,
            "Prepaid gas is not enough for withdraw flow"
        );
        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );
        assert!(
            amount.0
                <= self
                    .token
                    .accounts
                    .get(&env::signer_account_id())
                    .unwrap_or(0),
            "The account doesn't have enough digital tokens to do withdraw"
        );
        self.mutex_account_lock(Actions::Withdraw, amount, GAS_FOR_WITHDRAW)
    }

    #[private]
    pub fn withdraw_supplies_callback(
        &mut self,
        user_account: AccountId,
        token_amount: WBalance,
        dtoken_amount: WBalance,
        whole_amount: WBalance,
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
            self.to_decimals_token(whole_amount),
            Some(format!(
                "Withdraw with token_amount {}",
                Balance::from(whole_amount)
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
            self.terra_gas(30),
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
            self.set_accrued_supply_interest(env::signer_account_id(), AccruedInterest::default());

            self.decrease_contract_balance(dtoken_amount);

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
                self.terra_gas(10),
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
        self.update_campaigns_market_total_by_type(CampaignType::Supply);
        self.mutex_account_unlock();
        log!(
            "{}",
            Events::WithdrawFallbackSuccess(env::signer_account_id(), Balance::from(token_amount))
        );
        PromiseOrValue::Value(token_amount)
    }
}
