// IMPORTANT! Update previous version after migration
near_sdk_sim::lazy_static_include::lazy_static_include_bytes! {
    DTOKEN_CURRENT_WASM_BYTES => "../target/wasm32-unknown-unknown/release/market.wasm",
    DTOKEN_PREVIOUS_WASM_BYTES => "tests/sim/contracts/dtoken_v1.wasm",
}

const PREVIOUS_VERSION: &str = "0.0.1";

use crate::utils::{
    add_market, initialize_controller, initialize_utoken, new_user, set_price, upgrade_dtoken,
    view_balance,
};
use controller::ActionType::Supply;
use market::Config as dConfig;
use market::ContractContract as Dtoken;
use market::InterestRateModel;
use general::ratio::Ratio;
use general::Price;
use near_sdk::json_types::U128;
use near_sdk::{AccountId, Balance};
use near_sdk_sim::{call, deploy, init_simulator, to_yocto, view, ContractAccount, UserAccount};

fn upgrade_fixture() -> (
    ContractAccount<market::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let controller = initialize_controller(&root);
    let droot = root.create_user("dtoken".parse().unwrap(), to_yocto("1200000"));
    let contract_id = AccountId::new_unchecked("dtoken_contract".to_string());

    let utoken = initialize_utoken(&root);

    let dtoken = deploy!(
        contract: Dtoken,
        contract_id: contract_id,
        bytes: &DTOKEN_PREVIOUS_WASM_BYTES,
        signer_account: droot
    );

    call!(
        dtoken.user_account,
        dtoken.new(dConfig {
            initial_exchange_rate: U128::from(Ratio::one()),
            underlying_token_id: utoken.account_id(),
            owner_id: droot.account_id,
            controller_account_id: controller.account_id(),
            interest_rate_model: InterestRateModel::default(),
            disable_transfer_token: true
        }),
        deposit = 0
    )
    .assert_success();

    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(10000)),
        0,
        100000000000000
    );

    let user_balance: U128 = view!(dtoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 10000);

    let version: String = view!(dtoken.get_version()).unwrap_json();
    assert_eq!(version, PREVIOUS_VERSION);

    add_market(
        &controller,
        utoken.account_id(),
        dtoken.account_id(),
        "weth".to_string(),
    );

    set_price(
        &controller,
        dtoken.account_id(),
        &Price {
            ticker_id: "weth".to_string(),
            value: U128(10000),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    (dtoken, controller, utoken, user)
}

#[test]
fn test_upgrade_without_field() {
    let (dtoken, controller, _, user) = upgrade_fixture();

    let old_total_supplies = view!(dtoken.view_total_supplies()).unwrap_json::<U128>();
    let old_total_borrows = view!(dtoken.view_total_borrows()).unwrap_json::<U128>();
    let old_total_reserves = view!(dtoken.view_total_reserves()).unwrap_json::<U128>();
    let old_user_borrows =
        view!(dtoken.get_account_borrows(user.account_id())).unwrap_json::<Balance>();
    let old_user_supplies: Balance =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());

    upgrade_dtoken(&dtoken, &DTOKEN_CURRENT_WASM_BYTES).assert_success();

    assert_eq!(
        view!(dtoken.get_version()).unwrap_json::<String>(),
        env!("CARGO_PKG_VERSION").to_string()
    );

    // there are no such field so Err occurred
    // no method named `view_mock_field` found for struct `market::ContractContract` in the current scope
    // let mock_field_after_upgrade_check = view!(dtoken.view_mock_field(user.account_id())).unwrap_json();

    assert_eq!(
        old_total_supplies,
        view!(dtoken.view_total_supplies()).unwrap_json::<U128>()
    );
    assert_eq!(
        old_total_borrows,
        view!(dtoken.view_total_borrows()).unwrap_json::<U128>()
    );
    assert_eq!(
        old_total_reserves,
        view!(dtoken.view_total_reserves()).unwrap_json::<U128>()
    );
    assert_eq!(
        old_user_borrows,
        view!(dtoken.get_account_borrows(user.account_id())).unwrap_json::<Balance>()
    );
    assert_eq!(
        old_user_supplies,
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id())
    );
}
