use crate::*;
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Default)]
pub struct UserProfile {
    /// Dtoken address -> Supplies balance
    pub account_supplies: HashMap<AccountId, Balance>,

    /// Dtoken address -> Borrow balance
    pub account_borrows: HashMap<AccountId, Balance>,
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
}
