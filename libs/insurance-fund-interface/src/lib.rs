#![no_std]
use soroban_sdk::{Address, Env, contractclient, contracttype};

#[contractclient(name = "InsuranceFundClient")]
pub trait InsuranceFund {
    /// Accounts for new reserves added to the Fund contract. Required, since some implementations can account for balances
    /// apart from the plain token's amount
    ///
    /// # Panics
    /// If the Market contract hasn't authorized the call
    ///
    /// # Arguments
    /// * `token` - asset sent
    /// * `amount` - amount of the sent asset
    fn add_reserves(e: Env, token: Address, amount: i128);

    /// Notifies the Fund of bad debt.
    /// The Fund records the request and starts its internal process (Auction/Vote/etc.)
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
    /// [`IssueRequestResult::Recorded(u64)`] - where `u64` value represents a unique `request_id` tracking this specific coverage event,
    /// [`IssueRequestResult::Immediate(i128)`] - where `i128` is the amount covered if the fund can(and decides) immediately cover the request
    fn request_coverage(e: Env, token: Address, amount: i128) -> IssueRequestResult;

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
    /// [`i128`] amount of tokens that are covered and sent to the Market contract(`0` is considered as a valid amount)
    fn claim_coverage(e: Env, request_id: u64) -> i128;

    /// Cancels a still-`Pending` request without paying out. Used by the Market to
    /// resolve coverage requests whose deadline has elapsed, so the pool can socialize
    /// the loss and unlock without depending on Insurance Fund admin liveness.
    ///
    /// # Panics
    /// If the Market contract hasn't authorized the call, the request does not exist,
    /// or the request is already in [`CoverageStatus::Ready`] state.
    fn cancel(e: Env, request_id: u64);
}

#[contracttype]
#[derive(Debug)]
pub enum IssueRequestResult {
    Recorded(u64),
    Immediate(i128),
}

#[contracttype]
#[derive(Debug)]
pub enum CoverageStatus {
    Pending,
    Ready(i128),
}
