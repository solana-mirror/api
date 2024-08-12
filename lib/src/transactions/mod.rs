use std::{collections::HashMap, str::FromStr};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use solana_sdk::signature::Signature;
use solana_transaction_status::{EncodedConfirmedTransactionWithStatusMeta, UiTransactionEncoding};

use crate::{transactions::types::{BalanceChange, FormattedAmount, GetTransactionsError, ParsedTransaction}, utils::get_rpc};

pub mod types;

fn get_signatures(connection: &RpcClient, address: &str) -> Result<Vec<String>, GetTransactionsError> {
    let pubkey = Pubkey::from_str(&address);

    match pubkey {
        Ok(pubkey) => {
            let signatures = connection.get_signatures_for_address(&pubkey);
            
            match signatures {
                Ok(signatures) => Ok(signatures.into_iter().map(|x| x.signature).collect()),
                Err(_) => Err(GetTransactionsError::FetchError),
            }
        }
        Err(_) => Err(GetTransactionsError::InvalidAddress),
    }
}

async fn get_confirmed_transaction(connection: &RpcClient, signature: &str) -> Result<EncodedConfirmedTransactionWithStatusMeta, GetTransactionsError> {
    let sign = Signature::from_str(signature).unwrap();
    connection.get_transaction(&sign, UiTransactionEncoding::Base64)
        .map_err(|_| GetTransactionsError::FetchError)
}

pub async fn get_parsed_transactions(address: String) -> Result<Vec<ParsedTransaction>, GetTransactionsError> {
    let connection = RpcClient::new(get_rpc());
    let signatures = match get_signatures(&connection, &address) {
         Ok(sigs) => sigs,
         Err(err) => return Err(err),
    };

    // (only one tx) -> TODO: handle all txs (too many requests)
    let sig = &signatures[0];
    let future = async move {
        get_confirmed_transaction(&connection, sig).await
    };
    let result: Result<EncodedConfirmedTransactionWithStatusMeta, GetTransactionsError> = future.await;
   
    println!("{:?}", result);

    // mock data
    let transaction1 = ParsedTransaction {
        block_time: 1625097600,
        signatures: vec!["signature1".to_string(), "signature2".to_string()],
        logs: vec!["log1".to_string(), "log2".to_string()],
        balances: {
            let mut balances = HashMap::new();
            balances.insert(
                "address1".to_string(),
                BalanceChange {
                    pre: FormattedAmount { amount: 1000, formatted: 10.0 },
                    post: FormattedAmount { amount: 800, formatted: 8.0 },
                },
            );
            balances.insert(
                "address2".to_string(),
                BalanceChange {
                    pre: FormattedAmount { amount: 500, formatted: 5.0 },
                    post: FormattedAmount { amount: 700, formatted: 7.0 },
                },
            );
            balances
        },
        parsed_instructions: vec!["instruction1".to_string(), "instruction2".to_string()],
    };

    let transaction2 = ParsedTransaction {
        block_time: 1625097700,
        signatures: vec!["signature3".to_string(), "signature4".to_string()],
        logs: vec!["log3".to_string(), "log4".to_string()],
        balances: {
            let mut balances = HashMap::new();
            balances.insert(
                "address3".to_string(),
                BalanceChange {
                    pre: FormattedAmount { amount: 2000, formatted: 20.0 },
                    post: FormattedAmount { amount: 1500, formatted: 15.0 },
                },
            );
            balances.insert(
                "address4".to_string(),
                BalanceChange {
                    pre: FormattedAmount { amount: 100, formatted: 1.0 },
                    post: FormattedAmount { amount: 200, formatted: 2.0 },
                },
            );
            balances
        },
        parsed_instructions: vec!["instruction3".to_string(), "instruction4".to_string()],
    };

    // return mocked data to avoid type errors
    Ok(vec![transaction1, transaction2])
}