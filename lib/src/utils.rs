use dotenv::dotenv;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;
use std::env;

pub fn clean_string(s: String) -> String {
    s.trim_matches('\0').trim_matches('"').to_string()
}

pub fn get_rpc() -> String {
    dotenv().ok();
    env::var("RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

pub async fn get_token_data(connection: &RpcClient, token: &Pubkey) -> Option<Mint> {
    let account_info = connection.get_account(token).await.unwrap();

    match Mint::unpack(&account_info.data) {
        Ok(mint) => Some(mint),
        Err(_) => None,
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
