use near_sdk_sim::{call, view, init_simulator, ContractAccount, UserAccount};
use crate::utils::{initialize_controller, initialize_dtoken, initialize_utoken, assert_failure, view_balance};
use near_sdk::json_types::U128;
use general::Price;
use controller::ActionType::Supply;

fn withdraw_with_no_supply_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) =
        initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    );

    (dtoken, d_user)
}

fn withdraw_more_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) =
        initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        d_user,
        dtoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        controller.increase_supplies(d_user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        utoken.ft_transfer(
            dtoken.account_id(),
            U128(20),
            Some("Supply with token_amount 20".to_string())
        ),
        1,
        100000000000000
    );

    let user_balance: u128 =
        view_balance(&controller, Supply, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, d_user)
}

fn withdraw_less_same_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) =
        initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        d_user,
        dtoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        controller.increase_supplies(d_user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        utoken.ft_transfer(
            dtoken.account_id(),
            U128(20),
            Some("Supply with token_amount 20".to_string())
        ),
        1,
        100000000000000
    );

    let user_balance: u128 =
        view_balance(&controller, Supply, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, d_user, root)
}

fn supply_borrow_withdraw_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) =
        initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(300)),
        0,
        100000000000000
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken.account_id(),
            &Price {
                ticker_id: "weth".to_string(),
                value: U128(20),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    (dtoken, controller, utoken, d_user)
}

fn withdraw_error_transfer_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) =
        initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        d_user,
        dtoken.mint(d_user.account_id(), U128(3)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        dtoken.mint(d_user.account_id(), U128(7)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        dtoken.mint(d_user.account_id(), U128(10)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        controller.increase_supplies(d_user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        d_user,
        utoken.ft_transfer(
            dtoken.account_id(),
            U128(10),
            Some("Supply with token_amount 10".to_string())
        ),
        1,
        100000000000000
    );

    let user_balance: u128 =
        view_balance(&controller, Supply, d_user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, utoken, d_user)
}

#[test]
fn scenario_withdraw_with_no_supply() {
    let (dtoken, user) = withdraw_with_no_supply_fixture();

    let result = call!(user, dtoken.withdraw(U128(20)), deposit = 0);

    assert_failure(
        result,
        "Cannot calculate utilization rate as denominator is equal 0",
    );
}

#[test]
fn scenario_withdraw_more() {
    let (dtoken, controller, user) = withdraw_more_fixture();

    call!(user, dtoken.withdraw(U128(30)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");
}

#[test]
fn scenario_withdraw_less_same() {
    let (dtoken, controller, user, root) = withdraw_less_same_fixture();

    call!(user, dtoken.withdraw(U128(10)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 10, "Balance should be 10");

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    // Withdraw same
    call!(user, dtoken.withdraw(U128(10)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Balance should be 0");
}

#[test]
fn scenario_supply_borrow_withdraw() {
    let (dtoken, controller, utoken, user) = supply_borrow_withdraw_fixture();

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

    call!(user, dtoken.borrow(U128(5)), deposit = 0).assert_success();

    call!(user, dtoken.withdraw(U128(10)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 19, "Balance should be 19");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        114.to_string(),
        "After withdraw balance should be 114"
    );
}

#[test]
fn scenario_withdraw_error_transfer() {
    let (dtoken, controller, _utoken, user) = withdraw_error_transfer_fixture();

    call!(user, dtoken.withdraw(U128(10)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");
}
