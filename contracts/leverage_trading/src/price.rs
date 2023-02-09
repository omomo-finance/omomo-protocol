use crate::big_decimal::BigDecimal;
use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn update_or_insert_price(&mut self, token_id: AccountId, price: Price) {
        require!(
            BigDecimal::from(price.value) != BigDecimal::zero(),
            "Token price cannot be zero"
        );

        self.prices.insert(&token_id, &price);
    }

    pub fn get_price(&self, token_id: AccountId) -> BigDecimal {
        BigDecimal::from(
            self.prices
                .get(&token_id)
                .unwrap_or_else(|| {
                    panic!("Price for token: {token_id} not found");
                })
                .value,
        )
    }

    pub fn calculate_xrate(&self, token_id_1: AccountId, token_id_2: AccountId) -> BigDecimal {
        BigDecimal::from(self.view_price(token_id_1).value)
            / BigDecimal::from(self.view_price(token_id_2).value)
    }

    pub fn get_market_by(&self, token: &AccountId) -> AccountId {
        self.tokens_markets.get(token).unwrap_or_else(|| {
            panic!("Market for token: {token} was not found");
        })
    }
}
