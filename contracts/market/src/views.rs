use crate::*;
use general::ratio::Ratio;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub underlying_token: AccountId,
    /// WARN: should be the same as `underlying_token.ft_metadata.decimals`
    pub underlying_token_decimals: u8,

    /// Total supplies with precision 10^24
    pub total_supplies: WBalance,
    /// Total borrows with precision 10^24
    pub total_borrows: WBalance,
    /// Total reserves with precision 10^24
    pub total_reserves: WBalance,

    pub exchange_rate_ratio: WRatio,
    pub interest_rate_ratio: WRatio,
    pub borrow_rate_ratio: WRatio,
}

#[near_bindgen]
impl Contract {
    pub fn view_contract_balance(&self) -> WBalance {
        WBalance::from(self.contract_balance)
    }

    pub fn view_total_supplies(&self) -> WBalance {
        WBalance::from(self.get_total_supplies())
    }

    pub fn view_total_borrows(&self) -> WBalance {
        WBalance::from(self.get_total_borrows())
    }

    pub fn view_total_reserves(&self) -> WBalance {
        WBalance::from(self.get_total_reserves())
    }

    pub fn view_market_data(&self) -> MarketData {
        let total_supplies = self.get_total_supplies();
        let total_borrows = self.get_total_borrows();
        let total_reserves = self.get_total_reserves();

        let contract_ft_balance = self.view_contract_balance();

        let exchange_rate = self.get_exchange_rate(contract_ft_balance);
        let reserve_factor = self
            .config
            .get()
            .unwrap()
            .interest_rate_model
            .get_reserve_factor();

        let interest_rate = self.get_supply_rate(
            contract_ft_balance,
            WBalance::from(total_borrows),
            WBalance::from(total_reserves),
            reserve_factor,
        );
        let borrow_rate = self.get_borrow_rate(
            contract_ft_balance,
            WBalance::from(total_borrows),
            WBalance::from(total_reserves),
        );

        MarketData {
            underlying_token: self.underlying_token.clone(),
            underlying_token_decimals: self.underlying_token_decimals,
            total_supplies: WBalance::from(total_supplies),
            total_borrows: WBalance::from(total_borrows),
            total_reserves: WBalance::from(total_reserves),
            exchange_rate_ratio: WRatio::from(exchange_rate),
            interest_rate_ratio: WRatio::from(interest_rate),
            borrow_rate_ratio: WRatio::from(borrow_rate),
        }
    }

    pub fn view_repay_info(&self, user_id: AccountId, ft_balance: WBalance) -> RepayInfo {
        self.get_repay_info(user_id, ft_balance)
    }

    pub fn view_exchange_rate(&self, ft_balance: WBalance) -> Ratio {
        self.get_exchange_rate(ft_balance)
    }

    pub fn view_withdraw_info(&self, user_id: AccountId, ft_balance: WBalance) -> WithdrawInfo {
        self.get_withdraw_info(user_id, ft_balance)
    }

    pub fn view_reward_campaigns(&self) -> Vec<RewardCampaignExtended> {
        self.get_reward_campaigns_extended()
    }

    pub fn view_rewards_list(&self, user_id: AccountId) -> HashMap<String, Reward> {
        self.get_rewards_list(user_id)
    }

    pub fn view_reward_state(&self, user_id: AccountId, campaign_id: String) -> RewardState {
        self.get_reward_state(user_id, campaign_id)
    }
}

#[cfg(test)]
mod tests {
    use crate::InterestRateModel;
    use general::ratio::Ratio;
    use general::WBalance;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    use crate::views::MarketData;
    use crate::{Config, Contract};

    pub fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(alice())
            .signer_account_id(alice())
            .is_view(is_view)
            .build()
    }

    pub fn init_test_env(is_admin: bool) -> Contract {
        let (dtoken_account, underlying_token_account, controller_account) =
            (alice(), bob(), carol());

        if is_admin {
            testing_env!(get_context(false));
        }

        let mut contract = Contract::new(Config {
            initial_exchange_rate: U128::from(Ratio::one()),
            underlying_token_id: underlying_token_account,
            underlying_token_decimals: 24,
            owner_id: dtoken_account,
            controller_account_id: controller_account,
            interest_rate_model: InterestRateModel::default(),
            disable_transfer_token: true,
        });

        if is_admin {
            contract.set_total_reserves(200);
        }

        contract
    }

    #[test]
    fn test_view_repay_info() {
        let contract = init_test_env(false);

        let repay = contract.view_repay_info(bob(), WBalance::from(0));

        assert_eq!(
            // as there are no borrows yet accrued_interest_per_block = 0
            repay.accrued_interest_per_block,
            WBalance::from(Ratio::one()),
            "RepayInfo accrued_interest_per_block is not matches to expected"
        );
        assert_eq!(
            repay.total_amount,
            WBalance::from(0u128),
            "RepayInfo total_amount is not matches to expected"
        );
        assert_eq!(
            repay.borrow_amount,
            WBalance::from(0u128),
            "RepayInfo borrow_amount is not matches to expected"
        );
        assert_eq!(
            repay.accumulated_interest,
            WBalance::from(0u128),
            "RepayInfo accumulated_interest is not matches to expected"
        );
    }

    #[test]
    fn test_view_market_data() {
        let contract = init_test_env(true);

        let gotten_md = contract.view_market_data();

        let _expected_md = MarketData {
            underlying_token: contract.underlying_token,
            underlying_token_decimals: contract.underlying_token_decimals,
            total_supplies: U128(0),
            total_borrows: U128(0),
            total_reserves: U128(200),
            exchange_rate_ratio: U128::from(Ratio::one()),
            interest_rate_ratio: U128(0),
            borrow_rate_ratio: U128::from(Ratio::one()),
        };

        assert_eq!(
            &gotten_md.total_supplies, &_expected_md.total_supplies,
            "Market total supplies values check has been failed"
        );
        assert_eq!(
            &gotten_md.total_borrows, &_expected_md.total_borrows,
            "Market total borrows values check has been failed"
        );
        assert_eq!(
            &gotten_md.total_reserves, &_expected_md.total_reserves,
            "Market total reserves values check has been failed"
        );
        assert_eq!(
            &gotten_md.exchange_rate_ratio, &_expected_md.exchange_rate_ratio,
            "Exchange rate values check has been failed"
        );
        assert_eq!(
            &gotten_md.interest_rate_ratio, &_expected_md.interest_rate_ratio,
            "Interest rate values check has been failed"
        );
        assert_eq!(
            &gotten_md.borrow_rate_ratio, &_expected_md.borrow_rate_ratio,
            "Borrow rate values check has been failed"
        );
    }

    #[test]
    fn test_view_withdraw_info() {
        let contract = init_test_env(false);

        let withdraw_info = contract.view_withdraw_info(bob(), U128(1000));

        // total interest should be 0
        // exchange_rate = initial_exchange_rate = 1000000000000000000000000

        assert_eq!(
            withdraw_info.exchange_rate,
            U128::from(Ratio::one()),
            "Withdraw exchange_rate is not matches to expected"
        );
        assert_eq!(
            withdraw_info.total_interest,
            U128(0),
            "Withdraw total_interest is not matches to expected"
        );
    }
}
