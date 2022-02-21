use crate::*;

#[near_bindgen]
impl Contract {
    // use near_sdk::AccountId;
// use near_sdk::test_utils::test_env::{alice, bob, carol};

// use crate::{Config, Contract};

// pub fn init_test_env() -> (Contract, AccountId, AccountId) {
//     let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

//     let eth_contract = Contract::new(Config { owner_id: owner_account, oracle_account_id: oracle_account });

//     let token_address: AccountId = "near".parse().unwrap();

//     return (eth_contract, token_address, user_account);
// }

    fn is_repay_allowed(
        &self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> bool {
        true
    }

    pub fn repay_borrows(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    )-> Balance{
        assert_eq!(
            self.is_repay_allowed(
                account_id.clone(),
                token_address.clone(),
                token_amount.clone(),
            ),
            true,
            "repay operation is not allowed for account {} on market {}, repay amount {}",
            account_id,
            token_address,
            Balance::from(token_amount)
        );

        return self.decrease_borrows(account_id, token_address, token_amount);
    }

}