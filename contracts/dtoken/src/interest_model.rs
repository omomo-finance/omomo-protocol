use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_borrow_rate(&self) -> Balance {
        1
    }

    pub fn get_supply_rate(&self) -> Balance {
        1
    }

    pub fn get_borrow_rate_test(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance)-> Balance{

        1
    }

    fn get_util(&self, underlying_balance: WBalance, total_borrows: WBalance, total_reserves:WBalance)-> Balance{
        return total_borrows * RAT
    }

    
}