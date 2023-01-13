use crate::*;
use near_sdk::json_types::U128;

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// The exchange rate which will be used by default
    pub initial_exchange_rate: U128,

    /// The account ID of underlying_token
    pub underlying_token_id: AccountId,
    pub underlying_token_decimals: u8,

    /// The account ID of the contract owner that allows to modify config
    pub owner_id: AccountId,

    /// The account ID of the controller contract
    pub controller_account_id: AccountId,

    /// The interest rate model with custom values
    pub interest_rate_model: InterestRateModel,

    /// The flag that dtoken can be transferred
    pub disable_transfer_token: bool,
}

impl Contract {
    pub fn get_contract_config(&self) -> Config {
        self.config.get().unwrap()
    }
}
