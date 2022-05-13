use crate::utils::{
    add_market, assert_failure, initialize_controller,
    initialize_three_dtokens_with_custom_interest_rate, initialize_three_utokens, new_user,
    view_balance, supply,
};
use controller::ActionType::Borrow;
use dtoken::{InterestRateModel, RepayInfo};
use general::Price;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: u128 = 60;
const WNEAR_AMOUNT: u128 = 70;
const WBTC_AMOUNT: u128 = 100;
const WETH_BORROW: u128 = 30;
const WNEAR_BORROW: u128 = 40;
const START_BALANCE: u128 = 100;

fn borrow_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (uroot1, uroot2, uroot3, weth, wnear, wbtc) = initialize_three_utokens(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dweth, dwnear, dwbtc) = initialize_three_dtokens_with_custom_interest_rate(
        &root,
        weth.account_id(),
        wnear.account_id(),
        wbtc.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
        InterestRateModel::default(),
        InterestRateModel::default(),
    );

    call!(
        uroot1,
        wnear.mint(dwnear.account_id(), U128(WNEAR_AMOUNT)),
        0,
        100000000000000
    );

    call!(
        uroot2,
        weth.mint(dweth.account_id(), U128(WETH_AMOUNT)),
        0,
        100000000000000
    );

    call!(
        uroot3,
        wbtc.mint(dwbtc.account_id(), U128(WBTC_AMOUNT)),
        0,
        100000000000000
    );

    call!(
        uroot1,
        weth.mint(user.account_id(), U128(START_BALANCE)),
        0,
        100000000000000
    );

    call!(
        uroot2,
        wnear.mint(user.account_id(), U128(START_BALANCE)),
        0,
        100000000000000
    );

    call!(
        uroot3,
        wbtc.mint(user.account_id(), U128(START_BALANCE)),
        0,
        100000000000000
    );

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

    add_market(
        &controller,
        wbtc.account_id(),
        dwbtc.account_id(),
        "wbtc".to_string(),
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            dweth.account_id(),
            &Price {
                ticker_id: "weth".to_string(),
                value: U128(10000),
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
            dwnear.account_id(),
            &Price {
                ticker_id: "wnear".to_string(),
                value: U128(10000),
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
            dwbtc.account_id(),
            &Price {
                ticker_id: "wbtc".to_string(),
                value: U128(10000),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    )
    .assert_success();

    supply(&user, &weth, dweth.account_id(), WETH_AMOUNT).assert_success();

    supply(&user, &wnear, dwnear.account_id(), WNEAR_AMOUNT).assert_success();

    call!(user, dweth.borrow(U128(WETH_BORROW)), deposit = 0).assert_success();

    call!(user, dwnear.borrow(U128(WNEAR_BORROW)), deposit = 0).assert_success();

    (dwnear, controller, wnear, user)
}

#[test]
fn scenario_repay_success_after_failure() {
    let (dwnear, controller, wnear, user) = borrow_fixture();

    let action = "\"Repay\"".to_string();

    let result = call!(
        user,
        wnear.ft_transfer_call(
            dwnear.account_id(),
            U128(0),
            Some("REPAY".to_string()),
            action.clone()
        ),
        deposit = 1
    );
    assert_failure(result, "The amount should be a positive number");

    let dwnear_balance: String = view!(wnear.ft_balance_of(dwnear.account_id())).unwrap_json();

    let repay_info = call!(
        user,
        dwnear.view_repay_info(user.account_id(), U128(dwnear_balance.parse().unwrap())),
        deposit = 0
    )
    .unwrap_json::<RepayInfo>();

    let repay_amount = u128::from(repay_info.total_amount);

    call!(
        user,
        wnear.ft_transfer_call(
            dwnear.account_id(),
            U128(repay_amount),
            Some("REPAY".to_string()),
            action
        ),
        deposit = 1
    )
    .assert_success();

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance.0,
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW - repay_amount,
        "Repay wasn`t done"
    );

    let user_balance: u128 = view!(dwnear.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dwnear.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}
