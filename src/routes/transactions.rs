use lib::transactions::{
    get_parsed_transactions,
    types::{GetTransactionsError, ParsedTransaction},
};
use rocket::{http::Status, serde::json::Json};

#[get("/transactions/<address>")]
pub async fn transactions_handler(address: &str) -> Result<Json<Vec<ParsedTransaction>>, Status> {
    let parsed_transactions = get_parsed_transactions(address.to_string()).await;

    match parsed_transactions {
        Ok(parsed_transactions) => Ok(Json(parsed_transactions)),
        Err(err) => {
            let status_code = match err {
                GetTransactionsError::ParseError => Status::InternalServerError,
                GetTransactionsError::FetchError => Status::InternalServerError,
                GetTransactionsError::InvalidAddress => Status::BadRequest,
            };
            Err(status_code)
        }
    }
}