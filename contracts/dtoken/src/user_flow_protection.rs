use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, BlockHeight, log, near_bindgen};
use near_sdk::collections::{UnorderedMap};
use near_sdk::env::block_height;

const BLOCKS_TO_NEXT_OPERATION: BlockHeight = 100;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserFlowProtection {
    blocked_accounts: UnorderedMap<AccountId, BlockHeight>,
}

impl Default for UserFlowProtection {
    fn default() -> Self {
        Self {
            blocked_accounts: UnorderedMap::new(b"s".to_vec()),
        }
    }
}

#[near_bindgen]
impl UserFlowProtection {
    pub fn block_account(&mut self, account_id: AccountId) {
        let block_height = block_height();
        log!("Block account: {} at block index: {}", account_id, block_height);
        self.blocked_accounts.insert(&account_id, &block_height);
    }

    pub fn unblock_account(&mut self, account_id: AccountId) {
        log!("Unblock operation for account: {}", account_id);
        self.blocked_accounts.remove(&account_id);
    }

    pub fn get_last_block_index(&self, account_id: AccountId) -> BlockHeight {
        match self.blocked_accounts.get(&account_id) {
            Some(index) => index,
            None => 0
        }
    }

    pub fn is_user_can_perform_operation(&mut self, account_id: AccountId) -> bool {
        log!("Account: {}  can do action", account_id);
        let mut access: bool = false;
        let current_block_height = block_height();
        let blocked_index = self.get_last_block_index(account_id.clone());
        if current_block_height - blocked_index >= BLOCKS_TO_NEXT_OPERATION {
            self.unblock_account(account_id);
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
    fn block_account() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserFlowProtection::default();
        contract.block_account(AccountId::try_from("alice_near".to_string()).unwrap());
        assert_eq!(
            101,
            contract.get_last_block_index(AccountId::try_from("alice_near".to_string()).unwrap())
        );
    }

    #[test]
    fn unblock_account() {
        let context = get_context(false);
        testing_env!(context);
        let contract = UserFlowProtection::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        assert_eq!(0, contract.get_last_block_index(account));
    }

    #[test]
    fn is_user_can_perform_operation() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserFlowProtection::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        contract.block_account(AccountId::try_from("alice_near".to_string()).unwrap());
        assert_eq!(false, contract.is_user_can_perform_operation(account));
    }
}
