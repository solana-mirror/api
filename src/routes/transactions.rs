use lib::client::SolanaMirrorClient;
use lib::transactions::get_parsed_transactions;
use lib::transactions::types::TransactionResponse;
use lib::utils::{get_rpc, parse_page};
use lib::Error::{InvalidAddress, TooManyRequests};
use rocket::{http::Status, serde::json::Json};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[get("/transactions/<address>?<index>")]
pub async fn transactions_handler(
    address: &str,
    index: Option<&str>,
) -> Result<Json<TransactionResponse>, Status> {
    let client = SolanaMirrorClient::new(get_rpc());

    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Status::BadRequest),
    };

    let page = match parse_page(index) {
        Ok(p) => p,
        Err(_) => return Err(Status::BadRequest),
    };

    let parsed_transactions = get_parsed_transactions(&client, &pubkey, page).await;

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
