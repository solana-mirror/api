use crate::Error;
use reqwest::Client;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;
use types::{Signature, Transaction};
use uuid::Uuid;

pub mod types;

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

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    #[serde(rename = "numReadonlySignedAccounts")]
    pub num_readonly_signed_accounts: u8,
    #[serde(rename = "numReadonlyUnsignedAccounts")]
    pub num_readonly_unsigned_accounts: u8,
    #[serde(rename = "numRequiredSignatures")]
    pub num_required_signatures: u8,
}

#[derive(Serialize, Deserialize)]
pub struct Request<Params> {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Params>,
    pub id: String,
}

pub struct SolanaMirrorClient {
    inner_client: Client,
    pub rpc_url: String,
}

// get_signatures_for_address method

#[derive(Serialize, Deserialize, Debug)]
pub struct GetSignaturesForAddressResponse {
    pub jsonrpc: String,
    pub result: Vec<Signature>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetSignaturesForAddressConfig {
    pub commitment: Option<String>,
    pub before: Option<String>,
    pub until: Option<String>,
    pub limit: Option<u16>,
}

pub type GetSignaturesForAddressParams = (String, Option<GetSignaturesForAddressConfig>);

// get_transactions method

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTransactionResponse {
    pub jsonrpc: String,
    pub result: Option<Transaction>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct GetTransactionConfig {
    pub commitment: Option<String>,
    #[serde(rename = "maxSupportedTransactionVersion")]
    pub max_supported_transaction_version: Option<u8>,
    pub encoding: Option<String>,
}

pub type GetTransactionParams = (String, Option<GetTransactionConfig>);

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
