use general::NO_DEPOSIT;
use near_sdk::env::block_height;
use near_sdk::{env, require, AccountId};

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
            block_height(),
            self.get_controller_address(),
            NO_DEPOSIT,
            self.terra_gas(5),
        );
    }
}

#[cfg(test)]
mod tests {
    use crate::InterestRateModel;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::bob;

    use crate::Config;

    use super::*;

    #[test]
    fn set_get_admin() {
        let dtoken_contract = Contract::new(Config {
            initial_exchange_rate: U128(10000),
            underlying_token_id: "weth".parse().unwrap(),
            owner_id: "dtoken".parse().unwrap(),
            controller_account_id: "controller".parse().unwrap(),
            interest_rate_model: InterestRateModel::default(),
        });

        assert_eq!(dtoken_contract.admin, dtoken_contract.get_admin());
        assert_eq!(
            AccountId::new_unchecked("dtoken".parse().unwrap()),
            dtoken_contract.get_admin()
        );
    }

    #[test]
    fn update_exchange_rate() {
        let mut dtoken_contract = Contract::new(Config {
            initial_exchange_rate: U128(10000),
            underlying_token_id: "weth".parse().unwrap(),
            owner_id: "dtoken".parse().unwrap(),
            controller_account_id: "controller".parse().unwrap(),
            interest_rate_model: InterestRateModel::default(),
        });
        dtoken_contract.mint(bob(), U128(1000));
        let exchange_rate = dtoken_contract.get_exchange_rate(U128(20000));
        assert_eq!(exchange_rate, 200000);

        dtoken_contract.set_total_reserves(10000);
        let exchange_rate = dtoken_contract.get_exchange_rate(U128(20000));
        assert_eq!(exchange_rate, 100000);
    }
}
