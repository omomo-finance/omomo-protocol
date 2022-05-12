use general::NO_DEPOSIT;
use near_sdk::env::block_height;
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

    pub fn is_valid_admin_call(&self) -> bool {
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
    use crate::InterestRateModel;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    use crate::Config;

    use super::*;

    pub fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(alice())
            .signer_account_id(alice())
            .is_view(is_view)
            .build()
    }

    pub fn init(is_admin: bool) -> Contract {
        let (dtoken_account, underlying_token_account, controller_account) =
            (alice(), bob(), carol());

        if is_admin {
            testing_env!(get_context(false));
        }

        let mut contract = Contract::new(Config {
            initial_exchange_rate: U128(1000000),
            underlying_token_id: underlying_token_account,
            owner_id: dtoken_account,
            controller_account_id: controller_account,
            interest_rate_model: InterestRateModel::default(),
        });

        if is_admin {
            contract.set_total_reserves(200);
        }

        contract
    }

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
    fn test_increase_total_reserve() {
        let mut contract = init(true);

        contract.increase_reserve(WBalance::from(300));

        // 200 is initial total_reserve set up in init_test_env
        assert_eq!( WBalance::from(200 + 300), contract.view_total_reserves());
    }
}
