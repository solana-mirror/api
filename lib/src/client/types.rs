use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::client::Header;

// get_signatures_for_address types

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
    pub err: Option<Value>,
    pub memo: Option<Value>,
    pub signature: String,
    pub slot: u64,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<String>,
}

// get_transactions method types

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub meta: Meta,
    pub slot: u64,
    pub transaction: InnerTransaction,
    pub version: Option<Version>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    #[serde(rename = "computeUnitsConsumed")]
    pub compute_units_consumed: u64,
    pub err: Option<Value>,
    pub fee: u64,
    #[serde(rename = "innerInstructions")]
    pub inner_instructions: Vec<Value>,
    #[serde(rename = "loadedAddresses")]
    pub loaded_addresses: Value,
    #[serde(rename = "logMessages")]
    pub log_messages: Option<Vec<String>>,
    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,
    #[serde(rename = "postTokenBalances")]
    pub post_token_balances: Vec<TokenBalance>,
    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,
    #[serde(rename = "preTokenBalances")]
    pub pre_token_balances: Vec<TokenBalance>,
    pub rewards: Vec<Rewards>,
    /// Deprecated
    pub status: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenBalance {
    #[serde(rename = "accountIndex")]
    pub account_index: u64,
    pub mint: String,
    pub owner: String,
    #[serde(rename = "programId")]
    pub program_id: String,
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: UiTokenAmount,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rewards {
    pub pubkey: String,
    pub lamports: i64,
    #[serde(rename = "postBalance")]
    pub post_balance: u64,
    #[serde(rename = "rewardType")]
    pub reward_type: String,
    pub commission: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InnerTransaction {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionMessage {
    #[serde(rename = "accountIndex")]
    pub account_index: Option<u64>,
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<String>,
    pub header: Header,
    pub instructions: Vec<Instruction>,
    #[serde(rename = "recentBlockhash")]
    pub recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instruction {
    pub accounts: Vec<u8>,
    pub data: String,
    #[serde(rename = "programIdIndex")]
    pub program_id_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Version {
    U8(u8),
    String(String),
}
