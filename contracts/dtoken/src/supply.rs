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
}

#[near_bindgen]
impl Contract {
    pub fn post_supply(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            return PromiseOrValue::Value(token_amount);
        }
        self.adjust_rewards_by_campaign_type(CampaignType::Supply);

        let balance_of = self.view_contract_balance();

        let exchange_rate = self.get_exchange_rate(balance_of);
        let dtoken_amount = WBalance::from(
            (BigBalance::from(Balance::from(token_amount)) / exchange_rate).round_u128(),
        );

        let interest_rate_model = self.config.get().unwrap().interest_rate_model;
        let supply_rate: Ratio = self.get_supply_rate(
            balance_of,
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

    pub fn get_account_supplies(&self, account: AccountId) -> Balance {
        self.token.accounts.get(&account).unwrap_or_default()
    }

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
        self.increase_contract_balance(amount);

        self.mutex_account_unlock();
        PromiseOrValue::Value(U128(0))
    }
}
