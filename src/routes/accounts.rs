use std::str::FromStr;

use lib::{
    accounts::{get_parsed_accounts, types::ParsedAta}, client::SolanaMirrorClient, utils::get_rpc, Error
};
use rocket::{http::Status, serde::json::Json};
use solana_sdk::pubkey::Pubkey;

#[get("/accounts/<address>")]
pub async fn accounts_handler(address: &str) -> Result<Json<Vec<ParsedAta>>, Status> {
    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Status::BadRequest),
    };


    let client = SolanaMirrorClient::new(get_rpc());
    let parsed_accounts = get_parsed_accounts(&client, &pubkey).await;

    match parsed_accounts {
        Ok(parsed_accounts) => Ok(Json(parsed_accounts)),
        Err(err) => {
            let status_code = match err {
                Error::InvalidAddress => Status::BadRequest,
                Error::TooManyRequests => Status::TooManyRequests,
                _ => Status::InternalServerError,
            };
            Err(status_code)
        }
    }
}
