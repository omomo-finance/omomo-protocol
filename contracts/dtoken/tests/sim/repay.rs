use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

use controller::ActionType::Borrow;

use crate::utils::{initialize_controller, initialize_dtoken, initialize_utoken, view_balance};

fn repay_no_borrow_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
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

    (dtoken, utoken, d_user)
}

fn repay_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let (_, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) =
        initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(d_user.account_id(), U128(300)),
        0,
        100000000000000
    );

    (dtoken, controller, utoken, d_user)
}

fn repay_more_than_borrow_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let (_, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) =
        initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(d_user.account_id(), U128(300)),
        0,
        100000000000000
    );

    (dtoken, controller, utoken, d_user)
}

#[test]
fn scenario_repay_no_borrow() {
    let (dtoken, utoken, user) = repay_no_borrow_fixture();

    let action = "\"Repay\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("REPAY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        20.to_string(),
        "As user has never borrowed, transfer shouldn't be done"
    );
}

#[test]
fn scenario_repay() {
    let (dtoken, controller, utoken, user) = repay_fixture();

    let action = "\"Supply\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(30),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    call!(user, dtoken.borrow(U128(5)), deposit = 0).assert_success();

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 5, "Borrow balance on dtoken should be 5");

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 5, "Borrow balance on controller should be 5");

    let action = "\"Repay\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(277),
            Some("REPAY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        23.to_string(),
        "After repay of 277 tokens (borrow was 5), balance should be 23"
    );

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}

#[test]
fn scenario_repay_more_than_borrow() {
    let (dtoken, controller, utoken, user) = repay_more_than_borrow_fixture();

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

    let action = "\"Repay\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(200),
            Some("REPAY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        280.to_string(),
        "As it was borrowed 5 tokens and repayed 13 tokens (rate 1.3333), balance should be 280"
    );

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}
