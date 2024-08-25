use dotenv::dotenv;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::env;

use crate::{
    client::{types::Decimals, GetDecimalsConfig, JsonRpcError, SolanaMirrorClient},
    Error,
};

pub fn clean_string(s: String) -> String {
    s.trim_matches('\0').trim_matches('"').to_string()
}

pub fn get_rpc() -> String {
    dotenv().ok();
    env::var("RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

pub async fn get_token_data(
    client: &SolanaMirrorClient,
    token: &Pubkey,
) -> Result<Option<Decimals>, Error> {
    let response = client
        .get_decimals(
            token,
            Some(GetDecimalsConfig {
                commitment: Some("confirmed".to_string()),
            }),
        )
        .await?;

    if let Some(result) = response.result {
        Ok(Some(result.value))
    } else {
        Ok(None)
    }
}

pub fn create_batches<T: Clone>(
    arr: &Vec<T>,
    batch_size: usize,
    limit: Option<u32>,
) -> Vec<Vec<T>> {
    let mut batches: Vec<Vec<T>> = Vec::new();
    let mut total_elements = 0;

    let limit = limit.map(|l| l as usize);

    for i in (0..arr.len()).step_by(batch_size) {
        let mut batch = arr[i..std::cmp::min(i + batch_size, arr.len())].to_vec();

        if let Some(limit) = limit {
            let remaining_limit = limit - total_elements;

            if batch.len() > remaining_limit {
                batch = batch[..remaining_limit].to_vec();
                batches.push(batch);
                break;
            }
        }

        total_elements += batch.len();
        batches.push(batch);
    }

    batches
}

pub fn parse_json_rpc_error(res: Value) -> Option<JsonRpcError> {
    match serde_json::from_value::<JsonRpcError>(res) {
        Ok(jsonrpc_error) => Some(jsonrpc_error),
        Err(_) => None,
    }
}
