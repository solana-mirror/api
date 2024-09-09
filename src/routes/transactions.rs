use lib::client::SolanaMirrorClient;
use lib::transactions::get_parsed_transactions;
use lib::transactions::types::ParsedTransaction;
use lib::utils::get_rpc;
use lib::Error::{InvalidAddress, TooManyRequests};
use lib::Page;
use rocket::{http::Status, serde::json::Json};
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[get("/transactions/<address>?<page>")]
pub async fn transactions_handler(
    address: &str, 
    page: &str
) -> Result<Json<Vec<ParsedTransaction>>, Status> {
    let client = SolanaMirrorClient::new(get_rpc());

    let split: Vec<&str> = page.split('-').collect();

    if split.len() != 2 {
        return Err(Status::BadRequest)
    }

    let start_idx = match split[0].parse::<usize>() {
        Ok(x) => x,
        _ => return Err(Status::BadRequest),
    };
    let end_idx = match split[1].parse::<usize>() {
        Ok(y) => y,
        _ => return Err(Status::BadRequest),
    };

    if end_idx < start_idx {
        return Err(Status::BadRequest)
    }
        
    let page = Page {
        start_idx,
        end_idx,
    };

    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Status::BadRequest),
    };

    let parsed_transactions = get_parsed_transactions(&client, &pubkey, Some(page)).await;

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
