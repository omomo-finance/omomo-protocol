use crate::*;

use std::collections::HashMap;

impl Contract {
    pub fn get_prices_for_dtokens(&self, dtokens: Vec<AccountId>) -> HashMap<AccountId, Price> {
        let mut result = HashMap::new();
        for dtoken in dtokens {
            if let Some(price) = self.get_price(dtoken.clone()) {
                result.insert(dtoken, price);
            }
        }
        result
    }

    pub fn get_price(&self, dtoken_id: AccountId) -> Option<Price> {
        self.prices.get(&dtoken_id)
    }
}

#[near_bindgen]
impl Contract {
    // TODO Do we really need to expose this via near_bindgen
    #[private]
    pub fn upsert_price(&mut self, dtoken_id: AccountId, price: &Price) {
        self.prices.insert(&dtoken_id, price);
    }
}

#[cfg(test)]
mod tests {
    use assert_matches::assert_matches;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::AccountId;

    use crate::{Config, Contract};

    pub fn init_test_env() -> (Contract, AccountId, AccountId) {
        let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

        let eth_contract = Contract::new(Config {
            owner_id: owner_account,
            oracle_account_id: oracle_account,
        });

        let token_address: AccountId = "near".parse().unwrap();

        (eth_contract, token_address, user_account)
    }

    use super::*;

    #[test]
    fn test_add_get_price() {
        let (mut near_contract, token_address, _user_account) = init_test_env();

        let price = Price {
            // adding price of Near
            ticker_id: "wnear".to_string(),
            value: U128(20),
            volatility: U128(100),
            fraction_digits: 4,
        };

        near_contract.upsert_price(token_address.clone(), &price);

        let gotten_price = near_contract.get_price(token_address).unwrap();
        assert_matches!(
            &gotten_price,
            _price,
            "Get price format check has been failed"
        );
        assert_eq!(
            &gotten_price.value, &price.value,
            "Get price values check has been failed"
        );
        assert_eq!(
            &gotten_price.volatility, &price.volatility,
            "Get price volatility check has been failed"
        );
        assert_eq!(
            &gotten_price.ticker_id, &price.ticker_id,
            "Get price asset_id check has been failed"
        );
        assert_eq!(
            &gotten_price.fraction_digits, &price.fraction_digits,
            "Get fraction digits check has been failed"
        );
    }
}
