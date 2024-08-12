use std::env;

use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::{program_pack::Pack, pubkey::Pubkey};
use spl_token::state::Mint;

pub fn clean_string(s: String) -> String {
    s.trim_matches('\0').trim_matches('"').to_string()
}

pub fn get_rpc() -> String {
    env::var("RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

pub async fn get_token_data(connection: &RpcClient, token: &Pubkey) -> Option<Mint> {
    let account_info = connection.get_account(token).await.unwrap();

    match Mint::unpack(&account_info.data) {
        Ok(mint) => Some(mint),
        Err(_) => None,
    }
}
