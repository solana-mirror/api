use std::str::FromStr;

use reqwest::Client;
use lib::{
    coingecko::CoingeckoClient,
    chart::{get_chart_data, types::ChartDataWithPrice, Timeframe},
    client::SolanaMirrorClient,
    utils::get_rpc,
    Error::{TooManyRequests, FetchError, InvalidAddress, InvalidTimeframe, ParseError},
};
use rocket::{http::Status, serde::json::Json};
use spl_token::solana_program::pubkey::Pubkey;

#[get("/chart/<address>/<timeframe>")]
pub async fn chart_handler(
    address: &str,
    timeframe: &str,
) -> Result<Json<Vec<ChartDataWithPrice>>, Status> {

    // Gets the last character of the timeframe string (either "d" or "h")
    let timeframe_str = &timeframe[timeframe.len() - 1..];
    let parsed_timeframe = match Timeframe::new(timeframe_str) {
        Some(parsed_timeframe) => parsed_timeframe,
        None => return Err(Status::BadRequest),
    };

    // Gets the rest of the timeframe string (the amount of hours/days)
    let range = match timeframe[..timeframe.len() - 1].parse::<u8>() {
        Ok(range) => {
            if timeframe_str.to_lowercase() == "h" && range > 24 * 90 {
                return Err(Status::BadRequest);
            }
            range
        }
        Err(_) => return Err(Status::BadRequest),
    };

    let pubkey = match Pubkey::from_str(address) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err(Status::BadRequest),
    };

    let reqwest = Client::new();
    let coingecko = CoingeckoClient::from_client(&reqwest);
    let client = SolanaMirrorClient::from_client(&reqwest, get_rpc());

    let chart_data = get_chart_data(&client, &coingecko, &pubkey, parsed_timeframe, range).await;

    match chart_data {
        Ok(data) => Ok(Json(data)),
        Err(err) => {
            let status_code = match err {
                ParseError => Status::InternalServerError,
                TooManyRequests => Status::TooManyRequests,
                FetchError => Status::InternalServerError,
                InvalidAddress => Status::BadRequest,
                InvalidTimeframe => Status::BadRequest,
                _ => Status::InternalServerError
            };
            Err(status_code)
        }
    }
}
