use crate::utils::{initialize_controller, initialize_dtoken, initialize_utoken, view_balance};
use controller::ActionType::{Borrow, Supply};
use general::Price;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

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
        utoken.mint(d_user.account_id(), U128(800)),
        0,
        100000000000000
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken.account_id(),
            &Price {
                ticker_id: "weth".to_string(),
                value: U128(20000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    let action = "\"Supply\"".to_string();

    call!(
        d_user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(15),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    // after supplying
    let user_balance: String = view!(utoken.ft_balance_of(d_user.account_id())).unwrap_json();
    assert_eq!(user_balance, 785.to_string(), "User balance should be 385");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        115.to_string(),
        "Dtoken balance should be 115"
    );

    let user_balance: u128 = view_balance(
        &controller,
        Supply,
        d_user.account_id(),
        dtoken.account_id(),
    );
    assert_eq!(user_balance, 15, "supplied assets should be 15");

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    call!(d_user, dtoken.borrow(U128(5)), deposit = 0).assert_success();

    // after borrowing
    let user_balance: u128 = view_balance(
        &controller,
        Borrow,
        d_user.account_id(),
        dtoken.account_id(),
    );
    assert_eq!(user_balance, 5, "User balance should be 5");

    let user_balance_borrows: u128 =
        view!(dtoken.get_account_borrows(d_user.account_id())).unwrap_json();
    assert_eq!(user_balance_borrows, 5, "User borrowed balance should be 5");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        110.to_string(),
        "Dtoken balance should be 50"
    );
    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    (dtoken, controller, utoken, d_user)
}

fn repay_more_than_borrow_fixture() -> (
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
        utoken.mint(d_user.account_id(), U128(800)),
        0,
        100000000000000
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken.account_id(),
            &Price {
                ticker_id: "weth".to_string(),
                value: U128(20000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    let action = "\"Supply\"".to_string();

    call!(
        d_user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(15),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    // after supplying
    let user_balance: String = view!(utoken.ft_balance_of(d_user.account_id())).unwrap_json();
    assert_eq!(user_balance, 785.to_string(), "User balance should be 285");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        115.to_string(),
        "Dtoken balance should be 115"
    );

    let user_balance: u128 = view_balance(
        &controller,
        Supply,
        d_user.account_id(),
        dtoken.account_id(),
    );
    assert_eq!(user_balance, 15, "supplied assets should be 15");

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    call!(d_user, dtoken.borrow(U128(5)), deposit = 0).assert_success();

    // after borrowing
    let user_balance: u128 = view_balance(
        &controller,
        Borrow,
        d_user.account_id(),
        dtoken.account_id(),
    );
    assert_eq!(user_balance, 5, "User balance should be 5");

    let user_balance_borrows: u128 =
        view!(dtoken.get_account_borrows(d_user.account_id())).unwrap_json();
    assert_eq!(user_balance_borrows, 5, "User borrowed balance should be 5");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        110.to_string(),
        "Dtoken balance should be 50"
    );

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

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

    let action = "\"Repay\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(594),
            Some("REPAY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        196.to_string(),
        "After repay of 277 tokens (borrow was 5), balance should be 196"
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

    let action = "\"Repay\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(790),
            Some("REPAY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        196.to_string(),
        "As it was borrowed 10 tokens and repayed 13 tokens (rate 1.3333), balance should be 7"
    );

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}
