use std::str::FromStr;

use lib::{
    accounts::{get_parsed_accounts, types::ParsedAta},
    utils::get_rpc,
    Error,
};
use rocket::{http::Status, serde::json::Json};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

#[get("/accounts/<address>")]
pub async fn accounts_handler(address: &str) -> Result<Json<Vec<ParsedAta>>, Status> {
    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Status::BadRequest),
    };

    let client = RpcClient::new(get_rpc());
    let parsed_accounts = get_parsed_accounts(&client, &pubkey).await;

    match parsed_accounts {
        Ok(parsed_accounts) => Ok(Json(parsed_accounts)),
        Err(err) => {
            let status_code = match err {
                Error::ParseError => Status::InternalServerError,
                Error::FetchError => Status::InternalServerError,
                Error::InvalidAddress => Status::BadRequest,
                _ => Status::InternalServerError,
            };
            Err(status_code)
        }
    }
}
