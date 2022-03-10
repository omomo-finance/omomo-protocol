use crate::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;


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
        assert_eq!(env::predecessor_account_id(), self.underlying_token, "The call should come from token account");
        assert!(Balance::from(amount) > 0, "Amount should be a positive number");
        assert!(msg == *"SUPPLY" || msg == *"REPAY", "There is no such command");

        log!(format!("sender_id {}, msg {}", sender_id, msg));
        match msg.as_str() {
            "SUPPLY" => self.supply(amount),
            "REPAY" => self.repay(amount),
            _ => {
                log!("Incorrect command in transfer: {}", msg);
                PromiseOrValue::Value(amount)
            }
        }
    }
}