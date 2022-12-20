extern crate core;

mod big_decimal;
mod cancel_order;
mod common;
mod config;
mod create_order;
mod deposit;
mod execute_order;
mod ft;
mod liquidate_order;
mod market;
mod metadata;
mod oraclehook;
mod price;
mod ref_finance;
mod utils;
mod view;
mod withdraw;

pub use crate::metadata::*;

use crate::big_decimal::*;
use crate::common::PairId;
use crate::config::Config;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, require, AccountId, Balance, PromiseOrValue};
use std::collections::HashMap;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// market ➝ MarketData
    market_infos: LookupMap<AccountId, MarketData>,

    /// Protocol fee
    protocol_fee: u128,

    /// token ➝ Price
    prices: UnorderedMap<AccountId, Price>,

    /// total orders created on contract
    order_nonce: u64,

    /// user ➝ order_id ➝ Order
    orders: UnorderedMap<AccountId, HashMap<u64, Order>>,

    /// (AccountId, AccountId) ➝ TradePair
    supported_markets: UnorderedMap<PairId, TradePair>,

    /// User ➝ Token ➝ Balance
    balances: UnorderedMap<AccountId, HashMap<AccountId, Balance>>,

    config: Config,

    /// token id -> market id
    tokens_markets: LookupMap<AccountId, AccountId>,

    /// Protocol profit token_id -> amount
    protocol_profit: LookupMap<AccountId, BigDecimal>,

    /// Ref finance accountId [ as default "dcl.ref-dev.testnet" ]
    ref_finance_account: AccountId,

    /// Liquidation threshold
    liquidation_threshold: u128,

    /// Volatility rate
    volatility_rate: BigDecimal,

    /// Max value for order amount
    max_order_amount: u128,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new_with_config(owner_id: AccountId, oracle_account_id: AccountId) -> Self {
        Self::new(Config {
            owner_id,
            oracle_account_id,
        })
    }

    #[init]
    #[private]
    pub fn new(config: Config) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            market_infos: LookupMap::new(StorageKeys::Markets),
            protocol_fee: 10u128.pow(23),
            prices: UnorderedMap::new(StorageKeys::Prices),
            order_nonce: 0,
            orders: UnorderedMap::new(StorageKeys::Orders),
            supported_markets: UnorderedMap::new(StorageKeys::SupportedMarkets),
            config,
            balances: UnorderedMap::new(StorageKeys::Balances),
            tokens_markets: LookupMap::new(StorageKeys::TokenMarkets),
            protocol_profit: LookupMap::new(StorageKeys::ProtocolProfit),
            ref_finance_account: "dcl.ref-dev.testnet".parse().unwrap(),
            liquidation_threshold: 10_u128.pow(23),
            volatility_rate: BigDecimal::from(U128(95 * 10_u128.pow(22))),
            max_order_amount: 10_u128.pow(30),
        }
    }

    #[private]
    pub fn add_market_data(&mut self, market: AccountId, data: MarketData) {
        self.market_infos.insert(&market, &data);
    }

    #[private]
    pub fn set_protocol_fee(&mut self, fee: U128) {
        self.protocol_fee = fee.0
    }

    #[private]
    pub fn add_token_market(&mut self, token_id: AccountId, market_id: AccountId) {
        self.tokens_markets.insert(&token_id, &market_id);
    }

    #[private]
    pub fn set_liquidation_threshold(&mut self, threshold: U128) {
        self.liquidation_threshold = threshold.0;
    }

    #[private]
    pub fn set_volatility_rate(&mut self, rate: U128) {
        self.volatility_rate = BigDecimal::from(rate)
    }

    #[private]
    pub fn set_max_order_amount(&mut self, value: U128) {
        self.max_order_amount = value.0
    }

    pub fn get_max_leverage(&self, sell_token: AccountId, buy_token: AccountId) -> U128 {
        self.supported_markets
            .get(&(sell_token.clone(), buy_token.clone()))
            .unwrap_or_else(|| {
                panic!(
                    "Max leverage for pair {} | {} not found",
                    sell_token, buy_token
                )
            })
            .max_leverage
    }

    #[private]
    pub fn set_max_leverage(
        &mut self,
        sell_token: AccountId,
        buy_token: AccountId,
        leverage: U128,
    ) {
        let mut pair = self.supported_markets
            .get(&(sell_token.clone(), buy_token.clone()))
            .unwrap_or_else(|| {
                panic!(
                    "Max leverage for pair {} | {} not found",
                    sell_token.clone(), buy_token.clone()
                )
            });
        
            pair.max_leverage = leverage;
            self.supported_markets.insert(&(sell_token, buy_token), &pair);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // use near_sdk::test_utils::test_env::alice;
    // use near_sdk::test_utils::VMContextBuilder;
    // use near_sdk::{serde_json, testing_env, FunctionError, VMContext};

    // fn get_context(is_view: bool) -> VMContext {
    //     VMContextBuilder::new()
    //         .current_account_id("margin.nearland.testnet".parse().unwrap())
    //         .signer_account_id(alice())
    //         .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
    //         .block_index(103930916)
    //         .block_timestamp(1)
    //         .is_view(is_view)
    //         .build()
    // }

    #[test]
    fn test_get_max_leverage() {
        // let context = get_context(false);
        // testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );
        let sell_token= "usdt.qa.v1.nearlend.testnet".parse().unwrap();
        let buy_token = "wnear.qa.v1.nearlend.testnet".parse().unwrap();

        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
        };
        contract.add_pair(pair_data.clone());

        // let pair_data2 = TradePair {
        //     sell_ticker_id: "wnear".to_string(),
        //     sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        //     sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
        //     buy_ticker_id: "usdt".to_string(),
        //     buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
        //     pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
        //     max_leverage: U128(25 * 10_u128.pow(23)),
        // };

        // contract.add_pair(pair_data2.clone());

        let result = pair_data.max_leverage;
        let max_leverage = contract.get_max_leverage(sell_token, buy_token);
        assert_eq!(max_leverage, result);
    }

    #[test]
    fn test_set_max_leverage() {
        // let context = get_context(false);
        // testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );
        let sell_token: AccountId = "usdt.qa.v1.nearlend.testnet".parse().unwrap();
        let buy_token: AccountId = "wnear.qa.v1.nearlend.testnet".parse().unwrap();

        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
            max_leverage: U128(25 * 10_u128.pow(23)),
        };
        contract.add_pair(pair_data.clone());

        // let pair_data2 = TradePair {
        //     sell_ticker_id: "wnear".to_string(),
        //     sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        //     sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
        //     buy_ticker_id: "usdt".to_string(),
        //     buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
        //     pool_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000".to_string(),
        //     max_leverage: U128(25 * 10_u128.pow(23)),
        // };

        // contract.add_pair(pair_data2.clone());

        // let result = pair_data.max_leverage;
        contract.set_max_leverage(sell_token.clone(), buy_token.clone(),  U128(10 * 10_u128.pow(24)));
        let max_leverage = contract.get_max_leverage(sell_token, buy_token);
        assert_eq!(max_leverage, U128(10 * 10_u128.pow(24)));
    }


}