use crate::*;

use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::serde_json;
use near_sdk::AccountId;

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

        let action: Actions = serde_json::from_str(&msg).expect("Incorrect command in transfer");

        match action {
            Actions::Supply => self.supply(amount),
            Actions::Repay => self.repay(amount),
            Actions::Liquidate {
                borrower,
                borrowing_dtoken,
                collateral_dtoken,
            } => self.liquidate(
                borrower,
                borrowing_dtoken,
                env::signer_account_id(),
                collateral_dtoken,
                amount,
            ),
            _ => {
                panic!("Incorrect action in transfer")
            }
        }
    }
}
