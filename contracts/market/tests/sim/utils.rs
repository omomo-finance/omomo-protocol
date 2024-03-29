use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::{AccountId, Balance, Gas};
use near_sdk_sim::{call, deploy, to_yocto, view, ContractAccount, ExecutionResult, UserAccount};
use std::str::FromStr;

use controller::ContractContract as Controller;
use controller::{ActionType, Config as cConfig};
use general::{ratio::Ratio, Price, WBalance};
use market::ContractContract as Dtoken;
use market::InterestRateModel;
use market::{Config as dConfig, RepayInfo};
use mock_token::ContractContract as Utoken;

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DTOKEN_WASM_BYTES => "../target/wasm32-unknown-unknown/release/market.wasm",
    UTOKEN_WASM_BYTES => "../target/wasm32-unknown-unknown/release/mock_token.wasm",
    CONTROLLER_WASM_BYTES => "../target/wasm32-unknown-unknown/release/controller.wasm",
}

pub fn new_user(root: &UserAccount, account_id: AccountId) -> UserAccount {
    root.create_user(
        account_id,
        to_yocto("10000"), // initial balance
    )
}

pub fn init_dtoken(root: &UserAccount, token_id: AccountId) -> ContractAccount<Dtoken> {
    let contract = deploy!(
        contract: Dtoken,
        contract_id: token_id,
        bytes: &DTOKEN_WASM_BYTES,
        signer_account: root
    );

    contract
}

pub fn init_utoken(root: &UserAccount, token_id: AccountId) -> ContractAccount<Utoken> {
    let contract = deploy!(
        contract: Utoken,
        contract_id: token_id,
        bytes: &UTOKEN_WASM_BYTES,
        signer_account: root
    );

    contract
}

pub fn init_controller(root: &UserAccount, token_id: AccountId) -> ContractAccount<Controller> {
    let contract = deploy!(
        contract: Controller,
        contract_id: token_id,
        bytes: &CONTROLLER_WASM_BYTES,
        signer_account: root
    );

    contract
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

fn internal_utoken_initialize(
    account: &UserAccount,
    utoken: &ContractAccount<mock_token::ContractContract>,
    owner: AccountId,
) {
    call!(
        account,
        utoken.new_default_meta(
            owner,
            String::from("Mock Token"),
            String::from("MOCK"),
            U128(10000),
            24
        ),
        deposit = 0
    )
    .assert_success();
}

pub fn initialize_utoken(root: &UserAccount) -> ContractAccount<mock_token::ContractContract> {
    let uroot = root.create_user("utoken".parse().unwrap(), to_yocto("1200000"));
    let utoken = init_utoken(
        &uroot,
        AccountId::new_unchecked("utoken_contract".to_string()),
    );
    internal_utoken_initialize(&utoken.user_account, &utoken, uroot.account_id());
    utoken
}

pub fn initialize_two_utokens(
    root: &UserAccount,
) -> (
    ContractAccount<mock_token::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
) {
    let uroot1 = root.create_user("utoken1".parse().unwrap(), to_yocto("1200000"));
    let utoken1 = init_utoken(
        &uroot1,
        AccountId::new_unchecked("utoken_contract1".to_string()),
    );
    internal_utoken_initialize(&utoken1.user_account, &utoken1, uroot1.account_id());

    let uroot2 = root.create_user("utoken2".parse().unwrap(), to_yocto("1200000"));
    let utoken2 = init_utoken(
        &uroot2,
        AccountId::new_unchecked("utoken_contract2".to_string()),
    );
    internal_utoken_initialize(&utoken2.user_account, &utoken2, uroot2.account_id());

    (utoken1, utoken2)
}

pub fn initialize_three_utokens(
    root: &UserAccount,
) -> (
    ContractAccount<mock_token::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
) {
    let uroot1 = root.create_user("utoken1".parse().unwrap(), to_yocto("1200000"));
    let utoken1 = init_utoken(
        &uroot1,
        AccountId::new_unchecked("utoken_contract1".to_string()),
    );
    internal_utoken_initialize(&utoken1.user_account, &utoken1, uroot1.account_id());

    let uroot2 = root.create_user("utoken2".parse().unwrap(), to_yocto("1200000"));
    let utoken2 = init_utoken(
        &uroot2,
        AccountId::new_unchecked("utoken_contract2".to_string()),
    );
    internal_utoken_initialize(&utoken2.user_account, &utoken2, uroot2.account_id());

    let uroot3 = root.create_user("utoken3".parse().unwrap(), to_yocto("1200000"));
    let utoken3 = init_utoken(
        &uroot3,
        AccountId::new_unchecked("utoken_contract3".to_string()),
    );
    internal_utoken_initialize(&utoken3.user_account, &utoken3, uroot3.account_id());

    (utoken1, utoken2, utoken3)
}

pub fn initialize_controller(root: &UserAccount) -> ContractAccount<controller::ContractContract> {
    let croot = root.create_user("controller".parse().unwrap(), to_yocto("1200000"));
    let controller = init_controller(
        &croot,
        AccountId::new_unchecked("controller_contract".to_string()),
    );
    call!(
        controller.user_account,
        controller.new(cConfig {
            owner_id: croot.account_id(),
            oracle_account_id: "oracle".parse().unwrap()
        }),
        deposit = 0
    )
    .assert_success();
    controller
}

fn internal_dtoken_initialize(
    account: &UserAccount,
    dtoken: &ContractAccount<market::ContractContract>,
    owner: AccountId,
    utoken_account: AccountId,
    controller_account: AccountId,
    model: InterestRateModel,
) {
    call!(
        account,
        dtoken.new(dConfig {
            initial_exchange_rate: U128::from(Ratio::one()),
            underlying_token_id: utoken_account,
            underlying_token_decimals: 24,
            owner_id: owner,
            controller_account_id: controller_account,
            interest_rate_model: model,
            disable_transfer_token: true,
        }),
        deposit = 0
    )
    .assert_success();
}

pub fn initialize_dtoken(
    root: &UserAccount,
    utoken_account: AccountId,
    controller_account: AccountId,
    interest_model: InterestRateModel,
) -> (UserAccount, ContractAccount<market::ContractContract>) {
    let droot = root.create_user("dtoken".parse().unwrap(), to_yocto("1200000"));
    let dtoken = init_dtoken(
        &droot,
        AccountId::new_unchecked("dtoken_contract".to_string()),
    );
    internal_dtoken_initialize(
        &dtoken.user_account,
        &dtoken,
        droot.account_id(),
        utoken_account,
        controller_account,
        interest_model,
    );

    (droot, dtoken)
}

pub fn initialize_two_dtokens(
    root: &UserAccount,
    utoken_account1: AccountId,
    utoken_account2: AccountId,
    controller_account: AccountId,
    interest_model1: InterestRateModel,
    interest_model2: InterestRateModel,
) -> (
    UserAccount,
    ContractAccount<market::ContractContract>,
    ContractAccount<market::ContractContract>,
) {
    let droot = root.create_user("dtoken".parse().unwrap(), to_yocto("1200000"));
    let dtoken1 = init_dtoken(
        &droot,
        AccountId::new_unchecked("dtoken_contract1".to_string()),
    );

    let dtoken2 = init_dtoken(
        &droot,
        AccountId::new_unchecked("dtoken_contract2".to_string()),
    );

    internal_dtoken_initialize(
        &dtoken1.user_account,
        &dtoken1,
        droot.account_id(),
        utoken_account1,
        controller_account.clone(),
        interest_model1,
    );

    internal_dtoken_initialize(
        &dtoken2.user_account,
        &dtoken2,
        droot.account_id(),
        utoken_account2,
        controller_account,
        interest_model2,
    );
    (droot, dtoken1, dtoken2)
}

pub fn initialize_three_dtokens(
    root: &UserAccount,
    utoken_account1: AccountId,
    utoken_account2: AccountId,
    utoken_account3: AccountId,
    controller_account: AccountId,
    interest_model1: InterestRateModel,
    interest_model2: InterestRateModel,
    interest_model3: InterestRateModel,
) -> (
    UserAccount,
    ContractAccount<market::ContractContract>,
    ContractAccount<market::ContractContract>,
    ContractAccount<market::ContractContract>,
) {
    let droot = root.create_user("dtoken".parse().unwrap(), to_yocto("1200000"));
    let dtoken1 = init_dtoken(
        &droot,
        AccountId::new_unchecked("dtoken_contract1".to_string()),
    );

    let dtoken2 = init_dtoken(
        &droot,
        AccountId::new_unchecked("dtoken_contract2".to_string()),
    );
    let dtoken3 = init_dtoken(
        &droot,
        AccountId::new_unchecked("dtoken_contract3".to_string()),
    );

    internal_dtoken_initialize(
        &dtoken1.user_account,
        &dtoken1,
        droot.account_id(),
        utoken_account1,
        controller_account.clone(),
        interest_model1,
    );

    internal_dtoken_initialize(
        &dtoken2.user_account,
        &dtoken2,
        droot.account_id(),
        utoken_account2,
        controller_account.clone(),
        interest_model2,
    );
    internal_dtoken_initialize(
        &dtoken3.user_account,
        &dtoken3,
        droot.account_id(),
        utoken_account3,
        controller_account,
        interest_model3,
    );
    (droot, dtoken1, dtoken2, dtoken3)
}

pub fn add_market(
    controller: &ContractAccount<controller::ContractContract>,
    utoken_id: AccountId,
    dtoken_id: AccountId,
    ticker_id: String,
) {
    let ltv = Ratio::from_str("0.6").unwrap();
    let lth = Ratio::from_str("0.8").unwrap();
    call!(
        controller.user_account,
        controller.add_market(utoken_id, dtoken_id, ticker_id, ltv, lth),
        deposit = 0
    );
}

pub fn mint_tokens(
    utoken: &ContractAccount<mock_token::ContractContract>,
    receiver: AccountId,
    amount: U128,
) {
    call!(
        utoken.user_account,
        utoken.mint(receiver, amount),
        0,
        100000000000000
    )
    .assert_success();
}

pub fn set_price(
    controller: &ContractAccount<controller::ContractContract>,
    dtoken_id: AccountId,
    price: &Price,
) {
    call!(
        controller.user_account,
        controller.upsert_price(dtoken_id, price),
        deposit = 0
    )
    .assert_success();
}

pub fn reserve_storage(
    dtoken_admin: &UserAccount,
    utoken: &ContractAccount<mock_token::ContractContract>,
    dtoken: &ContractAccount<market::ContractContract>,
) {
    call!(
        dtoken_admin,
        utoken.storage_deposit(Some(dtoken.account_id()), None),
        deposit = to_yocto("0.25")
    )
    .assert_success();
}

pub fn mint_and_reserve(
    dtoken_admin: &UserAccount,
    utoken: &ContractAccount<mock_token::ContractContract>,
    dtoken: &ContractAccount<market::ContractContract>,
    amount: Balance,
) {
    mint_tokens(utoken, dtoken_admin.account_id(), U128(amount));
    reserve_storage(dtoken_admin, utoken, dtoken);

    let action = "\"Reserve\"".to_string();
    call!(
        dtoken_admin,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(amount),
            Some("RESERVE".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let underlying_balance: WBalance =
        view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        underlying_balance,
        WBalance::from(amount),
        "Unexpected dtoken balance"
    );

    let total_reserves: WBalance = view!(dtoken.view_total_reserves()).unwrap_json();
    assert_eq!(
        total_reserves,
        WBalance::from(amount),
        "Unexpected total reserves"
    );
}

pub fn supply(
    user: &UserAccount,
    utoken: &ContractAccount<mock_token::ContractContract>,
    dtoken: AccountId,
    amount: Balance,
) -> ExecutionResult {
    let action = "\"Supply\"".to_string();
    call!(
        user,
        utoken.ft_transfer_call(dtoken, U128(amount), Some("SUPPLY".to_string()), action),
        deposit = 1
    )
}

pub fn withdraw(
    user: &UserAccount,
    dtoken: &ContractAccount<market::ContractContract>,
    amount: Balance,
) -> ExecutionResult {
    call!(user, dtoken.withdraw(U128(amount)), deposit = 0)
}

pub fn borrow(
    user: &UserAccount,
    dtoken: &ContractAccount<market::ContractContract>,
    amount: Balance,
) -> ExecutionResult {
    call!(user, dtoken.borrow(U128(amount)), deposit = 0)
}

pub fn repay(
    user: &UserAccount,
    dtoken: AccountId,
    utoken: &ContractAccount<mock_token::ContractContract>,
    amount: Balance,
) -> ExecutionResult {
    let action = "\"Repay\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(dtoken, U128(amount), Some("REPAY".to_string()), action),
        deposit = 1
    )
}

pub fn liquidate(
    borrower: &UserAccount,
    liquidator: &UserAccount,
    borrowing_dtoken: &ContractAccount<market::ContractContract>,
    collateral_dtoken: &ContractAccount<market::ContractContract>,
    borrowing_utoken: &ContractAccount<mock_token::ContractContract>,
    amount: Balance,
) -> ExecutionResult {
    let action = json!({
        "Liquidate":{
            "borrower": borrower.account_id.as_str(),
            "borrowing_dtoken": borrowing_dtoken.account_id().as_str(),
            "collateral_dtoken": collateral_dtoken.account_id().as_str(),
        }
    })
    .to_string();

    call!(
        liquidator,
        borrowing_utoken.ft_transfer_call(
            borrowing_dtoken.account_id(),
            U128::from(amount),
            None,
            action
        ),
        deposit = 1
    )
}

pub fn repay_info(
    user: &UserAccount,
    dtoken: &ContractAccount<market::ContractContract>,
    dtoken_balance: U128,
) -> RepayInfo {
    call!(
        user,
        dtoken.view_repay_info(user.account_id(), dtoken_balance),
        deposit = 0
    )
    .unwrap_json::<RepayInfo>()
}

pub fn upgrade_dtoken(
    dtoken: &ContractAccount<market::ContractContract>,
    contract_bytes: &[u8],
) -> ExecutionResult {
    const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);

    dtoken
        .user_account
        .create_transaction(dtoken.account_id())
        .function_call("upgrade".to_string(), contract_bytes.to_vec(), MAX_GAS.0, 0)
        .submit()
}
