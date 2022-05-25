use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken, initialize_utoken,
    mint_tokens, new_user, set_price, supply, view_balance, withdraw,
};
use controller::ActionType::Supply;
use dtoken::InterestRateModel;
use general::wbalance::WBalance;
use general::Price;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

const WNEAR_AMOUNT: Balance = 99;
const WITHDRAW_AMOUNT: Balance = 100;
const START_PRICE: Balance = 10000;

fn withdraw_more_than_supply_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let wnear = initialize_utoken(&root);
    let controller = initialize_controller(&root);
    let interest_model = InterestRateModel {
        kink: U128(0),
        multiplier_per_block: U128(0),
        base_rate_per_block: U128(0),
        jump_multiplier_per_block: U128(0),
        reserve_factor: U128(0),
    };
    let dwnear = initialize_dtoken(
        &root,
        wnear.account_id(),
        controller.account_id(),
        interest_model,
    );

    mint_tokens(&wnear, dwnear.account_id(), U128(100));
    mint_tokens(&wnear, user.account_id(), U128(WNEAR_AMOUNT));

    add_market(
        &controller,
        wnear.account_id(),
        dwnear.account_id(),
        "wnear".to_string(),
    );

    set_price(
        &controller,
        dwnear.account_id(),
        &Price {
            ticker_id: "wnear".to_string(),
            value: WBalance::from(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    supply(&user, &wnear, dwnear.account_id(), WNEAR_AMOUNT).assert_success();

    (dwnear, controller, wnear, user)
}

#[test]
fn scenario_withdraw_more_than_supply() {
    let (dwnear, controller, wnear, user) = withdraw_more_than_supply_fixture();

    let result = withdraw(&user, &dwnear, WITHDRAW_AMOUNT);
    assert_failure(
        result,
        "The account doesn't have enough digital tokens to do withdraw",
    );

    let user_supply_balance: Balance =
        view_balance(&controller, Supply, user.account_id(), dwnear.account_id());
    assert_eq!(
        user_supply_balance, WNEAR_AMOUNT,
        "Balance should be {}",
        WNEAR_AMOUNT
    );

    let user_balance: U128 = view!(wnear.ft_balance_of(user.account_id())).unwrap_json();
    assert_eq!(user_balance.0, 0);
}
