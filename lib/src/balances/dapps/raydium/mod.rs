use serde::de::DeserializeOwned;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

use crate::{
    balances::dapps::types::{ProtocolInfo, TokenPosition},
    client::{GetAccountDataConfig, SolanaMirrorClient},
    price::get_price,
    types::{FormattedAmount, FormattedAmountWithPrice},
    utils::{calculate_concentrated_liquidity_amounts, fetch_image, fetch_metadata},
    Error,
};

use super::types::ParsedPosition;
use types::{Pool, Position};

pub mod types;

const RAYDIUM_CL_PROGRAM_ID: &str = "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK";

pub async fn get_parsed_positions(
    client: &SolanaMirrorClient,
    mint_protocol: &str,
) -> Result<ParsedPosition, Error> {
    let position_address = get_position_address(mint_protocol).unwrap();
    let position = get_position_data(client, &position_address).await?;
    let pool = get_pool_data(client, &position.pool_id).await?;

    let (amount_a, amount_b) = calculate_concentrated_liquidity_amounts(
        position.liquidity,
        position.tick_lower,
        position.tick_upper,
        pool.sqrt_price_x64,
    );

    let metadata_protocol = fetch_metadata(client, mint_protocol).await;
    let image_protocol = fetch_image(&metadata_protocol).await;

    let mint_a = pool.mint_a;
    let mint_b = pool.mint_b;

    let metadata_token_a = fetch_metadata(client, &mint_a.to_string()).await;
    let metadata_token_b = fetch_metadata(client, &mint_b.to_string()).await;
    let image_a = fetch_image(&metadata_token_a).await;
    let image_b = fetch_image(&metadata_token_b).await;

    let decimals_a = pool.mint_decimals_a;
    let decimals_b = pool.mint_decimals_b;

    let formatted_amount_a = amount_a / (10_f64.powi(decimals_a as i32));
    let formatted_amount_b = amount_b / (10_f64.powi(decimals_b as i32));

    let price_a = get_price(client, mint_a, Some(decimals_a)).await;
    let price_b = get_price(client, mint_b, Some(decimals_b)).await;

    let total_value_usd = match (price_a, price_b) {
        (Some(price_a), Some(price_b)) => {
            Some((formatted_amount_a * price_a) + (formatted_amount_b * price_b))
        }
        (Some(price_a), None) => Some(formatted_amount_a * price_a),
        (None, Some(price_b)) => Some(formatted_amount_b * price_b),
        (None, None) => None,
    };

    let parsed_position = ParsedPosition {
        total_value_usd,
        protocol: ProtocolInfo {
            name: metadata_protocol.name,
            symbol: metadata_protocol.symbol,
            image: image_protocol,
            program_id: RAYDIUM_CL_PROGRAM_ID.to_string(),
        },
        token_a: TokenPosition {
            mint: mint_a.to_string(),
            name: metadata_token_a.name,
            symbol: metadata_token_a.symbol,
            image: image_a,
            amount: FormattedAmountWithPrice {
                amount: FormattedAmount {
                    amount: amount_a.to_string(),
                    formatted: formatted_amount_a,
                },
                price: price_a.unwrap(),
            },
        },
        token_b: TokenPosition {
            mint: mint_b.to_string(),
            name: metadata_token_b.name,
            symbol: metadata_token_b.symbol,
            image: image_b,
            amount: FormattedAmountWithPrice {
                amount: FormattedAmount {
                    amount: amount_b.to_string(),
                    formatted: formatted_amount_b,
                },
                price: price_b.unwrap(),
            },
        },
        // TODO: not sure
        fee_tier: String::new(),
    };

    Ok(parsed_position)
}

pub fn get_position_address(nft_mint: &str) -> Result<Pubkey, Error> {
    let nft_mint_pubkey = Pubkey::from_str(nft_mint).unwrap();
    let program_id = Pubkey::from_str(RAYDIUM_CL_PROGRAM_ID).unwrap();
    let seeds = &[b"position", nft_mint_pubkey.as_ref()];
    let (position_address, _bump) = Pubkey::find_program_address(seeds, &program_id);
    Ok(position_address)
}

async fn get_position_data(
    client: &SolanaMirrorClient,
    position_address: &Pubkey,
) -> Result<Position, Error> {
    let encoded_position = match client
        .get_account_info(
            position_address,
            Some(GetAccountDataConfig {
                commitment: None,
                encoding: Some("jsonParsed".to_string()),
            }),
        )
        .await
    {
        Ok(encoded_position) => encoded_position,
        Err(_) => return Ok(Position::default()),
    };
    decode_data(&encoded_position)
}

async fn get_pool_data(client: &SolanaMirrorClient, pool_id: &Pubkey) -> Result<Pool, Error> {
    let encoded_pool = client
        .get_account_info(
            pool_id,
            Some(GetAccountDataConfig {
                commitment: None,
                encoding: Some("jsonParsed".to_string()),
            }),
        )
        .await?;
    decode_data(&encoded_pool)
}

fn decode_data<T: DeserializeOwned>(data: &[u8]) -> Result<T, Error> {
    bincode::deserialize(data).map_err(|_| Error::ParseError)
}
