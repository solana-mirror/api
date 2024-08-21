use std::{collections::HashMap, fs::File, io::BufReader};

use reqwest::Client;
use serde::Deserialize;
use serde_json::from_reader;

use crate::{chart::types::GetCoinMarketChartParams, Error};

const BASE_URL: &str = "https://api.coingecko.com/api/v3";

#[derive(Deserialize, Debug)]
pub struct CoingeckoToken {
    pub name: String,
    pub id: String,
    pub symbol: String,
}

pub type CoingeckoData = HashMap<String, CoingeckoToken>;

pub async fn get_coingecko_data() -> Result<CoingeckoData, Box<dyn std::error::Error>> {
    let file = match File::open("lib/src/coingecko.json") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return Err(Box::new(e));
        }
    };

    let reader = BufReader::new(file);

    match from_reader(reader) {
        Ok(data) => Ok(data),
        Err(e) => {
            eprintln!("Failed to parse file: {}", e);
            return Err(Box::new(e));
        }
    }
}

pub async fn get_coingecko_id(mint: &str) -> Option<String> {
    match get_coingecko_data().await {
        Ok(data) => match data.get(mint) {
            Some(token) => Some(token.id.clone()),
            None => None,
        },
        Err(_) => None,
    }
}

pub struct CoingeckoClient {
    pub inner_client: Client,
}

impl CoingeckoClient {
    pub fn new() -> Self {
        Self {
            inner_client: Client::new(),
        }
    }

    pub fn from_reqwest_client(inner_client: Client) -> Self {
        return Self { inner_client };
    }

    pub async fn get_coin_market_chart(
        &self,
        params: GetCoinMarketChartParams,
    ) -> Result<Vec<(u64, f64)>, Error> {
        let endpoint = format!("{}/coins/{}/market_chart/range", BASE_URL, params.id);

        let response = self
            .inner_client
            .get(&endpoint)
            .query(&[
                ("vs_currency", params.vs_currency),
                ("from", params.from.to_string()),
                ("to", params.to.to_string()),
            ])
            .send()
            .await
            .unwrap();

        if response.status().is_success() {
            let json: serde_json::Value = response.json().await.unwrap();
            let prices = json["prices"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .map(|p| {
                    (
                        p[0].as_u64().unwrap_or_default(),
                        p[1].as_f64().unwrap_or_default(),
                    )
                })
                .collect();

            Ok(prices)
        } else {
            Err(Error::FetchError)
        }
    }
}
