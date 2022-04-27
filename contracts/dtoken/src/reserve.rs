use crate::*;

const GAS_FOR_RESERVE: Gas = Gas(120_000_000_000_000);

impl Contract {
    pub fn reserve(&mut self, token_amount: WBalance) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_RESERVE,
            "Prepaid gas is not enough for reserve flow"
        );

        require!(
            self.is_valid_admin_call(),
            "Reserve action can be called by admin only"
        );

        underlying_token::ft_balance_of(
            env::current_account_id(),
            self.get_underlying_contract_address(),
            NO_DEPOSIT,
            TGAS,
        )
        .then(ext_self::reserve_balance_of_callback(
            token_amount,
            env::current_account_id(),
            NO_DEPOSIT,
            self.terra_gas(50),
        ))
        .into()
    }

    fn set_total_reserve(&mut self, amount: Balance) -> Balance {
        self.total_reserves = amount;
        self.total_reserves
    }

    pub fn increase_reserve(&mut self, token_amount: WBalance) -> Balance {
        self.set_total_reserve(self.total_reserves + Balance::from(token_amount))
    }
}

#[near_bindgen]
impl Contract {
    #[private]
    pub fn reserve_balance_of_callback(
        &mut self,
        token_amount: WBalance,
    ) -> PromiseOrValue<WBalance> {
        if !is_promise_success() {
            log!(
                "{}",
                Events::ReserveFailedToGetUnderlyingBalance(
                    env::signer_account_id(),
                    Balance::from(token_amount),
                    self.get_contract_address(),
                    self.get_underlying_contract_address()
                )
            );
            return PromiseOrValue::Value(token_amount);
        }

        self.increase_reserve(token_amount);
        return PromiseOrValue::Value(U128(0));
    }
}
