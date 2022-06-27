use crate::*;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct AccruedInterest {
    pub last_recalculation_block: BlockHeight,
    pub accumulated_interest: Balance,
}

// Cannot derive Default as `last_recalculation_block` by default should be current block
impl Default for AccruedInterest {
    fn default() -> Self {
        AccruedInterest {
            last_recalculation_block: env::block_height(),
            accumulated_interest: 0,
        }
    }
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
