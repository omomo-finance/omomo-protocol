near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    CONTROLLER_WASM_BYTES => "../../res/controller.wasm",
    CONTROLLER_PREV_WASM_BYTES => "../../res/controller_prev.wasm",
}

const CURRENT_VERSION: &str = "0.0.1";

use controller::ContractContract as Controller;
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount, deploy, call, to_yocto};
use near_sdk::AccountId;
use controller::Config as cConfig;



fn upgrade_fixture() -> (UserAccount,
                         ContractAccount<controller::ContractContract>)
{
    let root = init_simulator(None);

    let croot = root.create_user("controller".parse().unwrap(), to_yocto("1200000"));

    let controller = deploy!(
        contract: Controller,
        contract_id: AccountId::new_unchecked("controller_contract".to_string()),
        bytes: &CONTROLLER_PREV_WASM_BYTES,
        signer_account: croot
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

    (root, controller)
}

#[test]
fn test_upgrade() {
    const NEXT_VERSION: &str = "0.0.2";

    let( root, controller) = upgrade_fixture();

    root.call(
        controller.user_account.account_id.clone(),
        "upgrade",
        &CONTROLLER_PREV_WASM_BYTES,
        near_sdk_sim::DEFAULT_GAS,
        0,
    )
        .assert_success();

    // upgrade_controller(&controller, &CONTROLLER_WASM_BYTES).assert_success();
    //
    let version: String = view!(controller.get_version()).unwrap_json();
    assert_eq!(version, NEXT_VERSION);
}


// #[test]
// fn test_upgrade_with_less_fields() {
//     let (user, dtoken) = upgrade_fixture();
//
//
//     const NEXT_VERSION: &str = "0.0.3";
//
//     let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();
//     assert_eq!(reserve.0, 0);
//
//     // New contract without reserve field
//     upgrade(&dtoken, &DTOKEN_WASM_BYTES_PREV).assert_success();
//
//     let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
//     assert_eq!(user_balance.0, 1000);
//
//     // Call view method of removed field
//     // FunctionCallError(MethodResolveError(MethodNotFound))
//     // let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();
//
//     let version: String = view!(dtoken.get_version()).unwrap_json();
//     assert_eq!(version, NEXT_VERSION);
// }

// #[test]
// fn test_upgrade_with_additional_field() {
//     let (user, dtoken) = upgrade_fixture();
//
//
//     const NEXT_VERSION: &str = "0.0.2";
//
//     let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();
//     assert_eq!(reserve.0, 0);
//
//     // New contract with additional Vector field
//     upgrade(&dtoken, &DTOKEN_WASM_BYTES_PREV).assert_success();
//
//     let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
//     assert_eq!(user_balance.0, 1000);
//
//     // Return the value of new empty vector
//     // FunctionCallError(HostError(GuestPanic { panic_msg: \"panicked at 'index out of bounds: the len is 0 but the index is 2'
//     // let reserve: U128 = view!(dtoken.view_total_reserves()).unwrap_json();
//
//     let version: String = view!(dtoken.get_version()).unwrap_json();
//     assert_eq!(version, NEXT_VERSION);
// }