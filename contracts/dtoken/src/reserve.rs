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

        self.increase_reserve(token_amount);
        PromiseOrValue::Value(U128(0))
    }

    fn set_total_reserve(&mut self, amount: Balance) -> Balance {
        self.total_reserves = amount;
        self.total_reserves
    }

    pub fn increase_reserve(&mut self, token_amount: WBalance) -> Balance {
        self.set_total_reserve(self.total_reserves + Balance::from(token_amount))
    }
}
