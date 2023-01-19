use crate::utils::{
    add_market, borrow, initialize_controller, initialize_two_dtokens, initialize_two_utokens,
    mint_and_reserve, mint_tokens, new_user, set_price, supply, view_balance, withdraw,
};
use controller::ActionType::Supply;
use general::Price;
use market::InterestRateModel;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WBTC_AMOUNT: Balance = 0;
const BORROW_AMOUNT: Balance = 50;
const START_BALANCE: Balance = 100;
const START_PRICE: Balance = 50000;
const RESERVE_AMOUNT: Balance = 100;

fn withdraw_fixture() -> (
    ContractAccount<market::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (weth, wbtc) = initialize_two_utokens(&root);
    let controller = initialize_controller(&root);
    let interest_model = InterestRateModel {
        kink: U128(0),
        multiplier_per_block: U128(0),
        base_rate_per_block: U128(0),
        jump_multiplier_per_block: U128(0),
        reserve_factor: U128(0),
    };
    let (droot, weth_market, dwbtc) = initialize_two_dtokens(
        &root,
        weth.account_id(),
        wbtc.account_id(),
        controller.account_id(),
        interest_model.clone(),
        interest_model,
    );

    let mint_amount = U128(START_BALANCE);
    mint_and_reserve(&droot, &weth, &weth_market, RESERVE_AMOUNT);
    mint_and_reserve(&droot, &wbtc, &dwbtc, RESERVE_AMOUNT);
    mint_tokens(&weth, user.account_id(), mint_amount);
    mint_tokens(&wbtc, user.account_id(), U128(WBTC_AMOUNT));

    add_market(
        &controller,
        weth.account_id(),
        weth_market.account_id(),
        "weth".to_string(),
    );

    add_market(
        &controller,
        wbtc.account_id(),
        dwbtc.account_id(),
        "wbtc".to_string(),
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

    set_price(
        &controller,
        dwbtc.account_id(),
        &Price {
            ticker_id: "wbtc".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &weth, weth_market.account_id(), START_BALANCE).assert_success();

    borrow(&user, &dwbtc, BORROW_AMOUNT).assert_success();

    (weth_market, controller, weth, user)
}

#[test]
fn scenario_withdraw_more_after_borrow() {
    let (weth_market, controller, weth, user) = withdraw_fixture();

    withdraw(&user, &weth_market, START_BALANCE).assert_success();

    let user_supply_balance: u128 = view_balance(
        &controller,
        Supply,
        user.account_id(),
        weth_market.account_id(),
    );
    assert_eq!(
        user_supply_balance, START_BALANCE,
        "Balance should be {}",
        START_BALANCE
    );

    let user_balance: U128 = view!(weth.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 0);
}
