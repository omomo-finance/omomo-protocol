use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Debug, Serialize, PartialEq, Clone, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum VestingPlans {
    None,
    Linear,
}

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct RewardSetting {
    /// Token address
    pub token: AccountId,

    /// Rewards token count per day
    pub reward_per_day: Balance,

    /// Lock block count      
    pub lock_time: u128,

    /// Percents of the locked tokens will be confiscated in case of an urgent claim. Possible values [0 .. 1] * 10^4
    /// When it's equal 1 === Unable for urgent unlock
    pub penalty: Ratio,

    /// Vesting plan type
    pub vesting: VestingPlans,
}
