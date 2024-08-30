use lib::client::SolanaMirrorClient;
use lib::transactions::get_parsed_transactions;
use lib::transactions::types::ParsedTransaction;
use lib::utils::get_rpc;
use lib::Error::{InvalidAddress, TooManyRequests};
use rocket::{http::Status, serde::json::Json};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[get("/transactions/<address>")]
pub async fn transactions_handler(address: &str) -> Result<Json<Vec<ParsedTransaction>>, Status> {
    let client = SolanaMirrorClient::new(get_rpc());

    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Status::BadRequest),
    };

    let parsed_transactions = get_parsed_transactions(&client, &pubkey).await;

    match parsed_transactions {
        Ok(txs) => Ok(Json(txs)),
        Err(err) => {
            let status_code = match err {
                InvalidAddress => Status::BadRequest,
                TooManyRequests => Status::TooManyRequests,
                _ => Status::InternalServerError,
            };
            Err(status_code)
        }
    }
}
