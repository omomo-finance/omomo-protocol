use crate::utils::{
    add_market, initialize_controller, initialize_dtoken, initialize_utoken, new_user, supply, borrow,
};
use general::Price;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

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

    add_market(
        &controller,
        utoken.account_id(),
        dtoken.account_id(),
        "weth".to_string(),
    );

    supply(&user, &utoken, dtoken.account_id(), 10).assert_success();

    borrow(&user, &dtoken, 5).assert_success();

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
