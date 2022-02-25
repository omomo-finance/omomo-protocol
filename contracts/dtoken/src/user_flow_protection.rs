use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, BlockHeight, log, near_bindgen};
use near_sdk::collections::{UnorderedMap, Vector};
use near_sdk::env::block_height;

const BLOCKS_TO_NEXT_OPERATION: BlockHeight = 100;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserFlowProtection {
    account_action_index: UnorderedMap<AccountId, BlockHeight>,
    blocked_accounts: Vector<AccountId>,
}

impl Default for UserFlowProtection {
    fn default() -> Self {
        Self {
            account_action_index: UnorderedMap::new(b"s".to_vec()),
            blocked_accounts: Vector::new(b"s".to_vec()),
        }
    }
}

#[near_bindgen]
impl UserFlowProtection {
    pub fn block_account(&mut self, account_id: AccountId) {
        log!("Block operation for account: {}", account_id);
        self.blocked_accounts.push(&account_id);
    }

    pub fn unblock_account(&mut self, account_id: AccountId) {
        log!("Unblock operation for account: {}", account_id);
        match self.blocked_accounts.iter().position(|value| value == account_id) {
            Some(index) => {
                self.blocked_accounts.swap_remove(index as u64);
                log!("Account unblocked successful!")
            }
            None => log!("Account not found!")
        }
    }

    pub fn set_account_action_index(&mut self, account_id: AccountId) {
        let block_height = block_height();
        log!("Set last perform account: {} block_height: {}", account_id, block_height);
        self.block_account(account_id.clone());
        self.account_action_index.insert(&account_id, &block_height);
    }

    pub fn get_last_user_perform(&self, account_id: AccountId) -> Option<BlockHeight> {
        log!("Get last perform account: {}", account_id);
        self.account_action_index.get(&account_id)
    }

    pub fn is_user_can_perform_operation(&mut self, account_id: AccountId) -> bool {
        log!("User can perform: {}", account_id);
        let mut access: bool = false;
        let last_perform_block_height = self.get_last_user_perform(account_id.clone());
        let current_block_height = block_height();
        if current_block_height - last_perform_block_height.unwrap() >= BLOCKS_TO_NEXT_OPERATION {
            match self.blocked_accounts.iter().position(|account| account == account_id) {
                Some(index) => {
                    self.blocked_accounts.swap_remove(index as u64);
                    log!("Account: {} unblocked successful!", account_id)
                }
                None => log!("Account: {} not blocked!", account_id)
            }
            access = true;
        }
        access
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
    fn set_last_user_perform() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserFlowProtection::default();
        contract.set_account_action_index(AccountId::try_from("alice_near".to_string()).unwrap());
        assert_eq!(
            101,
            contract.get_last_user_perform(AccountId::try_from("alice_near".to_string()).unwrap()).unwrap()
        );
    }

    #[test]
    fn get_last_user_perform() {
        let context = get_context(false);
        testing_env!(context);
        let contract = UserFlowProtection::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        assert_eq!(None, contract.get_last_user_perform(account));
    }

    #[test]
    fn is_user_can_perform_operation() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserFlowProtection::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        contract.set_account_action_index(AccountId::try_from("alice_near".to_string()).unwrap());
        assert_eq!(false, contract.is_user_can_perform_operation(account));
    }
}
