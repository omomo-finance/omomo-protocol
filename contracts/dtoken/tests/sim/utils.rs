use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk_sim::{call, deploy, to_yocto, view, ContractAccount, ExecutionResult, UserAccount};

use controller::ContractContract as Controller;
use controller::{ActionType, Config as cConfig};
use dtoken::Config as dConfig;
use dtoken::ContractContract as Dtoken;
use dtoken::InterestRateModel;
use test_utoken::ContractContract as Utoken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DTOKEN_WASM_BYTES => "../../res/dtoken.wasm",
    UTOKEN_WASM_BYTES => "../../res/test_utoken.wasm",
    CONTROLLER_WASM_BYTES => "../../res/controller.wasm",
}

pub fn new_user(root: &UserAccount, account_id: AccountId) -> UserAccount {
    root.create_user(
        account_id,
        to_yocto("10000"), // initial balance
    )
}

pub fn init_dtoken(
    root: UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Dtoken>) {
    let contract = deploy!(
        contract: Dtoken,
        contract_id: token_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    (root, contract)
}

pub fn init_two_dtokens(
    root: UserAccount,
    token1_id: AccountId,
    token2_id: AccountId,
) -> (
    UserAccount,
    ContractAccount<Dtoken>,
    ContractAccount<Dtoken>,
) {
    let contract1 = deploy!(
        contract: Dtoken,
        contract_id: token1_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    let contract2 = deploy!(
        contract: Dtoken,
        contract_id: token2_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    (root, contract1, contract2)
}

pub fn init_utoken(
    root: UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Utoken>) {
    let contract = deploy!(
        contract: Utoken,
        contract_id: token_id,
        bytes: &UTOKEN_WASM_BYTES,
        signer_account: root
    );

    (root, contract)
}

pub fn init_controller(
    root: UserAccount,
    token_id: AccountId,
) -> (UserAccount, ContractAccount<Controller>) {
    let contract = deploy!(
        contract: Controller,
        contract_id: token_id,
        bytes: &CONTROLLER_WASM_BYTES,
        signer_account: root
    );

    (root, contract)
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
) -> (UserAccount, ContractAccount<test_utoken::ContractContract>) {
    let uroot = root.create_user("utoken".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot, utoken) = init_utoken(
        uroot,
        AccountId::new_unchecked("utoken_contract".to_string()),
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
    (uroot, utoken)
}

pub fn initialize_two_utokens(
    root: &UserAccount,
) -> (
    UserAccount,
    UserAccount,
    ContractAccount<test_utoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
) {
    let uroot1 = root.create_user("utoken1".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot1, utoken1) = init_utoken(
        uroot1,
        AccountId::new_unchecked("utoken_contract1".to_string()),
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
    let (uroot2, utoken2) = init_utoken(
        uroot2,
        AccountId::new_unchecked("utoken_contract2".to_string()),
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

    (uroot1, uroot2, utoken1, utoken2)
}

pub fn initialize_controller(
    root: &UserAccount,
) -> (UserAccount, ContractAccount<controller::ContractContract>) {
    let croot = root.create_user(
        "controller".parse().unwrap(),
        1200000000000000000000000000000,
    );
    let (croot, controller) = init_controller(
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
    (croot, controller)
}

pub fn initialize_dtoken(
    root: &UserAccount,
    utoken_account: AccountId,
    controller_account: AccountId,
) -> (UserAccount, ContractAccount<dtoken::ContractContract>) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken) = init_dtoken(
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
            interest_rate_model: InterestRateModel::default()
        }),
        deposit = 0
    )
    .assert_success();
    (droot, dtoken)
}

pub fn initialize_dtoken_with_custom_interest_rate(
    root: &UserAccount,
    utoken_account: AccountId,
    controller_account: AccountId,
    interest_model: InterestRateModel,
) -> (UserAccount, ContractAccount<dtoken::ContractContract>) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken) = init_dtoken(
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
            interest_rate_model: interest_model
        }),
        deposit = 0
    )
    .assert_success();
    (droot, dtoken)
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
) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken1, dtoken2) = init_two_dtokens(
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
            interest_rate_model: InterestRateModel::default()
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
            interest_rate_model: InterestRateModel::default()
        }),
        deposit = 0
    )
    .assert_success();
    (droot, dtoken1, dtoken2)
}

pub fn initialize_two_dtokens_with_custom_interest_rate(
    root: &UserAccount,
    utoken_account1: AccountId,
    utoken_account2: AccountId,
    controller_account: AccountId,
    interest_model1: InterestRateModel,
    interest_model2: InterestRateModel,
) -> (
    UserAccount,
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<dtoken::ContractContract>,
) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken1, dtoken2) = init_two_dtokens(
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
            interest_rate_model: interest_model1
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
            interest_rate_model: interest_model2
        }),
        deposit = 0
    )
    .assert_success();
    (droot, dtoken1, dtoken2)
}

pub fn add_market(
    controller: &ContractAccount<controller::ContractContract>,
    utoken_id: AccountId,
    dtoken_id: AccountId,
    ticker_id: String,
) {
    call!(
        controller.user_account,
        controller.add_market(utoken_id, dtoken_id, ticker_id),
        deposit = 0
    );
}
