use crate::*;
use std::collections::HashMap;

#[derive(Default, BorshDeserialize, BorshSerialize)]
pub struct UserProfile {
    /// Dtoken address -> Supplies balance
    pub account_supplies: HashMap<AccountId, Balance>,

    /// Dtoken address -> Borrow balance
    pub account_borrows: HashMap<AccountId, Balance>,

    /// The flag which describe account consistency
    pub is_inconsistent: bool,
}

impl UserProfile {
    pub fn set(&mut self, action: ActionType, token_address: AccountId, token_amount: Balance) {
        if let ActionType::Supply = action {
            *self.account_supplies.entry(token_address).or_default() = token_amount;
        } else {
            *self.account_borrows.entry(token_address).or_default() = token_amount;
        }
    }

    pub fn get(&self, action: ActionType, token_address: AccountId) -> Balance {
        match action {
            ActionType::Supply => *self.account_supplies.get(&token_address).unwrap_or(&0u128),
            ActionType::Borrow => *self.account_borrows.get(&token_address).unwrap_or(&0u128),
        }
    }

    pub fn is_consistent(&self) -> bool {
        return !self.is_inconsistent;
    }

    pub fn set_consistency(&mut self, consistency: bool) {
        self.is_inconsistent = !consistency;
    }
}

#[near_bindgen]
impl Contract {
    /// The method can be called only by Admin, Controller, Dtoken contracts
    pub fn set_account_consistency(&mut self, account: AccountId, consistency: bool) {
        require!(
            self.is_valid_admin_call() || self.is_dtoken_caller(),
            "This functionality is allowed to be called by admin, contract or dtoken's contract only"
        );

        self.user_profiles
            .get(&account)
            .unwrap_or_default()
            .set_consistency(consistency);
    }
}
