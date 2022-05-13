use crate::utils::{
    add_market, initialize_controller, initialize_three_dtokens_with_custom_interest_rate,
    initialize_three_utokens, new_user, view_balance, supply, borrow,
};
use controller::ActionType::Borrow;
use dtoken::InterestRateModel;
use general::Price;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: u128 = 10;
const WNEAR_AMOUNT: u128 = 10;
const BORROW_AMOUNT: u128 = 21;
const WBTC_START_BALANCE: u128 = 100;

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
        wnear.mint(dwnear.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        uroot2,
        weth.mint(dweth.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        uroot3,
        wbtc.mint(dwbtc.account_id(), U128(WBTC_START_BALANCE)),
        0,
        100000000000000
    );

    call!(
        uroot1,
        weth.mint(user.account_id(), U128(WETH_AMOUNT)),
        0,
        100000000000000
    );

    call!(
        uroot2,
        wnear.mint(user.account_id(), U128(WNEAR_AMOUNT)),
        0,
        100000000000000
    );

    call!(
        uroot3,
        wbtc.mint(user.account_id(), U128(0)),
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

    (dwbtc, controller, wbtc, user)
}

#[test]
fn scenario_borrow_zero_tokens() {
    let (dwbtc, controller, wbtc, user) = borrow_fixture();

    borrow(&user, &dwbtc, BORROW_AMOUNT).assert_success();

    let user_balance: u128 =
        view_balance(&controller, Borrow, user.account_id(), dwbtc.account_id());
    assert_eq!(
        user_balance, 0,
        "User borrow balance on controller should be 0"
    );

    let user_balance: u128 = view!(dwbtc.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "User borrow balance on dtoken should be 0");

    let user_balance: U128 = view!(wbtc.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance, U128(0), "User utoken balance should be 0");

    let dtoken_balance: U128 = view!(wbtc.ft_balance_of(dwbtc.account_id())).unwrap_json();
    assert_eq!(
        dtoken_balance,
        U128(WBTC_START_BALANCE),
        "Dtoken balance on utoken should be {}",
        WBTC_START_BALANCE
    );
}
