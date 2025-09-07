#![no_std]

pub mod contract;
pub mod error;
pub mod storage;

// Aliases
pub type MMError = crate::error::MarketManagerError;
