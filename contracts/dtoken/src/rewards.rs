use crate::*;
use general::ratio::{BigBalance, Ratio};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use std::cmp::min;
use std::fmt;

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, PartialEq, Clone, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum CampaignType {
    Supply,
    Borrow,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Vesting {
    /// Campaign vesting start time, seconds
    start_time: u64,
    /// Campaign vesting end time, seconds
    end_time: u64,
    /// Penalty amount which will be arrested in case of early withdraw
    penalty: Ratio,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, Deserialize, Clone)]
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

impl fmt::Display for RewardCampaign {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardCampaignExtended {
    /// Reward campaign id
    campaign_id: String,
    /// Reward campaign data
    campaign: RewardCampaign,
    /// Market total Supply/Borrow depends on reward campaign type
    market_total: WBalance,
    /// Rewards per day token amount
    rewards_per_day: WBalance,
}

impl fmt::Display for RewardCampaignExtended {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardState {
    max_claim_amount: WBalance,

    max_unlock_amount: WBalance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Reward {
    /// Reward campaign id
    campaign_id: String,
    /// Total rewards amount, default = 0
    amount: WBalance,
    /// The last rewards_per_token which used for rewards adjustment, default = 0
    rewards_per_token_paid: WBalance,
    /// Tokens total amount that has been claimed by the user
    claimed: WBalance,
    /// Tokens amount which were unlocked with penalty
    unlocked: WBalance,
}

impl fmt::Display for Reward {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Reward {
    pub fn new(campaig_id: String) -> Reward {
        Reward {
            campaign_id: campaig_id,
            amount: U128(0),
            rewards_per_token_paid: U128(0),
            claimed: U128(0),
            unlocked: U128(0),
        }
    }
}

impl Contract {
    pub fn get_reward_campaign_by_id(&self, campaign_id: String) -> Option<RewardCampaign> {
        self.reward_campaigns.get(&campaign_id)
    }

    pub fn get_reward_campaigns_extended(&self) -> Vec<RewardCampaignExtended> {
        self.reward_campaigns
            .iter()
            .filter(|(_, campaign)| campaign.end_time > env::block_height())
            .map(|(id, campaign)| RewardCampaignExtended {
                campaign_id: id,
                campaign: campaign.clone(),
                market_total: self.get_market_total(campaign.clone()),
                rewards_per_day: self.get_reward_tokens_per_day(campaign),
            })
            .collect::<Vec<RewardCampaignExtended>>()
    }

    pub fn get_rewards_per_time(&self, campaign: RewardCampaign, seconds: u128) -> WBalance {
        let rewards_per_second: u128 = campaign.reward_amount.0 * seconds
            / u128::from(campaign.end_time - campaign.start_time);
        WBalance::from(rewards_per_second)
    }

    pub fn get_rewards_per_second(&self, campaign: RewardCampaign) -> WBalance {
        self.get_rewards_per_time(campaign, 1)
    }

    pub fn get_reward_tokens_per_day(&self, campaign: RewardCampaign) -> WBalance {
        self.get_rewards_per_time(campaign, 24 * 60 * 60)
    }

    pub fn get_market_total(&self, campaign: RewardCampaign) -> WBalance {
        let total_amount = match campaign.campaign_type {
            CampaignType::Supply => self.get_total_supplies(), // Tokens
            CampaignType::Borrow => self.get_total_borrows(),  // Tokens
        };
        WBalance::from(total_amount)
    }

    pub fn get_account_total(&self, campaign: RewardCampaign, account_id: AccountId) -> WBalance {
        let account_total = match campaign.campaign_type {
            CampaignType::Supply => self.get_account_supplies(account_id),
            CampaignType::Borrow => self.get_account_borrows(account_id),
        };
        WBalance::from(account_total)
    }

    fn insert_map_if_not_exists(&mut self, account_id: AccountId) {
        let rewards_map: HashMap<String, Reward> = HashMap::new();
        *self.rewards.entry(account_id).or_default() = rewards_map;
    }

    pub fn get_accrued_rewards_per_token(&self, campaign_id: String) -> WBalance {
        if let Some(campaign) = self.get_reward_campaign_by_id(campaign_id.clone()) {
            let total = self.get_market_total(campaign.clone());
            let current_time = min(env::block_timestamp(), campaign.end_time);
            if total.0 == 0 {
                return WBalance::from(0);
            };

            return WBalance::from(
                self.get_rewards_per_time(
                    campaign.clone(),
                    u128::from(current_time - campaign.last_update_time),
                )
                .0 / total.0,
            );
        }
        panic!(
            "Campaign {} wasn't found on the current contract",
            campaign_id
        );
    }

    pub fn update_reward_campaign(&mut self, campaign_id: String) -> RewardCampaign {
        let mut campaign = self.get_reward_campaign_by_id(campaign_id.clone()).unwrap();
        let accrued_tokens = self.get_accrued_rewards_per_token(campaign_id.clone());
        campaign.rewards_per_token =
            WBalance::from(campaign.rewards_per_token.0 + accrued_tokens.0);
        campaign.last_update_time = min(env::block_timestamp(), campaign.end_time);
        self.reward_campaigns.insert(&campaign_id, &campaign);
        campaign
    }

    pub fn get_updated_reward_amount(&self, reward: &Reward, account_id: AccountId) -> WBalance {
        self.get_updated_reward_amount_with_accrued(reward, account_id, 0)
    }

    pub fn get_updated_reward_amount_with_accrued(
        &self,
        reward: &Reward,
        account_id: AccountId,
        accrued: Balance,
    ) -> WBalance {
        let campaign_option = self.get_reward_campaign_by_id(reward.campaign_id.clone());

        assert!(
            campaign_option.is_some(),
            "Campaign for reward {} not exists",
            reward
        );
        let campaign = campaign_option.unwrap();
        let total = self.get_account_total(campaign.clone(), account_id);
        WBalance::from(
            reward.amount.0
                + (total.0
                    * (campaign.rewards_per_token.0 - reward.rewards_per_token_paid.0 + accrued)),
        )
    }

    pub fn update_reward(&mut self, campaign_id: String, account_id: AccountId) -> Reward {
        let campaign = self.update_reward_campaign(campaign_id.clone());
        let default_reward = Reward::new(campaign_id.clone());

        self.insert_map_if_not_exists(account_id.clone());

        let account_rewards = self.rewards.get(&account_id).unwrap();
        let old_reward = account_rewards.get(&campaign_id).unwrap_or(&default_reward);
        let mut reward = Reward::new(campaign_id.clone());

        reward.amount = self.get_updated_reward_amount(old_reward, account_id.clone());
        reward.rewards_per_token_paid = campaign.rewards_per_token;
        reward.claimed = old_reward.claimed;
        reward.unlocked = old_reward.unlocked;

        reward
    }

    pub fn update_reward_in_state(&mut self, account_id: AccountId, reward: Reward) -> Reward {
        let default_reward = Reward::new(reward.campaign_id.clone());
        *self
            .rewards
            .entry(account_id)
            .or_default()
            .entry(reward.campaign_id.clone())
            .or_insert(default_reward) = reward.clone();
        reward
    }

    pub fn adjust_rewards_by_campaign_type(&mut self, campaign_type: CampaignType) {
        let campaigns = self
            .reward_campaigns
            .iter()
            .filter(|(_, campaign)| {
                campaign.campaign_type == campaign_type && campaign.end_time > env::block_height()
            })
            .map(|(campaign_id, _)| campaign_id)
            .collect::<Vec<String>>();

        campaigns.iter().for_each(|campaign_id| {
            self.adjust_reward(campaign_id.clone());
        });
    }

    pub fn adjust_reward(&mut self, campaign_id: String) -> Reward {
        let account_id = env::signer_account_id();
        let reward = self.update_reward(campaign_id, account_id.clone());
        self.update_reward_in_state(account_id, reward)
    }

    pub fn get_view_reward_state_for_user(
        &self,
        account_id: AccountId,
        campaign_id: String,
        reward: Reward,
    ) -> Reward {
        let accrued = self.get_accrued_rewards_per_token(campaign_id);
        Reward {
            campaign_id: reward.campaign_id.clone(),
            amount: self.get_updated_reward_amount_with_accrued(&reward, account_id, accrued.0),
            rewards_per_token_paid: reward.rewards_per_token_paid,
            claimed: reward.claimed,
            unlocked: reward.unlocked,
        }
    }

    pub fn get_reward_state(&self, account_id: AccountId, campaign_id: String) -> RewardState {
        let default_reward = Reward::new(campaign_id.clone());
        let reward = self
            .rewards
            .get(&account_id)
            .unwrap()
            .get(&campaign_id)
            .unwrap_or(&default_reward);

        let updated_reward =
            self.get_view_reward_state_for_user(account_id, campaign_id, reward.clone());

        let available_to_claim_amount = self.get_amount_available_to_claim(updated_reward.clone());
        let available_to_unlock_amount = updated_reward.amount.0 - available_to_claim_amount;
        RewardState {
            max_claim_amount: WBalance::from(available_to_claim_amount),
            max_unlock_amount: WBalance::from(available_to_unlock_amount),
        }
    }

    pub fn get_rewards_list(&self, account_id: AccountId) -> HashMap<String, Reward> {
        let default_map: HashMap<String, Reward> = HashMap::new();
        let account_rewards = self.rewards.get(&account_id).unwrap_or(&default_map);
        let mut view_rewards: HashMap<String, Reward> = HashMap::new();

        account_rewards.iter().for_each(|(campaign_id, reward)| {
            view_rewards.insert(
                campaign_id.clone(),
                self.get_view_reward_state_for_user(
                    account_id.clone(),
                    campaign_id.clone(),
                    reward.clone(),
                ),
            );
        });
        view_rewards
    }

    pub fn get_amount_available_to_claim(&self, reward: Reward) -> Balance {
        let mut result: Balance = 0;
        if let Some(campaign) = self.get_reward_campaign_by_id(reward.campaign_id.clone()) {
            if campaign.vesting.start_time > env::block_timestamp() {
                return result;
            };
            result = ((reward.amount.0 - reward.claimed.0)
                * Balance::from(env::block_timestamp() - campaign.vesting.start_time))
                / Balance::from(campaign.vesting.end_time - campaign.vesting.start_time);
        }
        result
    }

    pub fn claim_or_unlock_request(
        &mut self,
        account_id: AccountId,
        transfer_amount: WBalance,
        msg: String,
        token_address: AccountId,
        claimed_amount: WBalance,
        unlocked_amount: WBalance,
        reward: Reward,
    ) {
        underlying_token::ft_transfer(
            account_id.clone(),
            transfer_amount,
            Some(msg),
            token_address,
            ONE_YOCTO,
            self.terra_gas(10),
        )
        .then(ext_self::claim_reward_ft_transfer_callback(
            reward,
            account_id,
            claimed_amount,
            unlocked_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(5),
        ));
    }
}

#[near_bindgen]
impl Contract {
    pub fn add_reward_campaign(&mut self, reward_campaign: RewardCampaign) -> String {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );
        let campaign_id = self.request_unique_id();
        self.reward_campaigns.insert(&campaign_id, &reward_campaign);
        campaign_id
    }

    pub fn get_all_rewards_by_campaign_id(
        &self,
        campaign_id: String,
    ) -> HashMap<AccountId, Reward> {
        let mut result: HashMap<AccountId, Reward> = HashMap::new();
        self.rewards.iter().for_each(|(account_id, rewards)| {
            if let Some(reward) = rewards.get(&campaign_id) {
                result.insert(account_id.clone(), reward.clone());
            }
        });
        result
    }

    pub fn remove_rewards_entries_by_campaign_id(&mut self, campaign_id: String) {
        self.get_all_rewards_by_campaign_id(campaign_id)
            .iter()
            .for_each(|(account_id, reward)| {
                self.rewards
                    .get_mut(account_id)
                    .unwrap()
                    .remove(&reward.campaign_id);
            });
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
        self.remove_rewards_entries_by_campaign_id(campaign_id.clone());
        self.reward_campaigns.remove(&campaign_id);
    }

    pub fn claim_rewards(&mut self, campaign_id: String, amount: WBalance) {
        let account_id = env::signer_account_id();
        let reward = self.adjust_reward(campaign_id.clone());
        let available_amount = self.get_amount_available_to_claim(reward.clone());
        let campaign = self.get_reward_campaign_by_id(campaign_id).unwrap();
        assert!(
            env::block_timestamp() > campaign.vesting.start_time,
            "No rewards amount available to claim, because vesting is not started"
        );
        assert!(
            amount.0 <= available_amount,
            "There are not enough amount to claim. Possible amount is {}",
            available_amount
        );

        let message = format!("Claim reward with token_amount {}", amount.0);
        self.claim_or_unlock_request(
            account_id,
            amount,
            message,
            campaign.token,
            amount,
            WBalance::from(0),
            reward,
        )
    }

    pub fn unlock_rewards(&mut self, campaign_id: String, amount: WBalance) {
        let account_id = env::signer_account_id();
        let reward = self.adjust_reward(campaign_id.clone());
        let available_to_claim_amount = self.get_amount_available_to_claim(reward.clone());
        let available_to_unlock_amount = reward.amount.0 - available_to_claim_amount;
        let campaign = self.get_reward_campaign_by_id(campaign_id).unwrap();
        assert!(
            env::block_timestamp() > campaign.vesting.start_time,
            "No unlock amount available to claim, because vesting is not started"
        );
        assert!(
            amount.0 <= available_to_unlock_amount,
            "There are not enough amount to unlock. Possible amount is {}",
            available_to_unlock_amount
        );

        let amount_with_penalty =
            WBalance::from(BigBalance::from(amount) * campaign.vesting.penalty);
        let message = format!(
            "Unlock rewards with amount {} and amount_with_penalty {}",
            amount.0, amount_with_penalty.0
        );
        self.claim_or_unlock_request(
            account_id,
            amount_with_penalty,
            message,
            campaign.token,
            amount,
            amount_with_penalty,
            reward,
        )
    }

    #[private]
    pub fn claim_reward_ft_transfer_callback(
        &mut self,
        reward: Reward,
        account_id: AccountId,
        amount: WBalance,
        unlocked: WBalance,
    ) {
        assert!(is_promise_success(), "Claim operation wasn't successful");
        self.update_reward_in_state(
            account_id,
            Reward {
                campaign_id: reward.campaign_id,
                amount: reward.amount,
                rewards_per_token_paid: reward.rewards_per_token_paid,
                claimed: WBalance::from(reward.claimed.0 + amount.0),
                unlocked: WBalance::from(reward.unlocked.0 + unlocked.0),
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::rewards::{CampaignType, Vesting};
    use crate::{Config, Contract, Reward};
    use crate::{InterestRateModel, RewardCampaign};
    use general::ratio::Ratio;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{env, testing_env, AccountId, Balance, BlockHeight, VMContext};

    use general::WBalance;
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

    fn get_custom_context(
        is_view: bool,
        block_timestamp: u64,
        block_index: BlockHeight,
    ) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(AccountId::try_from(alice().to_string()).unwrap())
            .signer_account_id(AccountId::try_from(alice().to_string()).unwrap())
            .predecessor_account_id(AccountId::try_from(alice().to_string()).unwrap())
            .block_index(block_index)
            .block_timestamp(block_timestamp)
            .is_view(is_view)
            .build()
    }

    fn get_context(is_view: bool) -> VMContext {
        get_custom_context(is_view, 0, 100)
    }

    fn get_campaign() -> RewardCampaign {
        let vesting = Vesting {
            start_time: 1651362400,
            end_time: 1651372400,
            penalty: Ratio::from(5000u128),
        };
        let campaign = RewardCampaign {
            campaign_type: CampaignType::Supply,
            start_time: 1651352400,
            end_time: 1651362400,
            token: carol(),
            ticker_id: "CAROL".to_string(),
            reward_amount: U128(100000),
            last_update_time: 1651352400,
            rewards_per_token: U128(0),
            vesting,
        };
        campaign
    }

    #[test]
    fn test_get_amount_available_to_claim() {
        let mut contract = init_env();
        let mut campaign = get_campaign();
        campaign.campaign_type = CampaignType::Borrow;

        let context = get_custom_context(false, 1651362400, 1);
        testing_env!(context);
        let account_id = env::signer_account_id();
        let campaign_id = contract.add_reward_campaign(campaign);

        contract.increase_borrows(account_id, WBalance::from(1000));
        let reward = contract.adjust_reward(campaign_id);

        println!("{}", reward);
        let amount_available_to_claim = contract.get_amount_available_to_claim(reward.clone());
        assert_eq!(
            amount_available_to_claim, 0,
            "Amount for claim doesn't match to expected"
        );

        let context1 = get_custom_context(false, 1651367400, 1);
        testing_env!(context1);

        let amount_available_to_claim1 = contract.get_amount_available_to_claim(reward.clone());
        assert_eq!(
            amount_available_to_claim1,
            reward.amount.0 / 2,
            "Amount for claim doesn't match to expected"
        );

        let context2 = get_custom_context(false, 1651372400, 1);
        testing_env!(context2);
        let amount_available_to_claim2 = contract.get_amount_available_to_claim(reward.clone());
        assert_eq!(
            amount_available_to_claim2, reward.amount.0,
            "Amount for claim doesn't match to expected"
        );
    }

    #[test]
    fn test_get_rewards_list() {
        let mut contract = init_env();
        let mut campaign = get_campaign();
        campaign.campaign_type = CampaignType::Borrow;

        let context = get_custom_context(false, 1651357400, 1);
        testing_env!(context);
        let account_id = env::signer_account_id();
        let campaign_id = contract.add_reward_campaign(campaign);

        contract.increase_borrows(account_id.clone(), WBalance::from(1000));
        contract.adjust_reward(campaign_id.clone());

        let result1 = contract.get_rewards_list(account_id.clone());
        assert_eq!(
            result1.len(),
            1,
            "Rewards list length doesn't match to expected"
        );

        let context = get_custom_context(false, 1651362400, 1);
        testing_env!(context);
        let result2 = contract.get_rewards_list(account_id);
        assert_eq!(
            result2.len(),
            1,
            "Rewards list length doesn't match to expected"
        );
        assert_ne!(
            result1.get(&campaign_id).unwrap().amount,
            result2.get(&campaign_id).unwrap().amount,
            "Amounts are shouldn't be equal"
        );
        assert_eq!(
            result1.get(&campaign_id).unwrap().rewards_per_token_paid,
            result2.get(&campaign_id).unwrap().rewards_per_token_paid,
            "Rewards per token paid are should be similar"
        )
    }

    #[test]
    pub fn test_get_updated_reward_amount() {
        let account_id = AccountId::try_from(alice().to_string()).unwrap();
        let mut contract = init_env();
        let mut campaign = get_campaign();
        campaign.rewards_per_token = WBalance::from(10);
        let context = get_context(false);
        testing_env!(context);
        campaign.campaign_type = CampaignType::Borrow;
        let campaign_id = contract.add_reward_campaign(campaign);

        let reward = Reward::new(campaign_id);
        contract.increase_borrows(account_id.clone(), WBalance::from(100));

        let result = contract.get_updated_reward_amount(&reward, account_id);

        assert_eq!(
            result.0, 1000,
            "Reward amount doesn't match to expected value"
        );
    }

    #[test]
    fn test_get_accrued_rewards_with_no_total() {
        let mut contract = init_env();
        let campaign = get_campaign();
        let context = get_context(false);
        testing_env!(context);
        let campaign_id = contract.add_reward_campaign(campaign);
        let campaign_result = contract.get_accrued_rewards_per_token(campaign_id);

        assert_eq!(
            0, campaign_result.0,
            "Get accrued rewards should return 0 due to empty total supply"
        );
    }

    #[test]
    fn test_get_accrued_rewards_per_token() {
        let mut contract = init_env();
        let campaign = get_campaign();
        let context = get_custom_context(false, 1651362400, 1);
        let total_supply: Balance = 100;
        testing_env!(context);
        contract.mint(contract.get_signer_address(), WBalance::from(total_supply));
        contract.set_account_supplies(contract.get_signer_address(), WBalance::from(total_supply));

        let campaign_id = contract.add_reward_campaign(campaign.clone());
        let campaign_result = contract.get_accrued_rewards_per_token(campaign_id);

        assert_eq!(
            campaign.reward_amount.0 / total_supply,
            campaign_result.0,
            "Get accrued rewards value doesn't match"
        );
    }

    #[test]
    fn test_get_market_total() {
        let contract = init_env();
        let mut campaign = get_campaign();
        campaign.campaign_type = CampaignType::Supply;
        let supply_total = contract.get_market_total(campaign.clone());
        assert_eq!(
            supply_total.0,
            contract.get_total_supplies(),
            "Supplies total doesn't match"
        );

        campaign.campaign_type = CampaignType::Borrow;
        let borrow_total = contract.get_market_total(campaign);
        assert_eq!(
            borrow_total.0,
            contract.get_total_borrows(),
            "Supplies total doesn't match"
        );
    }

    #[test]
    fn test_get_rewards_per_second() {
        let contract = init_env();
        let campaign = get_campaign();
        let amount = contract.get_rewards_per_second(campaign);

        assert_eq!(10, amount.0, "Rewards per second doesn't match");
    }

    #[test]
    fn test_get_reward_tokens_per_day() {
        let contract = init_env();
        let campaign = get_campaign();
        let amount_per_second = contract.get_rewards_per_second(campaign.clone());
        let amount = contract.get_reward_tokens_per_day(campaign);
        assert_eq!(
            24 * 60 * 60 * amount_per_second.0,
            amount.0,
            "Rewards per day doesn't match"
        );
    }

    #[test]
    fn test_remove_reward_campaign() {
        let mut contract = init_env();
        let campaign = get_campaign();
        let context = get_context(false);
        testing_env!(context);

        let campaign_id = contract.add_reward_campaign(campaign);
        assert!(
            contract
                .get_reward_campaign_by_id(campaign_id.clone())
                .is_some(),
            "Campaign with id {} wasn't added",
            campaign_id
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
    fn test_get_reward_campaigns_extended() {
        let mut contract = init_env();
        let campaign = get_campaign();
        let context = get_context(false);
        testing_env!(context);
        contract.add_reward_campaign(campaign.clone());
        contract.add_reward_campaign(campaign.clone());
        contract.add_reward_campaign(campaign.clone());

        let campaign_list = contract.get_reward_campaigns_extended();

        assert_eq!(campaign_list.len(), 3, "Campaign list len doesn't match");

        let gotten_campaign = campaign_list.get(0).unwrap();

        assert_eq!(
            gotten_campaign.rewards_per_day.0,
            contract.get_reward_tokens_per_day(campaign.clone()).0,
            "Values rewards_per_day don't match"
        );
        assert_eq!(
            gotten_campaign.market_total.0,
            contract.get_market_total(campaign).0,
            "Values rewards_per_day don't match"
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
