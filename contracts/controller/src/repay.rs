use crate::*;

#[near_bindgen]
impl Contract {

    fn is_repay_allowed(
        &self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> bool {
        true
    }

    pub fn repay_borrows(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    )-> Balance{
        
        assert_eq!(
            self.is_repay_allowed(
                account_id.clone(),
                token_address.clone(),
                tokens_amount.clone(),
            ),
            true,
            "Repay operation is not allowed for account {} token_address {} tokens_amount {}",
            account_id,
            token_address,
            Balance::from(tokens_amount)
        );

        return self.decrease_borrows(account_id, token_address, tokens_amount);
    }

}