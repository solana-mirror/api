use std::time::{SystemTime, UNIX_EPOCH};

use solana_sdk::pubkey::Pubkey;
use types::{ChartData, ChartDataWithPrice};

use crate::{
    client::SolanaMirrorClient,
    transactions::{get_parsed_transactions, types::ParsedTransaction},
    Error,
};

#[derive(Debug)]
pub enum Timeframe {
    Hour,
    Day,
}

pub mod types;

impl Timeframe {
    pub fn from_str(timeframe: &str) -> Option<Self> {
        match timeframe.to_lowercase().as_str() {
            "h" => Some(Self::Hour),
            "d" => Some(Self::Day),
            _ => None,
        }
    }

    pub fn to_string(timeframe: Self) -> String {
        match timeframe {
            Self::Hour => String::from("h"),
            Self::Day => String::from("d"),
        }
    }

    pub fn to_seconds(timeframe: Self) -> i64 {
        match timeframe {
            Self::Hour => 3600,
            Self::Day => 86400,
        }
    }
}

pub async fn get_chart_data(
    client: &SolanaMirrorClient,
    pubkey: &Pubkey,
    timeframe: Timeframe,
    range: u8,
) -> Result<Vec<ChartData>, Error> {
    let txs = get_parsed_transactions(client, pubkey).await?;
    let states = get_balance_states(&txs);
    let filtered_states = filter_balance_states(states, timeframe, range);

    Ok(filtered_states)
}

fn get_balance_states(txs: &Vec<ParsedTransaction>) -> Vec<ChartData> {
    let mut states: Vec<ChartData> = Vec::new();

    for tx in txs {
        let mut state = ChartData {
            timestamp: tx.block_time,
            ..Default::default()
        };

        // Clone the last state if there's any
        if let Some(last_state) = states.last() {
            state.balances = last_state.balances.clone();
        }

        for (mint, formatted_balance) in &tx.balances {
            if formatted_balance.post.formatted == 0.0 {
                state.balances.remove(mint);
            } else {
                state
                    .balances
                    .insert(mint.to_string(), formatted_balance.post.clone());
            }
        }

        states.push(state);
    }

    states
}

fn filter_balance_states(
    states: Vec<ChartData>,
    timeframe: Timeframe,
    range: u8,
) -> Vec<ChartData> {
    let t_seconds = Timeframe::to_seconds(timeframe);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64;

    // We're pushing to _states to make sure all the states up to now are included
    // then we push this last state to the filtered states array
    let mut _states = states.clone();
    _states.push(ChartData {
        timestamp: now,
        ..states.last().unwrap().clone()
    });
    let mut filtered_states: Vec<ChartData> = Vec::new();

    let final_t = (now as f64 / t_seconds as f64).floor() as i64 * t_seconds;
    let initial_t = final_t - (range as i64 * t_seconds);

    let mut last_idx = 0;

    for i in 0..range {
        let t = initial_t + (i as i64 * t_seconds);

        for j in last_idx.._states.len() {
            if _states[j].timestamp >= t {
                if j == 0 {
                    break;
                }

                let state_to_push = ChartData {
                    timestamp: t,
                    balances: _states[j - 1].balances.clone(),
                };

                filtered_states.push(state_to_push);
                last_idx = j;
                break;
            }
        }

        // Fill empty periods
        let last_state = filtered_states.last().unwrap();
        if !filtered_states.is_empty() && last_state.timestamp != t {
            let state_to_push = ChartData {
                timestamp: t,
                ..last_state.clone()
            };

            filtered_states.push(state_to_push);
        }
    }

    // We can simply push "now" after it did the checks
    filtered_states.push(_states.last().unwrap().clone());

    filtered_states
}

#[allow(dead_code)]
fn get_price_states(_states: Vec<ChartData>) -> Vec<ChartDataWithPrice> {
    todo!()
}
