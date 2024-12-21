use std::str::FromStr;

use futures::future::join_all;
use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use spl_token::id as spl_token_id;

use crate::{
    client::{
        types::AccountData, GetTokenAccountsByOwnerConfig, GetTokenAccountsByOwnerFilter,
        SolanaMirrorClient,
    },
    coingecko::get_coingecko_id,
    price::get_price,
    types::FormattedAmount,
    utils::{fetch_image, fetch_metadata},
    Error, SOL_ADDRESS, SOL_IMAGE,
};

use types::ParsedAta;

pub mod types;

/// Fetches the token accounts associated with the given address and parses them.
pub async fn get_parsed_accounts(
    client: &SolanaMirrorClient,
    address: &Pubkey,
) -> Result<Vec<ParsedAta>, Error> {
    let accounts = get_accounts(client, address).await?;
    let parse_futures = accounts
        .iter()
        .map(|account| parse_account(client, account));

    let parsed_results = join_all(parse_futures).await;

    let mut parsed_accounts: Vec<ParsedAta> = Vec::new();
    for result in parsed_results {
        match result {
            Ok(parsed_account) => parsed_accounts.push(parsed_account),
            Err(e) => return Err(e),
        }
    }

    parsed_accounts.push(get_solana(client, address).await);
    Ok(parsed_accounts)
}

/// Fetches the SOL account associated with the given address.
async fn get_solana(client: &SolanaMirrorClient, pubkey: &Pubkey) -> ParsedAta {
    let price = get_price(client, Pubkey::from_str(SOL_ADDRESS).unwrap(), Some(9)).await;

    let amount = client.get_balance(pubkey, None).await.unwrap_or(0);

    let formatted = amount as f64 / LAMPORTS_PER_SOL as f64;

    ParsedAta {
        mint: SOL_ADDRESS.to_string(),
        ata: pubkey.to_string(),
        coingecko_id: Some("wrapped-solana".to_string()),
        decimals: 9,
        name: "Solana".to_string(),
        symbol: "SOL".to_string(),
        image: SOL_IMAGE.to_string(),
        price,
        balance: FormattedAmount {
            amount: amount.to_string(),
            formatted,
        },
    }
}

/// Fetches the token accounts associated with the given address.
async fn get_accounts(
    client: &SolanaMirrorClient,
    pubkey: &Pubkey,
) -> Result<Vec<AccountData>, Error> {
    let accounts = client
        .get_token_accounts_by_owner(
            pubkey,
            Some(GetTokenAccountsByOwnerFilter {
                program_id: spl_token_id().to_string(),
            }),
            Some(GetTokenAccountsByOwnerConfig {
                commitment: None,
                min_context_slot: None,
                data_slice: None,
                encoding: Some("jsonParsed".to_string()),
            }),
        )
        .await?;

    Ok(accounts.result.value)
}

/// Parses the given account.
async fn parse_account(
    client: &SolanaMirrorClient,
    account: &AccountData,
) -> Result<ParsedAta, Error> {
    let data = &account.account.data;
    let info = &data.parsed.info;
    let mint = &info.mint;

    let metadata = fetch_metadata(client, mint).await;

    let ata = &account.pubkey;
    let decimals = info.token_amount.decimals;
    let amount = info.token_amount.amount.parse::<u64>().unwrap();
    let formatted = info.token_amount.ui_amount;

    let mint_pubkey = Pubkey::from_str(mint).unwrap();
    let price = get_price(client, mint_pubkey, Some(decimals)).await;

    let coingecko_id = get_coingecko_id(mint).await;
    let image = fetch_image(&metadata).await;

    Ok(ParsedAta {
        mint: mint.to_string(),
        ata: ata.to_string(),
        coingecko_id,
        decimals,
        name: metadata.name,
        symbol: metadata.symbol,
        image,
        price,
        balance: FormattedAmount {
            amount: amount.to_string(),
            formatted,
        },
    })
}
