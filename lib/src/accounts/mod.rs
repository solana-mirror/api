use crate::utils::{clean_string, get_rpc};
use mpl_token_metadata::{accounts::Metadata, programs::MPL_TOKEN_METADATA_ID};
use rocket::futures::stream::{self, StreamExt};
use solana_account_decoder::UiAccountData;
use solana_client::{
    nonblocking::rpc_client::RpcClient, rpc_request::TokenAccountsFilter,
    rpc_response::RpcKeyedAccount,
};
use solana_sdk::pubkey::Pubkey;
use spl_token::id as spl_token_id;
use std::str::FromStr;
use types::{Balance, GetAccountsError, ParsedAta};

pub mod types;

/**
 * Fetches and parses the token accounts associated with the given address.
 */
pub async fn get_parsed_accounts(address: String) -> Result<Vec<ParsedAta>, GetAccountsError> {
    let accounts = get_accounts(address).await?;

    let parsed_accounts: Result<Vec<ParsedAta>, GetAccountsError> = stream::iter(accounts)
        .then(|acc| async { parse_account(acc).await })
        .collect::<Vec<_>>()
        .await
        .into_iter()
        .collect();

    parsed_accounts
}

/**
 * Fetches the token accounts associated with the given address.
 */
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

/**
 * Parses the given token account.
 */
pub async fn parse_account(account: RpcKeyedAccount) -> Result<ParsedAta, GetAccountsError> {
    if let UiAccountData::Json(parsed_account) = account.account.data {
        let info = parsed_account.parsed["info"].as_object().unwrap();

        let metadata = fetch_metadata(&info["mint"].as_str().unwrap())
            .await
            .unwrap();

        let mint = clean_string(info["mint"].as_str().unwrap().to_string());
        let ata = account.pubkey.to_string();

        let decimals = info["tokenAmount"]["decimals"].as_u64().unwrap();
        let amount = info["tokenAmount"]["amount"]
            .as_str()
            .unwrap()
            .parse::<u64>()
            .unwrap();
        let formatted = info["tokenAmount"]["uiAmount"].as_f64().unwrap();

        return Ok(ParsedAta {
            mint,
            ata,
            coingecko_id: None,
            decimals,
            name: clean_string(metadata.name),
            symbol: clean_string(metadata.symbol),
            image: clean_string(metadata.uri),
            price: 1.23, // Add Jup
            balance: Balance { amount, formatted },
        });
    }

    Err(GetAccountsError::ParseError)
}

/**
 * Fetches the metadata associated with the given mint address.
 */
pub async fn fetch_metadata(mint_address: &str) -> Result<Metadata, ()> {
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

    let account_data = client.get_account_data(&metadata_pubkey).await.unwrap();
    let metadata = Metadata::safe_deserialize(&account_data).unwrap();

    Ok(metadata)
}
