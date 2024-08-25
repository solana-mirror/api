use crate::{client::SolanaMirrorClient, utils::get_token_data, USDC_ADDRESS};
use jupiter_swap_api_client::{quote::QuoteRequest, JupiterSwapApiClient};
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

pub struct GetPriceConfig {
    pub decimals: Option<u8>,
}

/// Optionally fetches the decimals for a given token and its price and returns it
pub async fn get_price(
    client: &SolanaMirrorClient,
    token_a: Pubkey,
    config: GetPriceConfig,
) -> Option<f64> {
    let jupiter = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());

    let token_b = Pubkey::from_str(USDC_ADDRESS).unwrap();

    // Using USDC as $, if it's comparing USDC to itself return 1
    if token_a.to_string() == token_b.to_string() {
        return Some(1.0);
    }

    let decimals_b = 6;

    let decimals_a = match config.decimals {
        Some(decimals) => decimals,
        None => {
            get_token_data(&client, &token_a)
                .await
                .unwrap()
                .unwrap()
                .decimals
        }
    };

    let amount = 10_u64.pow(decimals_a as u32);

    let input_mint = ProgramPubkey::from_str(token_a.to_string().as_str()).unwrap();
    let output_mint = ProgramPubkey::from_str(token_b.to_string().as_str()).unwrap();

    let quote_request = QuoteRequest {
        amount,
        input_mint,
        output_mint,
        ..QuoteRequest::default()
    };

    match jupiter.quote(&quote_request).await {
        Ok(quote) => {
            let price = quote.out_amount as f64 / 10_f64.powi(decimals_b as i32);
            Some(price)
        }
        Err(_) => None,
    }
}
