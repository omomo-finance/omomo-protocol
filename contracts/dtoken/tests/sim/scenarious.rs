use near_sdk::AccountId;
use near_sdk::json_types::U128;
use near_sdk_sim::{call, ContractAccount, ExecutionResult, init_simulator, UserAccount, view};
use controller::{Config as cConfig};
use controller::ActionType;
use controller::ActionType::{Supply, Borrow};
use dtoken::Config as dConfig;
use crate::utils::{init_controller, init_dtoken, init_utoken};


fn assert_failure(outcome: ExecutionResult, error_message: &str) {
    assert!(!outcome.is_ok());
    let exe_status = format!("{:?}", outcome.promise_errors()[0].as_ref().unwrap().status());
    println!("{}", exe_status);
    assert!(exe_status.contains(error_message));
}

fn view_balance(contract: &ContractAccount<controller::ContractContract>, action: ActionType, user_account: AccountId, dtoken_account: AccountId) -> u128{
    view!(
        contract.get_entity_by_token(action, user_account, dtoken_account)
    ).unwrap_json()
}

fn initialize_utoken(root: &UserAccount) -> (UserAccount, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let uroot = root.create_user("utoken".parse().unwrap(), 1200000000000000000000000000000);
    let (uroot, utoken, u_user) = init_utoken(
        uroot,
        AccountId::new_unchecked("utoken_contract".to_string()),
    );
    call!(
        uroot,
        utoken.new_default_meta(uroot.account_id(), U128(10000)),
        deposit = 0
    )
        .assert_success();
    (uroot, utoken, u_user)
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
                initial_exchange_rate: U128(1), 
                underlying_token_id: utoken_account ,
                owner_id: droot.account_id().clone(), 
                controller_account_id: controller_account,
            }),
        deposit = 0
    )
        .assert_success();
    (droot, dtoken, d_user)
}

fn base_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount){
     // Supply
     let root = init_simulator(None);
     //  Initialize
 
     let (uroot, utoken, _u_user) = initialize_utoken(&root);
     let (_croot, controller, _c_user) = initialize_controller(&root);
     let (_droot, dtoken, d_user) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());
 
     // Supply preparation 
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

     (dtoken, controller, utoken, d_user)
}

fn withdraw_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount){
    let (dtoken, controller, utoken, user) = base_fixture();

    call!(
        user,
        dtoken.mint(&user.account_id(), U128(20)),
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

    (dtoken, controller, utoken, user)
}

fn repay_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let (dtoken, controller, utoken, user) = base_fixture();

    call!(
        user,
        dtoken.increase_borrows(user.account_id(),U128(10)),
        0,
        100000000000000
    ).assert_success();

    let user_balance: u128 = view!(
        dtoken.get_borrows_by_account(
            user.account_id()
        )
    ).unwrap_json();
    assert_eq!(user_balance, 10, "Borrow balance on dtoken should be 10");

    call!(
        user,
        controller.increase_borrows(user.account_id(), dtoken.account_id() ,U128(10)),
        0,
        100000000000000
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 10, "Borrow balance on controller should be 10");

    (dtoken, controller, utoken, user)
}

fn borrow_fixture() -> (ContractAccount<dtoken::ContractContract>, ContractAccount<controller::ContractContract>, ContractAccount<test_utoken::ContractContract>, UserAccount) {
    let root = init_simulator(None);
              
    let (uroot, utoken, _u_user) = initialize_utoken(&root);
    let (_croot, controller, _c_user) = initialize_controller(&root);
    let (_droot, dtoken, d_user) = initialize_dtoken(&root, utoken.account_id(), controller.account_id());

    call!(
        uroot,
        utoken.mint(dtoken.account_id(), U128(20)),
        0,
        100000000000000
    );

    call!(
        uroot,
        utoken.mint(d_user.account_id(), U128(0)),
        0,
        100000000000000
    );
    (dtoken, controller, utoken, d_user)

}

#[test]
fn scenario_supply_error_command(){
    let (dtoken, _controller, utoken, user) = base_fixture();
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
fn scenario_supply_zero_tokens(){
    let (dtoken, _controller, utoken, user) = base_fixture();
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
fn scenario_supply_error_contract(){
    let (dtoken, _controller, _utoken, user) = base_fixture();

    let json = r#"
       {
          "action":"SUPPLY",
          "memo":{
             "borrower":"123",
             "borrowing_dtoken":"123",
             "liquidator":"123",
             "collateral_dtoken":"123",
             "liquidation_amount":"123"
          }
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
fn scenario_supply_not_enough_balance(){
    let (dtoken, _controller, utoken, user) = base_fixture();
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
    let (dtoken, controller, utoken, user) = base_fixture();

    let json = r#"
       {
          "action":"SUPPLY",
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
            U128(20),
            Some("SUPPLY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();


    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 0.to_string(), "User balance should be 0");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 20.to_string(), "Dtoken balance should be 20");

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance on controller should be 20");
    
}

#[test]
fn scenario_withdraw_with_no_supply(){
    let (dtoken, _controller, _utoken, user) = base_fixture();

    let result = call!(
        user,
        dtoken.withdraw(U128(20)),
        deposit = 0
    );

    assert_failure(result, "Withdrawal operation is not allowed");
}

#[test]
fn scenario_withdraw_more(){
    let (dtoken, controller, _utoken, user) = withdraw_fixture();

    let result = call!(
        user,
        dtoken.withdraw(U128(30)),
        deposit = 0
    );

    assert_failure(result, "Withdrawal operation is not allowed");

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 20, "Balance should be 20");
}

#[test]
fn scenario_withdraw_less_same(){
    let (dtoken, controller, _utoken, user) = withdraw_fixture();

    // Withdraw less
    call!(
        user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 10, "Balance should be 10");

    // Withdraw the same
    call!(
        user,
        dtoken.withdraw(U128(10)),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Supply, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Balance should be 0");

}

#[test]
fn scenario_repay_no_borrow(){
    let (dtoken, _controller, utoken, user) = base_fixture();

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
fn scenario_repay(){
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
            U128(10),
            Some("REPAY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 10.to_string(), "After repay of 10 tokens, balance should be 10");
    
    let user_balance: u128 = view!(
        dtoken.get_borrows_by_account(
            user.account_id()
        )
    ).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}

#[test]
fn scenario_repay_more_than_borrow(){
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
            U128(20),
            Some("REPAY".to_string()),
            String::from(json)
        ),
        deposit = 1
    ).assert_success();

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 10.to_string(), "As it was borrowed 10 tokens and repayed 20 tokens, balance should be 10");
    
    let user_balance: u128 = view!(
        dtoken.get_borrows_by_account(
            user.account_id()
        )
    ).unwrap_json();
    assert_eq!(user_balance, 0, "Borrow balance on dtoken should be 0");

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");
}

#[test]
fn scenario_borrow(){
    let (dtoken, controller, utoken, user) = borrow_fixture();

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
        dtoken.get_borrows_by_account(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 20, "User borrow balance on dtoken should be 20");

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 20.to_string(), "User utoken balance should be 20");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 0.to_string(), "Dtoken balance on utoken should be 0");
}

#[test]
fn scenatio_borrow_more_than_on_dtoken(){
    let (dtoken, controller, utoken, user) = borrow_fixture();

    call!(
        user,
        dtoken.borrow(
            U128(40)
        ),
        deposit = 0
    ).assert_success();

    let user_balance: u128 = view_balance(&controller, Borrow, user.account_id(), dtoken.account_id());
    assert_eq!(user_balance, 0, "Borrow balance on controller should be 0");

    let user_balance: u128 = view!(
        dtoken.get_borrows_by_account(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 0, "User borrow balance on dtoken should be 0");

    let user_balance: String = view!(
        utoken.ft_balance_of(user.account_id())
    ).unwrap_json();
    assert_eq!(user_balance, 0.to_string(), "User balance on utoken should be 0");

    let dtoken_balance: String = view!(
        utoken.ft_balance_of(dtoken.account_id())
    ).unwrap_json();
    assert_eq!(dtoken_balance, 20.to_string(), "Dtoken balance on utoken should be 20");
}

