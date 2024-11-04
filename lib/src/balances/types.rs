use super::{accounts::types::ParsedAta, dapps::types::ParsedPosition};

#[derive(serde::Serialize)]
#[serde(untagged)]
pub enum BalancesResponse {
    All(Vec<ParsedAta>, Vec<ParsedPosition>),
    AccountsOnly(Vec<ParsedAta>),
}
