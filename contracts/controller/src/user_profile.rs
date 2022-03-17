use near_sdk::collections::UnorderedMap;
use crate::*;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserProfileControllerStorage {
    /// User Account ID -> Dtoken address -> Supplies balance
    pub account_supplies: UnorderedMap<AccountId, Balance>,

    /// User Account ID -> Dtoken address -> Borrow balance
    pub account_borrows: UnorderedMap<AccountId, Balance>,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserProfileController {
    user_profiles: LookupMap<AccountId, UserProfileControllerStorage>,
}

impl Default for UserProfileController {
    fn default() -> Self {
        Self {
            user_profiles: LookupMap::new(b"s".to_vec()),
        }
    }
}

impl UserProfileController {
    pub fn store_supply_amount(&mut self, account_id: AccountId, amount: Balance) {
        if self.is_exist(account_id.clone()) {
            let mut profile = self.get_user_profile(account_id.clone());
            profile.account_supplies.insert(&account_id, &amount);
        } else {
            let mut profile = UserProfileControllerStorage {
                account_supplies: UnorderedMap::new(b"s".to_vec()),
                account_borrows: UnorderedMap::new(b"s".to_vec()),
            };
            profile.account_supplies.insert(&account_id, &amount);
            self.user_profiles.insert(&account_id, &profile);
        }
        log!("Stored supply amount: {} for account: {}", amount, account_id);
    }

    pub fn store_borrow_amount(&mut self, account_id: AccountId, amount: Balance) {
        if self.is_exist(account_id.clone()) {
            let mut profile = self.get_user_profile(account_id.clone());
            profile.account_borrows.insert(&account_id, &amount);
        } else {
            let mut profile = UserProfileControllerStorage {
                account_supplies: UnorderedMap::new(b"s".to_vec()),
                account_borrows: UnorderedMap::new(b"s".to_vec()),
            };
            profile.account_borrows.insert(&account_id.clone(), &amount);
            self.user_profiles.insert(&account_id, &profile);
        }
        log!("Stored borrow amount: {} for account: {}", amount, account_id);
    }

    pub fn remove(&mut self, account_id: AccountId) {
        if self.is_exist(account_id.clone()) {
            self.user_profiles.remove(&account_id);
        }
    }

    pub fn get_user_profile(&self, account_id: AccountId) -> UserProfileControllerStorage {
        self.user_profiles.get(&account_id).unwrap()
    }

    pub fn is_exist(&self, account_id: AccountId) -> bool {
        match self.user_profiles.get(&account_id) {
            Some(_profile) => true,
            None => false
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use super::*;
    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::{VMContextBuilder};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(AccountId::try_from("alice_near".to_string()).unwrap())
            .signer_account_id(AccountId::try_from("bob_near".to_string()).unwrap())
            .predecessor_account_id(AccountId::try_from("carol_near".to_string()).unwrap())
            .block_index(101)
            .block_timestamp(0)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn store_supply_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.store_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200_u128);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn check_supply_amount_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.store_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200_u128);
        let profile = contract.get_user_profile(
            AccountId::try_from("alice_near".to_string()).unwrap()).account_supplies;
        assert_eq!(200_u128, profile.get(&AccountId::try_from("alice_near".to_string()).unwrap()).unwrap());
    }

    #[test]
    fn store_borrow_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.store_borrow_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 300_u128);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn check_borrow_amount_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.store_borrow_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200_u128);
        let profile = contract.get_user_profile(
            AccountId::try_from("alice_near".to_string()).unwrap()).account_borrows;
        assert_eq!(200_u128, profile.get(&AccountId::try_from("alice_near".to_string()).unwrap()).unwrap());
    }

    #[test]
    fn is_exist_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.store_borrow_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 100_u128);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn is_no_exist_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.store_borrow_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 1000_u128);
        assert!(!contract.is_exist(AccountId::try_from("bob_near".to_string()).unwrap()));
    }
}