#![allow(dead_code)]

use near_sdk::{AccountId, env};

use crate::Contract;

pub enum MetricType {
    ReserveFactor,
    HealthFactor,
    LiquidationIncentive,
}

pub enum MethodType {
    Withdraw,
    Repay,
    Supply,
    Liquidate,
    Borrow,
}


impl Contract {
    fn get_admin(&self) -> AccountId {
        return self.admin.clone();
    }

    fn set_admin(&mut self, account: AccountId) {
        assert!(self.is_valid_admin_call(), "This functionality is allowed to be called by admin or contract only");
        self.admin = account;
    }

    pub fn is_valid_admin_call(&self) -> bool {
        env::predecessor_account_id() == self.admin || env::predecessor_account_id() == env::current_account_id()
    }


    fn add_market_asset(&mut self, key: AccountId, value: AccountId) {
        assert!(self.is_valid_admin_call(), "This functionality is allowed to be called by admin or contract only");

        self.markets.insert(&key, &value);
    }

    fn remove_market_asset(&mut self, key: AccountId) {
        assert!(self.is_valid_admin_call(), "This functionality is allowed to be called by admin or contract only");

        assert!(self.markets.contains_key(&key), "Asset by this key doesnt exist");

        self.markets.remove(&key);
    }


    fn get_metric(self, user: AccountId, metric: MetricType) -> u128 {
        assert!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        // TODO implement other respective get metric functionality
        match metric {
            MetricType::ReserveFactor => 0,
            MetricType::HealthFactor => self.get_health_factor(user),
            MetricType::LiquidationIncentive => 0,
        }
    }


    // TODO implement the set_metric once metrics itself has been implemented
    // fn set_metric(mut self, metric: MetricType, value: u128) {
    //     assert!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");
    //
    //     match metric {
    //         MetricType::ReserveFactor => self.reserve_factor = value,
    //         MetricType::HealthFactor => self.health_factor = value,
    //         MetricType::LiquidationIncentive => self.liquidation_incentives = value,
    //     }
    // }

    fn pause_method(mut self, method: MethodType) {
        assert!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        match method {
            MethodType::Withdraw => self.is_action_paused.withdraw = true,
            MethodType::Repay => self.is_action_paused.repay = true,
            MethodType::Supply => self.is_action_paused.supply = true,
            MethodType::Liquidate => self.is_action_paused.liquidate = true,
            MethodType::Borrow => self.is_action_paused.borrow = true
        }
    }

    fn proceed_method(mut self, method: MethodType) {
        assert!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        match method {
            MethodType::Withdraw => self.is_action_paused.withdraw = false,
            MethodType::Repay => self.is_action_paused.repay = false,
            MethodType::Supply => self.is_action_paused.supply = false,
            MethodType::Liquidate => self.is_action_paused.liquidate = false,
            MethodType::Borrow => self.is_action_paused.borrow = false
        }
    }
}


#[cfg(test)]
mod tests {
    use near_sdk::log;

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

        return (near_contract, token_address, user_account);
    }

    #[test]
    fn get_set_admin() {
        let (near_contract, _, _) = init_();
        assert_eq!(near_contract.admin, near_contract.get_admin());
    }

    fn

}