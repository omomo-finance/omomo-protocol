use crate::utils::{
    add_market, initialize_controller, initialize_dtoken, initialize_utoken, new_user, view_balance, supply,
};
use controller::ActionType::Supply;
use general::Price;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: u128 = 100;

fn supply_borrow_withdraw_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, weth) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let (_droot, dweth) = initialize_dtoken(&root, weth.account_id(), controller.account_id());

    call!(
        weth.user_account,
        weth.mint(dweth.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        weth.user_account,
        weth.mint(user.account_id(), U128(WETH_AMOUNT)),
        0,
        100000000000000
    );

    add_market(
        &controller,
        weth.account_id(),
        dweth.account_id(),
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

    supply(&user, &weth, dweth.account_id(), WETH_AMOUNT).assert_success();

    (dweth, controller, weth, user)
}

#[test]
fn scenario_withdraw() {
    let (dweth, controller, weth, user) = supply_borrow_withdraw_fixture();

    call!(user, dweth.withdraw(U128(WETH_AMOUNT)), deposit = 0).assert_success();

    let user_supply_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dweth.account_id());
    assert_eq!(
        user_supply_balance,
        WETH_AMOUNT / 2,
        "Balance should be {}",
        WETH_AMOUNT / 2
    );

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, WETH_AMOUNT / 2);
}
