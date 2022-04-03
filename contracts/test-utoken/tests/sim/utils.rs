use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{init_simulator, to_yocto, UserAccount, DEFAULT_GAS, STORAGE_AMOUNT};

const CONTRACT_ID: &str = "token_contract";

// Load in contract bytes at runtime
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
   TOKEN_WASM_BYTES => "../../res/test_utoken.wasm",
}

pub fn register_user(user: &near_sdk_sim::UserAccount) {
    user.call(
        CONTRACT_ID.to_string(),
        "storage_deposit",
        &json!({
            "account_id": user.valid_account_id()
        })
        .to_string()
        .into_bytes(),
        near_sdk_sim::DEFAULT_GAS / 2,
        near_sdk::env::storage_byte_cost() * 125, // attached deposit
    )
    .assert_success();
}

pub fn init_no_macros(initial_balance: u128) -> (UserAccount, UserAccount, UserAccount) {
    let root = init_simulator(None);

    let token_contract = root.deploy(&TOKEN_WASM_BYTES, CONTRACT_ID.into(), STORAGE_AMOUNT);

    token_contract
        .call(
            CONTRACT_ID.into(),
            "new_default_meta",
            &json!({
                "owner_id": root.valid_account_id(),
                "name": "utoken_name",
                "symbol": "utoken_symbol",
                "total_supply": U128::from(initial_balance),
            })
            .to_string()
            .into_bytes(),
            DEFAULT_GAS / 2,
            0, // attached deposit
        )
        .assert_success();

    let alice = root.create_user("alice".to_string(), to_yocto("100"));
    register_user(&alice);

    (root, token_contract, alice)
}
