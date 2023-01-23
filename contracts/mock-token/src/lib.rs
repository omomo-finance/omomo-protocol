use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider, FT_METADATA_SPEC,
};
use near_contract_standards::fungible_token::FungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::json_types::U128;
use near_sdk::require;
use near_sdk::{env, near_bindgen, AccountId, PanicOnDefault, PromiseOrValue};

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,
    metadata: LazyOption<FungibleTokenMetadata>,
}

// example from near
const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg viewBox='0 0 40 41' fill='none' xmlns='http://www.w3.org/2000/svg'%3E%3Cpath fill-rule='evenodd' clip-rule='evenodd' d='M20 40.3967C31.0457 40.3967 40 31.4424 40 20.3967C40 9.35103 31.0457 0.396729 20 0.396729C8.9543 0.396729 0 9.35103 0 20.3967C0 31.4424 8.9543 40.3967 20 40.3967ZM9.94128 26.8949C10.8013 27.6658 11.8831 27.8967 12.92 27.8967C14.0181 27.8967 15.0686 27.635 15.7432 27.467L15.7518 27.4648C15.8896 27.4288 16.0125 27.3988 16.1084 27.3778C18.6046 26.829 21.1098 26.805 23.5551 27.2939C23.6803 27.3172 23.8337 27.3575 24.01 27.4037L24.0256 27.4078L24.0427 27.4121C25.4456 27.7647 28.3397 28.4921 30.2856 26.652C31.0407 25.9442 31.5442 24.9274 31.703 23.7996L31.7042 23.791C31.9467 22.0799 32.2474 19.9585 31.658 17.0091C31.5352 16.3792 31.0677 15.0625 30.3485 14.2287C29.1828 12.87 27.2919 12.5521 24.7238 13.2689L24.6369 13.2929C21.859 14.0578 19.0451 14.1357 16.2762 13.5239L16.0395 13.4699L16.0332 13.4684C15.1327 13.2611 13.4619 12.8765 11.973 13.086C10.6275 13.2779 9.43185 14.1627 8.77558 15.4614C8.52686 15.9503 8.40699 16.4452 8.33208 16.8231C7.90056 18.9827 7.88857 21.5411 8.30211 23.8506C8.52086 25.0653 9.10221 26.1481 9.94128 26.8949ZM12.4165 16.2383C12.5723 16.2173 12.7372 16.2083 12.908 16.2083C13.75 16.2083 14.7509 16.4362 15.3263 16.5772L15.59 16.6402C18.8683 17.363 22.1916 17.2731 25.4729 16.3702L25.5808 16.3433C27.3159 15.8514 27.8103 16.1543 27.9422 16.3073C28.1999 16.6042 28.4936 17.36 28.5475 17.639C29.033 20.0624 28.7842 21.8141 28.5625 23.3587C28.5056 23.7726 28.3407 24.1326 28.11 24.3515C27.4265 24.997 25.7497 24.5746 24.8276 24.3423L24.8047 24.3365C24.559 24.2735 24.3462 24.2225 24.1754 24.1865C21.3016 23.6077 18.3589 23.6437 15.4341 24.2795C15.3141 24.3059 15.1654 24.3437 14.9937 24.3874L14.9817 24.3905L14.9487 24.3986C14.3211 24.5535 12.5601 24.9881 12.0479 24.5315C11.7393 24.2585 11.5115 23.8086 11.4216 23.2987C11.083 21.4001 11.089 19.2136 11.4426 17.459C11.4756 17.291 11.5295 17.0541 11.6044 16.9041C11.7902 16.5322 12.0959 16.2833 12.4165 16.2383Z' fill='%2386EC8A'/%3E%3C/svg%3E";

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new_default_meta(
        owner_id: AccountId,
        name: String,
        symbol: String,
        total_supply: U128,
        decimals: u8,
    ) -> Self {
        Self::new(
            owner_id,
            total_supply,
            FungibleTokenMetadata {
                spec: FT_METADATA_SPEC.to_string(),
                name,
                symbol,
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                reference: None,
                reference_hash: None,
                decimals,
            },
        )
    }

    #[init]
    pub fn new(owner_id: AccountId, total_supply: U128, metadata: FungibleTokenMetadata) -> Self {
        require!(!env::state_exists(), "Already initialized");

        metadata.assert_valid();
        let mut this = Self {
            token: FungibleToken::new(b"a".to_vec()),
            metadata: LazyOption::new(b"m".to_vec(), Some(&metadata)),
        };
        this.token.internal_register_account(&owner_id);
        this.token.internal_deposit(&owner_id, total_supply.into());
        this
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        if self.token.accounts.get(&account_id).is_none() {
            self.token.internal_register_account(&account_id);
        };
        self.token.internal_deposit(&account_id, amount.into());
    }

    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.token.internal_withdraw(&account_id, amount.into());
    }
}

// main implementation for token and storage
near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(test)]
mod tests {
    use near_sdk::test_utils::test_env::{alice, bob};
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::{env, testing_env, Balance};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;

    fn init() -> (VMContextBuilder, AccountId, Contract) {
        // get VM builer
        let context = VMContextBuilder::new();

        // account for contract
        let _contract_account = alice();

        // init the contract

        let contract = Contract::new_default_meta(
            _contract_account.clone(),
            String::from("Mock Token"),
            String::from("MOCK"),
            TOTAL_SUPPLY.into(),
            24,
        );

        (context, _contract_account, contract)
    }

    #[allow(dead_code)]
    fn get_context(predecessor_account_id: AccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    #[test]
    fn check_total_supply() {
        let (context, _contract_account, contract) = init();

        testing_env!(context.build());

        assert_eq!(contract.ft_total_supply(), 1_000_000_000_000_000.into());
    }

    #[test]
    fn test_mint_bob() {
        let (context, _, mut contract) = init();

        testing_env!(context.build());

        let bob_account = bob();

        contract.mint(bob_account.clone(), (TOTAL_SUPPLY / 100).into());

        assert_eq!(
            contract.ft_balance_of(bob_account),
            (TOTAL_SUPPLY / 100).into()
        )
    }

    #[test]
    fn test_burn_bob() {
        let (context, _, mut contract) = init();

        testing_env!(context.build());

        let bob_account = bob();

        contract.mint(bob_account.clone(), (TOTAL_SUPPLY / 100).into());
        contract.burn(bob_account.clone(), (TOTAL_SUPPLY / 100).into());

        assert_eq!(contract.ft_balance_of(bob_account), 0.into())
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(2));
        testing_env!(context.build());

        let mut contract = Contract::new_default_meta(
            accounts(2),
            String::from("Mock Token"),
            String::from("MOCK"),
            TOTAL_SUPPLY.into(),
            24,
        );

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .predecessor_account_id(accounts(1))
            .build());
        // Paying for account registration, aka storage deposit
        contract.storage_deposit(None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(2))
            .build());

        let transferred_tokens = TOTAL_SUPPLY / 100;
        contract.ft_transfer(
            accounts(1),
            transferred_tokens.into(),
            Some("you have received some tokens bro".to_string()),
        );

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());

        assert_eq!(
            contract.ft_balance_of(accounts(2)).0,
            (TOTAL_SUPPLY - transferred_tokens)
        );
        assert_eq!(contract.ft_balance_of(accounts(1)).0, transferred_tokens);
    }
}
