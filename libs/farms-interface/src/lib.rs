//! Farms Contract Interface
//!
//! This crate provides a client interface for cross-contract calls to the Farms contract.
//! Use this crate when your contract needs to interact with Farms without depending on
//! the full implementation.
//!
//! # Design
//!
//! The [`FarmsClient`] is generated from the [`Farms`] trait using Soroban's
//! `#[contractclient]` macro. This allows contracts to call Farms without
//! importing the full contract implementation, keeping WASM binaries lean.
//!
//! The [`Delegatee`] type supports flexible stake identification:
//! - Simple mode: just owner address (for contracts with single position per user)
//! - Seeded mode: owner address + seed (for contracts with multiple obligations per user)

#![no_std]

use soroban_sdk::{Address, BytesN, Env, contractclient, contracttype};

/// Delegatee identifier for farm stakes.
///
/// Supports multiple stake identities per owner address:
/// - Simple: just owner address (for contracts where user has single position)
/// - With seed: owner address + seed (for contracts with multiple obligations per user)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delegatee {
    /// The owner's address
    pub owner: Address,
    /// Optional seed to distinguish multiple positions per owner
    pub seed: Option<BytesN<32>>,
}

impl Delegatee {
    /// Creates a delegatee from just an owner address (no seed).
    ///
    /// Use this when the delegating contract only tracks one position per user.
    pub fn new(owner: Address) -> Self {
        Self { owner, seed: None }
    }

    /// Creates a delegatee with an owner address and seed.
    ///
    /// Use this when the delegating contract tracks multiple positions per user
    /// (e.g., lending protocols with multiple obligation types).
    pub fn new_with_seed(owner: Address, seed: BytesN<32>) -> Self {
        Self { owner, seed: Some(seed) }
    }
}

impl From<Address> for Delegatee {
    fn from(owner: Address) -> Self {
        Self::new(owner)
    }
}

impl From<(Address, BytesN<32>)> for Delegatee {
    fn from((owner, seed): (Address, BytesN<32>)) -> Self {
        Self::new_with_seed(owner, seed)
    }
}

/// Farms contract interface for cross-contract calls.
///
/// This trait defines the subset of Farms functions available for external contracts.
/// The generated [`FarmsClient`] can be used to invoke these functions on a deployed
/// Farms contract.
#[contractclient(name = "FarmsClient")]
pub trait Farms {
    /// Updates a delegatee's stake via delegated authority.
    ///
    /// This is the core function for the delegated staking pattern (push model).
    /// The delegate authority (e.g., a lending protocol) calls this function
    /// to sync a delegatee's stake whenever their position changes.
    ///
    /// # Arguments
    /// * `delegatee` - The delegatee identifier (owner address + optional seed)
    /// * `farm_id` - The unique identifier of the farm (32-byte hash)
    /// * `new_stake` - The new total stake amount (replaces previous stake)
    ///
    /// # Authorization
    /// The calling contract must be registered as the `delegate_authority` for this farm.
    /// Calls from unauthorized addresses will fail.
    fn set_stake_delegated(e: Env, delegatee: Delegatee, farm_id: BytesN<32>, new_stake: i128);
}
