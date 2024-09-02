use serde::{Deserialize, Serialize};

use crate::transactions::types::FormattedAmount;

#[derive(Default, Debug, Serialize)]
pub struct ParsedAta {
    pub mint: String,
    pub ata: String,
    #[serde(rename = "coingeckoId")]
    pub coingecko_id: Option<String>,
    pub decimals: u8,
    pub name: String,
    pub symbol: String,
    pub image: String,
    pub price: Option<f64>,
    pub balance: FormattedAmount,
}

#[derive(Default, Debug, Serialize)]
pub struct ParsedMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
}

#[derive(Deserialize)]
pub struct ImageResponse {
    pub image: String,
}
