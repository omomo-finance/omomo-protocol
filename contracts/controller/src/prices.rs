use crate::*;

use std::collections::HashMap;

impl Contract {

    pub fn get_prices_for_assets(&self, assets: Vec<AccountId>) -> HashMap<AccountId, Price> {
        let mut result = HashMap::new();
        for asset in assets {
            if self.prices.contains_key(&asset) {
                let price_raw = self.get_price(asset);
                if(price_raw.is_some()){
                    let price = price_raw.unwrap();
                    result.insert(price.asset_id.clone(), price);
                }

            }
        }
        return result;
    }

    pub fn get_price(&self, asset_id: AccountId) -> Option<Price> {
        return self.prices.get(&asset_id);
    }

}

#[near_bindgen]
impl Contract {

    #[private]
    pub fn upsert_price(&mut self, price: &Price) {
        // Update & insert operation
        self.prices.insert(&price.asset_id, &price);
    }

}

#[cfg(test)]
mod tests {

    use near_sdk::AccountId;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use assert_matches::assert_matches;

    use crate::{Config, Contract};

    pub fn init_test_env() -> (Contract, AccountId, AccountId) {
        let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

        let eth_contract = Contract::new(Config { owner_id: owner_account, oracle_account_id: oracle_account });

        let token_address: AccountId = "near".parse().unwrap();

        return (eth_contract, token_address, user_account);
    }

    use super::*;

    #[test]
    fn test_add_get_price() {
        let (mut near_contract, token_address, _user_account) = init_test_env();

        let price = Price {
            // adding price of Near
            asset_id: token_address.clone(),
            value: U128(20),
            volatility: U128(100),
            fraction_digits: 4
        };

        near_contract.upsert_price(&price);

        let gotten_price = near_contract.get_price(token_address).unwrap();
        assert_matches!(&gotten_price, _price, "Get price format check has been failed");
        assert_eq!(&gotten_price.value, &price.value, "Get price values check has been failed");
        assert_eq!(&gotten_price.volatility, &price.volatility,  "Get price volatility check has been failed");
        assert_eq!(&gotten_price.asset_id, &price.asset_id, "Get price asset_id check has been failed");
        assert_eq!(&gotten_price.fraction_digits, &price.fraction_digits, "Get fraction digits check has been failed");
    }
}