use crate::*;
pub use crate::borrows_supplies::ActionType::{Borrow, Supply};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    Supply,
    Borrow,
}


#[near_bindgen]
impl Contract {
    #[private]
    fn set_entity_by_token(&mut self, action: ActionType, account: AccountId, token_address: AccountId, tokens_amount: Balance) -> Balance {
        // Receive ActionType whether its Supply or Borrow so that
        // it will be doing respective variable configuration

        let (accounts, key_prefix) = self.get_params_by_action(action);


        if !accounts.contains_key(&account) {
            let mut account_map: LookupMap<AccountId, u128> =
                LookupMap::new(key_prefix);
            account_map.insert(&token_address, &tokens_amount);
            accounts.insert(&account, &account_map);
        } else {
            accounts
                .get(&account)
                .unwrap()
                .insert(&token_address, &tokens_amount);
        }
        return tokens_amount;
    }


    pub fn get_by_token(&self, action: ActionType, account: AccountId, token_address: AccountId) -> Balance {
        let balance: Balance = 0;

        let (accounts, _) = match action {
            ActionType::Supply => (&self.account_supplies, StorageKeys::SuppliesToken),
            ActionType::Borrow => (&self.account_borrows, StorageKeys::BorrowsToken)
        };

        if !accounts.contains_key(&account) {
            return balance;
        }

        let accounts_map = accounts.get(&account).unwrap();

        accounts_map.get(&token_address).unwrap_or(balance)
    }

    fn get_params_by_action(&mut self, action: ActionType) -> (&mut LookupMap<AccountId, LookupMap<AccountId, Balance>>, StorageKeys) {
        // return parameters respective to ActionType
        match action {
            ActionType::Supply => (&mut self.account_supplies, StorageKeys::SuppliesToken),
            ActionType::Borrow => (&mut self.account_borrows, StorageKeys::BorrowsToken)
        }
    }


    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_borrows: Balance = self.get_by_token(Borrow, account.clone(), token_address.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(tokens_amount);

        self.set_entity_by_token(Borrow, account.clone(), token_address.clone(), increased_borrows);
    }


    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_borrows: Balance = self.get_by_token(Borrow, account.clone(), token_address.clone());

        assert!(existing_borrows >= Balance::from(tokens_amount), "Too much borrowed assets trying to pay out");

        let decreased_borrows: Balance = existing_borrows - Balance::from(tokens_amount);

        self.set_entity_by_token(Borrow, account.clone(), token_address.clone(), decreased_borrows);
    }


    pub fn increase_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) {
        let existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());
        let increased_supplies: Balance = existing_supplies + Balance::from(tokens_amount);

        self.set_entity_by_token(Supply, account.clone(), token_address.clone(), increased_supplies);
    }

    pub fn decrease_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        tokens_amount: WBalance,
    ) -> Balance {
        let existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());

        assert!(
            Balance::from(tokens_amount) <= existing_supplies,
            "Not enough existing supplies"
        );
        let decreased_supplies: Balance = existing_supplies - Balance::from(tokens_amount);

        return self.set_entity_by_token(Supply,
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
        let existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());

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
                tokens_amount.clone(),
            ),
            true,
            "Withdrawal operation is not allowed for account {} token_address {} tokens_amount {}",
            account_id,
            token_address,
            Balance::from(tokens_amount)
        );

        return self.decrease_supplies(account_id, token_address, tokens_amount);
    }


    #[warn(dead_code)]
    fn is_borrow_allowed(&mut self, account: AccountId, token_address: AccountId, _tokens_amount: WBalance) -> bool {
        let _existing_borrows = self.get_by_token(Borrow, account.clone(), token_address.clone());


        let _existing_supplies = self.get_by_token(Supply, account.clone(), token_address.clone());
        // TODO add check if allowed  (USD-estimated ACCOUNT SUPPLIES > USD-estimated ACCOUNT BORROWED  * ratio ? (or just 0.8) )

        // FIXME mock-checking for now
        return true;
    }
}


#[cfg(test)]
mod tests {
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};

    use crate::borrows_supplies::ActionType::{Borrow, Supply};

    use super::*;

    fn init() -> (Contract, AccountId, AccountId) {
        let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

        let eth_contract = Contract::new(Config { owner_id: owner_account, oracle_account_id: oracle_account });

        let token_address: AccountId = "near".parse().unwrap();

        return (eth_contract, token_address, user_account);
    }


    #[test]
    fn test_for_supply_and_borrow_getters() {
        let (mut near_contract, token_address, user_account) = init();
        assert_eq!(near_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 0);
        assert_eq!(near_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 0);
    }


    #[test]
    fn test_for_supply_and_borrow_setters() {
        let (mut near_contract, token_address, user_account) = init();
        near_contract.set_entity_by_token(Supply, user_account.clone(), token_address.clone(), 100);
        assert_eq!(near_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 100);


        near_contract.set_entity_by_token(Borrow, user_account.clone(), token_address.clone(), 50);
        assert_eq!(near_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 50);
    }


    #[test]
    fn success_increase_n_decrease_borrows() {
        let (mut near_contract, token_address, user_account) = init();

        near_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));

        assert_eq!(near_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 10);

        near_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(2));

        assert_eq!(near_contract.get_by_token(Borrow, user_account.clone(), token_address.clone()), 8);
    }


    #[test]
    fn success_increase_n_decrease_supplies() {
        let (mut near_contract, token_address, user_account) = init();

        near_contract.increase_supplies(user_account.clone(), token_address.clone(), U128(10));

        assert_eq!(near_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 10);

        near_contract.decrease_supplies(user_account.clone(), token_address.clone(), U128(2));

        assert_eq!(near_contract.get_by_token(Supply, user_account.clone(), token_address.clone()), 8);
    }


    #[test]
    #[should_panic]
    fn failed_decrease_borrows() {
        /*
        Test for decrease flow behavior computation
        */
        let (mut near_contract, token_address, user_account) = init();

        near_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));


        near_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(20));
    }
}