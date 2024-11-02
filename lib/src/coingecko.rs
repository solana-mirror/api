use std::{collections::HashMap, env, fs::File, io::BufReader};

use reqwest::Client;
use serde::Deserialize;
use serde_json::{from_reader, Value};

use crate::{chart::types::GetCoinMarketChartParams, Error};

const BASE_URL: &str = "https://api.coingecko.com/api/v3";

#[derive(Deserialize, Debug)]
pub struct CoingeckoToken {
    pub name: String,
    pub id: String,
    pub symbol: String,
}

pub type CoingeckoData = HashMap<String, CoingeckoToken>;

/// Reads the coingecko.json file with all the coingecko IDs available
pub async fn get_coingecko_data() -> Result<CoingeckoData, Error> {
    let file = match File::open("lib/src/coingecko.json") {
        Ok(file) => file,
        Err(e) => {
            eprintln!("Failed to open file: {}", e);
            return Err(Error::ParseError);
        }
    };

    let reader = BufReader::new(file);

    match from_reader(reader) {
        Ok(data) => Ok(data),
        Err(e) => {
            eprintln!("Failed to parse file: {}", e);
            Err(Error::ParseError)
        }
    }
}

/// Returns the coingecko ID from a mint from the coingecko.json file
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
    pub api_key: Option<String>,
}

impl CoingeckoClient {
    pub fn new() -> Self {
        let api_key = match env::var("COINGECKO_API_KEY") {
            Ok(key) => Some(key),
            _ => None,
        };

        Self {
            inner_client: Client::new(),
            api_key,
        }
    }

    pub fn from_client(inner_client: &Client) -> Self {
        let api_key = match env::var("COINGECKO_API_KEY") {
            Ok(key) => Some(key),
            _ => None,
        };

        Self {
            inner_client: inner_client.clone(),
            api_key,
        }
    }

    async fn make_request(&self, endpoint: &str, query: &[(&str, String)]) -> Result<Value, Error> {
        let request = self.inner_client.get(endpoint).query(query);

        match request.send().await {
            Ok(response) => {
                let res = response
                    .json::<Value>()
                    .await
                    .map_err(|_| Error::ParseError)?;
                Ok(res)
            }
            Err(_) => Err(Error::FetchError),
        }
    }

    pub async fn get_coin_market_chart(
        &self,
        params: GetCoinMarketChartParams,
    ) -> Result<Vec<(u64, f64)>, Error> {
        let endpoint = format!("{}/coins/{}/market_chart/range", BASE_URL, params.id);

        let mut query = vec![
            ("vs_currency", params.vs_currency),
            ("from", params.from.to_string()),
            ("to", params.to.to_string()),
        ];

        if let Some(key) = &self.api_key {
            query.push(("x_cg_demo_api_key", key.clone()));
        };

        let res = self.make_request(&endpoint, &query).await?;

        // TODO: set a type for the response and deserialize with serde
        let prices = res["prices"]
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
    }
}
