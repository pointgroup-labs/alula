#![no_std]
pub mod computations;
pub mod contract;
pub mod error;
pub mod storage;
<<<<<<< Updated upstream
=======
#[cfg(test)]
mod tests;
>>>>>>> Stashed changes

pub use contract::AggregatedPriceFeedClient;
