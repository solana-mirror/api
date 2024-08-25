use crate::{utils::parse_json_rpc_error, Error};
use base64::Engine;
use reqwest::Client;
use serde::{self, Deserialize, Serialize};
use serde_json::Value;
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

#[derive(Serialize, Deserialize, Debug)]
struct ErrorDetails {
    code: i32,
    message: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcError {
    error: ErrorDetails,
    #[serde(rename = "jsonrpc")]
    json_rpc: String,
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

// get_token_accounts_by_owner

#[derive(Serialize, Deserialize, Debug)]
pub struct GetTokenAccountsByOwnerResponse {
    pub jsonrpc: String,
    pub result: Option<AccountsResultData>,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTokenAccountsByOwnerFilter {
    #[serde(rename = "programId")]
    pub program_id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetTokenAccountsByOwnerConfig {
    pub commitment: Option<String>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<u64>,
    #[serde(rename = "dataSlice")]
    pub data_slice: Option<DataSlice>,
    pub encoding: Option<String>,
}

#[derive(Serialize, Deserialize)]
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

#[derive(Serialize, Deserialize, Debug)]
pub struct GetBalanceResponse {
    pub jsonrpc: String,
    pub result: Option<BalanceResultData>,
    pub id: String,
}

#[derive(Serialize, Deserialize)]
pub struct GetBalanceConfig {
    pub commitment: Option<String>,
    #[serde(rename = "minContextSlot")]
    pub min_context_slot: Option<usize>,
}

pub type GetBalanceParams = (String, Option<GetBalanceConfig>);

// get_account_info

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAccountDataResponse {
    pub jsonrpc: String,
    pub result: Option<AccountDataResultData>,
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetAccountDataConfig {
    pub commitment: Option<String>,
    pub encoding: Option<String>,
}

pub type GetAccountDataParams = (String, Option<GetAccountDataConfig>);

// get_decimals

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDecimalsResponse {
    pub id: String,
    pub jsonrpc: String,
    pub result: Option<DecimalsResultData>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetDecimalsConfig {
    pub commitment: Option<String>,
}

pub type GetDecimalsParams = (String, Option<GetDecimalsConfig>);

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

    pub fn from_reqwest_client(inner_client: Client, rpc_url: String) -> Self {
        return Self {
            inner_client,
            rpc_url,
        };
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

    pub async fn get_token_accounts_by_owner(
        &self,
        owner: &Pubkey,
        filter: Option<GetTokenAccountsByOwnerFilter>,
        config: Option<GetTokenAccountsByOwnerConfig>,
    ) -> Result<GetTokenAccountsByOwnerResponse, Error> {
        let params: GetTokenAccountsByOwnerParams = (owner.to_string(), filter, config);

        let body = Request {
            jsonrpc: "2.0".to_string(),
            method: JsonRpcMethod::GetTokenAccountsByOwner.to_string(),
            params: Some(params),
            id: Uuid::new_v4().to_string(),
        };

        let res = self.make_request(&body).await?;
        match serde_json::from_value::<GetTokenAccountsByOwnerResponse>(res) {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("Error parsing JSON: {:?}", e);
                Err(Error::ParseError)
            }
        }
    }

    pub async fn get_balance(
        &self,
        owner: &Pubkey,
        config: Option<GetBalanceConfig>,
    ) -> Result<u64, Error> {
        let params: GetBalanceParams = (owner.to_string(), config);

        let body = Request {
            jsonrpc: "2.0".to_string(),
            method: JsonRpcMethod::GetBalance.to_string(),
            params: Some(params),
            id: Uuid::new_v4().to_string(),
        };

        let res = self.make_request(&body).await?;
        match serde_json::from_value::<GetBalanceResponse>(res) {
            Ok(res) => Ok(res.result.unwrap().value),
            Err(e) => {
                println!("Error parsing balance JSON: {:?}", e);
                Err(Error::ParseError)
            }
        }
    }

    pub async fn get_account_info(
        &self,
        pubkey: &Pubkey,
        config: Option<GetAccountDataConfig>,
    ) -> Result<Vec<u8>, Error> {
        let params: GetAccountDataParams = (pubkey.to_string(), config);

        let body = Request {
            jsonrpc: "2.0".to_string(),
            method: JsonRpcMethod::GetAccountInfo.to_string(),
            params: Some(params),
            id: Uuid::new_v4().to_string(),
        };

        let res = self.make_request(&body).await?;
        match serde_json::from_value::<GetAccountDataResponse>(res) {
            Ok(res) => {
                let base64_data = &res.result.as_ref().unwrap().value.data[0];

                let decoded_data = base64::prelude::BASE64_STANDARD
                    .decode(base64_data)
                    .expect("Failed to decode base64 data.");

                Ok(decoded_data)
            }
            Err(e) => {
                println!("Error parsing JSON: {:?}", e);
                Err(Error::ParseError)
            }
        }
    }

    pub async fn get_decimals(
        &self,
        mint: &Pubkey,
        config: Option<GetDecimalsConfig>,
    ) -> Result<GetDecimalsResponse, Error> {
        let params: GetDecimalsParams = (mint.to_string(), config);

        let body = Request {
            jsonrpc: "2.0".to_string(),
            method: JsonRpcMethod::GetDecimals.to_string(),
            params: Some(params),
            id: Uuid::new_v4().to_string(),
        };

        let res = self.make_request(&body).await?;
        match serde_json::from_value::<GetDecimalsResponse>(res) {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("Error parsing JSON from decimals: {:?}", e);
                Err(Error::ParseError)
            }
        }
    }

    pub async fn get_signatures_for_address(
        &self,
        address: &Pubkey,
        config: Option<GetSignaturesForAddressConfig>,
    ) -> Result<GetSignaturesForAddressResponse, Error> {
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

        let mut res = self.make_batch_request(&body).await?;

        // If we get rate limited, retry once more
        for i in 1..=3 {
            if let Some(jsonrpc_error) = parse_json_rpc_error(res.clone()) {
                print!("JsonRpcError: {:?}", jsonrpc_error);
                if i != 3 {
                    res = self.make_batch_request(&body).await?;
                } else {
                    println!("Retry limit exceeded");
                    return Err(Error::FetchError);
                }
            } else {
                break;
            }
        }

        match serde_json::from_value::<Vec<GetTransactionResponse>>(res) {
            Ok(res) => Ok(res),
            Err(e) => {
                println!("Error parsing JSON: {:?}", e);
                Err(Error::ParseError)
            }
        }
    }
}
