//! Farms Contract Interface
#![no_std]

use soroban_sdk::{Address, BytesN, Env, contractclient};

/// Farms contract interface for cross-contract calls.
///
/// This trait defines the subset of Farms functions available for external contracts.
/// The generated [`FarmsClient`] can be used to invoke these functions on a deployed
/// Farms contract
#[contractclient(name = "FarmsClient")]
pub trait Farms {
    /// Updates a user's stake via delegated authority.
    ///
    /// This is the core function for the delegated staking pattern (push model).
    /// The delegate authority (e.g., a lending protocol) calls this function
    /// to sync a user's stake whenever their position changes.
    ///
    /// # Arguments
    /// * `user` - user whose stake is being updated
    /// * `farm_id` - unique identifier of the farm (32-byte hash)
    /// * `new_stake` - new total stake amount (replaces previous stake)
    ///
    /// # Authorization
    /// The calling contract must be registered as the `delegate_authority` for this farm.
    /// Calls from unauthorized addresses will fail
    fn set_stake_delegated(e: Env, user: Address, farm_id: BytesN<32>, new_stake: i128);
}
