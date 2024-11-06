use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct FormattedAmount {
    pub amount: String,
    pub formatted: f64,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct FormattedAmountWithPrice {
    pub amount: FormattedAmount,
    pub price: f64,
}
