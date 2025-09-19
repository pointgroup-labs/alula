#![no_std]
pub mod computations;
pub mod contract;
pub mod error;
pub mod storage;

pub use contract::AggregatedPriceFeedClient;
