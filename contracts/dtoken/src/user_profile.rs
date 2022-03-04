use near_sdk::collections::Vector;
use crate::*;

#[derive(BorshSerialize)]
pub enum Action {
    Supply,
    Borrow,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserProfile {
    ///  User profile account address
    account_id: AccountId,

    ///  Supply user amount
    supply_amount: WBalance,

    ///  Borrow user amount
    borrow_amount: WBalance,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserProfileController {
    user_profile_list: Vector<UserProfile>,
}

impl Default for UserProfileController {
    fn default() -> Self {
        Self {
            user_profile_list: Vector::new(b"s".to_vec()),
        }
    }
}

#[near_bindgen]
impl UserProfileController {
    pub fn set_supply_amount(&mut self, account_id: AccountId, amount: WBalance) {
        log!("Set supply amount: {} for account: {}", amount as u128, account_id);
        if !self.is_exist(account_id.clone()) {
            let user_profile = UserProfile { account_id, supply_amount: amount, borrow_amount: 0 as WBalance };
            self.user_profile_list.push(&user_profile);
        }else{
            let index = self.get_profile_index(account_id);
            let user_profile = self.user_profile_list.swap_remove_raw(index as u64);
            user_profile.supply_amount = amount;
            self.user_profile_list.push_raw(&user_profile);
        }
    }

    pub fn set_borrow_amount(&mut self, account_id: AccountId, amount: WBalance) {
        log!("Set borrow amount: {} for account: {}", amount as u128, account_id);
        if !self.is_exist(account_id.clone()) {
            let user_profile = UserProfile { account_id, supply_amount: 0 as WBalance, borrow_amount: amount };
            self.user_profile_list.push(&user_profile);
        }else {
            let index = self.get_profile_index(account_id);
            let user_profile = self.user_profile_list.swap_remove_raw(index as u64);
            user_profile.borrow_amount = amount;
            self.user_profile_list.push_raw(&user_profile);
        }
    }

    pub fn is_exist(&self, account_id: AccountId) -> bool {
        let mut exist: bool = false;
        if self.get_profile_index(account_id) != usize::MAX {
            exist = true;
        }
        exist
    }

    pub fn get_profile_index(&self, account_id: AccountId) -> usize {
        self.user_profile_list.iter().position(|profile| profile.account_id == account_id).unwrap_or(usize::MAX)
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
        let mut contract = UserProfileController::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn set_borrow_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.set_borrow_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn get_profile_index_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400 as WBalance);
        assert_eq!(
            1,
            contract.get_profile_index(AccountId::try_from("bob_near".to_string()).unwrap())
        );
    }

    #[test]
    fn is_exist_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400 as WBalance);
        contract.set_supply_amount(AccountId::try_from("marly_near".to_string()).unwrap(), 400 as WBalance);
        assert!(contract.is_exist(AccountId::try_from("bob_near".to_string()).unwrap()));
    }

    #[test]
    fn is_exist_fail_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200 as WBalance);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400 as WBalance);
        assert!(!contract.is_exist(AccountId::try_from("marly_near".to_string()).unwrap()));
    }
}