use ethers::types::{Address, H256, U128, U256};
use serde::{Deserialize, Serialize};
use serde_repr::Deserialize_repr;

/// Many queries can take a block range. To see how to use this as a parameter, see
/// some examples.
#[derive(Clone, Debug, Default, Serialize)]
#[non_exhaustive]
pub struct QueryOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end: Option<u64>,
}

impl QueryOptions {
    pub fn start(self, value: u64) -> Self {
        Self {
            start: Some(value),
            ..self
        }
    }
    pub fn end(self, value: u64) -> Self {
        Self {
            end: Some(value),
            ..self
        }
    }
    pub fn with_start(self, value: Option<u64>) -> Self {
        Self {
            start: value,
            ..self
        }
    }
    pub fn with_end(self, value: Option<u64>) -> Self {
        Self {
            start: value,
            ..self
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct BlockHeader {
    pub hash: U256,
    pub block_number: u64,
    pub timestamp: i64,
}

/// A uniswap v2 `PairCreated` event
/// <https://docs.uniswap.org/protocol/V2/reference/smart-contracts/factory#paircreated>
#[derive(Clone, Debug, Deserialize)]
pub struct PairCreated {
    pub block_number: u64,
    pub factory: Address,
    pub pair: Address,
    pub token0: Address,
    pub token1: Address,
    pub pair_index: U256,
    pub timestamp: i64,
    pub transaction_hash: H256,
    pub transaction_index: i64,
}

/// A uniswap v2 price quote
#[derive(Clone, Debug, Deserialize)]
pub struct Price {
    pub block_number: u64,
    pub address: Address,
    // actually u112
    pub reserve0: U128,
    // actually u112
    pub reserve1: U128,
    pub price: f64,
    pub decimals0: u8,
    pub decimals1: u8,
    pub timestamp: i64,
    pub transaction_hash: H256,
    pub transaction_index: i64,
}

/// The direction of transaction
#[derive(Clone, Copy, Debug, Deserialize_repr, PartialEq, Eq)]
#[repr(u8)]
pub enum Side {
    Buy,
    Sell,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq, Eq)]
pub struct Reserves {
    pub block_number: i64,
    pub timestamp: i64,
    pub transaction_hash: H256,
    pub transaction_index: i64,
    pub event: Type,
    pub reserve0: u128,
    pub reserve1: u128,
    pub amount0: U256,
    pub amount1: U256,
    pub lp_amount: U256,
    pub protocol_fee: Option<U256>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug, Deserialize_repr)]
#[repr(u8)]
pub enum Type {
    Mint,
    Burn,
    Swap,
    Sync,
}

#[derive(Clone, Debug, serde::Deserialize, PartialEq)]
pub struct Trade {
    pub block_number: i64,
    pub address: Address,
    pub sender: Address,
    pub receiver: Address,
    pub price: f64,
    pub last_traded_price: f64,
    pub volume0: f64,
    pub volume1: f64,
    pub fixed0: U256,
    pub fixed1: U256,
    pub decimals0: u8,
    pub decimals1: u8,
    pub side: Side,
    pub timestamp: i64,
    pub transaction_hash: H256,
    pub transaction_index: i64,
}
