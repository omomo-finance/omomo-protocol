use crate::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde_json;
use near_sdk::serde_json::Value;

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    /// Receives the transfer from the underlying fungible token and executes method call on controller
    /// Requires to be called by the fungible underlying token account.
    /// amount - Token amount
    #[allow(unreachable_code)]
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        assert_eq!(
            env::predecessor_account_id(),
            self.underlying_token,
            "The call should come from token account"
        );
        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );

        log!(format!("sender_id {}, msg {}", sender_id, msg));

        let msg: Value =
            serde_json::from_str(msg.to_string().as_str()).expect("Can't parse JSON message");

        if !msg["memo"].is_null() {
            let memo_data = msg["memo"].clone();
            log!("borrower: {}", memo_data["borrower"]);
            log!("borrowing_dtoken: {}", memo_data["borrowing_dtoken"]);
            log!("liquidator: {}", memo_data["liquidator"]);
            log!("collateral_dtoken: {}", memo_data["collateral_dtoken"]);
            log!("liquidation_amount: {}", memo_data["liquidation_amount"]);
        }

        // TODO: In future make action not a single one, but array in JSON message
        let action: &str = msg["action"].as_str().unwrap();
        match action {
            "SUPPLY" => self.supply(amount),
            "REPAY" => self.repay(amount),
            _ => {
                log!("Incorrect command in transfer: {}", action);
                PromiseOrValue::Value(amount)
            }
        }
    }
}
