use crate::utils::{initialize_dtoken, new_user, upgrade};
use dtoken::InterestRateModel;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DTOKEN_WASM_BYTES_0_0_2 => "../../res/dtoken_0_0_2_test.wasm",
    DTOKEN_WASM_BYTES_0_0_3 => "../../res/dtoken_0_0_3_test.wasm",
    DTOKEN_WASM_BYTES_0_0_4 => "../../res/dtoken_0_0_4_test.wasm",
}

const CURRENT_VERSION: &str = "0.0.1";

fn upgrade_fixture() -> (ContractAccount<dtoken::ContractContract>, UserAccount) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let dtoken = initialize_dtoken(
        &root,
        "utoken".parse().unwrap(),
        "controller".parse().unwrap(),
        InterestRateModel::default(),
    );

    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(1000)),
        0,
        100000000000000
    );

    let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 1000);

    let version: String = view!(dtoken.view_version()).unwrap_json();
    assert_eq!(version, CURRENT_VERSION);

    (dtoken, user)
}

#[test]
fn test_upgrade() {
    let (dtoken, user) = upgrade_fixture();
    const NEXT_VERSION: &str = "0.0.4";

    upgrade(&dtoken, &DTOKEN_WASM_BYTES_0_0_4).assert_success();

    let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 1000);

    let version: String = view!(dtoken.view_version()).unwrap_json();
    assert_eq!(version, NEXT_VERSION);
}

#[test]
fn test_upgrade_with_less_fields() {
    let (dtoken, user) = upgrade_fixture();

    const NEXT_VERSION: &str = "0.0.3";

    let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();
    assert_eq!(reserve.0, 0);

    // New contract without reserve field
    upgrade(&dtoken, &DTOKEN_WASM_BYTES_0_0_3).assert_success();

    let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 1000);

    // Call view method of removed field
    // FunctionCallError(MethodResolveError(MethodNotFound))
    // let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();

    let version: String = view!(dtoken.view_version()).unwrap_json();
    assert_eq!(version, NEXT_VERSION);
}

#[test]
fn test_upgrade_with_additional_field() {
    let (dtoken, user) = upgrade_fixture();

    const NEXT_VERSION: &str = "0.0.2";

    let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();
    assert_eq!(reserve.0, 0);

    // New contract with additional Vector field
    upgrade(&dtoken, &DTOKEN_WASM_BYTES_0_0_2).assert_success();

    let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 1000);

    // Return the value of new empty vector
    // FunctionCallError(HostError(GuestPanic { panic_msg: \"panicked at 'index out of bounds: the len is 0 but the index is 2'
    // let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();

    let version: String = view!(dtoken.view_version()).unwrap_json();
    assert_eq!(version, NEXT_VERSION);
}
