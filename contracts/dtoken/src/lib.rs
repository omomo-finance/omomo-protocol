use near_contract_standards::fungible_token::FungibleToken;

use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{
    env, ext_contract, log, near_bindgen, AccountId, Balance, Gas, PanicOnDefault, Promise,
    PromiseOrValue, PromiseResult,
};

use std::convert::TryFrom;

const NO_DEPOSIT: Balance = 0;
const BASE_GAS: Gas = 80_000_000_000_000; // Need to attach --gas=200000000000000 to 'borrow' call (80TGas here and 200TGas for call)
const CONTROLLER_ACCOUNT_ID: &str = "ctrl.nearlend.testnet";
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
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
    fn internal_unwrap_balance_of(&mut self, account_id: AccountId) -> Balance;
}

#[ext_contract(ext_controller)]
trait ControllerInterface {
    fn borrow_allowed(
        &mut self,
        dtoken_address: AccountId,
        user_address: AccountId,
        amount: u128,
    ) -> bool;

    fn get_interest_rate(
        &mut self,
        dtoken_address: AccountId,
        underlying_balance: Balance,
        total_borrows: Balance,
        total_reserve: Balance,
    ) -> Promise;

    fn increase_user_supply(&mut self, user_address: AccountId, dtoken: AccountId, amount: Balance);

    fn decrease_user_supply(&mut self, user_address: AccountId, dtoken: AccountId, amount: Balance);
}

#[ext_contract(ext_self)]
trait DtokenInterface {
    fn borrow_callback(amount: Balance) -> Promise;
    fn withdraw_callback(&mut self, account_id: AccountId, amount: Balance);
    fn supply_callback(&mut self, amount: Balance);
    fn repay_callback_get_balance(&self) -> Promise;
    fn repay_callback_get_interest_rate(&mut self);
    fn exchange_rate_callback(&self) -> PromiseOrValue<u128>;
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Dtoken {
    initial_exchange_rate: u128,
    total_reserve: u128,
    total_borrows: u128,
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
            total_borrows: 0,
            borrow_of: UnorderedMap::new(b"b".to_vec()),
            token: FungibleToken::new(b"t".to_vec()),
            underlying_token,
        }
    }

    #[private]
    pub fn borrow_callback(&mut self, amount: Balance) -> Promise {
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

        let borrow_amount: u128 = amount
            + self
            .borrow_of
            .get(&env::signer_account_id())
            .unwrap_or(0_u128);
        self.borrow_of
            .insert(&env::signer_account_id(), &borrow_amount);
        log!(
            "user {} total borrow {}",
            env::signer_account_id(),
            borrow_amount
        );

        return erc20_token::internal_transfer_with_registration(
            env::current_account_id(),
            env::signer_account_id(),
            amount,
            None,
            &self.underlying_token,
            NO_DEPOSIT,
            10_000_000_000_000,
        );
    }

    pub fn repay_callback_get_interest_rate(&mut self) -> Promise {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ASSERT|repay_callback_get_interest_rate:promise_results_count"
        );

        let interest_rate: u128 = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Failed => env::panic(b"ext_controller::get_interest_rate failed"),
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let borrow = self.borrow_of.get(&env::signer_account_id().clone()).unwrap();
        let amount: Balance = borrow * interest_rate / RATIO_DECIMALS;


        log!("{} repay_callback_get_interest_rate {}; borrow {}; amount {}", env::signer_account_id().clone(), interest_rate, borrow, amount);
        let new_value: u128 = 0;
        self.borrow_of.insert(&env::signer_account_id().clone(), &new_value);

        return erc20_token::internal_transfer_with_registration(
            env::signer_account_id().clone(),
            env::current_account_id(),
            amount,
            None,
            &self.underlying_token,
            NO_DEPOSIT,
            5_000_000_000_000,
        );
    }

    pub fn repay_callback_get_balance(&self) -> Promise {
        assert_eq!(
            env::promise_results_count(),
            1,
            "ASSERT|repay_callback_get_balance:promise_results_count"
        );

        let underlying_balance_of_dtoken = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<u128>(&result)
                .unwrap()
                .into(),
        };

        let controller_account_id: AccountId = AccountId::try_from(CONTROLLER_ACCOUNT_ID).unwrap();

        return ext_controller::get_interest_rate(
            env::current_account_id(),
            underlying_balance_of_dtoken,
            self.get_total_borrows(),
            self.total_reserve,
            &controller_account_id,
            NO_DEPOSIT,
            70_000_000_000_000,
        )
            .then(
                ext_self::repay_callback_get_interest_rate(
                    &env::current_account_id(),
                    NO_DEPOSIT,
                    15_000_000_000_000,
                )
            );
    }

    #[private]
    pub fn withdraw_callback(&mut self, account_id: AccountId, amount: Balance) {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Withdraw callback method called"
        );
        let balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let exchange_rate: u128;
        if self.token.total_supply <= 0 {
            exchange_rate = self.initial_exchange_rate
        } else {
            exchange_rate = (balance + self.total_borrows - self.total_reserve) * RATIO_DECIMALS
                / self.token.total_supply
        }

        erc20_token::internal_transfer_with_registration(
            env::current_account_id(),
            account_id.clone(),
            amount * exchange_rate / RATIO_DECIMALS,
            None,
            &self.underlying_token,
            NO_DEPOSIT,
            10_000_000_000_000,
        );

        self.burn(&account_id.to_string(), amount);
    }

    #[private]
    pub fn exchange_rate_callback(&self) -> PromiseOrValue<u128> {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Exchange rate callback method called"
        );
        let balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let exchange_rate: u128 = if self.token.total_supply == 0 {
            self.initial_exchange_rate
        } else {
            (balance + self.total_borrows - self.total_reserve) * RATIO_DECIMALS
                / self.token.total_supply
        };

        return near_sdk::PromiseOrValue::Value(exchange_rate);
    }

    #[private]
    pub fn supply_callback(&mut self, amount: Balance) -> Promise {
        assert_eq!(
            env::promise_results_count(),
            1,
            "Supply callback method called"
        );
        let balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        let exchange_rate: u128;
        if self.token.total_supply <= 0 {
            exchange_rate = self.initial_exchange_rate
        } else {
            exchange_rate = (balance + self.total_borrows - self.total_reserve) * RATIO_DECIMALS
                / self.token.total_supply
        }

        self.mint(&env::signer_account_id().clone(), amount);
        return erc20_token::internal_transfer_with_registration(
            env::signer_account_id(),
            env::current_account_id(),
            amount * RATIO_DECIMALS / exchange_rate,
            None,
            &self.underlying_token.clone(),
            NO_DEPOSIT,
            40_000_000_000_000,
        );
    }

    pub fn supply(&mut self, amount: Balance) {
        erc20_token::ft_balance_of(
            env::current_account_id(),
            &self.underlying_token,
            NO_DEPOSIT,
            20_000_000_000_000,
        )
            .then(ext_self::supply_callback(
                amount,
                &env::current_account_id().to_string(),
                NO_DEPOSIT,
                80_000_000_000_000,
            ));
    }

    pub fn withdraw(&self, amount: Balance) {
        erc20_token::ft_balance_of(
            env::current_account_id(),
            &self.underlying_token,
            NO_DEPOSIT,
            40_000_000_000_000,
        )
            .then(ext_self::withdraw_callback(
                env::signer_account_id(),
                amount,
                &env::current_account_id().to_string(),
                NO_DEPOSIT,
                20_000_000_000_000,
            ));
    }

    pub fn borrow(amount: Balance) -> Promise {
        let controller_account_id: AccountId =
            AccountId::try_from(CONTROLLER_ACCOUNT_ID.to_string()).unwrap();

        return ext_controller::borrow_allowed(
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

    pub fn repay(&self) -> Promise {
        log!("repay env::prepaid_gas {} env::used_gas {}", &env::prepaid_gas(), &env::used_gas());

        return erc20_token::internal_unwrap_balance_of(
            env::current_account_id(),
            &self.underlying_token,
            NO_DEPOSIT,
            15_000_000_000_000,
        )
            .then(ext_self::repay_callback_get_balance(
                &env::current_account_id(),
                NO_DEPOSIT,
                100_000_000_000_000,
            ));
    }

    pub fn add_reserve(amount: Balance) {
        //TODO: add_reserve implementation
    }

    pub fn get_exchange_rate(&self) -> Promise {
        return erc20_token::ft_balance_of(
            env::current_account_id(),
            &self.underlying_token,
            NO_DEPOSIT,
            40_000_000_000_000,
        )
            .then(ext_self::exchange_rate_callback(
                &env::current_account_id().to_string(),
                NO_DEPOSIT,
                20_000_000_000_000,
            ));
    }

    pub fn get_supplies(&self, account: AccountId) -> Balance {
        if !self.token.accounts.contains_key(&account) {
            return 0;
        }

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
        for (_, value) in self.borrow_of.iter() {
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
