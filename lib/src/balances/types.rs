use super::{accounts::types::ParsedAta, dapps::types::ParsedPosition};

#[derive(serde::Serialize)]
pub struct BalancesResponse {
    pub accounts: Vec<ParsedAta>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub raydium: Option<Vec<ParsedPosition>>,
}
