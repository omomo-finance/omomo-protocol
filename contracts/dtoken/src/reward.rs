use crate::*;
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
    pub fn get_user_rewards(&self, account_id: AccountId) -> Vec<Reward> {
        self.rewards
            .get(&account_id)
            .expect("This user doesn`t have rewards")
    }
}

#[near_bindgen]
impl Contract {
    pub fn adjust_reward(&mut self, account_id: AccountId, reward: Reward) {
        if self.rewards.get(&account_id).is_none() {
            self.rewards.insert(&account_id, &[reward].to_vec());
        } else {
            let mut user_rewards = self.rewards.get(&account_id).unwrap();
            user_rewards.push(reward);
        }
    }

    pub fn remove_reward(&mut self, account_id: AccountId, reward_id: String) {
        let mut user_rewards = self
            .rewards
            .get(&account_id)
            .expect("This user doesn`t have rewards");
        let reward_index = user_rewards
            .iter()
            .position(|x| *x.id == reward_id)
            .unwrap();
        user_rewards.remove(reward_index);
    }

    pub fn claim_reward(&mut self, account_id: AccountId, reward_id: String) -> Promise {
        assert!(
            self.rewards.get(&account_id).is_none(),
            "This user has no rewards"
        );
        let rewards = self.rewards.get(&account_id).unwrap();
        assert_eq!(rewards.len(), 0, "This user has no rewards");
        let reward_index = rewards.iter().position(|x| *x.id == reward_id).unwrap();
        let reward = rewards[reward_index].clone();

        underlying_token::ft_transfer(
            account_id.clone(),
            reward.amount,
            Some(format!(
                "Borrow with token_amount {}",
                Balance::from(reward.amount)
            )),
            reward.token.clone(),
            ONE_YOCTO,
            self.terra_gas(10),
        )
        .then(ext_self::reward_ft_transfer_callback(
            reward_index,
            account_id,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(5),
        ))
    }

    pub fn reward_ft_transfer_callback(&mut self, reward_index: usize, account_id: AccountId) {
        if !is_promise_success() {
            log!(format!(
                "There is an error transferring token to account {}",
                account_id
            ));
        }
        self.rewards.get(&account_id).unwrap().remove(reward_index);
        if self.rewards.len() == 0 {
            self.rewards.remove(&account_id);
        }
    }
}
