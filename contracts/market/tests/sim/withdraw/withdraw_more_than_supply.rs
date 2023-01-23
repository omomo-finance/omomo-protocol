use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken, initialize_utoken,
    mint_and_reserve, mint_tokens, new_user, set_price, supply, view_balance, withdraw,
};
use controller::ActionType::Supply;
use general::ratio::Ratio;
use general::Price;
use market::InterestRateModel;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WNEAR_AMOUNT: Balance = 50;
const WITHDRAW_AMOUNT: Balance = 100;
const START_PRICE: Balance = 10000;

fn withdraw_more_than_supply_fixture() -> (
    ContractAccount<market::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<mock_token::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let wnear = initialize_utoken(&root);
    let controller = initialize_controller(&root);
    let interest_model = InterestRateModel {
        kink: U128::from(Ratio::zero()),
        multiplier_per_block: U128::from(Ratio::zero()),
        base_rate_per_block: U128::from(Ratio::zero()),
        jump_multiplier_per_block: U128::from(Ratio::zero()),
        reserve_factor: U128::from(Ratio::zero()),
    };
    let (droot, wnear_market) = initialize_dtoken(
        &root,
        wnear.account_id(),
        controller.account_id(),
        interest_model,
    );

    mint_and_reserve(&droot, &wnear, &wnear_market, WNEAR_AMOUNT);
    mint_tokens(&wnear, user.account_id(), U128(WNEAR_AMOUNT));

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

    supply(&user, &wnear, wnear_market.account_id(), WNEAR_AMOUNT).assert_success();

    (wnear_market, controller, wnear, user)
}

#[test]
fn scenario_withdraw_more_than_supply() {
    let (wnear_market, controller, wnear, user) = withdraw_more_than_supply_fixture();

    let result = withdraw(&user, &wnear_market, WITHDRAW_AMOUNT);

    assert_failure(
        result,
        "The account doesn't have enough digital tokens to do withdraw",
    );

    let user_supply_balance: Balance = view_balance(
        &controller,
        Supply,
        user.account_id(),
        wnear_market.account_id(),
    );
    assert_eq!(
        user_supply_balance, WNEAR_AMOUNT,
        "Balance should be {}",
        WNEAR_AMOUNT
    );

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 0);
}
