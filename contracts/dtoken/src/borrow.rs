use crate::*;

#[near_bindgen]
impl Contract {
    fn borrow(&mut self, dtoken_amount: Balance) -> Promise {
        // somehow add if allowed functionality

        return controller::increase_borrows(
            env::signer_account_id(),
            self.get_underlying_contract_address(),
            dtoken_amount.into(),
            self.get_controller_address(),
            NO_DEPOSIT,
            TGAS * 20u64,
        )
    }
}
