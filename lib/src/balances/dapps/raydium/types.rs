use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Position {
    pub discriminator: [u8; 8],
    pub bump: u8,
    pub nft_mint: Pubkey,
    pub pool_id: Pubkey,
    pub tick_lower: i32,
    pub tick_upper: i32,
    pub liquidity: u128,
    pub fee_growth_inside_last_x64_a: u128,
    pub fee_growth_inside_last_x64_b: u128,
    pub token_fees_owed_a: u64,
    pub token_fees_owed_b: u64,
    pub reward_infos: [PositionRewardInfo; 3],
    pub padding: [u64; 8],
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct PositionRewardInfo {
    pub growth_inside_last_x64: u128,
    pub reward_amount_owed: u64,
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct Pool {
    pub discriminator: [u8; 8],
    pub bump: u8,
    pub amm_config: Pubkey,
    pub creator: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub vault_a: Pubkey,
    pub vault_b: Pubkey,
    pub observation_id: Pubkey,
    pub mint_decimals_a: u8,
    pub mint_decimals_b: u8,
    pub tick_spacing: u16,
    pub liquidity: u128,
    pub sqrt_price_x64: u128,
    pub tick_current: i32,
    pub observation_index: u16,
    pub observation_update_duration: u16,
    pub fee_growth_global_x64_a: u128,
    pub fee_growth_global_x64_b: u128,
    pub protocol_fees_token_a: u64,
    pub protocol_fees_token_b: u64,

    pub swap_in_amount_token_a: u128,
    pub swap_out_amount_token_b: u128,
    pub swap_in_amount_token_b: u128,
    pub swap_out_amount_token_a: u128,

    pub status: u8,
    pub reserved: [u8; 7],

    pub reward_infos: [RewardInfo; 3],
    pub tick_array_bitmap: [u64; 16],

    pub total_fees_token_a: u64,
    pub total_fees_claimed_token_a: u64,
    pub total_fees_token_b: u64,
    pub total_fees_claimed_token_b: u64,

    pub fund_fees_token_a: u64,
    pub fund_fees_token_b: u64,

    pub start_time: u64,

    pub padding: [u64; 8],
}

#[derive(Debug, Deserialize, Serialize, Default)]
pub struct RewardInfo {
    pub reward_state: u8,
    pub open_time: u64,
    pub end_time: u64,
    pub last_update_time: u64,
    pub emissions_per_second_x64: u128,
    pub reward_total_emissioned: u64,
    pub reward_claimed: u64,
    pub token_mint: Pubkey,
    pub token_vault: Pubkey,
    pub creator: Pubkey,
    pub reward_growth_global_x64: u128,
}
