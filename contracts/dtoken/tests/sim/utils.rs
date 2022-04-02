use near_sdk::AccountId;
use near_sdk_sim::{deploy, to_yocto, ContractAccount, UserAccount};

use controller::ContractContract as Controller;
use dtoken::ContractContract as Dtoken;
use test_utoken::ContractContract as Utoken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DTOKEN_WASM_BYTES => "../../res/dtoken.wasm",
    UTOKEN_WASM_BYTES => "../../res/test_utoken.wasm",
    CONTROLLER_WASM_BYTES => "../../res/controller.wasm",
}

pub fn init_dtoken(
    root: UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Dtoken>, UserAccount) {
    let contract = deploy!(
        contract: Dtoken,
        contract_id: token_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    let user_account = root.create_user(
        "user_account".parse().unwrap(),
        to_yocto("1000000"), // initial balance
    );

    (root, contract, user_account)
}

pub fn init_two_dtokens(
    root: UserAccount,
    token1_id: AccountId,
    token2_id: AccountId,
) -> (
    UserAccount,
    ContractAccount<Dtoken>,
    ContractAccount<Dtoken>,
    UserAccount,
    UserAccount,
) {
    let contract1 = deploy!(
        contract: Dtoken,
        contract_id: token1_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    let user_account1 = root.create_user(
        "user10_account".parse().unwrap(),
        to_yocto("10000"), // initial balance
    );

    let contract2 = deploy!(
        contract: Dtoken,
        contract_id: token2_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    let user_account2 = root.create_user(
        "user11_account".parse().unwrap(),
        to_yocto("10000"), // initial balance
    );

    (root, contract1, contract2, user_account1, user_account2)
}

pub fn init_utoken(
    root: UserAccount,
    token_id: AccountId,
    account_name: String,
) -> (UserAccount, ContractAccount<Utoken>, UserAccount) {
    let contract = deploy!(
        contract: Utoken,
        contract_id: token_id,
        bytes: &UTOKEN_WASM_BYTES,
        signer_account: root
    );

    let user_account = root.create_user(
        account_name.as_str().parse().unwrap(),
        to_yocto("1000000"), // initial balance
    );

    (root, contract, user_account)
}

pub fn init_controller(
    root: UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Controller>, UserAccount) {
    let contract = deploy!(
        contract: Controller,
        contract_id: token_id,
        bytes: &CONTROLLER_WASM_BYTES,
        signer_account: root
    );

    let user_account = root.create_user(
        "user3_account".parse().unwrap(),
        to_yocto("1000000"), // initial balance
    );

    (root, contract, user_account)
}
