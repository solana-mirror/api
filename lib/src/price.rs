use crate::client::GetDecimalsConfig;
use crate::{client::SolanaMirrorClient, USDC_ADDRESS};
use jupiter_swap_api_client::{quote::QuoteRequest, JupiterSwapApiClient};
use solana_program::pubkey::Pubkey as ProgramPubkey;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

/// Gets the price of the mint against USDC
/// Lets the caller pass the decimals beforehand. If they're not passed, they will be fetched
pub async fn get_price(
    client: &SolanaMirrorClient,
    token: Pubkey,
    decimals: Option<u8>,
) -> Option<f64> {
    let jupiter = JupiterSwapApiClient::new("https://quote-api.jup.ag/v6".to_string());
    let usdc_pubkey = Pubkey::from_str(USDC_ADDRESS).unwrap();

    // If it's comparing USDC to itself return 1
    if token.to_string() == *USDC_ADDRESS.to_string() {
        return Some(1.0);
    }

    let decimals_b = 6;
    let decimals_a = match decimals {
        Some(d) => d,
        None => {
            let response = client
                .get_decimals(
                    &token,
                    Some(GetDecimalsConfig {
                        commitment: Some("confirmed".to_string()),
                    }),
                )
                .await;

            response.unwrap().result.value.decimals
        }
    };

    let amount = 10_u64.pow(decimals_a as u32);

    let input_mint = ProgramPubkey::from_str(token.to_string().as_str()).unwrap();
    let output_mint = ProgramPubkey::from_str(usdc_pubkey.to_string().as_str()).unwrap();

    let quote_request = QuoteRequest {
        amount,
        input_mint,
        output_mint,
        ..QuoteRequest::default()
    };

    match jupiter.quote(&quote_request).await {
        Ok(quote) => {
            let price = quote.out_amount as f64 / 10_f64.powi(decimals_b);
            Some(price)
        }
        Err(_) => None,
    }
}
