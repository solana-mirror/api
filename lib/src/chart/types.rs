use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::transactions::types::FormattedAmount;

#[derive(Serialize, Deserialize, Debug)]
pub struct FormattedAmountWithPrice {
    pub amount: FormattedAmount,
    pub price: f64,
}

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
