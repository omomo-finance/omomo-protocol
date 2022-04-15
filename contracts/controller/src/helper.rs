use crate::*;

#[near_bindgen]
impl Contract {
    pub fn min(f1: u128, f2: u128) -> u128 {
        if f1 < f2 {
            f1
        } else {
            f2
        }
    }

    pub fn max(f1: u128, f2: u128) -> u128 {
        if f1 > f2 {
            f1
        } else {
            f2
        }
    }
}
