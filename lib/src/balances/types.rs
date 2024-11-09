use super::{accounts::types::ParsedAta, dapps::types::ParsedPosition};

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum BalancesResponse {
    All(AllBalances),
    AccountsOnly(Vec<ParsedAta>),
}

#[derive(serde::Serialize)]
pub struct AllBalances {
    pub accounts: Vec<ParsedAta>,
    pub raydium: Vec<ParsedPosition>
}