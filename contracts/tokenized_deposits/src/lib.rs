#![no_std]
// Amounts are written as `<units>_<7 stroop digits>`, which reads far better for asset figures than
// uniform three-digit grouping
#![allow(clippy::inconsistent_digit_grouping)]

pub mod constants;
pub mod contract;
pub mod error;
pub mod events;
pub mod market;
pub mod storage;

#[cfg(test)]
mod test;
