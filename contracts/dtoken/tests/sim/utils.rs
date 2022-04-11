use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, to_yocto, view, ContractAccount, ExecutionResult, UserAccount};

use controller::ContractContract as Controller;
use controller::{ActionType, Config as cConfig};
use dtoken::Config as dConfig;
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

pub fn assert_failure(outcome: ExecutionResult, error_message: &str) {
    assert!(!outcome.is_ok());
    let exe_status = format!(
        "{:?}",
        outcome.promise_errors()[0].as_ref().unwrap().status()
    );
    println!("{}", exe_status);
    assert!(exe_status.contains(error_message));
}

pub fn view_balance(
    contract: &ContractAccount<controller::ContractContract>,
    action: ActionType,
    user_account: AccountId,
    dtoken_account: AccountId,
) -> u128 {
    view!(contract.get_entity_by_token(action, user_account, dtoken_account)).unwrap_json()
}

pub fn initialize_utoken(
    root: &UserAccount,
) -> (
    UserAccount,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let uroot = root.create_user("utoken".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot, utoken, u_user) = init_utoken(
        uroot,
        AccountId::new_unchecked("utoken_contract".to_string()),
        String::from("user2_account"),
    );
    call!(
        uroot,
        utoken.new_default_meta(
            uroot.account_id(),
            String::from("Mock Token"),
            String::from("MOCK"),
            U128(10000)
        ),
        deposit = 0
    )
    .assert_success();
    (uroot, utoken, u_user)
}

pub fn initialize_two_utokens(
    root: &UserAccount,
) -> (
    UserAccount,
    UserAccount,
    ContractAccount<test_utoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let uroot1 = root.create_user("utoken1".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot1, utoken1, u_user1) = init_utoken(
        uroot1,
        AccountId::new_unchecked("utoken_contract1".to_string()),
        String::from("user4_account"),
    );
    call!(
        uroot1,
        utoken1.new_default_meta(
            uroot1.account_id(),
            String::from("Mock Token"),
            String::from("MOCK"),
            U128(10000)
        ),
        deposit = 0
    )
    .assert_success();

    let uroot2 = root.create_user("utoken2".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot2, utoken2, u_user2) = init_utoken(
        uroot2,
        AccountId::new_unchecked("utoken_contract2".to_string()),
        String::from("user5_account"),
    );
    call!(
        uroot2,
        utoken2.new_default_meta(
            uroot2.account_id(),
            String::from("Mock Token"),
            String::from("MOCK"),
            U128(10000)
        ),
        deposit = 0
    )
    .assert_success();

    (uroot1, uroot2, utoken1, utoken2, u_user1, u_user2)
}

pub fn initialize_controller(
    root: &UserAccount,
) -> (
    UserAccount,
    ContractAccount<controller::ContractContract>,
    UserAccount,
) {
    let croot = root.create_user(
        "controller".parse().unwrap(),
        1200000000000000000000000000000,
    );
    let (croot, controller, c_user) = init_controller(
        croot,
        AccountId::new_unchecked("controller_contract".to_string()),
    );
    call!(
        croot,
        controller.new(cConfig {
            owner_id: croot.account_id(),
            oracle_account_id: "oracle".parse().unwrap()
        }),
        deposit = 0
    )
    .assert_success();
    (croot, controller, c_user)
}

pub fn initialize_dtoken(
    root: &UserAccount,
    utoken_account: AccountId,
    controller_account: AccountId,
) -> (
    UserAccount,
    ContractAccount<dtoken::ContractContract>,
    UserAccount,
) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken, d_user) = init_dtoken(
        droot,
        AccountId::new_unchecked("dtoken_contract".to_string()),
    );
    call!(
        droot,
        dtoken.new(dConfig {
            initial_exchange_rate: U128(10000),
            underlying_token_id: utoken_account,
            owner_id: droot.account_id(),
            controller_account_id: controller_account,
        }),
        deposit = 0
    )
    .assert_success();
    (droot, dtoken, d_user)
}

pub fn initialize_two_dtokens(
    root: &UserAccount,
    utoken_account1: AccountId,
    utoken_account2: AccountId,
    controller_account: AccountId,
) -> (
    UserAccount,
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<dtoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken1, dtoken2, d_user1, d_user2) = init_two_dtokens(
        droot,
        AccountId::new_unchecked("dtoken_contract1".to_string()),
        AccountId::new_unchecked("dtoken_contract2".to_string()),
    );
    call!(
        droot,
        dtoken1.new(dConfig {
            initial_exchange_rate: U128(10000),
            underlying_token_id: utoken_account1,
            owner_id: droot.account_id(),
            controller_account_id: controller_account.clone(),
        }),
        deposit = 0
    )
    .assert_success();

    call!(
        droot,
        dtoken2.new(dConfig {
            initial_exchange_rate: U128(10000),
            underlying_token_id: utoken_account2,
            owner_id: droot.account_id(),
            controller_account_id: controller_account,
        }),
        deposit = 0
    )
    .assert_success();
    (droot, dtoken1, dtoken2, d_user1, d_user2)
}
