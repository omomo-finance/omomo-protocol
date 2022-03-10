use near_sdk::{AccountId, env, require};

use general::{Percent, Ratio};

use crate::Contract;

pub enum MethodType {
    Withdraw,
    Repay,
    Supply,
    Liquidate,
    Borrow,
}


impl Contract {
    pub fn get_admin(&self) -> AccountId {
        return self.admin.clone();
    }

    pub fn set_admin(&mut self, account: AccountId) {
        require!(self.is_valid_admin_call(), "This functionality is allowed to be called by admin or contract only");
        self.admin = account;
    }

    fn is_valid_admin_call(&self) -> bool {
        env::predecessor_account_id() == self.admin || env::predecessor_account_id() == env::current_account_id()
    }


    pub fn add_market_asset(&mut self, key: AccountId, value: AccountId) {
        require!(self.is_valid_admin_call(), "This functionality is allowed to be called by admin or contract only");

        self.markets.insert(&key, &value);
    }

    pub fn remove_market_asset(&mut self, key: AccountId) {
        require!(self.is_valid_admin_call(), "This functionality is allowed to be called by admin or contract only");

        require!(self.markets.contains_key(&key), "Asset by this key doesnt exist");

        self.markets.remove(&key);
    }


    pub fn get_reserve_factor(self) -> Percent {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        self.reserve_factor
    }

    pub fn get_liquidation_incentive(self) -> Ratio {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        self.liquidation_incentive
    }

    pub fn get_health_factor_threshold(self) -> Ratio {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        self.health_factor_threshold
    }

    pub fn set_health_factor_threshold(mut self, value: Ratio) {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        self.health_factor_threshold = value;
    }


    pub fn set_liquidation_incentive(mut self, value: Ratio) {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        self.liquidation_incentive = value;
    }

    pub fn set_reserve_factor(mut self, value: Percent) {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        self.reserve_factor = value;
    }

    pub fn pause_method(mut self, method: MethodType) {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

        match method {
            MethodType::Withdraw => self.is_action_paused.withdraw = true,
            MethodType::Repay => self.is_action_paused.repay = true,
            MethodType::Supply => self.is_action_paused.supply = true,
            MethodType::Liquidate => self.is_action_paused.liquidate = true,
            MethodType::Borrow => self.is_action_paused.borrow = true
        }
    }

    pub fn proceed_method(mut self, method: MethodType) {
        require!(self.is_valid_admin_call(), "this functionality is allowed to be called by admin or contract only");

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
}