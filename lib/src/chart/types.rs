use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::types::{FormattedAmount, FormattedAmountWithPrice};

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct ChartData {
    pub timestamp: i64,
    pub balances: HashMap<String, FormattedAmount>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ChartDataWithPrice {
    pub timestamp: i64,
    pub balances: HashMap<String, FormattedAmountWithPrice>,
    #[serde(rename = "usdValue")]
    pub usd_value: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MinimalChartData {
    pub timestamp: i64,
    #[serde(rename = "usdValue")]
    pub usd_value: f64,
}

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum ChartResponse {
    Detailed(Vec<ChartDataWithPrice>),
    Minimal(Vec<MinimalChartData>),
}

pub struct GetCoinMarketChartParams {
    pub id: String,
    pub vs_currency: String,
    pub from: i64,
    pub to: i64,
}
