use near_sdk_sim::{UserAccount, deploy, ContractAccount, to_yocto};
use near_sdk::{AccountId };

use test_utoken::ContractContract as Utoken;
use dtoken::ContractContract as Dtoken;
use controller::ContractContract as Controller;



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
        to_yocto("1000000") // initial balance
    );

    (root, contract, user_account)
}

pub fn init_utoken(
    root: UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Utoken>, UserAccount) {


    let contract = deploy!(
        contract: Utoken,
        contract_id: token_id,
        bytes: &UTOKEN_WASM_BYTES,
        signer_account: root
    );

    let user_account = root.create_user(
        "user2_account".parse().unwrap(),
        to_yocto("1000000") // initial balance
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
        to_yocto("1000000") // initial balance
    );

    (root, contract, user_account)
}