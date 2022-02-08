use near_sdk_sim::{UserAccount, call, deploy, ContractAccount, init_simulator, to_yocto};
use near_sdk::{AccountId};

use test_utoken::ContractContract as Utoken;
use dtoken::ContractContract as Dtoken;


near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DTOKEN_WASM_BYTES => "../../res/dtoken.wasm",
    UTOKEN_WASM_BYTES => "../../res/test_utoken.wasm",
    CONTROLLER_WASM_BYTES => "../../res/controller.wasm",
}



pub fn dai() -> AccountId {
    AccountId::new_unchecked("dai001".to_string())
}

pub fn eth() -> AccountId {
    AccountId::new_unchecked("eth002".to_string())
}

pub fn usdt() -> AccountId {
    AccountId::new_unchecked("usdt".to_string())
}

pub fn usdc() -> AccountId {
    AccountId::new_unchecked("usdc".to_string())
}


pub fn init_dtoken(
    root: &UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Dtoken>, UserAccount) {
    let root = init_simulator(None);

    let contract = deploy!(
        contract: Dtoken,
        contract_id: token_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    let user_account = root.create_user(
        "user_account".parse().unwrap(),
        to_yocto("1000"), // initial balance
    );

    (root, contract, user_account)
}

pub fn init_utoken(
    root: &UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Utoken>, UserAccount) {
    let root = init_simulator(None);

    let contract = deploy!(
        contract: Utoken,
        contract_id: token_id,
        bytes: &UTOKEN_WASM_BYTES,
        signer_account: root
    );

    let user_account = root.create_user(
        "user_account".parse().unwrap(),
        to_yocto("1000"), // initial balance
    );

    (root, contract, user_account)
}