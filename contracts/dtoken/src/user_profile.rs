use near_sdk::{AccountId, log, near_bindgen};
use near_sdk::collections::Vector;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

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
        log!("Set supply amount: {} for account: {}", amount, account_id);
        if !self.is_exist(account_id.clone()) {
            let user_profile = UserProfile { account_id, supply_amount: amount, borrow_amount: 0 };
            self.user_profile_list.push(&user_profile);
        }
    }

    pub fn set_borrow_amount(&mut self, account_id: AccountId, amount: WBalance) {
        log!("Set borrow amount: {} for account: {}", amount, account_id);
        if !self.is_exist(account_id.clone()) {
            let user_profile = UserProfile { account_id, supply_amount: 0, borrow_amount: amount };
            self.user_profile_list.push(&user_profile);
        }
    }

    pub fn is_exist(&self, account_id: AccountId) -> bool {
        let mut exist: bool = false;
        if self.get_profile_index(account_id) > -1 {
            exist = true
        }
        exist
    }

    //self.user_profile_list.iter().position(|profile| profile.account_id == account_id).unwrap_or(-1) not work!
    pub fn get_profile_index(&self, account_id: AccountId) -> usize {
        match self.user_profile_list.iter().position(|profile| profile.account_id == account_id).unwrap_or(unsize::Max) {
            index => index,
            None => -1
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
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
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn set_borrow_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.set_borrow_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200);
        assert!(contract.is_exist(AccountId::try_from("alice_near".to_string()).unwrap()));
    }

    #[test]
    fn get_profile_index_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400);
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
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400);
        contract.set_supply_amount(AccountId::try_from("marly_near".to_string()).unwrap(), 400);
        assert!(contract.is_exist(AccountId::try_from("bob_near".to_string()).unwrap()));
    }

    #[test]
    fn is_exist_fail_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserProfileController::default();
        contract.set_supply_amount(AccountId::try_from("alice_near".to_string()).unwrap(), 200);
        contract.set_supply_amount(AccountId::try_from("bob_near".to_string()).unwrap(), 400);
        assert!(!contract.is_exist(AccountId::try_from("marly_near".to_string()).unwrap()));
    }
}