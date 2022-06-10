use dtoken::InterestRateModel;
use near_sdk::{json_types::U128, Balance};
use near_sdk_sim::{init_simulator, ContractAccount, UserAccount};

use general::Price;

use crate::utils::{
    add_market, assert_failure, initialize_controller, initialize_dtoken, initialize_utoken,
    mint_tokens, new_user, set_price, supply,
};

const WNEAR_BALANCE: Balance = 50;
const SUPPLY_AMOUNT: Balance = 0;
const START_PRICE: Balance = 10000;

fn supply_amount_should_be_positive_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
) {
    let root = init_simulator(None);

    let user = new_user(&root, "user".parse().unwrap());
    let wnear = initialize_utoken(&root);
    let controller = initialize_controller(&root);
    let (_, dwnear) = initialize_dtoken(
        &root,
        wnear.account_id(),
        controller.account_id(),
        InterestRateModel::default(),
    );

    mint_tokens(&wnear, dwnear.account_id(), U128(100));
    mint_tokens(&wnear, user.account_id(), U128(WNEAR_BALANCE));

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
            value: U128(START_PRICE),
            volatility: U128(100),
            fraction_digits: 4,
        },
    );

    (dwnear, wnear, user)
}

#[test]
fn scenario_supply_amount_should_be_positive() {
    let (dwnear, wnear, user) = supply_amount_should_be_positive_fixture();

    let result = supply(&user, &wnear, dwnear.account_id(), SUPPLY_AMOUNT);
    assert_failure(result, "The amount should be a positive number");
}
