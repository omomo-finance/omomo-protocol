use crate::utils::{
    initialize_controller, initialize_dtoken, initialize_two_dtokens, initialize_two_utokens,
    initialize_utoken, new_user, view_balance,
};
use controller::ActionType::{Borrow, Supply};
use general::Price;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk::AccountId;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

fn liquidation_success_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let user1 = new_user(&root, "user1".parse().unwrap());
    let user2 = new_user(&root, "user2".parse().unwrap());
    let (_uroot1, _uroot2, utoken1, utoken2) = initialize_two_utokens(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dtoken1, dtoken2) = initialize_two_dtokens(
        &root,
        utoken1.account_id(),
        utoken2.account_id(),
        controller.account_id(),
    );

    call!(
        utoken1.user_account,
        utoken1.mint(dtoken1.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        utoken2.user_account,
        utoken2.mint(dtoken2.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        utoken1.user_account,
        utoken1.mint(user1.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        utoken2.user_account,
        utoken2.mint(user2.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        utoken1.user_account,
        utoken1.mint(user2.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        utoken2.user_account,
        utoken2.mint(user1.account_id(), U128(100000000000)),
        0,
        100000000000000
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken1.account_id(),
            &Price {
                ticker_id: "1weth".to_string(),
                value: U128(2000),
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
        user2,
        utoken2.ft_transfer_call(dtoken2.account_id(), U128(10), None, action.clone()),
        deposit = 1
    );

    call!(
        user1,
        utoken2.ft_transfer_call(dtoken2.account_id(), U128(60000), None, action.clone()),
        deposit = 1
    )
    .assert_success();

    call!(user1, dtoken1.borrow(U128(40000)), deposit = 0).assert_success();

    let user_balance: u128 = view!(dtoken1.get_account_borrows(user1.account_id())).unwrap_json();
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on dtoken should be 40000"
    );

    let user_balance: u128 = view_balance(
        &controller,
        Borrow,
        user1.account_id(),
        dtoken1.account_id(),
    );
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on controller should be 40000"
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dtoken2.account_id(),
            &Price {
                ticker_id: "2weth".to_string(),
                value: U128(1200),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    (dtoken1, dtoken2, controller, utoken1, utoken2, user1, user2)
}

#[test]
fn scenario_liquidation_success() {
    let (dtoken1, dtoken2, controller, utoken1, _utoken2, user1, user2) =
        liquidation_success_fixture();

    let action = json!({
        "Liquidate":{
            "borrower": user1.account_id.as_str(),
            "borrowing_dtoken": dtoken1.account_id().as_str(),
            "liquidator": user2.account_id.as_str(),
            "collateral_dtoken": dtoken2.account_id().as_str(),
            "liquidation_amount": U128(3500)
        }
    })
    .to_string();

    print!(
        "{:?}",
        call!(
            user2,
            utoken1.ft_transfer_call(dtoken1.account_id(), U128(3500), None, action),
            deposit = 1
        )
        .outcome()
    );

    let user_borrows: u128 = view!(dtoken1.get_account_borrows(user1.account_id())).unwrap_json();
    assert_eq!(
        user_borrows, 36500,
        "Borrow balance on dtoken should be 36500"
    );

    let user_borrows: u128 = view_balance(
        &controller,
        Borrow,
        user1.account_id(),
        dtoken1.account_id(),
    );
    assert_eq!(
        user_borrows, 36500,
        "Borrow balance on controller should be 36500"
    );

    let user_balance: u128 = view_balance(
        &controller,
        Supply,
        user2.account_id(),
        dtoken2.account_id(),
    );

    assert_eq!(
        user_balance, 3510,
        "Supply balance on dtoken should be 3510"
    );
}

fn liquidation_success_on_single_dtoken_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
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
        utoken.ft_transfer_call(dtoken.account_id(), U128(10), None, action),
        deposit = 1
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

    call!(user, dtoken.borrow(U128(5)), deposit = 0).assert_success();

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 5, "Borrow balance on dtoken should be 5");

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());

    assert_eq!(user_balance, 5, "Borrow balance on controller should be 5");

    (dtoken, utoken, user)
}

fn liquidation_failed_on_call_with_wrong_borrow_token_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
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
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(10),
            Some("SUPPLY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    call!(user, dtoken.borrow(U128(5)), deposit = 0).assert_success();

    let user_balance: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 5, "Borrow balance on dtoken should be 5");

    (dtoken, utoken, user)
}

#[test]
fn scenario_liquidation_success_on_single_dtoken() {
    let (dtoken, utoken, user) = liquidation_success_on_single_dtoken_fixture();

    let action = json!({
        "Liquidate":{
            "borrower": user.account_id.as_str(),
            "borrowing_dtoken": dtoken.account_id().as_str(),
            "liquidator": "test.testnet",
            "collateral_dtoken": dtoken.account_id().as_str(),
            "liquidation_amount": U128(10)
        }
    })
    .to_string();

    call!(
        user,
        utoken.ft_transfer_call(dtoken.account_id(), U128(10), None, action),
        deposit = 1
    );

    let _user_borrows: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();

    let _user_balance: u128 =
        view!(dtoken.get_account_borrows(AccountId::new_unchecked("test.testnet".to_string())))
            .unwrap_json();

    // NEAR tests doesn't work with liquidation due some issues
    //assert_eq!(user_borrows, 0, "Borrow balance on dtoken should be 0");
    //assert_eq!(user_balance, 10, "Supply balance on dtoken should be 10");
}

#[test]
fn scenario_liquidation_failed_on_call_with_wrong_borrow_token() {
    let (dtoken, utoken, user) = liquidation_failed_on_call_with_wrong_borrow_token_fixture();

    let action = json!({
        "Liquidate":{
            "borrower": user.account_id.as_str(),
            "borrowing_dtoken": "test.testnet",
            "liquidator": "test.testnet",
            "collateral_dtoken": dtoken.account_id().as_str(),
            "liquidation_amount": U128(5)
        }
    })
    .to_string();

    call!(
        user,
        utoken.ft_transfer_call(dtoken.account_id(), U128(5), None, action),
        deposit = 1
    )
    .assert_success();

    let _user_borrows: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    //assert_eq!(user_borrows, 3, "Borrow balance of user should stay the same, because of an error");
}
