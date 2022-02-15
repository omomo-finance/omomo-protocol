use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    fn set_supplies_by_token(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: Balance,
    ) -> Balance {
        if !self.account_supplies.contains_key(&account) {
            let mut supplies_map: LookupMap<AccountId, u128> =
                LookupMap::new(StorageKeys::SuppliesToken);
            supplies_map.insert(&token_address, &tokens_amount);
            self.account_supplies.insert(&account, &supplies_map);
        } 
        else {
            self.account_supplies
                .get(&account)
                .unwrap()
                .insert(&token_address, &tokens_amount);
        }
        return tokens_amount;
    }

    // #[private]
    pub fn get_supplies_by_token(&self, account: AccountId, token_address: AccountId) -> Balance {
        let balance: Balance = 0;
        if !self.account_supplies.contains_key(&account) {
            return balance;
        }
        let supplies_map = self.account_supplies.get(&account).unwrap();

        return supplies_map.get(&token_address).unwrap_or(balance);
    }

    pub fn increase_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_supplies = self.get_supplies_by_token(account.clone(), token_address.clone());
        let increased_supplies: Balance = existing_supplies + Balance::from(tokens_amount);

        self.set_supplies_by_token(account.clone(), token_address.clone(), increased_supplies);
    }

    pub fn decrease_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_supplies = self.get_supplies_by_token(account.clone(), token_address.clone());
        
        assert!(
            Balance::from(tokens_amount) <= existing_supplies,
            "Not enough existing supplies"
        );
        let decreased_supplies: Balance = existing_supplies - Balance::from(tokens_amount);

        return self.set_supplies_by_token(
            account.clone(),
            token_address.clone(),
            decreased_supplies,
        );
    }

    fn is_withdraw_allowed(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> bool {
        let existing_supplies = self.get_supplies_by_token(account.clone(), token_address.clone());
        return existing_supplies >= Balance::from(tokens_amount);
    }

    pub fn withdraw_supplies(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        assert_eq!(
            self.is_withdraw_allowed(
                account_id.clone(),
                token_address.clone(),
                tokens_amount.clone()
            ),
            true,
            "Withdrawal operation is not allowed for account {} token_address {} tokens_amount {}",
            account_id,
            token_address,
            Balance::from(tokens_amount)
        );

        return self.decrease_supplies(account_id, token_address, tokens_amount);
    }
}
