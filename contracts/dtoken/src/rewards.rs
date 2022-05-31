use crate::*;
use general::ratio::Ratio;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, PartialEq, Clone, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum CampaignType {
    Supply,
    Borrow,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Vesting {
    /// Campaign vesting start time, seconds
    start_time: u64,
    /// Campaign vesting end time, seconds
    end_time: u64,
    /// Penalty amount which will be arrested in case of early withdraw
    penalty: Ratio,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardCampaign {
    /// Reward campaign type
    campaign_type: CampaignType,
    /// Campaign start time seconds
    start_time: u64,
    /// Campaign end time seconds
    end_time: u64,
    /// Reward token address
    token: AccountId,
    /// Token ticker id
    ticker_id: String,
    /// Reward tokens total amount
    reward_amount: WBalance,
    /// Last time when rewardPerToken was recomputed/updated
    last_update_time: u64,
    /// Represent the token rewards amount which contract should pay for 1 token putted into liquidity
    rewards_per_token: WBalance,
    /// Vesting configuration
    vesting: Vesting,
}

impl Contract {
    pub fn get_reward_campaign_by_id(&self, campaign_id: String) -> Option<RewardCampaign> {
        self.reward_campaigns.get(&campaign_id)
    }
}

#[near_bindgen]
impl Contract {
    pub fn add_reward_campaign(&mut self, reward_campaign: RewardCampaign) -> String {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );
        let campaign_id: String = self.request_unique_id();
        self.reward_campaigns.insert(&campaign_id, &reward_campaign);
        campaign_id
    }

    pub fn remove_reward_campaign(&mut self, campaign_id: String) {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );
        require!(
            self.get_reward_campaign_by_id(campaign_id.clone())
                .is_some(),
            "Reward campaign by this key doesn't exists"
        );
        self.reward_campaigns.remove(&campaign_id);
    }
}

#[cfg(test)]
mod tests {
    use crate::rewards::{CampaignType, Vesting};
    use crate::{Config, Contract};
    use crate::{InterestRateModel, RewardCampaign};
    use general::ratio::Ratio;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, AccountId, VMContext};

    use std::convert::TryFrom;

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

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(AccountId::try_from(alice().to_string()).unwrap())
            .signer_account_id(AccountId::try_from(alice().to_string()).unwrap())
            .predecessor_account_id(AccountId::try_from(alice().to_string()).unwrap())
            .block_index(101)
            .block_timestamp(0)
            .is_view(is_view)
            .build()
    }

    fn get_campaign() -> RewardCampaign {
        let vesting = Vesting {
            start_time: 1652562000,
            end_time: 1653858000,
            penalty: Ratio(99999),
        };
        let campaign = RewardCampaign {
            campaign_type: CampaignType::Supply,
            start_time: 1651352400,
            end_time: 1652562000,
            token: carol(),
            ticker_id: "CAROL".to_string(),
            reward_amount: U128(10000000000),
            last_update_time: 1651352400,
            rewards_per_token: U128(77777777),
            vesting,
        };
        return campaign;
    }

    #[test]
    fn test_remove_reward_campaign() {
        let mut contract = init_env();
        let campaign = get_campaign();
        let context = get_context(false);
        testing_env!(context);

        let campaign_id = contract.add_reward_campaign(campaign.clone());
        assert!(
            contract
                .get_reward_campaign_by_id(campaign_id.clone())
                .is_some(),
            "Campaign with id {} wasn't added",
            campaign_id.clone()
        );

        contract.remove_reward_campaign(campaign_id.clone());
        assert!(
            contract
                .get_reward_campaign_by_id(campaign_id.clone())
                .is_none(),
            "Campaign with id {} wasn't removed",
            campaign_id
        );
    }

    #[test]
    fn test_add_reward_campaign() {
        let mut contract = init_env();
        let campaign = get_campaign();
        let context = get_context(false);
        testing_env!(context);
        let campaign_id = contract.add_reward_campaign(campaign.clone());
        assert_eq!(
            campaign_id,
            contract.get_unique_id(),
            "CampaignId doesn't match for expected result"
        );
        let received_campaign = contract.get_reward_campaign_by_id(campaign_id);
        assert!(received_campaign.is_some(), "Campaign wasn't found");
        let received_campaign_unwrapped = received_campaign.unwrap();
        assert_eq!(
            campaign.campaign_type, received_campaign_unwrapped.campaign_type,
            "Campaigns are not similar"
        );
        assert_eq!(
            campaign.start_time, received_campaign_unwrapped.start_time,
            "Campaigns are not similar"
        );
        assert_eq!(
            campaign.end_time, received_campaign_unwrapped.end_time,
            "Campaigns are not similar"
        );
        assert_eq!(
            campaign.token, received_campaign_unwrapped.token,
            "Campaigns are not similar"
        );
        assert_eq!(
            campaign.last_update_time, received_campaign_unwrapped.last_update_time,
            "Campaigns are not similar"
        );
    }
}
