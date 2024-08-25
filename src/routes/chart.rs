use std::str::FromStr;

use lib::{
    chart::{get_chart_data, get_price_states, types::ChartDataWithPrice, Timeframe},
    client::SolanaMirrorClient,
    utils::get_rpc,
    Error::{FetchError, InvalidAddress, InvalidTimeframe, ParseError},
};
use rocket::{http::Status, serde::json::Json};
use spl_token::solana_program::pubkey::Pubkey;

#[get("/chart/<address>/<timeframe>")]
pub async fn chart_handler(
    address: &str,
    timeframe: &str,
) -> Result<Json<Vec<ChartDataWithPrice>>, Status> {
    let timeframe_str = &timeframe[timeframe.len() - 1..];
    let parsed_timeframe = match Timeframe::from_str(timeframe_str) {
        Some(parsed_timeframe) => parsed_timeframe,
        None => return Err(Status::BadRequest),
    };

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

    let client = SolanaMirrorClient::new(get_rpc());
    let chart_data = get_chart_data(&client, &pubkey, parsed_timeframe, range).await;
    let chart_data_with_price = get_price_states(&client, chart_data.unwrap()).await;

    match chart_data_with_price {
        Ok(data) => Ok(Json(data)),
        Err(err) => {
            let status_code = match err {
                ParseError => Status::InternalServerError,
                FetchError => Status::InternalServerError,
                InvalidAddress => Status::BadRequest,
                InvalidTimeframe => Status::BadRequest,
            };
            Err(status_code)
        }
    }
}
