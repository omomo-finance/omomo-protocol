use crate::*;

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AccruedInterest {
    pub last_recalculation_block: BlockHeight,
    pub accumulated_interest: Balance,
}

#[derive(Default, BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UserProfile {
    pub borrows: Balance,
    pub supplies: Balance,

    pub borrow_interest: AccruedInterest,
    pub supply_interest: AccruedInterest,

    pub is_consistent: bool,
}
