use near_sdk::collections::LookupMap;
use crate::*;

#[derive(BorshSerialize)]
pub enum Action {
    Supply,
    Borrow,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserProfile {
    ///  Supply user amount
    pub(crate) supply_amount: WBalance,

    ///  Borrow user amount
    pub(crate) borrow_amount: WBalance,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserProfileDtoken {
    user_profile_list: LookupMap<AccountId, UserProfile>,
}

impl Default for UserProfileDtoken {
    fn default() -> Self {
        Self {
            user_profile_list: LookupMap::new(b"s".to_vec()),
        }
    }
}

#[near_bindgen]
impl UserProfileDtoken {
    pub fn set_supply_amount(&mut self, account_id: AccountId, amount: WBalance) {
        log!("Set supply amount: {} for account: {}", amount as u128, account_id);
        if !self.is_exist(account_id.clone()) {
            let user_profile = UserProfile { supply_amount: amount, borrow_amount: 0 as WBalance };
            self.user_profile_list.insert(&account_id, &user_profile);
        } else {
            let mut user_profile = self.user_profile_list.get(&account_id).unwrap();
            self.user_profile_list.remove(&account_id);
            user_profile.supply_amount = amount;
            self.user_profile_list.insert(&account_id, &user_profile);
        }
    }

    pub fn set_borrow_amount(&mut self, account_id: AccountId, amount: WBalance) {
        log!("Set borrow amount: {} for account: {}", amount as u128, account_id);
        if !self.is_exist(account_id.clone()) {
            let user_profile = UserProfile { supply_amount: 0 as WBalance, borrow_amount: amount };
            self.user_profile_list.insert(&account_id, &user_profile);
        } else {
            let mut user_profile = self.user_profile_list.get(&account_id).unwrap();
            user_profile.borrow_amount = amount;
            self.user_profile_list.insert(&account_id, &user_profile);
        }
    }

    pub fn is_exist(&self, account_id: AccountId) -> bool {
        match self.user_profile_list.get(&account_id) {
            Some(_profile) => true,
            None => false
        }
    }

    pub fn get_profile(&self, account_id: AccountId) -> UserProfile {
        self.user_profile_list.get(&account_id).unwrap()
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;
    use super::*;
    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::VMContextBuilder;

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
    fn set_supply_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileDtoken::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn set_borrow_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileDtoken::default();
        contract.set_borrow_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn is_exist_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileDtoken::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400 as WBalance);
        contract.set_supply_amount(AccountId::try_from("marly_near".to_string()).unwrap(), 400 as WBalance);
        assert!(contract.is_exist(AccountId::try_from("bob_near".to_string()).unwrap()));
    }

    #[test]
    fn is_exist_fail_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileDtoken::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400 as WBalance);
        assert!(!contract.is_exist(AccountId::try_from("marly_near".to_string()).unwrap()));
    }
}