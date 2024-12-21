use crate::types::FormattedAmountWithPrice;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct ParsedPosition {
    #[serde(rename = "totalValueUsd")]
    pub total_value_usd: Option<f64>,
    pub protocol: ProtocolInfo,
    #[serde(rename = "tokenA")]
    pub token_a: TokenPosition,
    #[serde(rename = "tokenB")]
    pub token_b: TokenPosition,
    #[serde(rename = "feeTier")]
    pub fee_tier: String,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct TokenPosition {
    pub mint: String,
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
    #[serde(rename = "poolId")]
    pub pool_id: String,
}
