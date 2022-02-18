use crate::*;

impl Contract {
    pub fn get_borrow_rate(&self, underlying_balance: WBalance) -> Balance {
        1
    }

    pub fn get_supply_rate(&self, underlying_balance: WBalance) -> Balance {
        1
    }
}