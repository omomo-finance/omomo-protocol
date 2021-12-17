use near_contract_standards::fungible_token::FungibleToken;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, PromiseOrValue,
    PromiseResult,
};

use std::convert::TryFrom;

const NO_DEPOSIT: Balance = 0;
const BASE_GAS: Gas = 80_000_000_000_000; // Need to atach --gas=200000000000000 to 'borrow' call (80TGas here and 200TGas for call)
const CONTROLLER_ACCOUNT_ID: &str = "ctrl.nearlend.testnet";
const WETH_TOKEN_ACCOUNT_ID: &str = "weth.nearlend.testnet";
const WNEAR_TOKEN_ACCOUNT_ID: &str = "wnear.nearlend.testnet";
const RATIO_DECIMALS: u128 = 10_u128.pow(8);

#[ext_contract(erc20_token)]
trait Erc20Interface {
    fn internal_transfer_with_registration(
        &mut self,
        sender_id: AccountId,
        receiver_id: AccountId,
        amount: Balance,
        memo: Option<String>,
    );
}

#[ext_contract(ext_controller)]
trait ControllerInterface {
    fn borrow_allowed(
        &mut self,
        dtoken_address: AccountId,
        user_address: AccountId,
        amount: u128,
    ) -> bool;
}

#[ext_contract(ext_self)]
trait DtokenInterface {
    fn borrow_callback(amount: Balance);
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Dtoken {
    initial_exchange_rate: u128,
    total_reserve: u128,
    borrow_of: UnorderedMap<AccountId, Balance>,
    token: FungibleToken,
    underlying_token: AccountId,
}

#[near_bindgen]
impl Dtoken {
    #[init]
    pub fn new(underlying_token: AccountId) -> Self {
        Self {
            initial_exchange_rate: 100000000,
            total_reserve: 0,
            borrow_of: UnorderedMap::new(b"b".to_vec()),
            token: FungibleToken::new(b"t".to_vec()),
            underlying_token,
        }
    }

    #[private]
    pub fn borrow_callback(&mut self, amount: Balance) {
        // Borrow allowed response
        let is_allowed: bool = match env::promise_result(0) {
            PromiseResult::NotReady => {
                unreachable!()
            }
            PromiseResult::Failed => env::panic(b"Unable to make comparison"),
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<bool>(&result)
                .unwrap()
                .into(),
        };

        assert!(is_allowed, "You are not allowed to borrow");

        erc20_token::internal_transfer_with_registration(
            env::current_account_id(),
            env::predecessor_account_id(),
            amount,
            None,
            &self.underlying_token.to_string(), // Attention here!
            NO_DEPOSIT,
            10_000_000_000_000,
        );


        let borrow: u128 = amount + self.borrow_of.get(&env::predecessor_account_id()).unwrap_or(0_u128);
        self.borrow_of.insert(&env::predecessor_account_id(), &borrow);
        log!("user {} total borrow {}", env::predecessor_account_id(), borrow);
    }

    pub fn supply(&mut self, amount: Balance) {
        let dtoken_account_id = env::current_account_id();
        let predecessor_account_id = env::predecessor_account_id();

        log!("dtoken_account_id: {}", dtoken_account_id);
        log!("signer_account_id: {}", predecessor_account_id);

        erc20_token::internal_transfer_with_registration(
            predecessor_account_id.clone(),
            dtoken_account_id.clone(),
            amount,
            None,
            &self.underlying_token.clone(),
            NO_DEPOSIT,
            BASE_GAS,
        );
        log!(
            "internal_transfer_with_registration from predecessor_account_id: {} \
        to dtoken_account_id: {} with amount: {}",
            predecessor_account_id.clone(),
            dtoken_account_id.clone(),
            amount
        );

        self.mint(&predecessor_account_id.clone(), amount);
        log!(
            "predecessor_account_id dtoken balance: {}",
            self.internal_unwrap_balance_of(&predecessor_account_id)
        );
    }

    pub fn withdraw(&mut self, amount: Balance) {
        let dtoken_account_id = env::current_account_id();
        let predecessor_account_id = env::predecessor_account_id();

        log!("dtoken_account_id: {}", dtoken_account_id);
        log!("signer_account_id: {}", predecessor_account_id);

        let ext_rate = self.get_exchange_rate();
        erc20_token::internal_transfer_with_registration(
            dtoken_account_id.clone(),
            predecessor_account_id.clone(),
            amount * ext_rate / RATIO_DECIMALS,
            None,
            &self.underlying_token.clone(),
            NO_DEPOSIT,
            BASE_GAS,
        );
        log!(
            "internal_transfer_with_registration from dtoken_account_id: {} \
        to predecessor_account_id: {} with amount {}",
            predecessor_account_id.clone(),
            dtoken_account_id.clone(),
            amount * ext_rate / RATIO_DECIMALS
        );

        self.burn(&predecessor_account_id, amount);
        log!(
            "predecessor_account_id dtoken balance: {}",
            self.internal_unwrap_balance_of(&predecessor_account_id)
        );
    }

    pub fn borrow(amount: Balance) {
        let controller_account_id: AccountId =
            AccountId::try_from(CONTROLLER_ACCOUNT_ID.to_string()).unwrap();

        ext_controller::borrow_allowed(
            env::current_account_id().to_string(),
            env::predecessor_account_id(),
            amount,
            &controller_account_id.to_string(),
            NO_DEPOSIT,
            10_000_000_000_000,
        )
        .then(ext_self::borrow_callback(
            amount,
            &env::current_account_id().to_string(),
            NO_DEPOSIT,
            20_000_000_000_000,
        ));
    }

    pub fn repay() {
        //TODO: repay implementation
    }

    pub fn add_reserve(amount: Balance) {
        //TODO: add_reserve implementation
    }

    pub fn get_exchange_rate(&self) -> u128 {
        //TODO: get exchange rate by formula
        return 1 * RATIO_DECIMALS;
    }

    pub fn get_supplies(&self, account: AccountId) -> Balance {
        return self.internal_unwrap_balance_of(&account);
    }

    pub fn get_borrows(&self, account: AccountId) -> Balance {
        return self.borrow_of.get(&account).unwrap_or(0);
    }

    pub fn get_total_reserve(&self) -> u128 {
        return self.total_reserve;
    }

    pub fn get_total_supplies(&self) -> u128 {
        return self.token.total_supply;
    }

    pub fn get_total_borrows(&self) -> u128 {
        let mut total_borrows: Balance = 0;
        for (key, value) in self.borrow_of.iter() {
            total_borrows += value;
        }
        
        return total_borrows;
    }

    pub fn internal_unwrap_balance_of(&self, account_id: &AccountId) -> Balance {
        self.token
            .internal_unwrap_balance_of(&account_id.to_string())
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        self.token.internal_deposit(&account_id.to_string(), amount);
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        self.token
            .internal_withdraw(&account_id.to_string(), amount);
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        self.token.internal_transfer(
            &sender_id.to_string(),
            &receiver_id.to_string(),
            amount,
            memo,
        );
    }

    fn mint(&mut self, account_id: &AccountId, amount: Balance) {
        if !self
            .token
            .accounts
            .contains_key(&account_id.clone().to_string())
        {
            self.token
                .internal_register_account(&account_id.clone().to_string());
        }
        self.internal_deposit(account_id, amount);
    }

    fn burn(&mut self, account_id: &AccountId, amount: Balance) {
        if !self.token.accounts.contains_key(&account_id.to_string()) {
            self.token
                .internal_register_account(&account_id.to_string());
        }
        self.internal_withdraw(account_id, amount);
    }

    // Callbacks
    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Dtoken, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Dtoken, token, on_account_closed);

/*
 * the rest of this file sets up unit tests
 * to run these, the command will be:
 * cargo test --package rust-template -- --nocapture
 * Note: 'rust-template' comes from Cargo.toml's 'name' key
 */

// use the attribute below for unit tests
#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::{get_logs, VMContextBuilder};
    use near_sdk::{testing_env, AccountId};

    // part of writing unit tests is setting up a mock context
    // provide a `predecessor` here, it'll modify the default context
    fn get_context(predecessor: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
    }

    // TESTS HERE
}
