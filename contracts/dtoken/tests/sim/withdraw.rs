use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken,
    initialize_two_dtokens_with_custom_interest_rate, initialize_two_utokens, initialize_utoken,
    new_user, view_balance,
};
use controller::AccountData;
use controller::ActionType::{Borrow, Supply};
use dtoken::InterestRateModel;
use general::ratio::RATIO_DECIMALS;
use general::Price;
use near_sdk::json_types::U128;
use near_sdk::test_utils::test_env::bob;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

fn withdraw_with_no_supply_fixture() -> (ContractAccount<dtoken::ContractContract>, UserAccount) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    add_market(
        &controller,
        utoken.account_id(),
        dtoken.account_id(),
        "weth".to_string(),
    );

    (dtoken, user)
}

fn withdraw_more_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        user,
        controller.increase_supplies(user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    add_market(
        &controller,
        utoken.account_id(),
        dtoken.account_id(),
        "weth".to_string(),
    );

    call!(
        user,
        utoken.ft_transfer(
            dtoken.account_id(),
            U128(20),
            Some("Supply with token_amount 20".to_string())
        ),
        1,
        100000000000000
    );

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, user)
}

fn withdraw_less_same_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        user,
        controller.increase_supplies(user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    add_market(
        &controller,
        utoken.account_id(),
        dtoken.account_id(),
        "weth".to_string(),
    );

    call!(
        user,
        utoken.ft_transfer(
            dtoken.account_id(),
            U128(20),
            Some("Supply with token_amount 20".to_string())
        ),
        1,
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

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, user, root)
}

fn supply_borrow_withdraw_fixture() -> (
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

fn withdraw_error_transfer_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, utoken) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        utoken.user_account,
        utoken.mint(dtoken.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        utoken.user_account,
        utoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(3)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(7)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        dtoken.user_account,
        dtoken.mint(user.account_id(), U128(10)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        user,
        controller.increase_supplies(user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    )
    .assert_success();

    call!(
        user,
        utoken.ft_transfer(
            dtoken.account_id(),
            U128(10),
            Some("Supply with token_amount 10".to_string())
        ),
        1,
        100000000000000
    );

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, utoken, user)
}

fn withdraw_with_borrow_on_another_dtoken_fixure() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot1, _uroot2, utoken1, utoken2) = initialize_two_utokens(&root);
    let (_croot, controller) = initialize_controller(&root);

    let interest_model = InterestRateModel {
        kink: U128(0),
        multiplier_per_block: U128(0),
        base_rate_per_block: U128(0),
        jump_multiplier_per_block: U128(0),
        reserve_factor: U128(0),
        rewards_config: Vec::new(),
    };
    let (_droot, dtoken1, dtoken2) = initialize_two_dtokens_with_custom_interest_rate(
        &root,
        utoken1.account_id(),
        utoken2.account_id(),
        controller.account_id(),
        interest_model.clone(),
        interest_model,
    );

    call!(
        utoken2.user_account,
        utoken2.mint(dtoken2.account_id(), U128(10000)),
        0,
        100000000000000
    );

    call!(
        utoken1.user_account,
        utoken1.mint(dtoken1.account_id(), U128(1000000)),
        0,
        100000000000000
    );

    call!(
        utoken1.user_account,
        utoken1.mint(user.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        utoken2.user_account,
        utoken2.mint(user.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    add_market(
        &controller,
        utoken1.account_id(),
        dtoken1.account_id(),
        "1weth".to_string(),
    );

    add_market(
        &controller,
        utoken2.account_id(),
        dtoken2.account_id(),
        "2weth".to_string(),
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken1.account_id(),
            &Price {
                ticker_id: "1weth".to_string(),
                value: U128(1000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken2.account_id(),
            &Price {
                ticker_id: "2weth".to_string(),
                value: U128(2000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    let action = "\"Supply\"".to_string();

    call!(
        user,
        utoken2.ft_transfer_call(dtoken2.account_id(), U128(60000), None, action),
        deposit = 1
    )
    .assert_success();

    call!(user, dtoken1.borrow(U128(40000)), deposit = 0).assert_success();

    let user_balance: U128 = view!(utoken1.ft_balance_of(user.account_id())).unwrap_json::<U128>();
    assert_eq!(
        user_balance,
        U128(100000040000),
        "User utoken balance should be 100000040000"
    );

    let user_balance: u128 = view!(dtoken1.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on dtoken should be 40000"
    );

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken1.account_id());
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on controller should be 40000"
    );

    (dtoken2, controller, utoken2, user)
}

fn withdraw_failed_due_to_low_health_factor_fixure() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot1, _uroot2, utoken1, utoken2) = initialize_two_utokens(&root);
    let (_croot, controller) = initialize_controller(&root);

    let interest_model = InterestRateModel {
        kink: U128(0),
        multiplier_per_block: U128(0),
        base_rate_per_block: U128(0),
        jump_multiplier_per_block: U128(0),
        reserve_factor: U128(0),
        rewards_config: Vec::new(),
    };
    let (_droot, dtoken1, dtoken2) = initialize_two_dtokens_with_custom_interest_rate(
        &root,
        utoken1.account_id(),
        utoken2.account_id(),
        controller.account_id(),
        interest_model.clone(),
        interest_model,
    );

    call!(
        utoken2.user_account,
        utoken2.mint(dtoken2.account_id(), U128(10000)),
        0,
        100000000000000
    );

    call!(
        utoken1.user_account,
        utoken1.mint(dtoken1.account_id(), U128(1000000)),
        0,
        100000000000000
    );

    call!(
        utoken1.user_account,
        utoken1.mint(user.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        utoken2.user_account,
        utoken2.mint(user.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    add_market(
        &controller,
        utoken1.account_id(),
        dtoken1.account_id(),
        "1weth".to_string(),
    );

    add_market(
        &controller,
        utoken2.account_id(),
        dtoken2.account_id(),
        "2weth".to_string(),
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken1.account_id(),
            &Price {
                ticker_id: "1weth".to_string(),
                value: U128(1000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken2.account_id(),
            &Price {
                ticker_id: "2weth".to_string(),
                value: U128(2000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    let action = "\"Supply\"".to_string();

    call!(
        user,
        utoken2.ft_transfer_call(dtoken2.account_id(), U128(60000), None, action),
        deposit = 1
    )
    .assert_success();

    call!(user, dtoken1.borrow(U128(40000)), deposit = 0).assert_success();

    let user_balance: U128 = view!(utoken1.ft_balance_of(user.account_id())).unwrap_json::<U128>();
    assert_eq!(
        user_balance,
        U128(100000040000),
        "User utoken balance should be 100000040000"
    );

    let user_balance: u128 = view!(dtoken1.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on dtoken should be 40000"
    );

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken1.account_id());
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on controller should be 40000"
    );

    (dtoken2, controller, utoken2, user)
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

    let dtoken_balance_before: U128 =
        view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    let exchange_rate: u128 = view!(dtoken.view_exchange_rate(dtoken_balance_before)).unwrap_json();
    let dtoken_amount: u128 = 10;
    let token_amount: u128 = dtoken_amount * RATIO_DECIMALS.0 / exchange_rate;

    call!(user, dtoken.withdraw(U128(dtoken_amount)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 19, "Balance should be 19");

    let dtoken_balance: U128 = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert!(dtoken_balance.0 < dtoken_balance_before.0 - token_amount);
}

#[test]
fn scenario_withdraw_error_transfer() {
    let (dtoken, controller, _utoken, user) = withdraw_error_transfer_fixture();

    call!(user, dtoken.withdraw(U128(10)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");
}

#[test]
fn scenario_view_accounts() {
    // TODO remove in future if we will make sure it works properly
    let (dtoken, controller, utoken, user) = supply_borrow_withdraw_fixture();

    let mut accounts = vec![user.account_id.clone()];

    accounts.push(bob());

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken.account_id(),
            &Price {
                ticker_id: "wnear".to_string(),
                value: U128(20),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

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

    let vec_acc_data: Vec<AccountData> =
        call!(controller.user_account, controller.view_accounts(accounts)).unwrap_json();

    let user_supply_on_dtoken = *vec_acc_data[0]
        .user_profile
        .account_supplies
        .get(&dtoken.account_id())
        .unwrap();
    let user_borrow_on_dtoken = *vec_acc_data[0]
        .user_profile
        .account_borrows
        .get(&dtoken.account_id())
        .unwrap();

    // borrow on dtoken should be 5 & supply 20
    assert_eq!(U128(20), user_supply_on_dtoken);
    assert_eq!(U128(5), user_borrow_on_dtoken);
}

#[test]
fn scenario_withdraw_with_borrow_on_another_dtoken() {
    let (dtoken2, controller, utoken2, user) = withdraw_with_borrow_on_another_dtoken_fixure();

    let dtoken_balance: U128 =
        view!(utoken2.ft_balance_of(dtoken2.account_id())).unwrap_json::<U128>();
    let exchange_rate: u128 = view!(dtoken2.view_exchange_rate(dtoken_balance)).unwrap_json();
    let dtoken_amount: u128 = 5000;
    let token_amount: u128 = dtoken_amount * RATIO_DECIMALS.0 / exchange_rate;

    let res_potential: u128 = view!(controller.get_potential_health_factor(
        user.account_id(),
        dtoken2.account_id(),
        U128(token_amount),
        Supply
    ))
    .unwrap_json();
    assert_eq!(res_potential, 27857);

    call!(user, dtoken2.withdraw(U128(dtoken_amount)), deposit = 0).assert_success();

    let res: u128 = view!(controller.get_health_factor(user.account_id())).unwrap_json();
    assert_eq!(res, 27857);

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken2.account_id());
    assert_eq!(user_balance, 55715, "Balance should be 55715");

    let dtoken_balance: U128 =
        view!(utoken2.ft_balance_of(user.account_id())).unwrap_json::<U128>();
    assert_eq!(
        dtoken_balance,
        U128(99999944285),
        "After withdraw balance should be 99999944285"
    );
}

#[test]
fn scenario_withdraw_failed_due_to_low_health_factor() {
    let (dtoken2, controller, utoken2, user) = withdraw_failed_due_to_low_health_factor_fixure();

    let dtoken_balance: U128 =
        view!(utoken2.ft_balance_of(dtoken2.account_id())).unwrap_json::<U128>();
    let exchange_rate: u128 = view!(dtoken2.view_exchange_rate(dtoken_balance)).unwrap_json();
    let dtoken_amount: u128 = 50000;
    let token_amount: u128 = dtoken_amount * RATIO_DECIMALS.0 / exchange_rate;

    let res_potential: u128 = view!(controller.get_potential_health_factor(
        user.account_id(),
        dtoken2.account_id(),
        U128(token_amount),
        Supply
    ))
    .unwrap_json();
    assert_eq!(res_potential, 8572);

    call!(user, dtoken2.withdraw(U128(dtoken_amount)), deposit = 0).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dtoken2.account_id());
    assert_eq!(
        user_balance, 60000,
        "Supply balance on controller should stay the same"
    );

    let dtoken_balance: U128 =
        view!(utoken2.ft_balance_of(user.account_id())).unwrap_json::<U128>();
    assert_eq!(
        dtoken_balance,
        U128(99999940000),
        "User balance on utoken should stay the same"
    );
}
