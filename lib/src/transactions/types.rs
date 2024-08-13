use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a formatted amount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormattedAmount {
    pub amount: u64,
    pub formatted: f64,
}

/// Stores the pre and post balances of a transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceChange {
    pub pre: FormattedAmount,
    pub post: FormattedAmount,
}

/// Represents a parsed transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedTransaction {
    pub block_time: u64,
    pub signatures: Vec<String>,
    pub logs: Vec<String>,
    pub balances: HashMap<String, BalanceChange>,
    pub parsed_instructions: Vec<String>,
}
