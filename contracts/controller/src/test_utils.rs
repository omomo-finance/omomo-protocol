use near_sdk::AccountId;
use near_sdk::test_utils::test_env::{alice, bob, carol};

use crate::{Config, Contract};

pub fn init_test_env() -> (Contract, AccountId, AccountId) {
    let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

    let eth_contract = Contract::new(Config { owner_id: owner_account, oracle_account_id: oracle_account });

    let token_address: AccountId = "near".parse().unwrap();

    return (eth_contract, token_address, user_account);
}