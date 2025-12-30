//! Farms integration module for delegated staking.
//!
//! Handles cross-contract calls to the Farms contract for updating user stakes
//! when their positions change (deposit, withdraw, borrow, repay, liquidate).

use farms_interface::FarmsClient;
use soroban_sdk::{Address, BytesN, Env};

use crate::{
    error::MCError,
    events,
    obligation::{Obligation, ObligationKey},
    pool::Pool,
    storage,
};

/// Farm kind: supply (j-token) or debt (d-token)
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FarmKind {
    Supply,
    Debt,
}

/// Syncs a user's stake with the farms contract for a specific pool.
/// No-op if no farm is configured for the pool/kind
pub fn refresh_pool_farm_stake(
    e: &Env,
    farms_contract: &Address,
    obligation: &Obligation,
    obligation_key: &ObligationKey,
    pool: &Pool,
    kind: FarmKind,
) -> Result<(), MCError> {
    let farm_id = match kind {
        FarmKind::Supply => pool.farm_supply.as_ref(),
        FarmKind::Debt => pool.farm_debt.as_ref(),
    };

    let Some(farm_id) = farm_id else {
        return Ok(());
    };

    let stake = match kind {
        FarmKind::Supply => {
            obligation.deposits.get(pool.pool_address.clone()).map(|d| d.j_tokens).unwrap_or(0)
        }
        FarmKind::Debt => {
            obligation.borrows.get(pool.pool_address.clone()).map(|b| b.d_tokens).unwrap_or(0)
        }
    };

    FarmsClient::new(e, farms_contract).set_stake_delegated(
        &obligation_key.into(),
        farm_id,
        &stake,
    );

    Ok(())
}

/// Refreshes all farms for all pools in an obligation.
/// Returns count of (supply_farms, debt_farms) refreshed
pub fn refresh_all_obligation_farms(
    e: &Env,
    farms_contract: &Address,
    obligation_key: &ObligationKey,
) -> Result<(u32, u32), MCError> {
    let obligation = Obligation::try_get(e, obligation_key)?;
    let mut num_supply_farms = 0u32;
    let mut num_debt_farms = 0u32;

    for pool_address in obligation.deposits.keys() {
        let pool = Pool::try_get(e, &pool_address)?;
        if pool.farm_supply.is_some() {
            refresh_pool_farm_stake(
                e,
                farms_contract,
                &obligation,
                obligation_key,
                &pool,
                FarmKind::Supply,
            )?;
            num_supply_farms += 1;
        }
    }

    for pool_address in obligation.borrows.keys() {
        let pool = Pool::try_get(e, &pool_address)?;
        if pool.farm_debt.is_some() {
            refresh_pool_farm_stake(
                e,
                farms_contract,
                &obligation,
                obligation_key,
                &pool,
                FarmKind::Debt,
            )?;
            num_debt_farms += 1;
        }
    }

    events::obligation_farms_refreshed(e, &obligation_key.user, num_supply_farms, num_debt_farms);

    Ok((num_supply_farms, num_debt_farms))
}

/// Refreshes a pool's farm stake if farms contract is configured.
/// No-op otherwise
pub fn try_refresh_pool_farm(
    e: &Env,
    obligation: &Obligation,
    obligation_key: &ObligationKey,
    pool: &Pool,
    kind: FarmKind,
) -> Result<(), MCError> {
    if let Some(farms_contract) = storage::get_farms_contract(e) {
        refresh_pool_farm_stake(e, &farms_contract, obligation, obligation_key, pool, kind)?;
    }
    Ok(())
}

/// Sets a farm ID for a pool (supply or debt)
pub fn set_pool_farm(
    e: &Env,
    pool_address: &Address,
    farm_id: BytesN<32>,
    kind: FarmKind,
) -> Result<(), MCError> {
    let mut pool = Pool::try_get(e, pool_address)?;
    // Emit event before moving farm_id to avoid clone
    events::pool_farm_set(e, pool_address, &farm_id, kind == FarmKind::Supply);
    match kind {
        FarmKind::Supply => pool.farm_supply = Some(farm_id),
        FarmKind::Debt => pool.farm_debt = Some(farm_id),
    }
    pool.set(e);
    Ok(())
}

/// Sets the supply farm ID for a pool
#[inline]
pub fn set_pool_supply_farm(
    e: &Env,
    pool_address: &Address,
    farm_id: BytesN<32>,
) -> Result<(), MCError> {
    set_pool_farm(e, pool_address, farm_id, FarmKind::Supply)
}

/// Sets the debt farm ID for a pool
#[inline]
pub fn set_pool_debt_farm(
    e: &Env,
    pool_address: &Address,
    farm_id: BytesN<32>,
) -> Result<(), MCError> {
    set_pool_farm(e, pool_address, farm_id, FarmKind::Debt)
}

/// Clears all farm configuration for a pool
pub fn clear_pool_farms(e: &Env, pool_address: &Address) -> Result<(), MCError> {
    let mut pool = Pool::try_get(e, pool_address)?;
    pool.farm_supply = None;
    pool.farm_debt = None;
    pool.set(e);
    events::pool_farms_cleared(e, pool_address);
    Ok(())
}
