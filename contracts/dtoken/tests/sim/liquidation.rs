use crate::utils::{
    add_market, initialize_controller, initialize_dtoken, initialize_two_dtokens,
    initialize_two_utokens, initialize_utoken, mint_tokens, new_user, set_price, view_balance,
};
use controller::ActionType::{Borrow, Supply};
use general::Price;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
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
    let borrower = new_user(&root, "borrower".parse().unwrap());
    let liquidator = new_user(&root, "liquidator".parse().unwrap());
    let (_uroot1, _uroot2, weth, wnear) = initialize_two_utokens(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dweth, dwnear) = initialize_two_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        controller.account_id(),
    );

    let mint_amount = U128(100000000000);
    mint_tokens(&weth, dweth.account_id(), mint_amount.clone());
    mint_tokens(&wnear, dwnear.account_id(), mint_amount.clone());
    mint_tokens(&weth, borrower.account_id(), mint_amount.clone());
    mint_tokens(&wnear, liquidator.account_id(), mint_amount.clone());
    mint_tokens(&weth, liquidator.account_id(), mint_amount.clone());
    mint_tokens(&wnear, borrower.account_id(), mint_amount.clone());
    mint_tokens(&wnear, borrower.account_id(), mint_amount.clone());

    add_market(
        &controller,
        weth.account_id(),
        dweth.account_id(),
        "weth".to_string(),
    );

    add_market(
        &controller,
        wnear.account_id(),
        dwnear.account_id(),
        "wnear".to_string(),
    );

    set_price(
        &controller,
        dweth.account_id(),
        &Price {
            ticker_id: "weth".to_string(),
            value: U128(2000),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    set_price(
        &controller,
        dwnear.account_id(),
        &Price {
            ticker_id: "wnear".to_string(),
            value: U128(2000),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    add_market(
        &controller,
        utoken1.account_id(),
        dtoken1.account_id(),
        "1weth".to_string(),
    );

    let action = "\"Supply\"".to_string();

    call!(
        borrower,
        wnear.ft_transfer_call(dwnear.account_id(), U128(60000), None, action),
        deposit = 1
    )
    .assert_success();

    call!(borrower, dweth.borrow(U128(40000)), deposit = 0).assert_success();

    let user_balance: u128 = view!(dweth.get_account_borrows(borrower.account_id())).unwrap_json();
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on dtoken should be 40000"
    );

    let user_balance: u128 = view_balance(
        &controller,
        Borrow,
        borrower.account_id(),
        dweth.account_id(),
    );
    assert_eq!(
        user_balance, 40000,
        "Borrow balance on controller should be 40000"
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dwnear.account_id(),
            &Price {
                ticker_id: "wnear".to_string(),
                value: U128(1200),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    (dweth, dwnear, controller, weth, wnear, borrower, liquidator)
}

#[test]
fn scenario_liquidation_success() {
    let (dweth, dwnear, controller, weth, _wnear, borrower, liquidator) =
        liquidation_success_fixture();

    let amount = U128(3500);
    let action = json!({
        "Liquidate":{
            "borrower": borrower.account_id.as_str(),
            "borrowing_dtoken": dweth.account_id().as_str(),
            "collateral_dtoken": dwnear.account_id().as_str(),
        }
    })
    .to_string();

    call!(
        liquidator,
        weth.ft_transfer_call(dweth.account_id(), amount.clone(), None, action),
        deposit = 1
    )
    .assert_success();

    let user_borrows: u128 = view!(dweth.get_account_borrows(borrower.account_id())).unwrap_json();

    assert_eq!(
        user_borrows, 36500,
        "Borrow balance on dtoken should be 36500"
    );

    let user_borrows: u128 = view_balance(
        &controller,
        Borrow,
        borrower.account_id(),
        dweth.account_id(),
    );
    assert_eq!(
        user_borrows, 36500,
        "Borrow balance on controller should be 36500"
    );

    let user_balance: u128 = view_balance(
        &controller,
        Supply,
        liquidator.account_id(),
        dwnear.account_id(),
    );

    assert_eq!(
        user_balance, 6125,
        "Supply balance on dtoken should be 6125"
    );

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
fn scenario_liquidation_failed_on_call_with_wrong_borrow_token() {
    let (dtoken, utoken, user) = liquidation_failed_on_call_with_wrong_borrow_token_fixture();

    let action = json!({
        "Liquidate":{
            "borrower": user.account_id.as_str(),
            "borrowing_dtoken": "test.testnet",
            "collateral_dtoken": dtoken.account_id().as_str(),
        }
    })
    .to_string();

    call!(
        user,
        utoken.ft_transfer_call(dtoken.account_id(), U128(5), None, action),
        deposit = 1
    )
    .assert_success();

    let user_borrows: u128 = view!(dtoken.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_borrows, 5,
        "Borrow balance of user should stay the same, because of an error"
    );
}

#[test]
fn scenario_liquidation_failed_on_too_much_for_liquidation() {
    let (dweth, dwnear, controller, weth, _wnear, borrower, liquidator) =
        liquidation_success_fixture();

    let amount = U128(70000);
    let action = json!({
        "Liquidate":{
            "borrower": borrower.account_id.as_str(),
            "borrowing_dtoken": dweth.account_id().as_str(),
            "collateral_dtoken": dwnear.account_id().as_str(),
        }
    })
    .to_string();

    call!(
        liquidator,
        weth.ft_transfer_call(dweth.account_id(), amount, None, action),
        deposit = 1
    );

    let user_borrows: u128 = view!(dweth.get_account_borrows(borrower.account_id())).unwrap_json();
    assert_eq!(
        user_borrows, 40000,
        "Borrow balance on dtoken should be 40000"
    );

    let user_borrows: u128 = view_balance(
        &controller,
        Borrow,
        borrower.account_id(),
        dweth.account_id(),
    );
    assert_eq!(
        user_borrows, 40000,
        "Borrow balance on controller should be 40000"
    );

    let user_balance: u128 = view_balance(
        &controller,
        Supply,
        liquidator.account_id(),
        dwnear.account_id(),
    );

    assert_eq!(user_balance, 0, "Supply balance on dtoken should be 0");
}
