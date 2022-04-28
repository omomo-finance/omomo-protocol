use crate::*;
use near_sdk::env::block_height;
use near_sdk::Promise;

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, PartialEq, Clone, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VestingPlans {
    None,
    Linear,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, PartialEq, Clone, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum RewardPeriod {
    Day,
    Week,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Reward {
    /// Unique id
    pub id: String,

    /// Token address
    pub token: AccountId,

    /// Reward token amount === RewardSetting.reward_per_day * (user_staker / total_stake) * (staked_blocks / blocks_per_day)
    pub amount: WBalance,

    /// BlockHeight when lock will be released
    pub locked_till: BlockHeight,

    /// RewardSetting.penalty
    pub penalty: Ratio,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardSetting {
    /// Token address
    pub token: AccountId,

    /// Rewards token count per day
    pub reward_per_period: RewardAmount,

    /// Lock block count      
    pub lock_time: BlockHeight,

    /// Percents of the locked tokens will be confiscated in case of an urgent claim. Possible values [0 .. 1] * 10^4
    /// When it's equal 1 === Unable for urgent unlock
    pub penalty: Ratio,

    /// Vesting plan type
    pub vesting: VestingPlans,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardAmount {
    pub period: RewardPeriod,
    pub amount: WBalance,
}

impl Contract {
    pub fn get_user_rewards(&self, account_id: AccountId) -> HashMap<String, Reward> {
        self.rewards
            .get(&account_id)
            .expect("This user doesn`t have rewards")
    }
}

#[near_bindgen]
impl Contract {
    pub fn adjust_reward(&mut self, account_id: AccountId, reward_id: String, reward: Reward) {
        if self.rewards.get(&account_id).is_none() {
            let mut new_reward = HashMap::new();
            new_reward.insert(reward_id, reward);
            self.rewards.insert(&account_id, &new_reward);
        } else {
            let mut user_rewards = self.rewards.get(&account_id).unwrap();
            user_rewards.insert(reward_id, reward);
        }
    }

    pub fn remove_reward(&mut self, account_id: AccountId, reward_id: String) {
        let mut user_rewards = self
            .rewards
            .get(&account_id)
            .expect("This user doesn`t have rewards");
        user_rewards.remove(&reward_id);
    }

    pub fn claim_reward(&mut self, reward_id: String) -> Promise {
        let account_id = env::signer_account_id();
        let rewards = self.rewards.get(&account_id).unwrap();
        assert!(rewards.is_empty(), "This user has no rewards");
        let reward = rewards.get(&reward_id).unwrap();
        assert!(
            reward.locked_till >= block_height(),
            "The reward is currently locked"
        );

        underlying_token::ft_transfer(
            account_id.clone(),
            reward.amount,
            Some(format!(
                "Claim reward with token_amount {}",
                Balance::from(reward.amount)
            )),
            reward.token.clone(),
            ONE_YOCTO,
            self.terra_gas(10),
        )
        .then(ext_self::reward_ft_transfer_callback(
            reward_id,
            account_id,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(5),
        ))
    }

    pub fn reward_ft_transfer_callback(&mut self, reward_id: String, account_id: AccountId) {
        if !is_promise_success() {
            log!(format!(
                "There is an error transferring token to account {}",
                account_id
            ));
        }
        self.rewards.get(&account_id).unwrap().remove(&reward_id);
        if self.rewards.get(&account_id).unwrap().is_empty() {
            self.rewards.remove(&account_id);
        }
    }
}
