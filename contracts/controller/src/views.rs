use std::collections::HashMap;
use crate::*;

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
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Market {
    pub asset_id: AccountId,
    pub dtoken: AccountId
}


#[near_bindgen]
impl Contract {

    pub fn view_total_borrows(&self, user_id: AccountId) -> Balance {
        self.get_total_borrows(user_id).into()
    }

    pub fn view_total_supplies(&self, user_id: AccountId) -> Balance {
        self.get_total_supplies(user_id).into()
    }

    pub fn view_markets(&self) -> Vec<Market>  {
        return self.markets.iter().map(|(asset_id, dtoken)| {
            Market {
                asset_id,
                dtoken
            }
        }).collect::<Vec<Market>>();
    }

    pub fn view_accounts(&self, user_ids: Vec<AccountId>) -> Vec<AccountData> {
        return user_ids.iter().map(|user_id| {
            let total_borrows = self.get_total_borrows(user_id.clone());
            let total_supplies = self.get_total_supplies(user_id.clone());
            let health_factor = self.get_health_factor(user_id.clone());
            AccountData {
                account_id: user_id.clone(),
                total_borrows: total_borrows.into(),
                total_supplies: total_supplies.into(),
                blocked: false,
                health_factor
            }
        }).collect::<Vec<AccountData>>();
    }

    pub fn view_prices(&self, assets: Vec<AccountId>) -> HashMap<AccountId, Price> {
        self.get_prices_for_assets(assets)
    }

}


#[cfg(test)]
mod tests {
    use near_sdk::{AccountId, testing_env};
    use near_sdk::test_utils::test_env::{alice, bob};
    use near_sdk::test_utils::VMContextBuilder;
    use crate::{Config, Contract};

    pub fn init_test_env() -> (Contract, AccountId) {

        let controller_contract = Contract::new(Config { owner_id: alice(), oracle_account_id: bob() });

        let context = VMContextBuilder::new()
            .signer_account_id(alice())
            .build();

        testing_env!(context);

        (controller_contract, alice())
    }

    #[test]
    fn test_view_prices() {

    }


    #[test]
    fn test_view_markets() {
        let (mut near_contract, _) = init_test_env();

        let asset_id_1 = AccountId::new_unchecked("weth".to_string());
        let dtoken_id_1 = AccountId::new_unchecked("dtoken.weth.testnet".to_string());
        let asset_id_2 = AccountId::new_unchecked("wnear".to_string());
        let dtoken_id_2 = AccountId::new_unchecked("dtoken.wnear.testnet".to_string());

        near_contract.add_market(asset_id_1.clone(), dtoken_id_1.clone());
        near_contract.add_market(asset_id_2.clone(), dtoken_id_2.clone());

        let accounts = near_contract.view_markets();

        assert_eq!(accounts.len(), 2, "View market response doesn't match");
        assert_eq!(accounts[0].asset_id, asset_id_1, "View market AssetId check has been failed");
        assert_eq!(accounts[0].dtoken, dtoken_id_1, "View market Dtoken check has been failed");
        assert_eq!(accounts[1].asset_id, asset_id_2, "View market AssetId check has been failed");
        assert_eq!(accounts[1].dtoken, dtoken_id_2, "View market Dtoken check has been failed");

    }

    #[test]
    fn test_view_accounts() {
        let (near_contract, _) = init_test_env();

        let mut accounts = Vec::new();
        accounts.push(alice());
        accounts.push(bob());

        let result = near_contract.view_accounts(accounts.clone());

        assert_eq!(result.len(), 2, "View accounts response doesn't match");
        assert_eq!(result[0].account_id, alice(), "View accounts account_id check has been failed");
        assert_eq!(result[1].account_id, bob(), "View accounts account_id check has been failed");
        assert_eq!(result[0].total_borrows,0, "View accounts total borrows check has been failed");
        assert_eq!(result[0].total_supplies,0, "View accounts total supplies check has been failed");
        assert_eq!(result[0].health_factor,10000, "View accounts health factor check has been failed");
    }
}