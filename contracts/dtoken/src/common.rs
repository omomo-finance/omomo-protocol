use crate::*;
use near_sdk::env::block_height;

impl Contract {
    pub fn get_controller_address(&self) -> AccountId {
        let config: Config = self.get_contract_config();

        AccountId::new_unchecked(config.controller_account_id.to_string())
    }

    pub fn get_contract_address(&self) -> AccountId {
        env::current_account_id()
    }

    pub fn get_signer_address(&self) -> AccountId {
        env::signer_account_id()
    }

    pub fn get_underlying_contract_address(&self) -> AccountId {
        self.underlying_token.clone()
    }

    pub fn get_exchange_rate(&self, underlying_balance: WBalance) -> Ratio {
        if self.token.total_supply == 0 {
            return self.initial_exchange_rate * RATIO_DECIMALS;
        }
        return (Balance::from(underlying_balance) + self.total_borrows - self.total_reserves) * RATIO_DECIMALS
            / self.token.total_supply;
    }

    pub fn terra_gas(&self, gas: u64) -> Gas {
        TGAS * gas
    }

    pub fn mutex_account_lock(&mut self, action: String) {
        if !self.mutex.try_lock(env::current_account_id()) {
            panic!(
                "failed to acquire {} action mutex for account {}",
                action,
                env::current_account_id()
            );
        }
    }

    pub fn mutex_account_unlock(&mut self) {
        self.mutex.unlock(env::signer_account_id());
    }
        
    pub fn add_inconsistent_account(&mut self, account: AccountId) {
        self.inconsistent_accounts.insert(&account, &block_height());
    }

    pub fn remove_inconsistent_account(&mut self, account: AccountId) {
        self.inconsistent_accounts.remove(&account);
    }

    pub fn custom_fail_log(event: String, account: AccountId, amount: Balance, reason: String) {
        log!(
            r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "{}", "data": {{"account_id": "{}", "amount": "{}", "reason": "{}"}}}}"#,  
            event, account, amount, reason
        ); 
    }

    pub fn custom_success_log(event: String, account: AccountId, amount: Balance) {
        log!(
            r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "{}", "data": {{"account_id": "{}", "amount": "{}"}}}}"#,  
            event, account, amount
        );
    }

    pub fn set_total_reserves(&mut self, amount: Balance) -> Balance {
        self.total_reserves = amount;
        self._get_total_reserves()
    }

    pub fn set_total_borrows(&mut self, amount: Balance) -> Balance {
        self.total_borrows = amount;
        self._get_total_borrows()
    }

    pub fn _get_total_supplies(&self) -> Balance {
        self.token.total_supply
    }

    pub fn _get_total_borrows(&self) -> Balance {
        self.total_borrows
    }

    pub fn _get_total_reserves(&self) -> Balance {
        self.total_reserves
    }
}

#[near_bindgen]
impl Contract {

    // TODO: this method should be private. Please move it and fix tests
    pub fn mint(&mut self, account_id: AccountId, amount: WBalance) {
        if self.token.accounts.get(&account_id).is_none() {
            self.token.internal_register_account(&account_id);
        };
        self.token.internal_deposit(&account_id, amount.into());
    }

    // TODO: this method should be private. Please move it and fix tests
    pub fn burn(&mut self, account_id: &AccountId, amount: WBalance) {
        if !self.token.accounts.contains_key(&account_id.clone()) {
            panic!("User with account {} wasn't found", account_id.clone());
        }
        self.token.internal_withdraw(account_id, amount.into());
    }
}
