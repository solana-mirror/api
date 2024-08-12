use crate::utils::{get_rpc, get_token_data};
use jupiter_swap_api_client::{quote::QuoteRequest, JupiterSwapApiClient};
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub async fn get_price(token_a: Pubkey, token_b: Pubkey) -> f64 {
    let connection = RpcClient::new(get_rpc());
    let jupiter = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

    // Using USDC as $, if it's comparing USDC to itself return 1
    if token_a.to_string() == token_b.to_string() {
        return 1.0;
    }

    let decimals_a = get_token_data(&connection, &token_a)
        .await
        .unwrap()
        .decimals;
    let decimals_b = get_token_data(&connection, &token_b)
        .await
        .unwrap()
        .decimals;

    let amount = 10_u64.pow(decimals_a as u32);

    let input_mint = ProgramPubkey::from_str(token_a.to_string().as_str()).unwrap();
    let output_mint = ProgramPubkey::from_str(token_b.to_string().as_str()).unwrap();

    let quote_request = QuoteRequest {
        amount,
        input_mint,
        output_mint,
        ..QuoteRequest::default()
    };

    let out_amount = jupiter.quote(&quote_request).await.unwrap().out_amount;
    out_amount as f64 / 10_f64.powi(decimals_b as i32)
}
