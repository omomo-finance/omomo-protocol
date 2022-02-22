use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::{AccountId, BlockHeight, log, near_bindgen};
use near_sdk::collections::{UnorderedMap};
use near_sdk::env::block_height;

const BLOCKS_TO_NEXT_OPERATION: BlockHeight = 100;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserPerform {
    last_user_perform: UnorderedMap<AccountId, BlockHeight>,
}

impl Default for UserPerform {
    fn default() -> Self {
        Self {
            last_user_perform: UnorderedMap::new(b"s".to_vec()),
        }
    }
}

#[near_bindgen]
impl UserPerform {
    pub fn set_last_user_perform(&mut self, account_id: AccountId) {
        let block_height = block_height();
        log!("Set last perform account: {} block_height: {}", account_id, block_height);
        self.last_user_perform.insert(&account_id, &block_height);
    }

    pub fn get_last_user_perform(&self, account_id: AccountId) -> Option<BlockHeight> {
        log!("Get last perform account: {}", account_id);
        self.last_user_perform.get(&account_id)
    }

    pub fn is_user_can_perform_operation(&self, account_id: AccountId) -> bool {
        log!("User can perform: {}", account_id);
        let last_perform_block_height = self.get_last_user_perform(account_id);
        let current_block_height = block_height();
        current_block_height - last_perform_block_height.unwrap() >= BLOCKS_TO_NEXT_OPERATION
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
        let context = get_context( false);
        testing_env!(context);
        let mut contract = UserPerform::default();
        contract.set_last_user_perform( AccountId::try_from("alice_near".to_string()).unwrap());
        assert_eq!(
            101,
            contract.get_last_user_perform( AccountId::try_from("alice_near".to_string()).unwrap()).unwrap()
        );
    }

    #[test]
    fn get_last_user_perform() {
        let context = get_context(false);
        testing_env!(context);
        let contract = UserPerform::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        assert_eq!(None, contract.get_last_user_perform(account));
    }

    #[test]
    fn is_user_can_perform_operation() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = UserPerform::default();
        let account = AccountId::try_from("alice_near".to_string()).unwrap();
        contract.set_last_user_perform( AccountId::try_from("alice_near".to_string()).unwrap());
        assert_eq!(false, contract.is_user_can_perform_operation(account));
    }
}