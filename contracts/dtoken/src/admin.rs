use general::NO_DEPOSIT;
use near_sdk::{env, require, AccountId, Balance};

use crate::*;

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

    fn is_valid_admin_call(&self) -> bool {
        env::signer_account_id() == self.admin
            || env::signer_account_id() == env::current_account_id()
    }

    pub fn add_inconsistent_account(&mut self, account: AccountId) {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );

        controller::set_account_consistency(
            account,
            false,
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        );
    }

    pub fn set_total_reserves(&mut self, amount: Balance) -> Balance {
        require!(
            self.is_valid_admin_call(),
            "This functionality is allowed to be called by admin or contract only"
        );

        self.total_reserves = amount;
        self.get_total_reserves()
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::json_types::U128;

    use crate::Config;

    use super::*;

    #[test]
    fn set_get_admin() {
        let dtoken_contract = Contract::new(Config {
            initial_exchange_rate: U128(10000),
            underlying_token_id: "weth".parse().unwrap(),
            owner_id: "dtoken".parse().unwrap(),
            controller_account_id: "controller".parse().unwrap(),
        });

        assert_eq!(dtoken_contract.admin, dtoken_contract.get_admin());
        assert_eq!(
            AccountId::new_unchecked("dtoken".parse().unwrap()),
            dtoken_contract.get_admin()
        );
    }
}
