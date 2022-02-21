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
    use near_sdk::AccountId;
    use near_sdk::test_utils::test_env::{alice, bob, carol};

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