#![no_std]

mod contract;
pub mod error;
pub mod storage;

// Aliases
pub type MMError = crate::error::MarketManagerError;
