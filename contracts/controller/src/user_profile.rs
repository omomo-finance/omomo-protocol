use crate::*;
use near_sdk::BlockHeight;
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Default)]
pub struct UserProfile {
    /// Dtoken address -> Supplies balance
    pub account_supplies: HashMap<AccountId, Balance>,

    /// Dtoken address -> Borrow balance
    pub account_borrows: HashMap<AccountId, Balance>,

    /// User consistency
    pub consistency: Consistency,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Default)]
pub struct WrappedUserProfile {
    /// Dtoken address -> Supplies balance
    pub account_supplies: HashMap<AccountId, WBalance>,

    /// Dtoken address -> Borrow balance
    pub account_borrows: HashMap<AccountId, WBalance>,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Default)]
pub struct Consistency {
    /// User consistency flag
    pub is_inconsistent: bool,

    /// Block that represents the time when consistency was affected
    pub block_height: BlockHeight,
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

    pub fn get_wrapped(&self) -> WrappedUserProfile {
        let mut result = WrappedUserProfile::default();
        for (key, value) in &self.account_supplies {
            result
                .account_supplies
                .insert(key.clone(), WBalance::from(*value));
        }
        for (key, value) in &self.account_borrows {
            result
                .account_borrows
                .insert(key.clone(), WBalance::from(*value));
        }
        result
    }

    pub fn is_consistent(&self) -> bool {
        !self.consistency.is_inconsistent
    }

    pub fn set_consistency(&mut self, consistency: bool, block: BlockHeight) {
        self.consistency.is_inconsistent = !consistency;
        self.consistency.block_height = block;
    }
}

#[near_bindgen]
impl Contract {
    /// The method can be called only by Admin, Controller, Dtoken contracts
    pub fn set_account_consistency(
        &mut self,
        account: AccountId,
        consistency: bool,
        block: BlockHeight,
    ) {
        require!(
            self.is_valid_admin_call() || self.is_dtoken_caller(),
            "This functionality is allowed to be called by admin, contract or dtoken's contract only"
        );

        self.user_profiles
            .get(&account)
            .unwrap_or_default()
            .set_consistency(consistency, block);
    }
}

#[cfg(test)]
mod tests {
    use crate::UserProfile;
    use general::ONE_TOKEN;
    use near_sdk::{AccountId, Balance};

    #[test]
    fn test_userprofile_get_wrapped() {
        let balance: Balance = 100 * ONE_TOKEN;
        let account = AccountId::new_unchecked("bob.near".to_string());
        let mut profile = UserProfile::default();
        profile.account_supplies.insert(account.clone(), balance);

        let wprofile = profile.get_wrapped();
        let supply_balance = wprofile.account_supplies.get(&account).unwrap();

        assert_eq!(
            profile.account_borrows.len(),
            wprofile.account_borrows.len(),
            "Structures has not similar length"
        );
        assert_eq!(
            profile.account_supplies.len(),
            wprofile.account_supplies.len(),
            "Structures has not similar length"
        );
        assert_eq!(
            Balance::from(*supply_balance),
            balance.clone(),
            "Wrapped structure doesn't match to expected value"
        );
    }
}
