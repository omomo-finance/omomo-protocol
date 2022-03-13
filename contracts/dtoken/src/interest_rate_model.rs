use crate::*;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct InterestRateModel {
    kink: Ratio,
    multiplier_per_block: Ratio,
    base_rate_per_block: Ratio,
    jump_multiplier_per_block: Ratio,
    reserve_factor: Ratio
}

#[near_bindgen]
impl InterestRateModel{
    pub fn get_kink(&self) -> Ratio{
        return self.kink;
    }

    pub fn get_multiplier_per_block(&self) -> Ratio{
        return self.multiplier_per_block;
    }

    pub fn get_base_rate_per_block(&self) -> Ratio{
        return self.base_rate_per_block;
    }

    pub fn get_jump_multiplier_per_block(&self) -> Ratio{
        return self.jump_multiplier_per_block;
    }

    pub fn get_reserve_factor(&self) -> Ratio{
        return self.reserve_factor;
    }

    #[private]
    pub fn set_kink(&mut self, value: Ratio){
        self.kink = value;
    }

    #[private]
    pub fn set_multiplier_per_block(&mut self, value: Ratio){
        self.multiplier_per_block = value;
    }

    #[private]
    pub fn set_base_rate_per_block(&mut self, value: Ratio){
        self.base_rate_per_block = value;
    }

    #[private]
    pub fn set_jump_multiplier_per_block(&mut self, value: Ratio){
        self.jump_multiplier_per_block = value;
    }

    #[private]
    pub fn set_reserve_factor(&mut self, value: Ratio){
        self.reserve_factor = value;
    }
}

#[near_bindgen]
impl InterestRateModel{
    pub fn get_with_ratio_decimals(value: f32) -> Ratio{
        return (value * RATIO_DECIMALS as f32) as Ratio;
    }
}

impl Default for InterestRateModel{
    fn default()-> Self{
        Self{
            kink: InterestRateModel::get_with_ratio_decimals(1.0),
            base_rate_per_block: InterestRateModel::get_with_ratio_decimals(1.0),
            multiplier_per_block: InterestRateModel::get_with_ratio_decimals(1.0),
            jump_multiplier_per_block: InterestRateModel::get_with_ratio_decimals(1.0),
            reserve_factor: InterestRateModel::get_with_ratio_decimals(0.05),
        }
    }
}