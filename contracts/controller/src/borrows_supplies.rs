use near_sdk::require;

use crate::*;
use crate::borrows_supplies::ActionType::{Borrow, Supply};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum ActionType {
    Supply,
    Borrow,
}

#[near_bindgen]
impl Contract {
    #[private]
    fn set_entity_by_token(&mut self, action: ActionType, user_id: AccountId, token_address: AccountId, token_amount: Balance) -> Balance {
        let mut user = self.user_profiles.get(&user_id).unwrap_or(UserProfile::default());
        user.update(action, token_address, token_amount);
        self.user_profiles.insert(&user_id, &user);

        return token_amount;
    }

    pub fn get_entity_by_token(&self, action: ActionType, user_id: AccountId, token_address: AccountId) -> Balance {
        let user = self.user_profiles.get(&user_id).unwrap_or(UserProfile::default());

        user.get(action, token_address)
    }

    pub fn increase_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) {
        let existing_borrows: Balance = self.get_entity_by_token(Borrow, account.clone(), token_address.clone());
        let increased_borrows: Balance = existing_borrows + Balance::from(token_amount);

        self.set_entity_by_token(Borrow, account.clone(), token_address.clone(), increased_borrows);
    }

    pub fn decrease_borrows(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_borrows: Balance = self.get_entity_by_token(Borrow, account.clone(), token_address.clone());

        assert!(existing_borrows >= Balance::from(token_amount), "Too much borrowed assets trying to pay out");

        let decreased_borrows: Balance = existing_borrows - Balance::from(token_amount);

        return self.set_entity_by_token(Borrow, account.clone(), token_address.clone(), decreased_borrows);
    }

    pub fn increase_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) {
        let existing_supplies = self.get_entity_by_token(Supply, account.clone(), token_address.clone());
        let increased_supplies: Balance = existing_supplies + Balance::from(token_amount);

        self.set_entity_by_token(Supply, account.clone(), token_address.clone(), increased_supplies);
    }

    pub fn decrease_supplies(
        &mut self,
        account: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        let existing_supplies = self.get_entity_by_token(Supply, account.clone(), token_address.clone());

        assert!(
            Balance::from(token_amount) <= existing_supplies,
            "Not enough existing supplies"
        );
        let decreased_supplies: Balance = existing_supplies - Balance::from(token_amount);

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
        token_amount: WBalance,
    ) -> bool {
        require!( 
            !self.is_action_paused.withdraw,
            "withdrawing is paused"
        );
        let existing_supplies = self.get_entity_by_token(Supply, account.clone(), token_address.clone());
        existing_supplies >= Balance::from(token_amount)
    }

    pub fn withdraw_supplies(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) -> Balance {
        assert_eq!(
            self.is_withdraw_allowed(
                account_id.clone(),
                token_address.clone(),
                token_amount.clone(),
            ),
            true,
            "Withdrawal operation is not allowed for account {} token_address {} token_amount` {}",
            account_id,
            token_address,
            Balance::from(token_amount)
        );

        return self.decrease_supplies(account_id, token_address, token_amount);
    }

    #[warn(dead_code)]
    fn is_borrow_allowed(&mut self, account: AccountId, token_address: AccountId, _token_amount: WBalance) -> bool {
        require!(
            !self.is_action_paused.borrow,
            "borrowing is paused"
        );
        let _existing_borrows = self.get_entity_by_token(Borrow, account.clone(), token_address.clone());

        let _existing_supplies = self.get_entity_by_token(Supply, account.clone(), token_address.clone());

        return self.get_health_factor(account.clone()) > self.get_health_factor_threshold();
    }

    pub fn make_borrow(
        &mut self,
        account_id: AccountId,
        token_address: AccountId,
        token_amount: WBalance,
    ) {
        assert_eq!(
            self.is_borrow_allowed(
                account_id.clone(),
                token_address.clone(),
                token_amount.clone(),
            ),
            true,
            "Borrow operation is not allowed for account {} token_address {} token_amount {}",
            account_id,
            token_address,
            Balance::from(token_amount)
        );
        self.increase_borrows(account_id, token_address, token_amount);
    }
    fn get_account_balance(&self, account_entry: Option<HashMap<AccountId, Balance>>) -> WBalance {
        let mut balance: Balance = 0;

        if account_entry.is_some() {
            let account_borrow = account_entry.unwrap();

            for (asset, asset_amount) in account_borrow.iter() {
                let asset_price: Balance = self.get_price(asset.clone()).unwrap().value.0;

                balance += asset_price * asset_amount;
            }
        }

        U128(balance)
    }

    pub fn get_total_borrows(&self, account: AccountId) -> WBalance {
        todo!();
    }

    pub fn get_total_supplies(&self, account: AccountId) -> WBalance {
        todo!();
    }
}


#[cfg(test)]
mod tests {
    use near_sdk::AccountId;
    use near_sdk::json_types::U128;
    use near_sdk::test_utils::test_env::{alice, bob, carol};

    use crate::{Config, Contract};
    use crate::borrows_supplies::ActionType::{Borrow, Supply};

    pub fn init_test_env() -> (Contract, AccountId, AccountId) {
        let (owner_account, oracle_account, user_account) = (alice(), bob(), carol());

        let near_contract = Contract::new(Config {
            owner_id: owner_account,
            oracle_account_id: oracle_account,
        });

        let token_address: AccountId = "near".parse().unwrap();

        return (near_contract, token_address, user_account);
    }

    #[test]
    fn test_for_supply_and_borrow_getters() {
        let (near_contract, token_address, user_account) = init_test_env();
        assert_eq!(near_contract.get_entity_by_token(Supply, user_account.clone(), token_address.clone()), 0);
        assert_eq!(near_contract.get_entity_by_token(Borrow, user_account.clone(), token_address.clone()), 0);
    }

    #[test]
    fn test_for_supply_and_borrow_setters() {
        let (mut near_contract, token_address, user_account) = init_test_env();
        near_contract.set_entity_by_token(Supply, user_account.clone(), token_address.clone(), 100);
        assert_eq!(near_contract.get_entity_by_token(Supply, user_account.clone(), token_address.clone()), 100);

        near_contract.set_entity_by_token(Borrow, user_account.clone(), token_address.clone(), 50);
        assert_eq!(near_contract.get_entity_by_token(Borrow, user_account.clone(), token_address.clone()), 50);
    }

    #[test]
    fn success_increase_n_decrease_borrows() {
        let (mut near_contract, token_address, user_account) = init_test_env();

        near_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));
        near_contract.increase_borrows(user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string()), U128(20));

        assert_eq!(near_contract.get_entity_by_token(Borrow, user_account.clone(), token_address.clone()), 10);
        assert_eq!(near_contract.get_entity_by_token(Borrow, user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string())), 20);

        near_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(2));
        near_contract.decrease_borrows(user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string()), U128(2));

        assert_eq!(near_contract.get_entity_by_token(Borrow, user_account.clone(), token_address.clone()), 8);
        assert_eq!(near_contract.get_entity_by_token(Borrow, user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string())), 18);
    }

    #[test]
    fn success_increase_n_decrease_supplies() {
        let (mut near_contract, token_address, user_account) = init_test_env();

        near_contract.increase_supplies(user_account.clone(), token_address.clone(), U128(10));
        near_contract.increase_supplies(user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string()), U128(20));

        assert_eq!(near_contract.get_entity_by_token(Supply, user_account.clone(), token_address.clone()), 10);
        assert_eq!(near_contract.get_entity_by_token(Supply, user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string())), 20);

        near_contract.decrease_supplies(user_account.clone(), token_address.clone(), U128(2));
        near_contract.decrease_supplies(user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string()), U128(2));

        assert_eq!(near_contract.get_entity_by_token(Supply, user_account.clone(), token_address.clone()), 8);
        assert_eq!(near_contract.get_entity_by_token(Supply, user_account.clone(), AccountId::new_unchecked("test.nearlend".to_string())), 18);
    }

    #[test]
    #[should_panic]
    fn failed_decrease_borrows() {
        /*
        Test for decrease flow behavior computation
        */
        let (mut near_contract, token_address, user_account) = init_test_env();

        near_contract.increase_borrows(user_account.clone(), token_address.clone(), U128(10));

        near_contract.decrease_borrows(user_account.clone(), token_address.clone(), U128(20));
    }
}