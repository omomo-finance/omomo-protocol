use crate::*;
use std::cmp::{min, max};

#[near_bindgen]
impl Contract {
    pub fn get_supply_rate(&self) -> Ratio {
        1
    }

    pub fn get_borrow_rate(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves: WBalance) -> Ratio{

        let util = self.get_util(underlying_balance, total_borrows, total_reserves);
        let kink = self.model.get_kink();
        let multiplier_per_block = self.model.get_multiplier_per_block();
        let base_rate_per_block =self.model.get_base_rate_per_block();
        let jump_multiplier_per_block = self.model.get_jump_multiplier_per_block();
    
        return min(util, kink) * multiplier_per_block / RATIO_DECIMALS + max(0, util as i128 - kink as i128) as Ratio * jump_multiplier_per_block / RATIO_DECIMALS + base_rate_per_block        
    }

    fn get_util(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance) -> Ratio{
        return Balance::from(total_borrows) * RATIO_DECIMALS / (Balance::from(underlying_balance) + Balance::from(total_borrows) - Balance::from(total_reserves));
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::AccountId;
    use crate::{Config, Contract};

    pub fn init_test_env() -> (Contract, AccountId, AccountId, AccountId) {
        let (user_account, underlying_token_account, controller_account) = (alice(), bob(), carol());
    
        let contract = Contract::new(Config { 
            initial_exchange_rate: U128(1), 
            underlying_token_id: underlying_token_account.clone() ,
            owner_id: user_account.clone(), 
            controller_account_id: controller_account.clone(), 
        });
    
        return (contract, underlying_token_account, controller_account, user_account);
    }

    #[test]
    fn test_get_util_rate(){
        let (contract, underlying_account, controller_account, user_account) = init_test_env();
        assert_eq!(contract.get_util(U128(20), U128(180), U128(0)), 9000);
    }

    #[test]
    fn test_get_borrow_rate(){
        let (mut contract, underlying_account, controller_account, user_account) = init_test_env();
        contract.model.set_base_rate_per_block(0);
        contract.model.set_multiplier_per_block(500);
        contract.model.set_kink(8000);
        contract.model.set_jump_multiplier_per_block(10900);
        assert_eq!(contract.get_borrow_rate(U128(20), U128(180), U128(0)), 1490);
    }

    
}