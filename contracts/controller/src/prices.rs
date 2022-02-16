use crate::*;

impl Contract {

    pub fn get_prices_for_assets(&self, assets: Vec<AccountId>) -> LookupMap<AccountId, Balance> {
        let mut result = LookupMap::new(b"t".to_vec());
        for asset in assets {
            if self.prices.contains_key(&asset) {
                let price = self.prices.get(&asset).unwrap();
                result.insert(&price.asset_id, &price.value);
            }
        }
        return  result;
    }

    pub fn get_price(&self, asset_id: AccountId) -> Option<Price> {
        return self.prices.get(&asset_id);
    }

}