use market::InterestRateModel;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

use controller::ActionType::Supply;
use general::Price;

use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_two_dtokens,
    initialize_two_utokens, mint_and_reserve, mint_tokens, new_user, set_price, supply,
    view_balance, withdraw,
};

const RESERVE_AMOUNT: Balance = 100;
const WNEAR_BALANCE: Balance = 50;
const WETH_BALANCE: Balance = 100;
const SUPPLY_WETH_AMOUNT: Balance = 100;
const START_PRICE: Balance = 10000;

fn withdraw_success_after_failure_fixture() -> (
    ContractAccount<market::ContractContract>,
    ContractAccount<market::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (weth, wnear) = initialize_two_utokens(&root);
    let controller = initialize_controller(&root);
    let (droot, weth_market, wnear_market) = initialize_two_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
        InterestRateModel::default(),
    );

    mint_and_reserve(&droot, &weth, &weth_market, RESERVE_AMOUNT);
    mint_and_reserve(&droot, &wnear, &wnear_market, RESERVE_AMOUNT);

    mint_tokens(&wnear, user.account_id(), U128(WNEAR_BALANCE));
    mint_tokens(&weth, user.account_id(), U128(WETH_BALANCE));

    add_market(
        &controller,
        weth.account_id(),
        weth_market.account_id(),
        "weth".to_string(),
    );

    add_market(
        &controller,
        wnear.account_id(),
        wnear_market.account_id(),
        "wnear".to_string(),
    );

    set_price(
        &controller,
        wnear_market.account_id(),
        &Price {
            ticker_id: "wnear".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    set_price(
        &controller,
        weth_market.account_id(),
        &Price {
            ticker_id: "weth".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &weth, weth_market.account_id(), SUPPLY_WETH_AMOUNT).assert_success();

    (weth_market, wnear_market, controller, weth, wnear, user)
}

#[test]
fn scenario_withdraw_success_after_failure() {
    let (weth_market, wnear_market, controller, weth, _wnear, user) =
        withdraw_success_after_failure_fixture();

    let result = withdraw(&user, &wnear_market, 0);
    assert_failure(result, "Amount should be a positive number");

    withdraw(&user, &weth_market, SUPPLY_WETH_AMOUNT).assert_success();

    let user_supply_balance: Balance = view_balance(
        &controller,
        Supply,
        user.account_id(),
        weth_market.account_id(),
    );
    assert_eq!(user_supply_balance, 0, "Balance should be {}", 0);

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, SUPPLY_WETH_AMOUNT);
}
