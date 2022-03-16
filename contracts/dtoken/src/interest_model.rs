use crate::*;
use std::cmp::{min, max};

const MAX_RESERVE_FACTOR_VALUE: Ratio = 1 * RATIO_DECIMALS;

#[near_bindgen]
impl Contract {
    pub fn get_supply_rate(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance, reserve_factor: WBalance) -> Ratio {
        assert!(Balance::from(reserve_factor) <= MAX_RESERVE_FACTOR_VALUE, "Reserve factor should be less {}", MAX_RESERVE_FACTOR_VALUE);
        let rest_of_supply_factor = RATIO_DECIMALS - Balance::from(reserve_factor);
        let borrow_rate = self.get_borrow_rate(underlying_balance, total_borrows, total_reserves);
        let rate_to_pool = borrow_rate * rest_of_supply_factor / RATIO_DECIMALS;
        let util_rate = self.get_util(underlying_balance, total_borrows, total_reserves);
        util_rate * rate_to_pool / RATIO_DECIMALS
    }

    pub fn get_borrow_rate(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves: WBalance) -> Ratio {
        let util = self.get_util(underlying_balance, total_borrows, total_reserves);
        let kink = self.model.get_kink();
        let multiplier_per_block = self.model.get_multiplier_per_block();
        let base_rate_per_block =self.model.get_base_rate_per_block();
        let jump_multiplier_per_block = self.model.get_jump_multiplier_per_block();
        return min(util, kink) * multiplier_per_block / RATIO_DECIMALS + max(0, util as i128 - kink as i128) as Ratio * jump_multiplier_per_block / RATIO_DECIMALS + base_rate_per_block        
    }

    fn get_util(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance) -> Ratio {
        let sum_balance_borrows = Balance::from(underlying_balance).checked_add(Balance::from(total_borrows));
        assert!(sum_balance_borrows.is_some(), "Overflowing occurs while adding undelying balance and total borrows");
        let denominator = sum_balance_borrows.unwrap().checked_sub(Balance::from(total_reserves));
        assert!(denominator.is_some(), "Overflowing occurs while subtracting total reserves from sum of underlying balance and total borrows");
        assert_ne!(denominator.unwrap(), 0, "Cannot calculate utilization rate as denominator is equal 0");
        return Balance::from(total_borrows) * RATIO_DECIMALS / denominator.unwrap();
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    
    use crate::{Config, Contract};

    pub fn init_test_env() -> Contract {
        let (user_account, underlying_token_account, controller_account) = (alice(), bob(), carol());
    
        let contract = Contract::new(Config { 
            initial_exchange_rate: U128(10000), 
            underlying_token_id: underlying_token_account.clone() ,
            owner_id: user_account.clone(), 
            controller_account_id: controller_account.clone(), 
        });
    
        return contract;
    }

    #[test]
    fn test_get_util_rate(){
        let contract = init_test_env();
        assert_eq!(contract.get_util(U128(20), U128(180), U128(0)), 9000);
    }

    #[test]
    fn test_get_borrow_rate(){
        let mut contract = init_test_env();
        contract.model.set_base_rate_per_block(0);
        contract.model.set_multiplier_per_block(500);
        contract.model.set_kink(8000);
        contract.model.set_jump_multiplier_per_block(10900);
        assert_eq!(contract.get_borrow_rate(U128(20), U128(180), U128(0)), 1490);
    }

    #[test]
    fn test_get_supply_rate(){
        let mut contract = init_test_env();
        contract.model.set_base_rate_per_block(0);
        contract.model.set_multiplier_per_block(500);
        contract.model.set_kink(8000);
        contract.model.set_jump_multiplier_per_block(10900);
        contract.model.set_reserve_factor(700);
        assert_eq!(contract.get_supply_rate(U128(20), U128(180), U128(0), U128(contract.model.get_reserve_factor())), 1246);
    }

    
}