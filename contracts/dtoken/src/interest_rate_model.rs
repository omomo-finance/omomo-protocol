use crate::*;
use near_sdk::env::block_height;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct InterestRateModel {
    kink: Ratio,
    multiplier_per_block: Ratio,
    base_rate_per_block: Ratio,
    jump_multiplier_per_block: Ratio,
    reserve_factor: Ratio,
}

#[near_bindgen]
impl InterestRateModel {
    pub fn get_kink(&self) -> Ratio {
        self.kink
    }

    pub fn get_multiplier_per_block(&self) -> Ratio {
        self.multiplier_per_block
    }

    pub fn get_base_rate_per_block(&self) -> Ratio {
        self.base_rate_per_block
    }

    pub fn get_jump_multiplier_per_block(&self) -> Ratio {
        self.jump_multiplier_per_block
    }

    pub fn get_reserve_factor(&self) -> Ratio {
        self.reserve_factor
    }

    #[private]
    pub fn set_kink(&mut self, value: Ratio) {
        self.kink = value;
    }

    #[private]
    pub fn set_multiplier_per_block(&mut self, value: Ratio) {
        self.multiplier_per_block = value;
    }

    #[private]
    pub fn set_base_rate_per_block(&mut self, value: Ratio) {
        self.base_rate_per_block = value;
    }

    #[private]
    pub fn set_jump_multiplier_per_block(&mut self, value: Ratio) {
        self.jump_multiplier_per_block = value;
    }

    #[private]
    pub fn set_reserve_factor(&mut self, value: Ratio) {
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
            * borrow_rate
            * (current_block_height - accrued_interest.last_recalculation_block) as u128
            / RATIO_DECIMALS;

        AccruedInterest {
            accumulated_interest: accrued_interest.accumulated_interest + accrued_rate,
            last_recalculation_block: current_block_height,
        }
    }
}

fn get_with_ratio_decimals(value: f32) -> Ratio {
    (value * RATIO_DECIMALS as f32) as Ratio
}

impl Default for InterestRateModel {
    fn default() -> Self {
        Self {
            kink: get_with_ratio_decimals(1.0),
            base_rate_per_block: get_with_ratio_decimals(1.0),
            multiplier_per_block: get_with_ratio_decimals(1.0),
            jump_multiplier_per_block: get_with_ratio_decimals(1.0),
            reserve_factor: get_with_ratio_decimals(0.05),
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
