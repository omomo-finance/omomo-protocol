use crate::*;

impl Contract {

    pub fn get_price(&self, asset_id: AccountId) -> Option<u128> {
        return self.prices.get(&asset_id);
    }

}