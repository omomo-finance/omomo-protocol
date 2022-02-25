use crate::*;

#[near_bindgen]
impl Contract {
    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());

        assert!(existing_borrows >= Balance::from(token_amount), "Repay amount is more than existing borrows");
        let decreased_borrows: Balance = existing_borrows - Balance::from(token_amount);

        let new_borrows = self.total_borrows.overflowing_sub(Balance::from(token_amount));
        assert_eq!(new_borrows.1, false, "Overflow occurs while decreasing total supply");
        self.total_borrows = new_borrows.0;
        
        return self.set_borrows(account.clone(), U128(decreased_borrows));
    }

    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_borrows_by_account(account.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(token_amount);

        let new_borrows = self.total_borrows.overflowing_add(Balance::from(token_amount));
        assert_eq!(new_borrows.1, false, "Overflow occurs while incresing total supply");
        self.total_borrows = new_borrows.0;
        return self.set_borrows(account.clone(), U128(increased_borrows));
    }

    #[private]
    pub fn set_borrows(&mut self, account: AccountId, token_amount: WBalance) -> Balance {
        self.borrows
            .insert(&account, &Balance::from(token_amount));
        return Balance::from(token_amount);
    }

    pub fn get_borrows_by_account(&self, account: AccountId) -> Balance{
        if self.borrows.get(&account).is_none(){
            return 0;
        }
        return self.borrows.get(&account).unwrap();
    }

}
