use crate::*;

impl Contract {
    pub fn get_prices_for_assets(&self, assets: Vec<AccountId>) -> LookupMap<AccountId, Balance> {
        let mut result = LookupMap::new(b"t".to_vec());
        for asset in assets {
            if self.prices.contains_key(&asset) {
                let price = self.get_price(asset).unwrap();
                result.insert(&price.asset_id, &price.value);
            }
        }
        return result;
    }

    pub fn get_price(&self, asset_id: AccountId) -> Option<Price> {
        return self.prices.get(&asset_id);
    }

    pub fn upsert_price(&mut self, price: &Price) {
        // Update & insert operation
        self.prices.insert(&price.asset_id, &price);
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::init_test_env;

    use super::*;

    #[test]
    fn test_add_get_price() {
        let (mut near_contract, token_address, user_account) = init_test_env();

        let price_1 = Price {
            // adding price of Near
            asset_id: token_address.clone(),
            value: 20,
        };

        let price_2 = Price {
            // adding price of Ether
            asset_id: "eth".parse().unwrap(),
            value: 3000,
        };

        near_contract.upsert_price(&price_1);
        near_contract.upsert_price(&price_2);
        dbg!(near_contract.get_price(token_address));
        dbg!(near_contract.get_price("eth".parse().unwrap()));
    }
}