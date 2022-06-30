near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DTOKEN_WASM_BYTES => "../../res/dtoken.wasm",
    PREV_DTOKEN_WASM_BYTES => "../../res/prev_dtoken.wasm",
}

const CURRENT_VERSION: &str = "0.0.1";


use near_sdk::AccountId;
use crate::utils::{new_user, upgrade_dtoken};
use dtoken::InterestRateModel;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount, deploy, to_yocto};
use dtoken::ContractContract as Dtoken;
use dtoken::Config as dConfig;
use general::ratio::Ratio;

fn upgrade_fixture() -> (ContractAccount<dtoken::ContractContract>, UserAccount) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());

    let droot = root.create_user("dtoken".parse().unwrap(), to_yocto("1200000"));
    let contract_id = AccountId::new_unchecked("dtoken_contract".to_string());
    let dtoken = deploy!(
        contract: Dtoken,
        contract_id: contract_id,
        bytes: &PREV_DTOKEN_WASM_BYTES,
        signer_account: droot
    );

    call!(
        dtoken.user_account,
        dtoken.new(dConfig {
            initial_exchange_rate: U128::from(Ratio::one()),
            underlying_token_id: "utoken".parse().unwrap(),
            owner_id: droot.account_id,
            controller_account_id: "controller".parse().unwrap(),
            interest_rate_model: InterestRateModel::default()
        }),
        deposit = 0
    )
        .assert_success();


    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(1000)),
        0,
        100000000000000
    );

    let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 1000);

    let version: String = view!(dtoken.get_version()).unwrap_json();
    assert_eq!(version, CURRENT_VERSION);

    (dtoken, user)
}


#[test]
fn test_upgrade() {
    let (dtoken, user) = upgrade_fixture();

    // ViewResult is an error: "wasm execution failed with error: FunctionCallError(MethodResolveError(MethodNotFound))"
    // let mock_field_check: u64 = view!(dtoken.view_mock_field(user.account_id())).unwrap_json();

    upgrade_dtoken(&dtoken, &DTOKEN_WASM_BYTES).assert_success();

    let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 1000);

    let mock_field_check_len: u64 = view!(dtoken.view_mock_field()).unwrap_json();
    assert_eq!(mock_field_check_len, 0);
}
