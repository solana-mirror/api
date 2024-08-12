use crate::{
    coingecko::get_coingecko_id,
    consts::USDC_ADDRESS,
    price::get_price,
    utils::{clean_string, get_rpc},
};
use mpl_token_metadata::{accounts::Metadata, programs::MPL_TOKEN_METADATA_ID};
use rocket::futures::stream::{self, StreamExt};
use solana_account_decoder::UiAccountData;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_request::TokenAccountsFilter,
    rpc_response::RpcKeyedAccount,
};
use solana_sdk::pubkey::Pubkey;
use spl_token::id as spl_token_id;
use std::{collections::HashMap, str::FromStr};
use types::{Balance, GetAccountsError, ImageResponse, ParsedAta, ParsedMetadata};

pub mod types;

/// Fetches the token accounts associated with the given address and parses them.
pub async fn get_parsed_accounts(address: String) -> Result<Vec<ParsedAta>, GetAccountsError> {
    let accounts = get_accounts(address).await?;

    stream::iter(accounts)
        .then(|acc| async { parse_account(acc).await })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect()
}

/// Fetches the token accounts associated with the given address.
pub async fn get_accounts(address: String) -> Result<Vec<RpcKeyedAccount>, GetAccountsError> {
    let client = RpcClient::new(get_rpc());
    let pubkey = Pubkey::from_str(&address);

    match pubkey {
        Ok(pubkey) => {
            let accounts = client
                .get_token_accounts_by_owner(
                    &pubkey,
                    TokenAccountsFilter::ProgramId(spl_token_id()),
                )
                .await;

            match accounts {
                Ok(accounts) => Ok(accounts),
                Err(_) => Err(GetAccountsError::FetchError),
            }
        }
        Err(_) => Err(GetAccountsError::InvalidAddress),
    }
}

/// Parses the given account.
pub async fn parse_account(account: RpcKeyedAccount) -> Result<ParsedAta, GetAccountsError> {
    if let UiAccountData::Json(parsed_account) = account.account.data {
        let info = parsed_account.parsed["info"].as_object().unwrap();

        let metadata = fetch_metadata(&info["mint"].as_str().unwrap()).await;

        let mint = clean_string(info["mint"].as_str().unwrap().to_string());
        let ata = account.pubkey.to_string();

        let decimals = info["tokenAmount"]["decimals"].as_u64().unwrap();
        let amount = info["tokenAmount"]["amount"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let formatted = info["tokenAmount"]["uiAmount"].as_f64().unwrap();

        let mint_pubkey = Pubkey::from_str(&mint).unwrap();
        let usdc_pubkey = Pubkey::from_str(USDC_ADDRESS).unwrap();
        let price = get_price(mint_pubkey, usdc_pubkey).await;

        let coingecko_id = get_coingecko_id(&mint).await;

        let image = fetch_image(&metadata).await;

        return Ok(ParsedAta {
            mint,
            ata,
            coingecko_id,
            decimals,
            name: metadata.name,
            symbol: metadata.symbol,
            image,
            price,
            balance: Balance { amount, formatted },
        });
    }

    Err(GetAccountsError::ParseError)
}

/// Fetches the metadata associated with the given mint address.
pub async fn fetch_metadata(mint_address: &str) -> ParsedMetadata {
    let client = RpcClient::new(get_rpc());

    let mint_pubkey = Pubkey::from_str(mint_address).unwrap();
    let mpl_program_id = Pubkey::from_str(MPL_TOKEN_METADATA_ID.to_string().as_str()).unwrap();

    // Get the metadata account address associated with the mint
    let metadata_pubkey = Pubkey::find_program_address(
        &[
            "metadata".as_ref(),
            &mpl_program_id.to_bytes(),
            &mint_pubkey.to_bytes(),
        ],
        &mpl_program_id,
    )
    .0;

    let account_data = client
        .get_account_data(&metadata_pubkey)
        .await
        .unwrap_or_default();

    match Metadata::safe_deserialize(&account_data) {
        Ok(metadata) => parse_metadata(metadata),
        Err(_) => ParsedMetadata::default(),
    }
}

/// Parses the given metadata.
fn parse_metadata(metadata: Metadata) -> ParsedMetadata {
    ParsedMetadata {
        name: clean_string(metadata.name),
        symbol: clean_string(metadata.symbol),
        uri: clean_string(metadata.uri),
    }
}

/// Fetches the image for each account
async fn fetch_image(metadata: &ParsedMetadata) -> String {
    let predefined_images = HashMap::from([
        ("USDC", "https://cryptologos.cc/logos/usd-coin-usdc-logo.png?v=032"),
        ("SOL", "https://cryptologos.cc/logos/solana-sol-logo.png?v=032"),
        ("RCL", "https://ipfs.io/ipfs/Qme9ErqmQaznzpfDACncEW48NyXJPFP7HgzfoNdto9xQ9P/02.jpg")
    ]);

    if let Some(&url) = predefined_images.get(metadata.symbol.as_str()) {
        return url.to_string();
    }

    if let Ok(response) = reqwest::get(&metadata.uri).await {
        if let Ok(image_response) = response.json::<ImageResponse>().await {
            return image_response.image;
        }
    }
    eprintln!("Failed to fetch the image. Returning a default image.");
    String::from("https://example.com/default-image.png")
}