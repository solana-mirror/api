use solana_program::native_token::LAMPORTS_PER_SOL;
use solana_sdk::pubkey::Pubkey;
use std::collections::HashMap;

pub mod types;

use crate::{
    client::{
        types::{TokenBalance, Transaction},
        GetSignaturesForAddressConfig, GetTransactionConfig, SolanaMirrorClient,
    }, transactions::types::{BalanceChange, FormattedAmount, ParsedTransaction}, utils::create_batches, Error, Page, SOL_ADDRESS
};

use self::types::TransactionResponse;

/// Get the parsed transactions for the given address
pub async fn get_parsed_transactions(
    client: &SolanaMirrorClient,
    pubkey: &Pubkey,
    page: Option<Page>,
) -> Result<TransactionResponse, Error> {
    let signatures = get_signatures(client, pubkey).await?;
    let batches = match page {
        Some(p) => {
            if p.start_idx >= signatures.len() {
                return Ok(TransactionResponse {
                    count: signatures.len(),
                    transactions: Vec::<ParsedTransaction>::new()
                });
            } else if p.end_idx >= signatures.len() {
                vec![signatures[p.start_idx..].to_vec()]
            } else {
                vec![signatures[p.start_idx..p.end_idx].to_vec()]
            }
        },
        None => create_batches(&signatures, 900, None)
    };

    let mut txs: Vec<Transaction> = Vec::new();

    for batch in batches {
        let transactions: Vec<crate::client::GetTransactionResponse> = client
            .get_transactions(
                &batch,
                Some(GetTransactionConfig {
                    max_supported_transaction_version: Some(0),
                    commitment: None,
                    encoding: None,
                }),
            )
            .await?;

        txs.extend(transactions.into_iter().filter_map(|tx| tx.result));
    }

    let mut parsed_transactions: Vec<ParsedTransaction> = txs
        .iter()
        .map(|tx| parse_transaction(tx, pubkey))
        .filter_map(|x| x.ok())
        .collect::<Vec<ParsedTransaction>>();

    parsed_transactions.sort_by_key(|x| x.block_time);
    
    Ok(TransactionResponse {
        transactions: parsed_transactions,
        count: signatures.len()
    })
}

async fn get_signatures(
    client: &SolanaMirrorClient,
    pubkey: &Pubkey,
) -> Result<Vec<String>, Error> {
    let mut before: Option<String> = None;
    let mut should_continue: bool = true;
    let mut signatures: Vec<String> = Vec::new();

    // Get signatures for the address until the length 
    // is not the max (1000) and set `before` accordingly
    while should_continue {
        let response = client
            .get_signatures_for_address(
                pubkey,
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
            let len = raw_signatures.len(); // Get the length before moving the value to the iter
            let mapped: Vec<String> = raw_signatures.into_iter().map(|x| x.signature).collect();

            if let Some(last_signature) = mapped.last() {
                before = Some(last_signature.to_string());
            }

            signatures.extend(mapped);
            if len < 1000 {
                should_continue = false;
            }
        }
    }

    Ok(signatures)
}

fn parse_transaction(tx: &Transaction, signer: &Pubkey) -> Result<ParsedTransaction, Error> {
    let mut balances: HashMap<String, BalanceChange> = HashMap::new();

    let owner_idx = match tx
        .transaction
        .message
        .account_keys
        .iter()
        .position(|x| *x == signer.to_string())
    {
        Some(idx) => idx,
        None => return Err(Error::InvalidAddress),
    };

    // Handle SOL
    let pre_sol = tx.meta.pre_balances[owner_idx];
    let post_sol = tx.meta.post_balances[owner_idx];

    if pre_sol != post_sol {
        balances.insert(
            SOL_ADDRESS.to_string(),
            BalanceChange {
                pre: FormattedAmount {
                    amount: pre_sol.to_string(),
                    formatted: pre_sol as f64 / LAMPORTS_PER_SOL as f64,
                },
                post: FormattedAmount {
                    amount: post_sol.to_string(),
                    formatted: post_sol as f64 / LAMPORTS_PER_SOL as f64,
                },
            },
        );
    }

    // Handle SPL
    let pre_token_balances: Vec<TokenBalance> = tx
        .meta
        .pre_token_balances
        .iter()
        .filter(|x| x.owner == signer.to_string())
        .cloned()
        .collect();

    let post_token_balances: Vec<TokenBalance> = tx
        .meta
        .post_token_balances
        .iter()
        .filter(|x| x.owner == signer.to_string())
        .cloned()
        .collect();

    for pre_balance in pre_token_balances {
        let balance_change = balances.entry(pre_balance.mint).or_insert(BalanceChange::default());

        balance_change.pre = FormattedAmount {
            amount: pre_balance.ui_token_amount.amount,
            formatted: pre_balance.ui_token_amount.ui_amount.unwrap_or_default(),
        };
    }

    for post_balance in post_token_balances {
        let balance_change = balances.entry(post_balance.mint).or_insert(BalanceChange::default());

        balance_change.post = FormattedAmount {
            amount: post_balance.ui_token_amount.amount,
            formatted: post_balance.ui_token_amount.ui_amount.unwrap_or_default(),
        };
    }

    // Handle ixs
    let parsed_instructions: Vec<String> = tx
        .meta
        .log_messages
        .as_ref()
        .unwrap_or(&vec![])
        .iter()
        .filter(|x| x.starts_with("Program log: Instruction: "))
        .map(|x| x.replace("Program log: Instruction: ", ""))
        .collect();

    Ok(ParsedTransaction {
        block_time: tx.block_time.unwrap_or_default(),
        signatures: tx.transaction.clone().signatures,
        balances,
        parsed_instructions,
    })
}
