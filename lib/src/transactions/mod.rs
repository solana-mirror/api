use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use std::{collections::HashMap, str::FromStr};

pub mod types;

use crate::{
    client::{
        types::TokenBalance, GetSignaturesForAddressConfig, GetTransactionConfig,
        GetTransactionResponse, SolanaMirrorClient,
    },
    transactions::types::{BalanceChange, FormattedAmount, ParsedTransaction},
    utils::create_batches,
    Error, SOL_ADDRESS,
};

/// Get the parsed transactions for the given address
pub async fn get_parsed_transactions(
    client: &SolanaMirrorClient,
    address: String,
) -> Result<Vec<ParsedTransaction>, Error> {
    let signatures = get_signatures(client, &address).await?;

    let batches = create_batches(&signatures, 900, None);

    let mut txs: Vec<GetTransactionResponse> = Vec::new();

    for batch in batches {
        let transactions: Vec<GetTransactionResponse> = client
            .get_transactions(
                &batch,
                Some(GetTransactionConfig {
                    max_supported_transaction_version: Some(0),
                    commitment: None,
                    encoding: None,
                }),
            )
            .await?;

        txs.extend(transactions);
    }

    let parsed_transactions: Vec<ParsedTransaction> = txs
        .iter()
        .map(|tx| parse_transaction(tx, Pubkey::from_str(&address).unwrap()))
        .filter_map(|x| x.ok())
        .collect();

    Ok(parsed_transactions)
}

async fn get_signatures(client: &SolanaMirrorClient, address: &str) -> Result<Vec<String>, Error> {
    // Validate the address
    Pubkey::from_str(address).map_err(|_| return Error::InvalidAddress)?;

    let mut before: Option<String> = None;
    let mut should_continue: bool = true;
    let mut signatures: Vec<String> = Vec::new();

    while should_continue {
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

fn parse_transaction(
    transaction: &GetTransactionResponse,
    signer: Pubkey,
) -> Result<ParsedTransaction, Error> {
    let mut balances: HashMap<String, BalanceChange> = HashMap::new();
    let tx = transaction.result.clone().unwrap();

    let owner_idx = tx
        .transaction
        .message
        .account_keys
        .iter()
        .position(|x| x.to_owned() == signer.to_string());

    match owner_idx {
        Some(idx) => {
            // Handle SOL
            let pre_sol = tx.meta.pre_balances[idx];
            let post_sol = tx.meta.post_balances[idx];

            if pre_sol != post_sol {
                balances.insert(
                    SOL_ADDRESS.to_string(),
                    BalanceChange {
                        pre: FormattedAmount {
                            amount: pre_sol,
                            formatted: pre_sol as f64 / LAMPORTS_PER_SOL as f64,
                        },
                        post: FormattedAmount {
                            amount: post_sol,
                            formatted: post_sol as f64 / LAMPORTS_PER_SOL as f64,
                        },
                    },
                );
            }

            // Handle SPL
            let pre_token_balances: Vec<TokenBalance> = tx
                .clone()
                .meta
                .pre_token_balances
                .into_iter()
                .filter(|x| x.owner == signer.to_string())
                .collect();

            let post_token_balances: Vec<TokenBalance> = tx
                .clone()
                .meta
                .post_token_balances
                .into_iter()
                .filter(|x| x.owner == signer.to_string())
                .collect();

            for pre_balance in pre_token_balances {
                if !balances.contains_key(pre_balance.mint.as_str()) {
                    balances.insert(pre_balance.mint.to_string(), BalanceChange::default());
                }

                let balance_change = balances.get_mut(pre_balance.mint.as_str()).unwrap();
                balance_change.pre = FormattedAmount {
                    amount: pre_balance.ui_token_amount.amount.parse::<u64>().unwrap(),
                    formatted: pre_balance.ui_token_amount.ui_amount.unwrap_or_default(),
                };
            }

            for post_balance in post_token_balances {
                if !balances.contains_key(post_balance.mint.as_str()) {
                    balances.insert(post_balance.mint.to_string(), BalanceChange::default());
                }

                let balance_change = balances.get_mut(post_balance.mint.as_str()).unwrap();
                balance_change.post = FormattedAmount {
                    amount: post_balance.ui_token_amount.amount.parse::<u64>().unwrap(),
                    formatted: post_balance.ui_token_amount.ui_amount.unwrap_or_default(),
                };
            }

            // Handle ixs
            let parsed_instructions: Vec<String> = tx
                .meta
                .log_messages
                .clone()
                .unwrap_or_default()
                .into_iter()
                .filter(|x| x.starts_with("Program log: Instruction: "))
                .map(|x| x.replace("Program log: Instruction: ", ""))
                .collect();

            Ok(ParsedTransaction {
                block_time: tx.block_time.unwrap_or_default(),
                signatures: tx.transaction.signatures,
                balances,
                parsed_instructions,
            })
        }
        None => return Err(Error::InvalidAddress),
    }
}
