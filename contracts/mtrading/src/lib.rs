use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, require, AccountId, Balance};

#[derive(BorshSerialize, BorshDeserialize)]
enum PositionType {
    Long,
    Short,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Position {
    active: bool,
    p_type: PositionType,
    sell_token: AccountId,
    buy_token: AccountId,
    collateral_amount:  Balance,
    buy_token_price: Balance,
    sell_token_price: Balance,
    leverage: u128
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// number of all positions
    total_positions: u128,

    /// list positions with data
    positions: LookupMap<u128, Position>,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            total_positions: 0,
            positions: LookupMap::new(b"positions".to_vec()),
        }
    }

    #[private]
    pub fn get_position(&self, position_id: U128) -> Position {
        self.positions
            .get(&position_id.0)
            .unwrap_or_else(|| panic!("Position with current position_id: {}", position_id.0))
    }

    pub fn open_position(
        &mut self,
        amount: U128,
        buy_token: AccountId,
        sell_token: AccountId,
        leverage: U128,
    )->u128 {
        self.total_positions += 1;
        self.total_positions
    }

    pub fn close_position(position_id: U128) {}

    pub fn liquidate_position(position_id: U128) {}
}
