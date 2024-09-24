pub mod accounts;
pub mod chart;
pub mod client;
pub mod coingecko;
pub mod price;
pub mod transactions;
pub mod utils;

pub const USDC_ADDRESS: &str = "EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v";
pub const SOL_ADDRESS: &str = "So11111111111111111111111111111111111111112";

#[derive(Debug)]
pub enum Error {
    InvalidAddress,
    InvalidIndex,
    InvalidTimeframe,
    FetchError,
    ParseError,
    TooManyRequests,
}

#[derive(Debug)]
pub struct Page {
    pub start_idx: usize,
    pub end_idx: usize,
}
