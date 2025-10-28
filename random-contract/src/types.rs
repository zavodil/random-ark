use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Coin flip choice: Heads (0) or Tails (1)
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, JsonSchema, Clone, Copy, Debug, PartialEq)]
#[borsh(crate = "near_sdk::borsh")]
#[serde(crate = "near_sdk::serde")]
pub enum CoinSide {
    Heads, // 0
    Tails, // 1
}

/// Random number response from WASM
#[derive(Serialize, Deserialize, JsonSchema, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct RandomResponse {
    pub random_number: u32,
}
