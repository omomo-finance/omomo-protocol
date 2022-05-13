use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

use controller::ActionType::Supply;
use general::Price;

use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_two_dtokens,
    initialize_two_utokens, new_user, supply, view_balance,
};

const WNEAR_BALANCE: u128 = 50;
const WETH_BALANCE: u128 = 100;
const SUPPLY_WETH_AMOUNT: u128 = 100;
const SUPPLY_WNEAR_AMOUNT: u128 = 0;

fn supply_success_after_failure_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (uroot1, uroot2, weth, wnear) = initialize_two_utokens(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dweth, dwnear) = initialize_two_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        controller.account_id(),
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
        uroot1,
        weth.mint(user.account_id(), U128(WETH_BALANCE)),
        0,
        100000000000000
    );

    call!(
        uroot2,
        wnear.mint(user.account_id(), U128(WNEAR_BALANCE)),
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
        "weth".to_string(),
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

    (dweth, dwnear, controller, weth, wnear, user)
}

#[test]
fn scenario_supply_success_after_failure() {
    let (dweth, dwnear, controller, weth, wnear, user) = supply_success_after_failure_fixture();

    let result = supply(&user, &wnear, dwnear.account_id(), SUPPLY_WNEAR_AMOUNT);
    assert_failure(result, "The amount should be a positive number");

    supply(&user, &weth, dweth.account_id(), SUPPLY_WETH_AMOUNT).assert_success();

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance, U128(0), "User balance should be 0");

    let user_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dweth.account_id());
    assert_eq!(
        user_balance, SUPPLY_WETH_AMOUNT,
        "Balance on controller should be {}",
        SUPPLY_WETH_AMOUNT
    );
}
