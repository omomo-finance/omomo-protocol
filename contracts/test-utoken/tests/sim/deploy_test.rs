use near_sdk::json_types::U128;
use near_sdk_sim::to_yocto;

use crate::utils::init_no_macros as init;

#[test]
fn simulate_total_supply() {
    let initial_balance = to_yocto("100");
    let (_, ft, _) = init(initial_balance);

    let total_supply: U128 = ft.view(ft.account_id(), "ft_total_supply", b"").unwrap_json();

    assert_eq!(initial_balance, total_supply.0);
}
