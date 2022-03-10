use crate::*;
use near_sdk::bs58::alphabet::Error;

#[near_bindgen]
impl Contract {
    pub fn liquidation(
        &mut self,
        borrower: AccountId,
        borrowing_dtoken: AccountId,
        _liquidator: AccountId,
        _collateral_dtoken: AccountId,
        liquidation_amount: WBalance,
    ) {
        match self.is_liquidation_allowed(liquidation_amount) {
            Ok(amount) => {}
            Error => {
                panic!("liquidation is not allowed")
            }
        }

        self.decrease_borrows(borrower, borrowing_dtoken, liquidation_amount);
    }

    #[private]
    pub fn is_liquidation_allowed(
        &mut self,
        amount_for_liquidation: WBalance,
    ) -> Result<WBalance, Error> {
        // TBD
        return Ok(amount_for_liquidation);
    }
}
