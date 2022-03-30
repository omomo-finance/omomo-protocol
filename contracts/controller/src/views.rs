use crate::*;

use std::collections::HashMap;
use crate::admin::Market;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct AccountData {
    pub account_id: AccountId,
    pub total_borrows: Balance,
    pub total_supplies: Balance,
    pub blocked: bool,
    pub health_factor: Ratio
}

impl Default for AccountData {
    fn default() -> Self {
        AccountData {
            account_id: AccountId::new_unchecked("".to_string()),
            total_borrows: 0,
            total_supplies: 0,
            blocked: false,
            health_factor: 1 * RATIO_DECIMALS,
        }
    }
}


#[near_bindgen]
impl Contract {

    pub fn view_total_borrows(&self, user_id: AccountId) -> Balance {
        return self.get_total_borrows(user_id).into();
    }

    pub fn view_total_supplies(&self, user_id: AccountId) -> Balance {
        return self.get_total_supplies(user_id).into();
    }

    pub fn view_markets(&self) -> Vec<Market>  {
        return self.get_markets_list();
    }

    pub fn view_accounts(&self, user_ids: Vec<AccountId>) -> Vec<AccountData> {
        return user_ids.iter()
        .filter(|user_id| { return self.user_profiles.get(user_id).is_some() })
        // .filter(|_user_id| { return true })
        .map(|user_id| {
            let total_borrows = self.get_total_borrows(user_id.clone());
            let total_supplies = self.get_total_supplies(user_id.clone());
            let health_factor = self.get_health_factor(user_id.clone());
            return AccountData {
                account_id: user_id.clone(),
                total_borrows: total_borrows.into(),
                total_supplies: total_supplies.into(),
                blocked: false,
                health_factor
            }
        }).collect::<Vec<AccountData>>();
    }

    pub fn view_prices(&self, dtokens: Vec<AccountId>) -> HashMap<AccountId, Price> {
        return self.get_prices_for_dtokens(dtokens);
    }

}


#[cfg(test)]
mod tests {
    use near_sdk::{AccountId, testing_env};
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use general::Price;
    use crate::{Config, Contract, OraclePriceHandlerHook, PriceJsonList};
    use crate::ActionType::Supply;

    pub fn init_test_env() -> (Contract, AccountId, AccountId) {
        let (owner_account, _oracle_account, user_account) = (alice(), bob(), carol());

        let mut controller_contract = Contract::new(Config {
            owner_id: owner_account.clone(),
            oracle_account_id:  owner_account.clone(),
        });

        let context = VMContextBuilder::new()
            .signer_account_id(owner_account.clone())
            .predecessor_account_id(owner_account.clone())
            .build();

        testing_env!(context);

        let ticker_id_1 = "weth".to_string();
        let asset_id_1 = AccountId::new_unchecked("token.weth".to_string());
        let dtoken_id_1 = AccountId::new_unchecked("dtoken.weth".to_string());

        let ticker_id_2 = "wnear".to_string();
        let asset_id_2 = AccountId::new_unchecked("token.wnear".to_string());
        let dtoken_id_2 = AccountId::new_unchecked("dtoken.wnear".to_string());

        controller_contract.add_market(asset_id_1.clone(), dtoken_id_1.clone(), ticker_id_1.clone());
        controller_contract.add_market(asset_id_2.clone(), dtoken_id_2.clone(), ticker_id_2.clone());


        let token_address: AccountId = "dtoken.wnear".parse().unwrap();

        let mut prices: Vec<Price> = Vec::new();
        prices.push(Price {
            ticker_id: ticker_id_2.clone(),
            value: U128(20000),
            volatility: U128(80),
            fraction_digits: 4
        });
        prices.push(Price {
            ticker_id: ticker_id_1.clone(),
            value: U128(20000),
            volatility: U128(100),
            fraction_digits: 4
        });

        controller_contract.oracle_on_data(PriceJsonList {
            block_height: 83456999,
            price_list: prices,
        });

        return (controller_contract, token_address, user_account);
    }


    #[test]
    fn test_view_markets() {
        let (near_contract, _, _) = init_test_env();

        let ticker_id_1 = "weth".to_string();
        let asset_id_1 = AccountId::new_unchecked("token.weth".to_string());
        let dtoken_id_1 = AccountId::new_unchecked("dtoken.weth".to_string());

        let ticker_id_2 = "wnear".to_string();
        let asset_id_2 = AccountId::new_unchecked("token.wnear".to_string());
        let dtoken_id_2 = AccountId::new_unchecked("dtoken.wnear".to_string());


        let accounts = near_contract.view_markets();

        assert_eq!(accounts.len(), 2, "View market response doesn't match");
        assert_eq!(accounts[0].asset_id, asset_id_1, "View market AssetId check has been failed");
        assert_eq!(accounts[0].dtoken, dtoken_id_1, "View market Dtoken check has been failed");
        assert_eq!(accounts[0].ticker_id, ticker_id_1, "View market Ticker check has been failed");
        assert_eq!(accounts[1].asset_id, asset_id_2, "View market AssetId check has been failed");
        assert_eq!(accounts[1].dtoken, dtoken_id_2, "View market Dtoken check has been failed");
        assert_eq!(accounts[1].ticker_id, ticker_id_2, "View market Ticker check has been failed");

    }

    #[test]
    fn test_view_accounts() {
        let (mut near_contract, token_address, _) = init_test_env();
        let mut accounts = Vec::new();

        accounts.push(alice());
        accounts.push(bob());

        near_contract.set_entity_by_token(Supply, accounts[0].clone(), token_address.clone(), 100);
        let result = near_contract.view_accounts(accounts);

        assert_eq!(result.len(), 1, "View accounts response doesn't match");
        assert_eq!(result[0].account_id, alice(), "View accounts account_id check has been failed");
        assert_eq!(result[0].total_borrows,0, "View accounts total borrows check has been failed");
        assert_eq!(result[0].total_supplies,100*20000, "View accounts total supplies check has been failed");
        assert_eq!(result[0].health_factor,10000, "View accounts health factor check has been failed");
    }
}