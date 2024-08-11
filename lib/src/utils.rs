use std::env;

pub fn clean_string(s: String) -> String {
    s.trim_matches('\0').trim_matches('"').to_string()
}

pub fn get_rpc() -> String {
    env::var("RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}
