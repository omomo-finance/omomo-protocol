use crate::utils::{
    initialize_controller, initialize_dtoken, initialize_utoken, new_user, view_balance,
};
use controller::ActionType::{Borrow, Supply};
use general::Price;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

fn borrow_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(user.account_id(), U128(300)),
        0,
        100000000000000
    );

    let action = "\"Supply\"".to_string();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(15),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

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

    (dtoken, controller, utoken, user)
}

fn borrow_more_than_on_dtoken_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(user.account_id(), U128(30)),
        0,
        100000000000000
    );

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

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    (dtoken, controller, utoken, user)
}

fn supply_borrow_repay_withdraw_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(user.account_id(), U128(800)),
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

    (dtoken, controller, utoken, user, root)
}

#[test]
fn scenario_borrow() {
    let (dtoken, controller, utoken, user) = borrow_fixture();

    call!(user, dtoken.borrow(U128(10)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(
        user_balance, 10,
        "User borrow balance on controller should be 10"
    );

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance, 10,
        "User borrow balance on dtoken should be 10"
    );

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        295.to_string(),
        "User utoken balance should be 295"
    );

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        105.to_string(),
        "Dtoken balance on utoken should be 105"
    );
}

#[test]
fn scenario_borrow_more_than_on_dtoken() {
    let (dtoken, controller, utoken, user) = borrow_more_than_on_dtoken_fixture();

    call!(user, dtoken.borrow(U128(60)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "User borrow balance on dtoken should be 0");

    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        0.to_string(),
        "User balance on utoken should be 0"
    );

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        50.to_string(),
        "Dtoken balance on utoken should be 50"
    );
}

#[test]
fn scenario_supply_borrow_repay_withdraw() {
    // initial dtoken_balance = 100; user_balance = 800;
    let (dtoken, controller, utoken, user, root) = supply_borrow_repay_withdraw_fixture();

    let action = "\"Supply\"".to_string();

    call!(
        user,
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
    let user_balance: String = view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 785.to_string(), "User balance should be 285");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        115.to_string(),
        "Dtoken balance should be 115"
    );

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 15, "supplied assets should be 15");

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    call!(user, dtoken.borrow(U128(5)), deposit = 0).assert_success();

    // after borrowing
    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 5, "User balance should be 5");

    let user_balance_borrows: u128 =
        view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance_borrows, 5, "User borrowed balance should be 5");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        110.to_string(),
        "Dtoken balance should be 50"
    );

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

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
    );

    // after repaying
    let user_borrowed_balance_after_repay: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(
        user_borrowed_balance_after_repay, 0,
        "User borrowed balance should be 0"
    );

    let user_balance_after_repay: String =
        view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance_after_repay,
        196.to_string(),
        "User balance should be 196"
    );

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    call!(user, dtoken.withdraw(U128(10)), deposit = 0).assert_success();

    // after withdrawing
    let user_balance_after_withdraw: String =
        view!(utoken.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance_after_withdraw,
        196.to_string(),
        "User balance should be 196"
    );

    let user_supply_balance_after_withdraw =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(
        user_supply_balance_after_withdraw, 15,
        "supply balance should be 15"
    );

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        704.to_string(),
        "After withdraw balance should be 167"
    );
}
