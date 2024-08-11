use serde::Serialize;

pub enum GetAccountsError {
    InvalidAddress,
    FetchError,
    ParseError,
}

#[derive(Default, Debug, Serialize)]
pub struct ParsedAta {
    pub mint: String,
    pub ata: String,
    pub coingecko_id: Option<String>,
    pub decimals: u64,
    pub name: String,
    pub symbol: String,
    pub image: String,
    pub price: f64,
    pub balance: Balance,
}

#[derive(Default, Debug, Serialize)]
pub struct Balance {
    pub amount: u64,
    pub formatted: f64,
}
