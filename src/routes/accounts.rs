use lib::{
    accounts::{get_parsed_accounts, types::ParsedAta},
    Error,
};
use rocket::{http::Status, serde::json::Json};

#[get("/accounts/<address>")]
pub async fn accounts_handler(address: &str) -> Result<Json<Vec<ParsedAta>>, Status> {
    let parsed_accounts = get_parsed_accounts(address.to_string()).await;

    match parsed_accounts {
        Ok(parsed_accounts) => Ok(Json(parsed_accounts)),
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
