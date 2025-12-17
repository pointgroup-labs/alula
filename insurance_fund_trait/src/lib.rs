#![no_std]
use soroban_sdk::{Address, Env, contractclient, contracttype};

#[contractclient(name = "InsuranceFundClient")]
pub trait InsuranceFund {
    /// Notifies the Fund of bad debt.
    /// The Fund records the request if needed and starts its internal process (Auction/Vote/etc.)
    /// or covers the entire amount immediately if possible
    ///
    /// # Arguments
    /// * `token` - asset needed (e.g., USDC)
    /// * `amount` - requested amount to cover
    ///
    /// # Panics
    /// If the Market contract hasn't authorized the call
    ///
    /// # Returns
    /// [`Some(u64)`] - unique `request_id` tracking this specific coverage event, or
    /// `None` if the fund can immediately cover the request
    fn request_coverage(e: Env, token: Address, amount: i128) -> Option<u64>;

    /// Returns the status of an active coverage request
    ///
    /// # Returns
    /// [`Some(CoverageStatus)`] - status of the request if active,
    /// [`None`] otherwise
    fn get_status(e: Env, request_id: u64) -> Option<CoverageStatus>;

    /// Finalizes the coverage, transfers tokens to the Market contract, and removes the request from storage
    ///
    /// # Panics
    /// If the Market contract hasn't authorized the call, the request does not exist or is in [`CoverageStatus::Pending`] state
    ///
    /// # Returns
    /// [`i128`] amount of tokens that are covered and sent to the Market contract
    fn claim_coverage(e: Env, request_id: u64) -> i128;
}

#[contracttype]
#[derive(Debug)]
pub enum CoverageStatus {
    Pending,
    Ready(i128),
}
