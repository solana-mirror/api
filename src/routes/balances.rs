use std::str::FromStr;

use rocket::{futures::future::join_all, http::Status, serde::json::Json};
use solana_sdk::pubkey::Pubkey;

use lib::{
    balances::{
        accounts::get_parsed_accounts,
        dapps::{raydium::get_raydium_position, types::ParsedPosition},
        types::{AllBalances, BalancesResponse},
    },
    client::SolanaMirrorClient,
    utils::get_rpc,
    Error,
};

#[get("/balances/<address>?<showApps>")]
pub async fn accounts_handler(
    address: &str,
    #[allow(non_snake_case)] showApps: Option<bool>,
) -> Result<Json<BalancesResponse>, Status> {
    let show_apps = showApps;

    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Status::BadRequest),
    };

    let client = SolanaMirrorClient::new(get_rpc());
    let parsed_accounts_results = get_parsed_accounts(&client, &pubkey).await;

    let parsed_accounts = match parsed_accounts_results {
        Ok(accounts) => accounts,
        Err(err) => {
            let status_code = match err {
                Error::InvalidAddress => Status::BadRequest,
                Error::TooManyRequests => Status::TooManyRequests,
                _ => Status::InternalServerError,
            };
            return Err(status_code);
        }
    };

    let (position_accounts, filtered_parsed_accounts): (Vec<_>, Vec<_>) = parsed_accounts
        .into_iter()
        .partition(|account| account.balance.amount == "1");

    if show_apps == Some(false) {
        return Ok(Json(BalancesResponse::AccountsOnly(
            filtered_parsed_accounts,
        )));
    }

    let position_mints: Vec<&str> = position_accounts
        .iter()
        .filter(|x| (x.balance.amount == "1"))
        .map(|x| x.mint.as_str())
        .collect();

    let parse_raydium_position_futures: Vec<_> = position_mints
        .iter()
        .map(|&mint| get_raydium_position(&client, mint))
        .collect();

    let parsed_raydium_results: Vec<Result<ParsedPosition, Error>> = join_all(parse_raydium_position_futures).await;

    let mut parsed_raydium_positions: Vec<ParsedPosition> = Vec::new();
    for result in parsed_raydium_results {
        match result {
            Ok(parsed_position) => parsed_raydium_positions.push(parsed_position),
            Err(err) => {
                let status_code = match err {
                    Error::InvalidAddress => Status::BadRequest,
                    Error::TooManyRequests => Status::TooManyRequests,
                    _ => Status::InternalServerError,
                };
                return Err(status_code);
            }
        }
    }
    Ok(Json(BalancesResponse::All(
        AllBalances {
            accounts: filtered_parsed_accounts,
            raydium: parsed_raydium_positions,
        }
    )))
}
