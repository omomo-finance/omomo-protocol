use crate::*;
use general::ratio::{Ratio, RATIO_DECIMALS};
use near_sdk::env::block_height;
use std::fmt;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct InterestRateModel {
    pub kink: WRatio,
    pub multiplier_per_block: WRatio,
    pub base_rate_per_block: WRatio,
    pub jump_multiplier_per_block: WRatio,
    pub reserve_factor: WRatio,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct RepayInfo {
    pub accrued_interest_per_block: WBalance,
    pub total_amount: WBalance,
    pub borrow_amount: WBalance,
    pub accumulated_interest: WBalance,
}

impl fmt::Display for RepayInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Debug, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct WithdrawInfo {
    pub exchange_rate: Ratio,
    pub total_interest: Balance,
}

impl fmt::Display for WithdrawInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl InterestRateModel {
    pub fn get_kink(&self) -> Ratio {
        Ratio(self.kink.0)
    }

    pub fn get_multiplier_per_block(&self) -> Ratio {
        Ratio(self.multiplier_per_block.0)
    }

    pub fn get_base_rate_per_block(&self) -> Ratio {
        Ratio(self.base_rate_per_block.0)
    }

    pub fn get_jump_multiplier_per_block(&self) -> Ratio {
        Ratio(self.jump_multiplier_per_block.0)
    }

    pub fn get_reserve_factor(&self) -> Ratio {
        Ratio(self.reserve_factor.0)
    }

    pub fn set_kink(&mut self, value: WRatio) {
        self.kink = value;
    }

    pub fn set_multiplier_per_block(&mut self, value: WRatio) {
        self.multiplier_per_block = value;
    }

    pub fn set_base_rate_per_block(&mut self, value: WRatio) {
        self.base_rate_per_block = value;
    }

    pub fn set_jump_multiplier_per_block(&mut self, value: WRatio) {
        self.jump_multiplier_per_block = value;
    }

    pub fn set_reserve_factor(&mut self, value: WRatio) {
        self.reserve_factor = value;
    }

    pub fn calculate_accrued_interest(
        &self,
        borrow_rate: Ratio,
        total_borrow: Balance,
        accrued_interest: AccruedInterest,
    ) -> AccruedInterest {
        let current_block_height = block_height();
        let accrued_rate = total_borrow
            * borrow_rate.0
            * (current_block_height - accrued_interest.last_recalculation_block) as u128
            / RATIO_DECIMALS.0;

        AccruedInterest {
            accumulated_interest: accrued_interest.accumulated_interest + accrued_rate,
            last_recalculation_block: current_block_height,
        }
    }
}

impl Default for InterestRateModel {
    fn default() -> Self {
        Self {
            kink: WRatio::from(RATIO_DECIMALS.0),
            base_rate_per_block: WRatio::from(RATIO_DECIMALS.0),
            multiplier_per_block: WRatio::from(RATIO_DECIMALS.0),
            jump_multiplier_per_block: WRatio::from(RATIO_DECIMALS.0),
            reserve_factor: WRatio::from(500),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn get_accrued_borrow_interest(&self, account: AccountId) -> AccruedInterest {
        self.user_profiles
            .get(&account)
            .unwrap_or_default()
            .borrow_interest
    }

    pub fn get_accrued_supply_interest(&self, account: AccountId) -> AccruedInterest {
        self.user_profiles
            .get(&account)
            .unwrap_or_default()
            .supply_interest
    }

    #[private]
    pub fn set_accrued_supply_interest(
        &mut self,
        account: AccountId,
        accrued_interest: AccruedInterest,
    ) {
        let mut user = self.user_profiles.get(&account).unwrap_or_default();
        user.supply_interest = accrued_interest;
        self.user_profiles.insert(&account, &user);
    }

    #[private]
    pub fn set_accrued_borrow_interest(
        &mut self,
        account: AccountId,
        accrued_interest: AccruedInterest,
    ) {
        let mut user = self.user_profiles.get(&account).unwrap_or_default();
        user.borrow_interest = accrued_interest;
        self.user_profiles.insert(&account, &user);
    }
}