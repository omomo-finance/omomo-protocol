use crate::*;
use near_sdk::env::block_height;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    SupplyBlock,
    SupplyInterest,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct InterestRateModel {
    kink: Ratio,
    multiplier_per_block: Ratio,
    base_rate_per_block: Ratio,
    jump_multiplier_per_block: Ratio,
    reserve_factor: Ratio,
    account_supply_block: LookupMap<AccountId, BlockHeight>,
    account_supply_interest: LookupMap<AccountId, Balance>
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

    pub fn get_supply_block_by_user(&self, account: AccountId) -> BlockHeight{
        self.account_supply_block.get(&account).unwrap_or(0)
    }

    pub fn get_supply_interest_by_user(&self, account: AccountId) -> Balance{
        self.account_supply_interest.get(&account).unwrap_or(0)
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

    fn set_supply_block_by_user(&mut self, account: AccountId, block_height: BlockHeight){
        self.account_supply_block.insert(&account, &block_height);
    }

    fn set_supply_interest_by_user(&mut self, account: AccountId, interest: Ratio){
        self.account_supply_interest.insert(&account, &interest);
    }

    pub fn calculate_interest_on_supply(&mut self, account: AccountId, supply_rate: Ratio, total_supply: Balance) {
        let current_block_height = block_height();
        if self.get_supply_block_by_user(account.clone()) == 0 {
            self.set_supply_block_by_user(account, current_block_height);
        } else {
            let accrued_rate = total_supply * supply_rate * (current_block_height - self.get_supply_block_by_user(account.clone())) as u128 / RATIO_DECIMALS;
            self.set_supply_block_by_user(account.clone(), current_block_height);
            self.set_supply_interest_by_user(account, accrued_rate);
        }
    }

    pub fn calculate_interest_on_withdraw(&mut self, account: AccountId, supply_rate: Ratio, total_supply: Balance) -> Balance {
        let current_block_height = block_height();
        if current_block_height == self.get_supply_block_by_user(account.clone()) {
            self.get_supply_interest_by_user(account)
        } else {
            let accrued_rate = total_supply * supply_rate * (current_block_height - self.get_supply_block_by_user(account.clone())) as u128 / RATIO_DECIMALS;
            let total_accrued_rate = self.get_supply_interest_by_user(account.clone()) + accrued_rate;
            self.set_supply_block_by_user(account.clone(), current_block_height);
            self.set_supply_interest_by_user(account, 0);
            total_accrued_rate
        }
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
            account_supply_block: LookupMap::new(StorageKeys::SupplyBlock),
            account_supply_interest: LookupMap::new(StorageKeys::SupplyInterest),
        }
    }
}