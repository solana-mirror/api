use std::future::Future;
use std::pin::Pin;

use crate::Error;
use base64::Engine;
use reqwest::Client;
use serde::{self, de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{from_value, Value};
use solana_sdk::pubkey::Pubkey;
use types::{
    AccountDataResultData, AccountsResultData, BalanceResultData, DecimalsResultData, Signature,
    Transaction,
};
use uuid::Uuid;

pub mod types;

#[derive(Serialize, Deserialize)]
pub enum JsonRpcMethod {
    GetTokenAccountsByOwner,
    GetBalance,
    GetAccountInfo,
    GetDecimals,
    GetTransaction,
    GetSignaturesForAddress,
}

impl JsonRpcMethod {
    pub fn to_string(&self) -> String {
        match self {
            JsonRpcMethod::GetTokenAccountsByOwner => "getTokenAccountsByOwner".to_string(),
            JsonRpcMethod::GetBalance => "getBalance".to_string(),
            JsonRpcMethod::GetAccountInfo => "getAccountInfo".to_string(),
            JsonRpcMethod::GetDecimals => "getTokenSupply".to_string(),
            JsonRpcMethod::GetTransaction => "getTransaction".to_string(),
            JsonRpcMethod::GetSignaturesForAddress => "getSignaturesForAddress".to_string(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcRequest<Params> {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Params>,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct JsonRpcResponse<Result> {
    pub jsonrpc: String,
    pub result: Result,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    error: ErrorDetails,
    #[serde(rename = "jsonrpc")]
    json_rpc: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct ErrorDetails {
    code: i32,
    message: String,
}

// get_token_accounts_by_owner
pub type GetTokenAccountsByOwnerResponse = JsonRpcResponse<AccountsResultData>;

#[derive(Serialize, Deserialize, Clone)]
pub struct GetTokenAccountsByOwnerFilter {
    #[serde(rename = "programId")]
    pub program_id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetTokenAccountsByOwnerConfig {
    pub commitment: Option<String>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
    #[serde(rename = "dataSlice")]
    pub data_slice: Option<DataSlice>,
    pub encoding: Option<String>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct DataSlice {
    pub length: usize,
    pub offset: usize,
}

pub type GetTokenAccountsByOwnerParams = (
    String,
    Option<GetTokenAccountsByOwnerFilter>,
    Option<GetTokenAccountsByOwnerConfig>,
);

// get_balance
pub type GetBalanceResponse = JsonRpcResponse<Option<BalanceResultData>>;

#[derive(Serialize, Deserialize)]
pub struct GetBalanceConfig {
    pub commitment: Option<String>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<usize>,
}

pub type GetBalanceParams = (String, Option<GetBalanceConfig>);

// get_account_info
pub type GetAccountDataResponse = JsonRpcResponse<AccountDataResultData>;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAccountDataConfig {
    pub commitment: Option<String>,
    pub encoding: Option<String>,
}

pub type GetAccountDataParams = (String, Option<GetAccountDataConfig>);

// get_decimals
pub type GetDecimalsResponse = JsonRpcResponse<DecimalsResultData>;

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDecimalsConfig {
    pub commitment: Option<String>,
}

pub type GetDecimalsParams = (String, Option<GetDecimalsConfig>);

// get_signatures_for_address
pub type GetSignaturesForAddressResponse = JsonRpcResponse<Vec<Signature>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct GetSignaturesForAddressConfig {
    pub commitment: Option<String>,
    pub before: Option<String>,
    pub until: Option<String>,
    pub limit: Option<u16>,
}

pub type GetSignaturesForAddressParams = (String, Option<GetSignaturesForAddressConfig>);

// get_transactions
pub type GetTransactionResponse = JsonRpcResponse<Option<Transaction>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct GetTransactionConfig {
    pub commitment: Option<String>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
    pub encoding: Option<String>,
}

pub type GetTransactionParams = (String, Option<GetTransactionConfig>);

// TODO: implement on all methods
#[allow(dead_code)]
async fn retry<T, F>(callback: F, max_retries: u8) -> Result<T, Error>
where
    F: Fn() -> Pin<Box<dyn Future<Output = Result<T, Error>>>>,
{
    for attempt in 0..=max_retries {
        match callback().await {
            Ok(res) => return Ok(res),
            Err(e) => {
                if attempt == max_retries {
                    return Err(e);
                } else {
                    match e {
                        Error::TooManyRequests => continue,
                        _ => return Err(e),
                    }
                }
            }
        }
    }

    Err(Error::FetchError)
}

fn deserialize<T: DeserializeOwned>(res: &Value) -> Result<T, Error> {
    // Try deserializing into an error first
    if let Ok(deserialized_err) = from_value::<JsonRpcError>(res.clone()) {
        match deserialized_err.error.code {
            429 => return Err(Error::TooManyRequests),
            _ => return Err(Error::FetchError),
        }
    }

    match from_value::<T>(res.clone()) {
        Ok(res) => Ok(res),
        Err(_) => Err(Error::ParseError),
    }
}

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

    pub fn from_client(inner_client: &Client, rpc_url: String) -> Self {
        Self {
            inner_client: inner_client.clone(),
            rpc_url,
        }
    }

    async fn make_batch_request<T: Serialize>(
        &self,
        body: &Vec<JsonRpcRequest<T>>,
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

    async fn make_request<T: Serialize>(
        &self,
        method: JsonRpcMethod,
        params: Option<T>,
    ) -> Result<Value, Error> {
        let body = &JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
            id: Uuid::new_v4().to_string(),
        };

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

    pub async fn get_token_accounts_by_owner(
        &self,
        owner: &Pubkey,
        filter: Option<GetTokenAccountsByOwnerFilter>,
        config: Option<GetTokenAccountsByOwnerConfig>,
    ) -> Result<GetTokenAccountsByOwnerResponse, Error> {
        let params: GetTokenAccountsByOwnerParams = (owner.to_string(), filter, config);

        let res = self
            .make_request(JsonRpcMethod::GetTokenAccountsByOwner, Some(params))
            .await?;
        println!("{}", res);
        deserialize::<GetTokenAccountsByOwnerResponse>(&res)
    }

    pub async fn get_balance(
        &self,
        owner: &Pubkey,
        config: Option<GetBalanceConfig>,
    ) -> Result<u64, Error> {
        let params: GetBalanceParams = (owner.to_string(), config);

        let res = self
            .make_request(JsonRpcMethod::GetBalance, Some(params))
            .await?;
        match deserialize::<GetBalanceResponse>(&res) {
            Ok(bal) => Ok(bal.result.unwrap().value),
            Err(e) => Err(e),
        }
    }

    pub async fn get_account_info(
        &self,
        pubkey: &Pubkey,
        config: Option<GetAccountDataConfig>,
    ) -> Result<Vec<u8>, Error> {
        let params: GetAccountDataParams = (pubkey.to_string(), config);

        let res = self
            .make_request(JsonRpcMethod::GetAccountInfo, Some(params))
            .await?;
        match deserialize::<GetAccountDataResponse>(&res) {
            Ok(acc) => {
                let base64_data = &acc.result.value.data[0];
                let decoded_data = base64::prelude::BASE64_STANDARD
                    .decode(base64_data)
                    .expect("Failed to decode base64 data.");

                Ok(decoded_data)
            }
            Err(e) => Err(e),
        }
    }

    pub async fn get_decimals(
        &self,
        mint: &Pubkey,
        config: Option<GetDecimalsConfig>,
    ) -> Result<GetDecimalsResponse, Error> {
        let params: GetDecimalsParams = (mint.to_string(), config);

        let res = self
            .make_request(JsonRpcMethod::GetDecimals, Some(params))
            .await?;
        deserialize::<GetDecimalsResponse>(&res)
    }

    pub async fn get_signatures_for_address(
        &self,
        address: &Pubkey,
        config: Option<GetSignaturesForAddressConfig>,
    ) -> Result<GetSignaturesForAddressResponse, Error> {
        let params: GetSignaturesForAddressParams = (address.to_string(), config);

        let res = self
            .make_request(JsonRpcMethod::GetSignaturesForAddress, Some(params))
            .await?;
        deserialize::<GetSignaturesForAddressResponse>(&res)
    }

    pub async fn get_transactions(
        &self,
        signatures: &[String],
        config: Option<GetTransactionConfig>,
    ) -> Result<Vec<GetTransactionResponse>, Error> {
        let body: Vec<JsonRpcRequest<GetTransactionParams>> = signatures
            .iter()
            .map(|signature| {
                let params: GetTransactionParams = (signature.to_string(), config.clone());
                JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: JsonRpcMethod::GetTransaction.to_string(),
                    params: Some(params),
                    id: Uuid::new_v4().to_string(),
                }
            })
            .collect();

        let res = self.make_batch_request(&body).await?;
        deserialize::<Vec<GetTransactionResponse>>(&res)
    }
}
