use near_contract_standards::fungible_token::FungibleToken;
use near_contract_standards::fungible_token::metadata::{
    FungibleTokenMetadata, FungibleTokenMetadataProvider,
};
use near_sdk::{AccountId, near_bindgen, PanicOnDefault, Balance, PromiseOrValue};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::json_types::U128;

#[near_bindgen]
#[derive(BorshSerialize, BorshDeserialize, PanicOnDefault)]
pub struct Contract {
    token: FungibleToken,

}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new() -> Self {
        Self {
            token: FungibleToken::new(b"t".to_vec()),
        }
    }

    pub fn mint(&mut self, account_id: AccountId, amount: U128) {
        self.token.internal_register_account(&account_id);
        self.token
            .internal_deposit(&account_id, amount.into());
    }

    pub fn burn(&mut self, account_id: AccountId, amount: U128) {
        self.token
            .internal_withdraw(&account_id, amount.into());
    }
}

// main implementation for token and storage
near_contract_standards::impl_fungible_token_core!(Contract, token);
near_contract_standards::impl_fungible_token_storage!(Contract, token);

#[near_bindgen]
impl FungibleTokenMetadataProvider for Contract {
    fn ft_metadata(&self) -> FungibleTokenMetadata {
        unimplemented!()
    }
}


#[cfg(test)]
mod tests {
    use near_sdk::{env, testing_env};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::test_utils::test_env::{alice, bob};

    use super::*;

    const TOTAL_SUPPLY: Balance = 1_000_000_000_000_000;


    fn init() -> (VMContextBuilder, AccountId, Contract) {
        // get VM builer
        let context = VMContextBuilder::new();

        // account for contract
        let _contract_account = alice();

        // init the contract
        let mut contract = Contract::new();

        (context, _contract_account, contract)
    }


    #[test]
    fn check_total_supply() {
        let (mut context, contract_account, mut contract) = init();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .build());

        // supply with 10 Near
        contract.mint(contract_account, TOTAL_SUPPLY.into());


        assert_eq!(contract.token.total_supply, TOTAL_SUPPLY);
    }

    #[test]
    fn check_balance_after_burning() {
        let (mut context, contract_account, mut contract) = init();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(contract.storage_balance_bounds().min.into())
            .build());

        let bob_account = bob();

        contract.mint(contract_account, TOTAL_SUPPLY.into());

        let transfer_amount = TOTAL_SUPPLY / 2;
        contract.mint(bob_account.clone(), transfer_amount.into());

        contract.burn(bob_account.clone(), transfer_amount.into());

        assert_eq!(contract.ft_balance_of(bob_account), 0.into());
    }
}
