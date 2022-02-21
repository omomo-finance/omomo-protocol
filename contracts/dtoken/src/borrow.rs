use crate::*;

#[near_bindgen]
impl Contract {
    

    pub fn borrow(token_amount: WBalance)   {

    }



    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());

        assert!(existing_borrows >= Balance::from(tokens_amount), "Too much borrowed assets trying to pay out");

        let decreased_borrows: Balance = existing_borrows - Balance::from(tokens_amount);
        self.total_borrows -= Balance::from(tokens_amount);
        return self.set_borrows(account.clone(), U128(decreased_borrows));
    }

    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(tokens_amount);

        self.total_borrows += Balance::from(tokens_amount);
        return self.set_borrows(account.clone(), U128(increased_borrows));
    }

    #[private]
    pub fn set_borrows(&mut self, account: AccountId, tokens_amount: WBalance) -> Balance {
        self.borrows
            .insert(&account, &Balance::from(tokens_amount));
        return Balance::from(tokens_amount);
    }

    pub fn get_borrows_by_account(&self, account: AccountId) -> Balance{
        if self.borrows.get(&account).is_none(){
            return 0;
        }
        return self.borrows.get(&account).unwrap();
    }

}
