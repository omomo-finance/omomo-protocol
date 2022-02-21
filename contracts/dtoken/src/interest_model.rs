use crate::*;

#[near_bindgen]
impl Contract {
    pub fn get_borrow_rate(&self) -> Balance {
        1
    }

    pub fn get_supply_rate(&self) -> Balance {
        1
    }

    
}