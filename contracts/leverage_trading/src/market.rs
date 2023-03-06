use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn add_pair(&mut self, pair_data: TradePair) {
        let pair = PairId {
            sell_token: pair_data.sell_token.clone(),
            buy_token: pair_data.buy_token.clone(),
        };

        self.supported_markets.insert(&pair, &pair_data);
    }

    #[private]
    pub fn remove_pair(&mut self, pair_data: TradePair) {
        let pair = PairId {
            sell_token: pair_data.sell_token.clone(),
            buy_token: pair_data.buy_token.clone(),
        };

        self.supported_markets.remove(&pair);
    }
}
