use crate::types::FormattedAmountWithPrice;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ParsedPosition {
    pub total_value_usd: Option<f64>,
    pub protocol: ProtocolInfo,
    pub token_a: TokenPosition,
    pub token_b: TokenPosition,
    pub fee_tier: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TokenPosition {
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub image: String,
    pub amount: FormattedAmountWithPrice,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ProtocolInfo {
    pub name: String,
    pub symbol: String,
    pub image: String,
    pub program_id: Pubkey,
}
