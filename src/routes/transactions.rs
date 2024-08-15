use lib::client::SolanaMirrorClient;
use lib::transactions::get_parsed_transactions;
use lib::transactions::types::ParsedTransaction;
use lib::utils::get_rpc;
use lib::Error;
use rocket::{http::Status, serde::json::Json};

#[get("/transactions/<address>")]
pub async fn transactions_handler(address: &str) -> Result<Json<Vec<ParsedTransaction>>, Status> {
    let client = SolanaMirrorClient::new(get_rpc());
    let parsed_transactions = get_parsed_transactions(&client, address.to_string()).await;

    match parsed_transactions {
        Ok(txs) => Ok(Json(txs)),
        Err(err) => {
            let status_code = match err {
                Error::ParseError => Status::InternalServerError,
                Error::FetchError => Status::InternalServerError,
                Error::InvalidAddress => Status::BadRequest,
            };
            Err(status_code)
        }
    }
}
