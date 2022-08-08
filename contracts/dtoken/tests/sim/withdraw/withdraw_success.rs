use crate::utils::{
    add_market, initialize_controller, initialize_dtoken, initialize_utoken, mint_and_reserve,
    mint_tokens, new_user, set_price, supply, view_balance, withdraw,
};
use controller::ActionType::Supply;
use dtoken::InterestRateModel;
use general::Price;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const SUPPLY_AMOUNT: Balance = 100;
const WITHDRAW_AMOUNT: Balance = SUPPLY_AMOUNT / 3;
const START_PRICE: Balance = 10000;

fn withdraw_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let weth = initialize_utoken(&root);
    let controller = initialize_controller(&root);
    let (droot, weth_market) = initialize_dtoken(
        &root,
        weth.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
    );

    mint_and_reserve(&droot, &weth, &weth_market, SUPPLY_AMOUNT);
    mint_tokens(&weth, user.account_id(), U128(SUPPLY_AMOUNT));

    add_market(
        &controller,
        weth.account_id(),
        weth_market.account_id(),
        "weth".to_string(),
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

    supply(&user, &weth, weth_market.account_id(), SUPPLY_AMOUNT).assert_success();

    (weth_market, controller, weth, user)
}

#[test]
fn scenario_partial_withdraw() {
    let (weth_market, controller, weth, user) = withdraw_fixture();

    withdraw(&user, &weth_market, WITHDRAW_AMOUNT).assert_success();
    let user_dtoken_balance: U128 = view!(weth_market.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_dtoken_balance.0, SUPPLY_AMOUNT - WITHDRAW_AMOUNT);

    let user_supply_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), weth_market.account_id());
    assert_eq!(
        user_supply_balance,
        SUPPLY_AMOUNT - WITHDRAW_AMOUNT,
        "Balance should be {}",
        SUPPLY_AMOUNT - WITHDRAW_AMOUNT
    );

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, WITHDRAW_AMOUNT);
}

#[test]
fn scenario_full_withdraw() {
    let (weth_market, controller, weth, user) = withdraw_fixture();

    withdraw(&user, &weth_market, SUPPLY_AMOUNT).assert_success();
    let user_dtoken_balance: U128 = view!(weth_market.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_dtoken_balance.0, 0);

    let user_supply_balance: u128 =
        view_balance(&controller, Supply, user.account_id(), weth_market.account_id());
    assert_eq!(user_supply_balance, 0, "Balance should be {}", 0);

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, SUPPLY_AMOUNT);
}
