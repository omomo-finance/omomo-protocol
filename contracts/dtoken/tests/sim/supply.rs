use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

use controller::ActionType::Supply;
use general::wbalance::WBalance;
use general::Price;

use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken, initialize_utoken,
    new_user, view_balance,
};

fn supply_error_command_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    (dtoken, utoken, user)
}

fn supply_zero_tokens_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    (dtoken, utoken, user)
}

fn supply_error_contract_fixture() -> (ContractAccount<dtoken::ContractContract>, UserAccount) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    (dtoken, user)
}

fn supply_not_enough_balance_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    (dtoken, utoken, user)
}

fn supply_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(user.account_id(), U128(300)),
        0,
        100000000000000
    );

    add_market(
        &controller,
        utoken.account_id(),
        dtoken.account_id(),
        "weth".to_string(),
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken.account_id(),
            &Price {
                ticker_id: "weth".to_string(),
                value: WBalance::from(20),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    (dtoken, controller, utoken, user)
}

#[test]
fn scenario_supply_error_command() {
    let (dtoken, utoken, user) = supply_error_command_fixture();
    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPL".to_string()),
            "SUPPL".to_string()
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        20.to_string(),
        "As to mistake in command, transfer shouldn't be done"
    );
}

#[test]
fn scenario_supply_zero_tokens() {
    let (dtoken, utoken, user) = supply_zero_tokens_fixture();

    let action = "\"Supply\"".to_string();

    let result = call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(0),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    );
    assert_failure(result, "The amount should be a positive number");
}

#[test]
fn scenario_supply_error_contract() {
    let (dtoken, user) = supply_error_contract_fixture();

    let action = "\"Supply\"".to_string();

    let result = call!(
        user,
        dtoken.ft_on_transfer(user.account_id(), U128(20), action),
        deposit = 0
    );

    assert_failure(result, "The call should come from token account");
}

#[test]
fn scenario_supply_not_enough_balance() {
    let (dtoken, utoken, user) = supply_not_enough_balance_fixture();

    let action = "\"Supply\"".to_string();

    let result = call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(50),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    );
    assert_failure(result, "The account doesn't have enough balance");
}

#[test]
fn scenario_supply() {
    let (dtoken, controller, utoken, user) = supply_fixture();

    let action = "\"Supply\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 280.to_string(), "User balance should be 280");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        120.to_string(),
        "Dtoken balance should be 120"
    );

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance on controller should be 20");
}
