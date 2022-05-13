use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken_with_custom_interest_rate,
    initialize_utoken, new_user, view_balance,
};
use controller::ActionType::Supply;
use dtoken::InterestRateModel;
use general::Price;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: u128 = 100;

fn withdraw_fail_due_to_zero_denominator_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (_uroot, weth) = initialize_utoken(&root);
    let (_croot, controller) = initialize_controller(&root);
    let interest_model = InterestRateModel {
        kink: U128(0),
        multiplier_per_block: U128(0),
        base_rate_per_block: U128(0),
        jump_multiplier_per_block: U128(0),
        reserve_factor: U128(0),
        rewards_config: Vec::new(),
    };
    let (_droot, dweth) = initialize_dtoken_with_custom_interest_rate(
        &root,
        weth.account_id(),
        controller.account_id(),
        interest_model,
    );

    call!(
        weth.user_account,
        weth.mint(dweth.account_id(), U128(0)),
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

    (dweth, controller, weth, user)
}

#[test]
fn scenario_withdraw_fail_due_to_zero_denominator() {
    let (dweth, controller, weth, user) = withdraw_fail_due_to_zero_denominator_fixture();

    let result = call!(user, dweth.withdraw(U128(WETH_AMOUNT)), deposit = 0);
    assert_failure(
        result,
        "Cannot calculate utilization rate as denominator is equal 0",
    );

    let user_supply_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), dweth.account_id());
    assert_eq!(user_supply_balance, 0, "Balance should be 0");

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, WETH_AMOUNT);
}
