use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};

use crate::{
    client::{GetSignaturesForAddressConfig, GetTransactionConfig, SolanaMirrorClient},
    transactions::types::{BalanceChange, FormattedAmount, ParsedTransaction},
    Error,
};

pub mod types;

async fn get_signatures(client: &SolanaMirrorClient, address: &str) -> Result<Vec<String>, Error> {
    // Validate the address
    Pubkey::from_str(address).map_err(|_| return Error::InvalidAddress)?;

    let mut before: Option<String> = None;
    let mut should_continue: bool = true;
    let mut signatures: Vec<String> = Vec::new();

    while should_continue {
        // Fetch the signatures for the address
        let response = client
            .get_signatures_for_address(
                &address,
                Some(GetSignaturesForAddressConfig {
                    before: before.clone(),
                    until: None,
                    limit: None,
                    commitment: Some("confirmed".to_string()),
                }),
            )
            .await?;

        let raw_signatures = response.result;

        if raw_signatures.is_empty() {
            should_continue = false;
        } else {
            let mapped: Vec<String> = raw_signatures.iter().map(|x| x.signature.clone()).collect();

            if let Some(last_signature) = mapped.last() {
                before = Some(last_signature.to_string());
            }

            signatures.extend(mapped);
            if raw_signatures.len() < 1000 {
                should_continue = false;
            }
        }
    }

    Ok(signatures)
}

pub async fn get_parsed_transactions(
    client: &SolanaMirrorClient,
    address: String,
) -> Result<Vec<ParsedTransaction>, Error> {
    let signatures = get_signatures(client, &address).await?;

    let txs = client
        .get_transactions(
            signatures,
            Some(GetTransactionConfig {
                max_supported_transaction_version: Some(0),
                commitment: None,
                encoding: None,
            }),
        )
        .await?;

    println!("{:?}", txs.last());

    let transaction1 = ParsedTransaction {
        block_time: 1625097600,
        signatures: vec!["signature1".to_string(), "signature2".to_string()],
        logs: vec!["log1".to_string(), "log2".to_string()],
        balances: {
            let mut balances = HashMap::new();
            balances.insert(
                "address1".to_string(),
                BalanceChange {
                    pre: FormattedAmount {
                        amount: 1000,
                        formatted: 10.0,
                    },
                    post: FormattedAmount {
                        amount: 800,
                        formatted: 8.0,
                    },
                },
            );
            balances.insert(
                "address2".to_string(),
                BalanceChange {
                    pre: FormattedAmount {
                        amount: 500,
                        formatted: 5.0,
                    },
                    post: FormattedAmount {
                        amount: 700,
                        formatted: 7.0,
                    },
                },
            );
            balances
        },
        parsed_instructions: vec!["instruction1".to_string(), "instruction2".to_string()],
    };

    // return mocked data to avoid type errors
    Ok(vec![transaction1])
}
