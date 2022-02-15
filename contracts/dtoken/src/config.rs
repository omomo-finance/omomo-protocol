use crate::*;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// The exchange rate which will be used by default
    pub initial_exchange_rate: u128,

    /// The account ID of underlying_token
    pub underlying_token_id: AccountId,

    /// The account ID of the contract owner that allows to modify config
    pub owner_id: AccountId,

    /// The account ID of the controller contract
    pub controller_account_id: AccountId,
}

#[near_bindgen]
impl Contract {
    pub fn get_contract_config(&self) -> Config {
        self.config.get().unwrap()
    }
}