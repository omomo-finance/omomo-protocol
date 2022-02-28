use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_supply_rate(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance, reserve_factor: u128) -> Balance {
        // If reserve factor = 1 * 10^4 , after next operation rest_of_supply_factor = 0, and as a result all rest operations will be equal 0,
        // and token = dtoken/supply_rate will get a mistake cannot divide on 0
        let rest_of_supply_factor = RATIO_DECIMALS - reserve_factor;

        let borrow_rate = self.get_borrow_rate(underlying_balance, total_borrows, total_reserves);
        let rate_to_pool = borrow_rate * rest_of_supply_factor / RATIO_DECIMALS;
        let util_rate = self.get_util(underlying_balance, total_borrows, total_reserves);

        util_rate * rate_to_pool / RATIO_DECIMALS
    }

    pub fn get_borrow_rate(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance)-> Balance{
        let util = self.get_util(underlying_balance, total_borrows, total_reserves);
        let kink = self.model.get_kink();
        let multiplier_per_block = self.model.get_multiplier_per_block();
        let base_rate_per_block =self.model.get_base_rate_per_block();
        let jump_multiplier_per_block = self.model.get_jump_multiplier_per_block();

        if util <= kink{
            return util*multiplier_per_block / RATIO_DECIMALS + base_rate_per_block;
        }
        else{
            let normal_rate = kink * multiplier_per_block / RATIO_DECIMALS + base_rate_per_block;
            let excess_util = util - kink;
            return excess_util * jump_multiplier_per_block/ RATIO_DECIMALS + normal_rate;
        }
    }

    fn get_util(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance)-> Balance{
        return Balance::from(total_borrows) * RATIO_DECIMALS / (Balance::from(underlying_balance) + Balance::from(total_borrows) - Balance::from(total_reserves));
    }
}