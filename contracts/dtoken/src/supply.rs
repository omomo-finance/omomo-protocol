use crate::*;
use general::ratio::{BigBalance, Ratio};

const GAS_FOR_SUPPLY: Gas = Gas(120_000_000_000_000);

impl Contract {
    pub fn supply(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_SUPPLY,
            "Prepaid gas is not enough for supply flow"
        );
        self.mutex_account_lock(Actions::Supply, token_amount, self.terra_gas(120))
    }
    pub fn post_supply(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
        }
        self.adjust_rewards_by_campaign_type(CampaignType::Supply);
        underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
            .then(ext_self::supply_balance_of_callback(
                token_amount,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(50),
            ))
            .into()
    }

    pub fn get_account_supplies(&self, account: AccountId) -> Balance {
        self.token.accounts.get(&account).unwrap_or_default()
    }
}

#[near_bindgen]
impl Contract {
    #[allow(dead_code)]
    #[private]
    pub fn supply_balance_of_callback(
        &mut self,
        token_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::SupplyFailedToGetUnderlyingBalance(
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
            PromiseResult::Successful(result) => {
                let actual_balance: WBalance = near_sdk::serde_json::from_slice::<WBalance>(&result)
                    .unwrap()
                    .into();
                let mut funded_by_underlying_token = WBalance::from(0);

                for hm in self.funded_reward_amount.values() {
                    match hm.get(&self.get_underlying_contract_address()) {
                        None => {}
                        Some(balance) => { funded_by_underlying_token = (funded_by_underlying_token.0 + balance).into() }
                    };
                }

                (actual_balance.0 - funded_by_underlying_token.0).into()
            }
        };

        let exchange_rate =
            self.get_exchange_rate((balance_of - Balance::from(token_amount)).into());
        let dtoken_amount =
            WBalance::from((BigBalance::from(token_amount.0) / exchange_rate).round_u128());

        let interest_rate_model = self.config.get().unwrap().interest_rate_model;
        let supply_rate: Ratio = self.get_supply_rate(
            U128(balance_of - Balance::from(token_amount)),
            U128(self.get_total_borrows()),
            U128(self.get_total_reserves()),
            interest_rate_model.get_reserve_factor(),
        );
        let accrued_interest = self.get_accrued_supply_interest(env::signer_account_id());
        let accrued_supply_interest = interest_rate_model.calculate_accrued_interest(
            supply_rate,
            self.get_account_supplies(env::signer_account_id()),
            accrued_interest,
        );
        self.set_accrued_supply_interest(env::signer_account_id(), accrued_supply_interest);

        // Dtokens minting and adding them to the user account
        self.mint(self.get_signer_address(), dtoken_amount);

        log!(
            "Supply from Account {} to Dtoken contract {} with tokens amount {} was successfully done!",
            self.get_signer_address(),
            self.get_contract_address(),
            Balance::from(token_amount)
        );

        controller::increase_supplies(
            env::signer_account_id(),
            self.get_contract_address(),
            dtoken_amount,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        )
            .then(ext_self::controller_increase_supplies_callback(
                token_amount,
                dtoken_amount,
                env::current_account_id(),
                NO_DEPOSIT,
                self.terra_gas(20),
            ))
            .into()
    }

    #[allow(dead_code)]
    #[private]
    pub fn controller_increase_supplies_callback(
        &mut self,
        amount: WBalance,
        dtoken_amount: WBalance,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::SupplyFailedToIncreaseSupplyOnController(
                    env::signer_account_id(),
                    Balance::from(amount)
                )
            );
            self.burn(&self.get_signer_address(), dtoken_amount);

            self.mutex_account_unlock();
            return PromiseOrValue::Value(amount);
        }
        self.update_campaigns_market_total_by_type(CampaignType::Supply);
        log!(
            "{}",
            Events::SupplySuccess(env::signer_account_id(), Balance::from(amount))
        );
        self.mutex_account_unlock();
        PromiseOrValue::Value(U128(0))
    }
}
