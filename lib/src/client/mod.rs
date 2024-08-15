use crate::Error;
use reqwest::Client;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use uuid::Uuid;

pub struct SolanaMirrorClient {
    inner_client: Client,
    pub rpc_url: String,
}

impl SolanaMirrorClient {
    pub fn new(rpc_url: String) -> Self {
        Self {
            inner_client: Client::new(),
            rpc_url,
        }
    }

    async fn make_batch_request<T: Serialize>(
        &self,
        body: &Vec<Request<T>>,
    ) -> Result<Value, Error> {
        let serialized = match serde_json::to_string(body) {
            Ok(serialized) => serialized,
            Err(_) => return Err(Error::ParseError),
        };

        let req = self
            .inner_client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .header("solana-client", "js/0.0.0-development")
            .body(serialized);

        match req.send().await {
            Ok(response) => {
                let res = response
                    .json::<Value>()
                    .await
                    .map_err(|_| Error::ParseError)?;
                Ok(res)
            }
            Err(_) => Err(Error::FetchError),
        }
    }

    async fn make_request<T: Serialize>(&self, body: &Request<T>) -> Result<Value, Error> {
        let serialized = match serde_json::to_string(body) {
            Ok(serialized) => serialized,
            Err(_) => return Err(Error::ParseError),
        };

        let req = self
            .inner_client
            .post(&self.rpc_url)
            .header("Content-Type", "application/json")
            .header("solana-client", "js/0.0.0-development")
            .body(serialized);

        match req.send().await {
            Ok(response) => {
                let res = response
                    .json::<Value>()
                    .await
                    .map_err(|_| Error::ParseError)?;
                Ok(res)
            }
            Err(e) => {
                println!("Failed to deserialize {:?}", e);
                Err(Error::FetchError)
            }
        }
    }

    pub async fn get_signatures_for_address(
        &self,
        address: &str,
        config: Option<GetSignaturesForAddressConfig>,
    ) -> Result<GetSignaturesForAddressResponse, Error> {
        // Validate address
        let _pubkey = Pubkey::from_str(&address).map_err(|_| return Error::InvalidAddress)?;

        let params: GetSignaturesForAddressParams = (address.to_string(), config);
        let body = &Request {
            jsonrpc: "2.0".to_string(),
            method: JsonRpcMethod::GetSignaturesForAddress.to_string(),
            params: Some(params),
            id: Uuid::new_v4().to_string(),
        };

        let res = self.make_request(&body).await?;
        match serde_json::from_value::<GetSignaturesForAddressResponse>(res) {
            Ok(res) => Ok(res),
            Err(_) => Err(Error::ParseError),
        }
    }

    pub async fn get_transactions(
        &self,
        signatures: &Vec<String>,
        config: Option<GetTransactionConfig>,
    ) -> Result<Vec<GetTransactionResponse>, Error> {
        let body: Vec<Request<GetTransactionParams>> = signatures
            .iter()
            .map(|signature| {
                let params: GetTransactionParams = (signature.to_string(), config.clone());
                Request {
                    jsonrpc: "2.0".to_string(),
                    method: JsonRpcMethod::GetTransaction.to_string(),
                    params: Some(params),
                    id: Uuid::new_v4().to_string(),
                }
            })
            .collect();

        let res = self.make_batch_request(&body).await?;
        match serde_json::from_value::<Vec<GetTransactionResponse>>(res) {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("Error parsing JSON: {:?}", e);
                Err(Error::ParseError)
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Rewards {
    pub pubkey: String,
    pub lamports: i64,
    #[serde(rename = "postBalance")]
    pub post_balance: u64,
    #[serde(rename = "rewardType")]
    pub reward_type: String,
    pub commission: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    #[serde(rename = "numReadonlySignedAccounts")]
    pub num_readonly_signed_accounts: u8,
    #[serde(rename = "numReadonlyUnsignedAccounts")]
    pub num_readonly_unsigned_accounts: u8,
    #[serde(rename = "numRequiredSignatures")]
    pub num_required_signatures: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Instruction {
    pub accounts: Vec<u8>,
    pub data: String,
    #[serde(rename = "programIdIndex")]
    pub program_id_index: u8,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionMessage {
    #[serde(rename = "accountIndex")]
    pub account_index: Option<u64>,
    #[serde(rename = "accountKeys")]
    pub account_keys: Vec<String>,
    pub header: Header,
    pub instructions: Vec<Instruction>,
    #[serde(rename = "recentBlockhash")]
    pub recent_blockhash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct InnerTransaction {
    pub message: TransactionMessage,
    pub signatures: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Meta {
    #[serde(rename = "computeUnitsConsumed")]
    pub compute_units_consumed: u64,
    pub err: Option<Value>,
    pub fee: u64,
    #[serde(rename = "innerInstructions")]
    pub inner_instructions: Vec<Value>,
    #[serde(rename = "loadedAddresses")]
    pub loaded_addresses: Value,
    #[serde(rename = "logMessages")]
    pub log_messages: Option<Vec<String>>,
    #[serde(rename = "postBalances")]
    pub post_balances: Vec<u64>,
    #[serde(rename = "postTokenBalances")]
    pub post_token_balances: Vec<TokenBalance>,
    #[serde(rename = "preBalances")]
    pub pre_balances: Vec<u64>,
    #[serde(rename = "preTokenBalances")]
    pub pre_token_balances: Vec<TokenBalance>,
    pub rewards: Vec<Rewards>,
    /// Deprecated
    pub status: Value,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UiTokenAmount {
    pub amount: String,
    pub decimals: u8,
    #[serde(rename = "uiAmount")]
    pub ui_amount: Option<f64>,
    #[serde(rename = "uiAmountString")]
    pub ui_amount_string: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TokenBalance {
    #[serde(rename = "accountIndex")]
    pub account_index: u64,
    pub mint: String,
    pub owner: String,
    #[serde(rename = "programId")]
    pub program_id: String,
    #[serde(rename = "uiTokenAmount")]
    pub ui_token_amount: UiTokenAmount,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionResponse {
    pub jsonrpc: String,
    pub result: Option<Transaction>,
    #[serde(rename = "blockTime")]
    pub block_time: Option<i64>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Version {
    U8(u8),
    String(String),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub meta: Meta,
    pub slot: u64,
    pub transaction: InnerTransaction,
    pub version: Option<Version>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
    pub err: Option<Value>,
    pub memo: Option<Value>,
    pub signature: String,
    pub slot: u64,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub block_time: Option<i64>,
    #[serde(rename = "confirmationStatus")]
    pub confirmation_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSignaturesForAddressResponse {
    pub jsonrpc: String,
    pub result: Vec<Signature>,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub enum JsonRpcMethod {
    GetTransaction,
    GetSignaturesForAddress,
}

impl JsonRpcMethod {
    pub fn to_string(&self) -> String {
        match self {
            JsonRpcMethod::GetTransaction => "getTransaction".to_string(),
            JsonRpcMethod::GetSignaturesForAddress => "getSignaturesForAddress".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct Request<Params> {
    jsonrpc: String,
    method: String,
    params: Option<Params>,
    id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetSignaturesForAddressConfig {
    pub commitment: Option<String>,
    pub before: Option<String>,
    pub until: Option<String>,
    pub limit: Option<u16>,
}

type GetSignaturesForAddressParams = (String, Option<GetSignaturesForAddressConfig>);

#[derive(Serialize, Deserialize, Clone)]
pub struct GetTransactionConfig {
    pub commitment: Option<String>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
    pub encoding: Option<String>,
}

type GetTransactionParams = (String, Option<GetTransactionConfig>);
