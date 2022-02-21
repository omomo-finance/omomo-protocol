use crate::*;

#[near_bindgen]
impl Contract {
    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());

        assert!(existing_borrows >= Balance::from(tokens_amount), "Too much borrowed assets trying to pay out");

        let decreased_borrows: Balance = existing_borrows - Balance::from(tokens_amount);
        self.total_borrows -= Balance::from(tokens_amount);
        return self.set_borrows(account.clone(), decreased_borrows);
    }

    #[private]
    fn set_borrows(&mut self, account: AccountId, tokens_amount: Balance) -> Balance {
        self.borrows
            .insert(&account, &tokens_amount);
        return tokens_amount;
    }

    pub fn get_borrows_by_account(&self, account: AccountId) -> Balance{
        assert!(self.borrows.get(&account).is_some(), "This account is never borrowed");
        return self.borrows.get(&account).unwrap();
    }

}
