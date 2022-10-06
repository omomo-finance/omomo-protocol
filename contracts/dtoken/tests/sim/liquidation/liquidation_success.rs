// use std::str::FromStr;

// use crate::utils::{
//     add_market, borrow, initialize_controller, initialize_two_dtokens, initialize_two_utokens,
//     liquidate, mint_and_reserve, mint_tokens, new_user, set_price, supply, view_balance,
// };
// use controller::get_default_liquidation_incentive;
// use controller::ActionType::{Borrow, Supply};
// use dtoken::InterestRateModel;
// use general::ratio::Ratio;
// use general::Price;
// use near_sdk::json_types::U128;
// use near_sdk::Balance;
// use near_sdk_sim::{init_simulator, view, ContractAccount, UserAccount};

// const RESERVE_AMOUNT: Balance = 100000000000;
// const BORROWER_SUPPLY: Balance = 60000;
// const BORROWER_BORROW: Balance = 40000;
// const MINT_BALANCE: Balance = 100000000000;
// const START_PRICE: Balance = 2000;
// const CHANGED_PRICE: Balance = 1200;

// fn liquidation_success_fixture() -> (
//     ContractAccount<dtoken::ContractContract>,
//     ContractAccount<dtoken::ContractContract>,
//     ContractAccount<controller::ContractContract>,
//     ContractAccount<test_utoken::ContractContract>,
//     ContractAccount<test_utoken::ContractContract>,
//     UserAccount,
//     UserAccount,
// ) {
//     let root = init_simulator(None);

//     // Initialize
//     let borrower = new_user(&root, "borrower".parse().unwrap());
//     let liquidator = new_user(&root, "liquidator".parse().unwrap());
//     let (weth, wnear) = initialize_two_utokens(&root);
//     let controller = initialize_controller(&root);
//     let (droot, weth_market, wnear_market) = initialize_two_dtokens(
//         &root,
//         weth.account_id(),
//         wnear.account_id(),
//         controller.account_id(),
//         InterestRateModel::default(),
//         InterestRateModel::default(),
//     );

//     mint_and_reserve(&droot, &weth, &weth_market, RESERVE_AMOUNT);
//     mint_and_reserve(&droot, &wnear, &wnear_market, RESERVE_AMOUNT);

//     let mint_amount = U128(MINT_BALANCE);
//     mint_tokens(&weth, borrower.account_id(), mint_amount);
//     mint_tokens(&wnear, liquidator.account_id(), mint_amount);
//     mint_tokens(&weth, liquidator.account_id(), mint_amount);
//     mint_tokens(&wnear, borrower.account_id(), mint_amount);

//     add_market(
//         &controller,
//         weth.account_id(),
//         weth_market.account_id(),
//         "weth".to_string(),
//     );

//     add_market(
//         &controller,
//         wnear.account_id(),
//         wnear_market.account_id(),
//         "wnear".to_string(),
//     );

//     set_price(
//         &controller,
//         weth_market.account_id(),
//         &Price {
//             ticker_id: "weth".to_string(),
//             value: U128(START_PRICE),
//             volatility: U128(100),
//             fraction_digits: 4,
//         },
//     );

//     set_price(
//         &controller,
//         wnear_market.account_id(),
//         &Price {
//             ticker_id: "wnear".to_string(),
//             value: U128(START_PRICE),
//             volatility: U128(100),
//             fraction_digits: 4,
//         },
//     );

//     supply(&borrower, &wnear, wnear_market.account_id(), BORROWER_SUPPLY).assert_success();
//     let health_factor: Ratio =
//         view!(controller.get_health_factor(borrower.account_id())).unwrap_json();
//     assert_eq!(
//         health_factor,
//         Ratio::from_str("1.5").unwrap(),
//         "health factor should be default eq to 150%"
//     );

//     borrow(&borrower, &weth_market, BORROWER_BORROW).assert_success();
//     let health_factor: Ratio =
//         view!(controller.get_health_factor(borrower.account_id())).unwrap_json();
//     assert_eq!(
//         health_factor,
//         Ratio::from_str("1.5").unwrap(),
//         "health factor should be eq to 150%"
//     );

//     let user_balance: Balance =
//         view!(weth_market.get_account_borrows(borrower.account_id())).unwrap_json();
//     assert_eq!(
//         user_balance, BORROWER_BORROW,
//         "Borrow balance on dtoken should be {}",
//         BORROWER_BORROW
//     );

//     let user_balance: Balance = view_balance(
//         &controller,
//         Borrow,
//         borrower.account_id(),
//         weth_market.account_id(),
//     );
//     assert_eq!(
//         user_balance, BORROWER_BORROW,
//         "Borrow balance on controller should be {}",
//         BORROWER_BORROW
//     );

//     set_price(
//         &controller,
//         wnear_market.account_id(),
//         &Price {
//             ticker_id: "wnear".to_string(),
//             value: U128(CHANGED_PRICE),
//             volatility: U128(100),
//             fraction_digits: 4,
//         },
//     );
//     let health_factor: Ratio =
//         view!(controller.get_health_factor(borrower.account_id())).unwrap_json();
//     assert_eq!(health_factor, Ratio::from_str("0.9").unwrap());

//     (weth_market, wnear_market, controller, weth, wnear, borrower, liquidator)
// }

// #[test]
// fn scenario_liquidation_success() {
//     let (weth_market, wnear_market, controller, weth, _wnear, borrower, liquidator) =
//         liquidation_success_fixture();

//     let liquidation_amount = 3500;

//     liquidate(
//         &borrower,
//         &liquidator,
//         &weth_market,
//         &wnear_market,
//         &weth,
//         liquidation_amount,
//     )
//     .assert_success();

//     let weth_ft_balance_of_for_weth_market: U128 =
//         view!(weth.ft_balance_of(weth_market.account_id())).unwrap_json();

//     assert_eq!(
//         Balance::from(weth_ft_balance_of_for_weth_market),
//         (MINT_BALANCE - BORROWER_BORROW + liquidation_amount),
//         "weth_market_balance_of_on_weth balance of should be {}",
//         (MINT_BALANCE - BORROWER_BORROW + liquidation_amount)
//     );

//     let user_borrows: Balance =
//         view!(weth_market.get_account_borrows(borrower.account_id())).unwrap_json();

//     let borrow_balance = BORROWER_BORROW - liquidation_amount;

//     assert_eq!(
//         user_borrows,
//         borrow_balance.clone(),
//         "Borrow balance on dtoken should be {}",
//         borrow_balance.clone()
//     );

//     let user_borrows: Balance = view_balance(
//         &controller,
//         Borrow,
//         borrower.account_id(),
//         weth_market.account_id(),
//     );
//     assert_eq!(
//         user_borrows,
//         borrow_balance.clone(),
//         "Borrow balance on controller should be {}",
//         borrow_balance
//     );

//     let user_balance: Balance = view_balance(
//         &controller,
//         Supply,
//         liquidator.account_id(),
//         wnear_market.account_id(),
//     );

//     // 100% + 5% * liquidation_amount * old_price / new_price
//     let revenue_amount: Balance = ((Ratio::one() + get_default_liquidation_incentive())
//         * Ratio::from(liquidation_amount)
//         * Ratio::from(START_PRICE)
//         / Ratio::from(CHANGED_PRICE))
//     .round_u128();

//     assert_eq!(
//         user_balance,
//         revenue_amount.clone(),
//         "Supply balance on dtoken should be {}",
//         revenue_amount.clone()
//     );

//     let borrower_wnear_market_balance: U128 =
//         view!(wnear_market.ft_balance_of(borrower.account_id())).unwrap_json();

//     assert_eq!(
//         Balance::from(borrower_wnear_market_balance),
//         BORROWER_SUPPLY - revenue_amount,
//         "Borrower balance on dtokn ft should be {}",
//         BORROWER_SUPPLY - revenue_amount
//     );

//     let liquidator_wnear_market_balance: U128 =
//         view!(wnear_market.ft_balance_of(liquidator.account_id())).unwrap_json();

//     assert_eq!(
//         Balance::from(liquidator_wnear_market_balance),
//         revenue_amount.clone(),
//         "Liquidator balance on utoken should be {}",
//         revenue_amount.clone()
//     );
// }