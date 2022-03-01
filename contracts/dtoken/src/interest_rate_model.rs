use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Default)]
pub struct InterestRateModel {

    pub kink: u128,

    pub multiplier_per_block: u128,

    pub base_rate_per_block: u128,

    pub jump_multiplier_per_block: u128,

    pub reserve_factor: u128
}

#[near_bindgen]
impl InterestRateModel{
    pub fn get_kink(&self) -> u128{
        return self.kink;
    }

    pub fn get_multiplier_per_block(&self) -> u128{
        return self.multiplier_per_block;
    }

    pub fn get_base_rate_per_block(&self) -> u128{
        return self.base_rate_per_block;
    }

    pub fn get_jump_multiplier_per_block(&self) -> u128{
        return self.jump_multiplier_per_block;
    }

    pub fn get_reserve_factor(&self) -> u128{
        return self.reserve_factor;
    }
}

#[near_bindgen]
impl InterestRateModel{
    pub fn get_with_ratio_decimals(value: f32) -> u128{
        return (value * RATIO_DECIMALS as f32) as u128;
    }
}