use crate::*;

#[near_bindgen]
impl OraclePriceHandlerHook for Contract {
    fn oracle_on_data(&mut self, price_data: PriceJsonList) {
        let config: Config = self.get_contract_config();

        assert_eq!(
            env::predecessor_account_id(),
            config.oracle_account_id,
            "Oracle account {} doesn't match to the signer {}",
            config.oracle_account_id,
            env::predecessor_account_id()
        );

        for price in price_data.price_list {
            self.upsert_price(&price);
        }
    }
}
