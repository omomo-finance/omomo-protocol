use near_sdk::{AccountId, BlockHeight, log, near_bindgen};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::env::block_height;

const BLOCKS_TO_NEXT_OPERATION: BlockHeight = 100;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct ActionMutex {
    blocked_accounts: LookupMap<AccountId, BlockHeight>,
}

impl Default for ActionMutex {
    fn default() -> Self {
        Self {
            blocked_accounts: LookupMap::new(b"s".to_vec()),
        }
    }
}

impl ActionMutex {
    pub fn lock(&mut self, account_id: &AccountId) {
        let block_height = block_height();
        log!("Lock account: {}", account_id);
        self.blocked_accounts.insert(&account_id, &block_height);
    }

    pub fn unlock(&mut self, account_id: &AccountId) {
        log!("Unlock operation for account: {}", account_id);
        self.blocked_accounts.remove(&account_id);
    }

    pub fn get_last_block_index(&self, account_id: &AccountId) -> BlockHeight {
        self.blocked_accounts.get(&account_id).unwrap_or(0)
    }

    pub fn try_lock(&mut self, account_id: &AccountId) -> bool {
        log!("Try lock account: {}", account_id);
        let mut is_locked: bool = true;
        let current_block_height = block_height();
        let blocked_index = self.get_last_block_index(&account_id);
        if blocked_index > 0 && current_block_height - blocked_index <= BLOCKS_TO_NEXT_OPERATION {
            is_locked = false;
        } else {
            self.blocked_accounts.insert(&account_id, &current_block_height);
        }
        is_locked
    }

    pub fn is_user_can_perform_operation(&mut self, account_id: &AccountId) -> bool {
        log!("Account: {}  can do action", account_id);
        let mut access: bool = false;
        let current_block_height = block_height();
        let blocked_index = self.get_last_block_index(&account_id);
        if current_block_height - blocked_index >= BLOCKS_TO_NEXT_OPERATION {
            if blocked_index > 0 {
                self.unlock(account_id);
            }
            access = true;
        }
        access
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use near_sdk::{testing_env, VMContext};
    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;

    use super::*;

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
    fn lock_account() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = ActionMutex::default();
        contract.lock(&AccountId::try_from("alice_near".to_string()).unwrap());
        assert_eq!(
            101,
            contract.get_last_block_index(&AccountId::try_from("alice_near".to_string()).unwrap())
        );
    }

    #[test]
    fn try_lock_after_unlock() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = ActionMutex::default();

        let alice = alice();

        // has to be present in blocked_account
        contract.lock(&alice);
        assert!(contract.blocked_accounts.get(&alice).is_some());

        // as the account is already blocked, but there no more blocks produced it has to fail
        assert!(!contract.try_lock(&alice));
        assert!(contract.blocked_accounts.get(&alice).is_some());

        // as we unlock account it should be absent in blocked_accounts
        (contract.unlock(&alice));
        assert!(contract.blocked_accounts.get(&alice).is_none());

        // should be added in blocked_accounts again
        assert!(contract.try_lock(&alice));
        assert!(contract.blocked_accounts.get(&alice).is_some());
    }

    #[test]
    fn unlock_account() {
        let context = get_context(false);
        testing_env!(context);
        let contract = ActionMutex::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        assert_eq!(0, contract.get_last_block_index(&account));
    }

    #[test]
    fn is_user_can_perform_operation() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = ActionMutex::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        contract.lock(&AccountId::try_from("alice_near".to_string()).unwrap());
        assert!(!contract.is_user_can_perform_operation(&account));
    }
}
