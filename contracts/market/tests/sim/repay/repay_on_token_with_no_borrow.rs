use crate::utils::{
    add_market, borrow, initialize_controller, initialize_two_dtokens, initialize_two_utokens,
    mint_tokens, new_user, repay, set_price, supply,
};
use market::{InterestRateModel, WRatio};
use general::Price;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: Balance = 60;
const WNEAR_AMOUNT: Balance = 70;
const WETH_BORROW: Balance = 30;
const START_BALANCE: Balance = 100;
const START_PRICE: Balance = 10000;

fn repay_fixture() -> (
    ContractAccount<market::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (weth, wnear) = initialize_two_utokens(&root);
    let controller = initialize_controller(&root);
    let interest_rate_model = InterestRateModel {
        kink: WRatio::from(0),
        base_rate_per_block: WRatio::from(0),
        multiplier_per_block: WRatio::from(0),
        jump_multiplier_per_block: WRatio::from(0),
        reserve_factor: WRatio::from(0),
    };
    let (_, weth_market, wnear_market) = initialize_two_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        controller.account_id(),
        interest_rate_model.clone(),
        interest_rate_model,
    );

    let mint_amount = U128(START_BALANCE);
    mint_tokens(&weth, weth_market.account_id(), U128(WETH_AMOUNT));
    mint_tokens(&wnear, wnear_market.account_id(), U128(WNEAR_AMOUNT));
    mint_tokens(&weth, user.account_id(), mint_amount);
    mint_tokens(&wnear, user.account_id(), mint_amount);

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

    supply(&user, &weth, weth_market.account_id(), WETH_AMOUNT).assert_success();

    borrow(&user, &weth_market, WETH_BORROW).assert_success();

    (wnear_market, controller, wnear, user)
}

#[test]
fn scenario_repay() {
    let (wnear_market, _controller, wnear, user) = repay_fixture();

    repay(&user, wnear_market.account_id(), &wnear, WETH_BORROW).assert_success();

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, START_BALANCE,);
}
