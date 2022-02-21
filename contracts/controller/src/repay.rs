use crate::*;

#[near_bindgen]
impl Contract {

    fn is_repay_allowed(
        &self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> bool {
        true
    }

    pub fn repay_borrows(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    )-> Balance{
        assert_eq!(
            self.is_repay_allowed(
                account_id.clone(),
                token_address.clone(),
                token_amount.clone(),
            ),
            true,
            "repay operation is not allowed for account {} on market {}, repay amount {}",
            account_id,
            token_address,
            Balance::from(token_amount)
        );

        return self.decrease_borrows(account_id, token_address, token_amount);
    }

}