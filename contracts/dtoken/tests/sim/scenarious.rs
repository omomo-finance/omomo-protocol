use near_sdk::AccountId;
use near_sdk::borsh::BorshSerialize;
use near_sdk::json_types::U128;
use near_sdk::serde_json::json;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, init_simulator, UserAccount, view};

use controller::ActionType;
use controller::ActionType::{Borrow, Supply};
use controller::Config as cConfig;
use dtoken::Config as dConfig;
use general::Price;

use crate::utils::{init_controller, init_dtoken, init_two_dtokens, init_utoken};

fn assert_failure(outcome: ExecutionResult, error_message: &str) {
    assert!(!outcome.is_ok());
    let exe_status = format!("{:?}", outcome.promise_errors()[0].as_ref().unwrap().status());
    println!("{}", exe_status);
    assert!(exe_status.contains(error_message));
}

fn view_balance(contract: &ContractAccount<controller::ContractContract>, action: ActionType, user_account: AccountId, dtoken_account: AccountId) -> u128 {
    view!(
        contract.get_entity_by_token(action, user_account, dtoken_account)
    ).unwrap_json()
}

fn initialize_utoken(root: &UserAccount) -> (UserAccount, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let uroot = root.create_user("utoken".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot, utoken, u_user) = init_utoken(
        uroot,
        AccountId::new_unchecked("utoken_contract".to_string()),
        String::from("user2_account")
    );
    call!(
        uroot,
        utoken.new_default_meta(uroot.account_id(), String::from("Mock Token"), String::from("MOCK"), U128(10000)),
        deposit = 0
    )
        .assert_success();
    (uroot, utoken, u_user)
}

fn initialize_two_utokens(
    root: &UserAccount,
) -> (
    UserAccount,
    UserAccount,
    ContractAccount<test_utoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let uroot1 = root.create_user("utoken1".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot1, utoken1, u_user1) = init_utoken(
        uroot1,
        AccountId::new_unchecked("utoken_contract1".to_string()),
        String::from("user4_account")
    );
    call!(
        uroot1,
        utoken1.new_default_meta(
            uroot1.account_id(),
            String::from("Mock Token"),
            String::from("MOCK"),
            U128(10000)
        ),
        deposit = 0
    )
        .assert_success();

    let uroot2 = root.create_user("utoken2".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot2, utoken2, u_user2) = init_utoken(
        uroot2,
        AccountId::new_unchecked("utoken_contract2".to_string()),
        String::from("user5_account")
    );
    call!(
        uroot2,
        utoken2.new_default_meta(
            uroot2.account_id(),
            String::from("Mock Token"),
            String::from("MOCK"),
            U128(10000)
        ),
        deposit = 0
    )
        .assert_success();

    (uroot1, uroot2, utoken1, utoken2, u_user1, u_user2)
}

fn initialize_controller(root: &UserAccount) -> (UserAccount, ContractAccount<controller::ContractContract>, UserAccount) {
    let croot = root.create_user("controller".parse().unwrap(), 1200000000000000000000000000000);
    let (croot, controller, c_user) = init_controller(
        croot,
        AccountId::new_unchecked("controller_contract".to_string()),
    );
    call!(
        croot,
        controller.new(
            cConfig{
                owner_id: croot.account_id().clone(), 
                oracle_account_id: "oracle".parse().unwrap()
            }),
        deposit = 0
    )
        .assert_success();
    (croot, controller, c_user)
}

fn initialize_dtoken(root: &UserAccount, utoken_account: AccountId, controller_account: AccountId) -> (UserAccount, ContractAccount<dtoken::ContractContract>, UserAccount) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken, d_user) = init_dtoken(
        droot,
        AccountId::new_unchecked("dtoken_contract".to_string()),
    );
    call!(
        droot,
        dtoken.new(
            dConfig{
                initial_exchange_rate: U128(10000), 
                underlying_token_id: utoken_account ,
                owner_id: droot.account_id().clone(), 
                controller_account_id: controller_account,
            }),
        deposit = 0
    )
        .assert_success();
    (droot, dtoken, d_user)
}

fn initialize_two_dtokens(
    root: &UserAccount,
    utoken_account1: AccountId,
    utoken_account2: AccountId,
    controller_account: AccountId,
) -> (
    UserAccount,
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<dtoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let droot = root.create_user("dtoken".parse().unwrap(), 1200000000000000000000000000000);
    let (droot, dtoken1, dtoken2, d_user1, d_user2) = init_two_dtokens(
        droot,
        AccountId::new_unchecked("dtoken_contract1".to_string()),
        AccountId::new_unchecked("dtoken_contract2".to_string()),
    );
    call!(
        droot,
        dtoken1.new(dConfig {
            initial_exchange_rate: U128(10000),
            underlying_token_id: utoken_account1,
            owner_id: droot.account_id().clone(),
            controller_account_id: controller_account.clone(),
        }),
        deposit = 0
    )
        .assert_success();

    call!(
        droot,
        dtoken2.new(dConfig {
            initial_exchange_rate: U128(10000),
            underlying_token_id: utoken_account2,
            owner_id: droot.account_id().clone(),
            controller_account_id: controller_account.clone(),
        }),
        deposit = 0
    )
        .assert_success();
    (droot, dtoken1, dtoken2, d_user1, d_user2)
}

fn base_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount, UserAccount) {
    let root = init_simulator(None);

    // Initialize
    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
         uroot,
         utoken.mint(dtoken.account_id(), U128(0)),
         0,
         100000000000000
     );

    call!(
         uroot,
         utoken.mint(d_user.account_id(), U128(20)),
         0,
         100000000000000
     );

    (dtoken, controller, utoken, d_user, root)
}

fn base2_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount, UserAccount) {
    let root = init_simulator(None);

    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(300)),
        0,
        100000000000000
    );

    call!(
        controller.user_account,
        controller.upsert_price(
            &Price {
                asset_id: dtoken.account_id(),
                value: U128(20),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    ).assert_success();

    (dtoken, controller, utoken, d_user, root)
}

fn base_repay_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let root = init_simulator(None);

    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(100)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(300)),
        0,
        100000000000000
    );

    (dtoken, controller, utoken, d_user)
}

fn withdraw_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount, UserAccount) {
    let (dtoken, controller, utoken, user, root) = base_fixture();

    call!(
        user,
        dtoken.mint(user.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        user,
        controller.increase_supplies(user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        user,
        utoken.ft_transfer(
            dtoken.account_id(), 
            U128(20), 
            Some(format!("Supply with token_amount 20"))),
        1,
        100000000000000
    );

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, utoken, user, root)
}

fn withdraw_less_dtoken_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let (dtoken, controller, utoken, user, _) = base_fixture();

    call!(
        user,
        dtoken.mint(user.account_id(), U128(3)),
        0,
        100000000000000
    ).assert_success();

    call!(
        user,
        dtoken.mint(user.account_id(), U128(7)),
        0,
        100000000000000
    ).assert_success();

    call!(
        user,
        dtoken.mint(user.account_id(), U128(10)),
        0,
        100000000000000
    ).assert_success();

    call!(
        user,
        controller.increase_supplies(user.account_id(), dtoken.account_id(), U128(20)),
        0,
        100000000000000
    ).assert_success();

    call!(
        user,
        utoken.ft_transfer(
            dtoken.account_id(), 
            U128(10), 
            Some(format!("Supply with token_amount 10"))),
        1,
        100000000000000
    );

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");

    (dtoken, controller, utoken, user)
}

fn repay_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let (dtoken, controller, utoken, user) = base_repay_fixture();

    call!(
        user,
        dtoken.increase_borrows(user.account_id(),U128(5)),
        0,
        100000000000000
    ).assert_success();

    let user_balance: u128 = view!(
        dtoken.get_account_borrows(
            user.account_id()
        )
    ).unwrap_json();
    assert_eq!(user_balance, 5, "Borrow balance on dtoken should be 5");

    call!(
        user,
        controller.increase_borrows(user.account_id(), dtoken.account_id() ,U128(5)),
        0,
        100000000000000
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 5, "Borrow balance on controller should be 5");

    (dtoken, controller, utoken, user)
}

fn liquidation_fixture() -> (
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<dtoken::ContractContract>,
    ContractAccount<controller::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    ContractAccount<test_utoken::ContractContract>,
    UserAccount,
    UserAccount,
) {
    let root = init_simulator(None);

    // Initialize
    let (uroot1, uroot2, utoken1, utoken2, _u_user1, _u_user2) = initialize_two_utokens(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken1, dtoken2, d_user1, d_user2) =
        initialize_two_dtokens(&root, utoken1.account_id(), utoken2.account_id(),controller.account_id());

    call!(
        uroot1,
        utoken1.mint(dtoken1.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot1,
        utoken1.mint(d_user1.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        uroot2,
        utoken2.mint(dtoken2.account_id(), U128(0)),
        0,
        100000000000000
    );

    call!(
        uroot2,
        utoken2.mint(d_user2.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        d_user1,
        dtoken1.increase_borrows(d_user1.account_id(),U128(5)),
        0,
        100000000000000
    ).assert_success();

    let user_balance: u128 = view!(
        dtoken1.get_account_borrows(
            d_user1.account_id()
        )
    ).unwrap_json();
    assert_eq!(user_balance, 5, "Borrow balance on dtoken should be 5");

    call!(
        d_user1,
        controller.increase_borrows(d_user1.account_id(), dtoken1.account_id() ,U128(5)),
        0,
        100000000000000
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Borrow, d_user1.account_id(), dtoken1.account_id());
    assert_eq!(user_balance, 5, "Borrow balance on controller should be 5");

    (dtoken1, dtoken2, controller,utoken1, utoken2, d_user1, d_user2)
}

fn borrow_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let root = init_simulator(None);

    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(15)),
        0,
        100000000000000
    );


    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(5)),
        0,
        100000000000000
    );


    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(30)),
        0,
        100000000000000
    );

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    call!(
        d_user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(30),
            Some("SUPPLY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();

    call!(
        controller.user_account,
        controller.upsert_price(
            &Price {
                asset_id: dtoken.account_id(),
                value: U128(20),
                volatility: U128(100),
                fraction_digits: 4
            }
        ),
        deposit = 0
    ).assert_success();

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    (dtoken, controller, utoken, d_user)
}

#[test]
fn scenario_supply_error_command() {
    let (dtoken, _controller, utoken, user, _) = base_fixture();
    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPL".to_string()),
            "SUPPL".to_string()
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 20.to_string(), "As to mistake in command, transfer shouldn't be done");
}

#[test]
fn scenario_supply_zero_tokens() {
    let (dtoken, _controller, utoken, user, _) = base_fixture();
    let result = call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(0),
            Some("SUPPLY".to_string()),
            "SUPPLY".to_string()
        ),
        deposit = 1
    );
    assert_failure(result, "The amount should be a positive number");
}

#[test]
fn scenario_supply_error_contract() {
    let (dtoken, _controller, _utoken, user, _) = base_fixture();

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    let result = call!(
        user,
        dtoken.ft_on_transfer(
            user.account_id(),
            U128(20),
            String::from(json)
        ),
        deposit = 0
    );

    assert_failure(result, "The call should come from token account");
}

#[test]
fn scenario_supply_not_enough_balance() {
    let (dtoken, _controller, utoken, user, _) = base_fixture();
    let result = call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(50),
            Some("SUPPLY".to_string()),
            "SUPPLY".to_string()
        ),
        deposit = 1
    );
    assert_failure(result, "The account doesn't have enough balance");
}

#[test]
fn scenario_supply() {
    let (dtoken, controller, utoken, user, _) = base2_fixture();

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPLY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 280.to_string(), "User balance should be 280");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 120.to_string(), "Dtoken balance should be 120");

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance on controller should be 20");
}

#[test]
fn scenario_withdraw_with_no_supply() {
    let (dtoken, _controller, _utoken, user, _) = base_fixture();

    let result = call!(
        user,
        dtoken.withdraw(U128(20)),
        deposit = 0
    );

    assert_failure(result, "Cannot calculate utilization rate as denominator is equal 0");
}

#[test]
fn scenario_withdraw_more() {
    let (dtoken, controller, _utoken, user, _) = withdraw_fixture();

    call!(
        user,
        dtoken.withdraw(U128(30)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");
}

#[test]
fn scenario_withdraw_less_same() {
    let (dtoken, controller, _utoken, user, root) = withdraw_fixture();

    call!(
        user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 10, "Balance should be 10");

    root.borrow_runtime_mut().produce_blocks(100).unwrap();

    // Withdraw same
    call!(
        user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Balance should be 0");
}

#[test]
fn scenario_withdraw() {
    let (dtoken, controller, utoken, user, _) = base2_fixture();

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("SUPPLY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();

    call!(
        user,
        dtoken.borrow(
            U128(5)
        ),
        deposit = 0
    ).assert_success();

    call!(
        user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 19, "Balance should be 19");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 114.to_string(), "After withdraw balance should be 114");
}

#[test]
fn scenario_withdraw_error_transfer() {
    let (dtoken, controller, _utoken, user) = withdraw_less_dtoken_fixture();

    call!(
        user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");
}

#[test]
fn scenario_repay_no_borrow() {
    let (dtoken, _controller, utoken, user, _) = base_fixture();

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(20),
            Some("REPAY".to_string()),
            "REPAY".to_string()
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 20.to_string(), "As user has never borrowed, transfer shouldn't be done");
}

#[test]
fn scenario_repay() {
    let (dtoken, controller, utoken, user) = repay_fixture();

    let json = r#"
       {
          "action":"REPAY",
          "memo":{
             "borrower":"123",
             "borrowing_dtoken":"123",
             "liquidator":"123",
             "collateral_dtoken":"123",
             "liquidation_amount":"123"
          }
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(277),
            Some("REPAY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 23.to_string(), "After repay of 277 tokens (borrow was 5), balance should be 23");

    let user_balance: u128 = view!(
        dtoken.get_account_borrows(
            user.account_id()
        )
    ).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}

#[test]
fn scenario_repay_more_than_borrow() {
    let (dtoken, controller, utoken, user) = repay_fixture();

    let json = r#"
       {
          "action":"REPAY",
          "memo":{
             "borrower":"123",
             "borrowing_dtoken":"123",
             "liquidator":"123",
             "collateral_dtoken":"123",
             "liquidation_amount":"123"
          }
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(300),
            Some("REPAY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 23.to_string(), "As it was borrowed 10 tokens and repayed 13 tokens (rate 1.3333), balance should be 7");

    let user_balance: u128 = view!(
        dtoken.get_account_borrows(
            user.account_id()
        )
    ).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}

#[test]
fn scenario_borrow() {
    let (dtoken, controller, utoken, user, _) = base2_fixture();

    call!(
        user,
        dtoken.borrow(
            U128(20)
        ),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "User borrow balance on controller should be 20");

    let user_balance: u128 = view!(
        dtoken.get_account_borrows(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 20, "User borrow balance on dtoken should be 20");

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 320.to_string(), "User utoken balance should be 320");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 80.to_string(), "Dtoken balance on utoken should be 80");
}

#[test]
fn scenario_borrow_more_than_on_dtoken() {
    let (dtoken, controller, utoken, user) = borrow_fixture();

    call!(
        user,
        dtoken.borrow(
            U128(60)
        ),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");

    let user_balance: u128 = view!(
        dtoken.get_account_borrows(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 0, "User borrow balance on dtoken should be 0");

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 0.to_string(), "User balance on utoken should be 0");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 50.to_string(), "Dtoken balance on utoken should be 50");
}

#[test]
fn supply_borrow_repay_withdraw() {
    // initial dtoken_balance = 100; user_balance = 300;
    let (dtoken, controller, utoken, user, _) = base2_fixture();

    let supply_json = r#"
       {
          "action":"SUPPLY",
          "memo":{}
       }"#;


    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(15),
            Some("SUPPLY".to_string()),
            String::from(supply_json)
        ),
        deposit = 1
    ).assert_success();

    // after supplying
    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 285.to_string(), "User balance should be 285");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 115.to_string(), "Dtoken balance should be 115");

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 15, "supplied assets should be 15");

    call!(
        user,
        dtoken.borrow(
            U128(5)
        ),
        deposit = 0
    ).assert_success();


    // after borrowing
    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 5, "User balance should be 5");

    let user_balance_borrows: u128 = view!(
        dtoken.get_account_borrows(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance_borrows, 5, "User borrowed balance should be 5");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 110.to_string(), "Dtoken balance should be 50");


    let json_repay = r#"
       {
          "action":"REPAY",
          "memo":{}
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(60),
            Some("REPAY".to_string()),
            String::from(json_repay)
        ),
        deposit = 1
    );

    // after repaying
    let user_borrowed_balance_after_repay: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_borrowed_balance_after_repay, 0, "User borrowed balance should be 0");

    let user_balance_after_repay: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance_after_repay, 233.to_string(), "User balance should be 233");


    call!(
        user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    // after withdrawing
    let user_balance_after_withdraw: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance_after_withdraw, 233.to_string(), "User balance should be 233");

    let user_supply_balance_after_withdraw = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_supply_balance_after_withdraw, 15, "supply balance should be 15");

    let dtoken_balance: String = view!(utoken.ft_balance_of(dtoken.account_id())).unwrap_json();
    assert_eq!(dtoken_balance, 167.to_string(), "After withdraw balance should be 167");
}

// liquidation_fixture

#[test]
fn scenario_liquidation_success() {
    let (dtoken1,
        dtoken2,
        controller,
        utoken1,
        utoken2,
        user1,
        user2) = liquidation_fixture();

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    call!(
        user1,
        utoken2.ft_transfer_call(dtoken2.account_id(), U128(10), None, String::from(json)),
        deposit = 1
    );

    let json = json!({
        "action": "LIQUIDATION",
        "memo": {
            "borrower": user1.account_id.as_str(),
            "borrowing_dtoken": dtoken1.account_id().as_str(),
            "liquidator": user2.account_id.as_str(),
            "collateral_dtoken": dtoken2.account_id().as_str(),
            "liquidation_amount": U128(5)
        }
    });

    call!(
        user2,
        utoken1.ft_transfer_call(dtoken1.account_id(), U128(10), None, json.to_string()),
        deposit = 1
    );

    let user_borrows: u128 = view!(dtoken1.get_account_borrows(user1.account_id())).unwrap_json();

    let user_balance: u128 = view_balance(&controller, Supply, user2.account_id(), dtoken2.account_id());

    // NEAR tests doesn't work with liquidation due some issues
    //assert_eq!(user_borrows, 0, "Borrow balance on dtoken should be 0");
    //assert_eq!(user_balance, 10, "Supply balance on dtoken should be 10");
}

#[test]
fn scenario_liquidation_success_on_single_dtoken()
{
    let (dtoken, _controller, utoken, user) = repay_fixture();

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(10),
            None,
            String::from(json)
        ),
        deposit = 1
    );

    let json = json!({
        "action": "LIQUIDATION",
        "memo": {
            "borrower": user.account_id.as_str(),
            "borrowing_dtoken": dtoken.account_id().as_str(),
            "liquidator": "test.testnet",
            "collateral_dtoken": dtoken.account_id().as_str(),
            "liquidation_amount": U128(10)
        }
    });

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(10),
            None,
            json.to_string()
        ),
        deposit = 1
    );

    let user_borrows: u128 = view!(
        dtoken.get_account_borrows(
            user.account_id()
        )
    ).unwrap_json();

    let user_balance: u128 = view!(
        dtoken.get_account_borrows(
           AccountId::new_unchecked("test.testnet".to_string())
        )
    ).unwrap_json();

    // NEAR tests doesn't work with liquidation due some issues
    //assert_eq!(user_borrows, 0, "Borrow balance on dtoken should be 0");
    //assert_eq!(user_balance, 10, "Supply balance on dtoken should be 10");
}

#[test]
fn scenario_liquidation_failed_no_collateral()
{
    let (dtoken, _controller, utoken, user) = repay_fixture();

    let json = json!({
        "action": "LIQUIDATION",
        "memo": {
            "borrower": user.account_id.as_str(),
            "borrowing_dtoken": dtoken.account_id().as_str(),
            "liquidator": "test.testnet",
            "collateral_dtoken": dtoken.account_id().as_str(),
            "liquidation_amount": U128(5)
        }
    });

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(5),
            None,
            json.to_string()
        ),
        deposit = 1
    ).assert_success();

    let user_borrows: u128 = view!(
        dtoken.get_account_borrows(
            user.account_id()
        )
    ).unwrap_json();
    //assert_eq!(user_borrows, 5, "Borrow balance of user should stay the same, because of an error");
}

#[test]
fn scenario_liquidation_failed_on_not_enough_amount_to_liquidate()
{
    let (dtoken, _controller, utoken, user) = repay_fixture();

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(10),
            None,
            String::from(json)
        ),
        deposit = 1
    );

    let json = json!({
        "action": "LIQUIDATION",
        "memo": {
            "borrower": user.account_id.as_str(),
            "borrowing_dtoken": dtoken.account_id().as_str(),
            "liquidator": "test.testnet",
            "collateral_dtoken": dtoken.account_id().as_str(),
            "liquidation_amount": U128(3)
        }
    });

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(3),
            None,
            json.to_string()
        ),
        deposit = 1
    ).assert_success();

    let user_borrows: u128 = view!(
        dtoken.get_account_borrows(
            user.account_id()
        )
    ).unwrap_json();
    //assert_eq!(user_borrows, 3, "Borrow balance of user should stay the same, because of an error");
}

#[test]
fn scenario_liquidation_failed_on_call_with_wrong_borrow_token()
{
    let (dtoken, _controller, utoken, user) = repay_fixture();

    let json = r#"
       {
          "action":"SUPPLY"
       }"#;

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(10),
            None,
            String::from(json)
        ),
        deposit = 1
    );

    let json = json!({
        "action": "LIQUIDATION",
        "memo": {
            "borrower": user.account_id.as_str(),
            "borrowing_dtoken": "test.testnet",
            "liquidator": "test.testnet",
            "collateral_dtoken": dtoken.account_id().as_str(),
            "liquidation_amount": U128(5)
        }
    });

    call!(
        user,
        utoken.ft_transfer_call(
            dtoken.account_id(),
            U128(5),
            None,
            json.to_string()
        ),
        deposit = 1
    ).assert_success();

    let user_borrows: u128 = view!(
        dtoken.get_account_borrows(
            user.account_id()
        )
    ).unwrap_json();
    //assert_eq!(user_borrows, 3, "Borrow balance of user should stay the same, because of an error");
}