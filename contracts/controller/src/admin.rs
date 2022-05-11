use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{env, near_bindgen, require, AccountId};
use std::collections::HashMap;

use general::percent::Percent;
use general::ratio::Ratio;

use crate::*;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum MethodType {
    Withdraw,
    Repay,
    Supply,
    Liquidate,
    Borrow,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Market {
    pub asset_id: AccountId,
    pub dtoken: AccountId,
    pub ticker_id: String,
}

#[near_bindgen]
impl Contract {
    pub fn get_admin(&self) -> AccountId {
        self.admin.clone()
    }

    pub fn set_admin(&mut self, account: AccountId) {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );
        self.admin = account;
    }

    #[private]
    pub fn is_valid_admin_call(&self) -> bool {
        env::signer_account_id() == self.admin
            || env::signer_account_id() == env::current_account_id()
    }

    #[private]
    pub fn is_dtoken_caller(&self) -> bool {
        self.markets
            .values()
            .any(|profile| profile.dtoken == env::predecessor_account_id())
    }

    pub fn get_markets_list(&self) -> Vec<Market> {
        return self
            .markets
            .iter()
            .map(|(asset_id, market)| Market {
                asset_id,
                dtoken: market.dtoken,
                ticker_id: market.ticker_id,
            })
            .collect::<Vec<Market>>();
    }

    pub fn get_tickers_dtoken_hash(&self) -> HashMap<String, AccountId> {
        let mut result: HashMap<String, AccountId> = HashMap::new();
        self.markets.iter().for_each(|(_, market)| {
            result.insert(market.ticker_id, market.dtoken);
        });
        result
    }

    pub fn get_utoken_tickers_hash(&self) -> HashMap<AccountId, String> {
        let mut result: HashMap<AccountId, String> = HashMap::new();
        self.markets.iter().for_each(|(asset_id, market)| {
            result.insert(asset_id, market.ticker_id);
        });
        result
    }

    pub fn add_market(&mut self, asset_id: AccountId, dtoken: AccountId, ticker_id: String) {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );

        let market = MarketProfile { dtoken, ticker_id };

        self.markets.insert(&asset_id, &market);
    }

    pub fn remove_market(&mut self, key: AccountId) {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );

        require!(
            self.markets.get(&key).is_some(),
            "Asset by this key doesnt exist"
        );

        self.markets.remove(&key);
    }

    pub fn get_reserve_factor(self) -> Percent {
        require!(
            self.is_valid_admin_call(),
            "this functionality is allowed to be called by admin or contract only"
        );

        self.reserve_factor
    }

    pub fn get_liquidation_incentive(&self) -> Ratio {
        // TODO: Move this kind of getter that don't require admin rights somewhere else
        // incentive % + 100 %
        self.liquidation_incentive + Ratio(10000)
    }

    pub fn get_health_threshold(&self) -> Ratio {
        self.health_threshold
    }

    pub fn set_health_factor_threshold(mut self, value: Ratio) {
        // TODO: Maybe change name of this funcction
        require!(
            self.is_valid_admin_call(),
            "this functionality is allowed to be called by admin or contract only"
        );

        self.health_threshold = value;
    }

    pub fn set_liquidation_incentive(mut self, value: Ratio) {
        require!(
            self.is_valid_admin_call(),
            "this functionality is allowed to be called by admin or contract only"
        );

        self.liquidation_incentive = value;
    }

    pub fn set_reserve_factor(mut self, value: Percent) {
        require!(
            self.is_valid_admin_call(),
            "this functionality is allowed to be called by admin or contract only"
        );

        self.reserve_factor = value;
    }

    pub fn pause_method(mut self, method: MethodType) {
        require!(
            self.is_valid_admin_call(),
            "this functionality is allowed to be called by admin or contract only"
        );

        match method {
            MethodType::Withdraw => self.is_action_paused.withdraw = true,
            MethodType::Repay => self.is_action_paused.repay = true,
            MethodType::Supply => self.is_action_paused.supply = true,
            MethodType::Liquidate => self.is_action_paused.liquidate = true,
            MethodType::Borrow => self.is_action_paused.borrow = true,
        }
    }

    pub fn proceed_method(mut self, method: MethodType) {
        require!(
            self.is_valid_admin_call(),
            "this functionality is allowed to be called by admin or contract only"
        );

        match method {
            MethodType::Withdraw => self.is_action_paused.withdraw = false,
            MethodType::Repay => self.is_action_paused.repay = false,
            MethodType::Supply => self.is_action_paused.supply = false,
            MethodType::Liquidate => self.is_action_paused.liquidate = false,
            MethodType::Borrow => self.is_action_paused.borrow = false,
        }
    }

    pub fn get_user_profile(&self, user_id: AccountId) -> UserProfile {
        self.user_profiles.get(&user_id).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use crate::Config;

    use super::*;

    pub fn init_() -> (Contract, AccountId, AccountId) {
        let owner_account: AccountId = "contract.near".parse().unwrap();
        let oracle_account: AccountId = "oracle.near".parse().unwrap();
        let user_account: AccountId = "user.near".parse().unwrap();

        let near_contract = Contract::new(Config {
            owner_id: owner_account,
            oracle_account_id: oracle_account,
        });

        let token_address: AccountId = "near".parse().unwrap();

        (near_contract, token_address, user_account)
    }

    #[test]
    fn get_set_admin() {
        let (near_contract, _, _) = init_();
        assert_eq!(near_contract.admin, near_contract.get_admin());
    }
}
