#![no_std]
pub mod computations;
pub mod contract;
pub mod error;
pub mod storage;
#[cfg(test)]
mod tests;

pub use contract::AggregatedPriceFeedClient;
