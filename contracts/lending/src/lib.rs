#![no_std]

pub mod constants;
pub mod contract;
pub mod error;
pub mod events;
pub mod interest_rate;
pub mod math_utils;
pub mod multiply_pair;
pub mod obligation;
pub mod pool;
pub mod soroswap_router;
pub mod storage;
pub mod swap;

// Aliases
pub type LCError = error::LendingContractError;
pub type LCResult<T> = Result<T, LCError>;
