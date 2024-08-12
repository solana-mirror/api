use std::{collections::HashMap, fs::File, io::BufReader};

use serde::Deserialize;
use serde_json::from_reader;

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
