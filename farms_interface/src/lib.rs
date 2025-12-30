//! Farms Contract Interface
//!
//! This crate provides a client interface for cross-contract calls to the Farms contract.
//! Use this crate when your contract needs to interact with Farms without depending on
//! the full implementation.
//!
//! # Usage
//!
//! ```ignore
//! use farms_interface::FarmsClient;
//!
//! let client = FarmsClient::new(&env, &farms_contract_address);
//! client.set_stake_delegated(&user, &farm_id, &new_stake);
//! ```
//!
//! # Design
//!
//! The [`FarmsClient`] is generated from the [`Farms`] trait using Soroban's
//! `#[contractclient]` macro. This allows contracts to call Farms without
//! importing the full contract implementation, keeping WASM binaries lean.

#![no_std]

use soroban_sdk::{contractclient, Address, BytesN, Env};

/// Farms contract interface for cross-contract calls.
///
/// This trait defines the subset of Farms functions available for external contracts.
/// The generated [`FarmsClient`] can be used to invoke these functions on a deployed
/// Farms contract.
#[contractclient(name = "FarmsClient")]
pub trait Farms {
    /// Updates a user's stake via delegated authority.
    ///
    /// This is the core function for the delegated staking pattern (push model).
    /// The delegate authority (e.g., a lending protocol) calls this function
    /// to sync a user's stake whenever their position changes.
    ///
    /// # Arguments
    /// * `user` - The user whose stake is being updated
    /// * `farm_id` - The unique identifier of the farm (32-byte hash)
    /// * `new_stake` - The new total stake amount (replaces previous stake)
    ///
    /// # Authorization
    /// The calling contract must be registered as the `delegate_authority` for this farm.
    /// Calls from unauthorized addresses will fail.
    ///
    /// # Example Use Cases
    /// - Lending protocols: sync stake after deposit, withdraw, borrow, repay
    /// - AMM integrations: sync stake after add/remove liquidity
    /// - Staking wrappers: sync stake after any position change
    fn set_stake_delegated(e: Env, user: Address, farm_id: BytesN<32>, new_stake: i128);
}
