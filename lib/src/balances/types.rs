use super::{accounts::types::ParsedAta, dapps::types::ParsedPosition};

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum BalancesResponse {
    All(AllBalances),
    AccountsOnly(AccountsOnly),
}

#[derive(serde::Serialize)]
pub struct AllBalances {
    pub accounts: Vec<ParsedAta>,
    pub raydium: Vec<ParsedPosition>,
}

#[derive(serde::Serialize)]
pub struct AccountsOnly {
    pub accounts: Vec<ParsedAta>,
}
