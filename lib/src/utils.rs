use crate::{Error, Page};
use std::env;

pub fn clean_string(s: String) -> String {
    s.trim_matches('\0').trim_matches('"').to_string()
}

pub fn get_rpc() -> String {
    env::var("RPC").unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string())
}

pub fn create_batches<T: Clone>(
    arr: &Vec<T>,
    batch_size: usize,
    limit: Option<u32>,
) -> Vec<Vec<T>> {
    let mut batches: Vec<Vec<T>> = Vec::new();
    let mut total_elements = 0;

    let limit = limit.map(|l| l as usize);

    for i in (0..arr.len()).step_by(batch_size) {
        let mut batch = arr[i..std::cmp::min(i + batch_size, arr.len())].to_vec();

        if let Some(limit) = limit {
            let remaining_limit = limit - total_elements;

            if batch.len() > remaining_limit {
                batch = batch[..remaining_limit].to_vec();
                batches.push(batch);
                break;
            }
        }

        total_elements += batch.len();
        batches.push(batch);
    }

    batches
}

pub fn parse_page(index: Option<&str>) -> Result<Option<Page>, Error> {
    if index.is_none() {
        return Ok(None);
    }

    let split: Vec<&str> = index.unwrap().split('-').collect();

    if split.len() != 2 {
        return Err(Error::InvalidIndex);
    }

    let start_idx = match split[0].parse::<usize>() {
        Ok(x) => x,
        _ => return Err(Error::InvalidIndex),
    };
    let end_idx = match split[1].parse::<usize>() {
        Ok(y) => y,
        _ => return Err(Error::InvalidIndex),
    };

    if end_idx < start_idx {
        return Err(Error::InvalidIndex);
    }

    Ok(Some(Page { start_idx, end_idx }))
}
