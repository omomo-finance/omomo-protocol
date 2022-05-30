// use crate::utils::{initialize_dtoken, new_user};
// use dtoken::InterestRateModel;
// use near_sdk::{json_types::U128, Gas};
// use near_sdk_sim::{call, init_simulator, view};

// near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
//     DTOKEN_WASM_BYTES_0_0_1 => "../../res/dtoken.wasm",
//     DTOKEN_WASM_BYTES_0_0_2 => "../../res/dtoken_0_0_2_test.wasm",
// }

// const CURRENT_VERSION: &str = "0.0.1";
// const NEXT_VERSION: &str = "0.0.2";
// const MAX_GAS: Gas = Gas(Gas::ONE_TERA.0 * 300);

// #[test]
// fn test_upgrade() {
//     let root = init_simulator(None);

//     let user = new_user(&root, "user".parse().unwrap());
//     let dtoken = initialize_dtoken(
//         &root,
//         "utoken".parse().unwrap(),
//         "controller".parse().unwrap(),
//         InterestRateModel::default(),
//     );

//     call!(
//         dtoken.user_account,
//         dtoken.mint(user.account_id(), U128(1000)),
//         0,
//         100000000000000
//     );

//     let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
//     assert_eq!(user_balance.0, 1000);

//     let version: String = view!(dtoken.view_version()).unwrap_json();
//     assert_eq!(version, CURRENT_VERSION);

//     // New contract with additional field
//     dtoken
//         .user_account
//         .create_transaction(dtoken.account_id())
//         .function_call(
//             "upgrade".to_string(),
//             DTOKEN_WASM_BYTES_0_0_2.to_vec(),
//             MAX_GAS.0,
//             0,
//         )
//         .submit()
//         .assert_success();

//     let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
//     assert_eq!(user_balance.0, 1000);

//     let version: String = view!(dtoken.view_version()).unwrap_json();
//     assert_eq!(version, NEXT_VERSION);
// }
