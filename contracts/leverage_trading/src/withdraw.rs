use crate::*;

#[near_bindgen]
impl Contract {
    //Method with mock data. Returns the incoming amount of tokens
    pub fn withdraw(&mut self, token: AccountId, amount: U128) -> U128 {
        amount
    }
}
