use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::non_fungible_token::Token;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::{ValidAccountId, U128};
use near_sdk::{env, log, near_bindgen, AccountId, Balance, PanicOnDefault, PromiseOrValue};

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
    owner_id: AccountId,
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(
        symbol: std::string::String,
        ft_name: std::string::String,
        total_supply: U128,
    ) -> Self {
        Self::new(
            env::current_account_id(),
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name: ft_name,
                symbol: symbol,
                icon: None,
                reference: None,
                reference_hash: None,
                decimals: 24,
            },
        )
    }

    #[init]
    fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
            owner_id: owner_id.clone(),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        this
    }

    pub fn give_tokens_to(&mut self, receiver_id: ValidAccountId, amount: U128) {
        if !self.token.accounts.contains_key(receiver_id.as_ref()) {
            self.token.internal_register_account(receiver_id.as_ref());
        }

        self.token
            .internal_transfer(&self.owner_id, receiver_id.as_ref(), amount.into(), None);

        log!("Gived {:?} tokens to account @{}", amount, receiver_id);
        log!("Supply left {}", self.token.total_supply);
        log!(
            "Full balance {:?}",
            self.token.accounts.get(receiver_id.as_ref())
        );
    }

    // Duplicate of ERC20 interface (which is current NEAR FT)
    pub fn internal_unwrap_balance_of(&mut self, account_id: &AccountId) -> Balance {
        self.token.internal_unwrap_balance_of(account_id)
    }

    pub fn internal_deposit(&mut self, account_id: &AccountId, amount: Balance) {
        self.token.internal_deposit(account_id, amount);
    }

    pub fn internal_withdraw(&mut self, account_id: &AccountId, amount: Balance) {
        self.token.internal_withdraw(account_id, amount);
    }

    pub fn internal_transfer(
        &mut self,
        sender_id: &AccountId,
        receiver_id: &AccountId,
        amount: Balance,
        memo: Option<String>,
    ) {
        self.token
            .internal_transfer(sender_id, receiver_id, amount, memo);
    }

    pub fn internal_register_account(&mut self, account_id: &AccountId) {
        self.token.internal_register_account(account_id);
    }

    // Callbacks
    fn on_account_closed(&mut self, account_id: AccountId, balance: Balance) {
        log!("Closed @{} with {}", account_id, balance);
    }

    fn on_tokens_burned(&mut self, account_id: AccountId, amount: Balance) {
        log!("Account @{} burned {}", account_id, amount);
    }
}

near_contract_standards::impl_fungible_token_core!(Contract, token, on_tokens_burned);
near_contract_standards::impl_fungible_token_storage!(Contract, token, on_account_closed);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}
