use crate::*;

#[near_bindgen]
impl Contract{
    pub fn is_liquidation_allowed(
        &self,
        borrower: AccountId,
        _borrowing_dtoken: AccountId,
        _collateral_dtoken: AccountId,
        amount_for_liquidation: WBalance,
    ) -> Result<WBalance, String> {
        if self.get_health_factor(borrower.clone()) > self.get_health_factor_threshold() {
            return Err(String::from("User can't be luqidated as he has normal value of health factor"));
        } else {
            return Ok(amount_for_liquidation);
        }
    }
}