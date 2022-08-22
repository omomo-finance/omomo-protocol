use crate::utils::{
    add_market, simple_borrow, initialize_controller, initialize_three_dtokens, initialize_three_utokens,
    mint_and_reserve, mint_tokens, new_user, repay, repay_info, set_price, supply, view_balance,
};
use controller::ActionType::Borrow;
use dtoken::{InterestRateModel, WRatio};
use general::{ratio::Ratio, Price};
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WETH_AMOUNT: Balance = 60;
const WNEAR_AMOUNT: Balance = 70;
const WBTC_AMOUNT: Balance = 100;
const WETH_BORROW: Balance = 30;
const WNEAR_BORROW: Balance = 40;
const START_BALANCE: Balance = 200;
const START_PRICE: Balance = 10000;
const FIRST_PART_TO_REPAY: Balance = 10;

fn repay_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let (weth, wnear, wbtc) = initialize_three_utokens(&root);
    let controller = initialize_controller(&root);
    let interest_rate_model = InterestRateModel {
        kink: WRatio::from(0),
        base_rate_per_block: WRatio::from(0),
        multiplier_per_block: WRatio::from(0),
        jump_multiplier_per_block: WRatio::from(0),
        reserve_factor: WRatio::from(0),
    };
    let (droot, weth_market, wnear_market, dwbtc) = initialize_three_dtokens(
        &root,
        weth.account_id(),
        wnear.account_id(),
        wbtc.account_id(),
        controller.account_id(),
        interest_rate_model.clone(),
        interest_rate_model.clone(),
        interest_rate_model,
    );

    mint_and_reserve(&droot, &weth, &weth_market, WETH_AMOUNT);
    mint_and_reserve(&droot, &wnear, &wnear_market, WNEAR_AMOUNT);
    mint_and_reserve(&droot, &wbtc, &dwbtc, WBTC_AMOUNT);

    let mint_amount = U128(START_BALANCE);
    mint_tokens(&weth, user.account_id(), mint_amount);
    mint_tokens(&wnear, user.account_id(), mint_amount);
    mint_tokens(&wbtc, user.account_id(), mint_amount);

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
        dwbtc.account_id(),
        &Price {
            ticker_id: "wbtc".to_string(),
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &weth, weth_market.account_id(), WETH_AMOUNT).assert_success();
    supply(&user, &wnear, wnear_market.account_id(), WNEAR_AMOUNT).assert_success();
    let wnear_market_balance: U128 =
        view!(wnear.ft_balance_of(wnear_market.account_id())).unwrap_json();
    let exchange_rate: Ratio =
        view!(wnear_market.view_exchange_rate(wnear_market_balance)).unwrap_json();
    assert_eq!(exchange_rate, Ratio::one(), "xrate should be 1.0");

    simple_borrow(&user, &weth_market, WETH_BORROW).assert_success();

    simple_borrow(&user, &wnear_market, WNEAR_BORROW).assert_success();

    (wnear_market, controller, wnear, user)
}

#[test]
fn scenario_repay_by_parts() {
    let (wnear_market, controller, wnear, user) = repay_fixture();

    let wnear_market_balance: U128 =
        view!(wnear.ft_balance_of(wnear_market.account_id())).unwrap_json();
    let repay_info = repay_info(&user, &wnear_market, wnear_market_balance);
    let repay_amount = Balance::from(repay_info.total_amount);

    repay(
        &user,
        wnear_market.account_id(),
        &wnear,
        FIRST_PART_TO_REPAY,
    )
    .assert_success();

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance.0,
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW - FIRST_PART_TO_REPAY,
        "Repay was partially done, user balance should be {}",
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW - FIRST_PART_TO_REPAY
    );

    let user_balance: Balance =
        view!(wnear_market.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance,
        WNEAR_BORROW - FIRST_PART_TO_REPAY,
        "Borrow balance on dtoken should be {}",
        WNEAR_BORROW - FIRST_PART_TO_REPAY
    );

    let user_balance: Balance = view_balance(
        &controller,
        Borrow,
        user.account_id(),
        wnear_market.account_id(),
    );
    assert_eq!(
        user_balance,
        WNEAR_BORROW - FIRST_PART_TO_REPAY,
        "Borrow balance on controller should be {}",
        WNEAR_BORROW - FIRST_PART_TO_REPAY
    );

    repay(
        &user,
        wnear_market.account_id(),
        &wnear,
        repay_amount - FIRST_PART_TO_REPAY,
    )
    .assert_success();

    let balance_after_first_repay =
        START_BALANCE - WNEAR_AMOUNT + WNEAR_BORROW - FIRST_PART_TO_REPAY;

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(
        user_balance.0,
        balance_after_first_repay - (repay_amount - FIRST_PART_TO_REPAY),
        "Repay was fully done, user balance should be {}",
        balance_after_first_repay - (repay_amount - FIRST_PART_TO_REPAY)
    );

    let user_balance: Balance =
        view!(wnear_market.get_account_borrows(user.account_id())).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: Balance = view_balance(
        &controller,
        Borrow,
        user.account_id(),
        wnear_market.account_id(),
    );
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
    let wnear_market_balance: U128 =
        view!(wnear.ft_balance_of(wnear_market.account_id())).unwrap_json();
    let exchange_rate: Ratio =
        view!(wnear_market.view_exchange_rate(wnear_market_balance)).unwrap_json();
    assert_eq!(exchange_rate, Ratio::one(), "xrate should be 1.0");
}
