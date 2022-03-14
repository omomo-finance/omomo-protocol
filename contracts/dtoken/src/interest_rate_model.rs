use crate::*;
use near_sdk::env::block_height;

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    BorrowBlock,
    BorrowInterest,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct InterestRateModel {
    kink: Ratio,
    multiplier_per_block: Ratio,
    base_rate_per_block: Ratio,
    jump_multiplier_per_block: Ratio,
    reserve_factor: Ratio,
    account_borrow_block: LookupMap<AccountId, BlockHeight>,
    account_borrow_interest: LookupMap<AccountId, Ratio>
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

    pub fn get_borrow_block_by_user(&self, account: AccountId) -> BlockHeight{
        self.account_borrow_block.get(&account).unwrap_or(0)
    }

    pub fn get_borrow_interest_by_user(&self, account: AccountId) -> Ratio{
        self.account_borrow_interest.get(&account).unwrap_or(0)
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

    #[private]
    pub fn set_borrow_block_by_user(&mut self, account: AccountId, block_height: BlockHeight){
        self.account_borrow_block.insert(&account, &block_height);
    }

    #[private]
    pub fn set_borrow_interest_by_user(&mut self, account: AccountId, interest: Ratio){
        self.account_borrow_interest.insert(&account, &interest);
    }

    #[private]
    pub fn get_accrued_borrow_interest(&mut self, account: AccountId, borrow_rate: Ratio) -> Ratio {
        let current_block_height = block_height();
        if current_block_height == self.get_borrow_block_by_user(account.clone()){
            self.get_borrow_interest_by_user(account)
        } else {
            let accrued_rate = borrow_rate * (current_block_height - self.get_borrow_block_by_user(account.clone())) as u128;
            self.set_borrow_block_by_user(account.clone(), current_block_height);
            self.set_borrow_interest_by_user(account, accrued_rate);
            accrued_rate
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
            account_borrow_block: LookupMap::new(StorageKeys::BorrowBlock),
            account_borrow_interest: LookupMap::new(StorageKeys::BorrowInterest),
        }
    }
}