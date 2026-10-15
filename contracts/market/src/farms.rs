//! Farms integration module for delegated staking.
//!
//! Handles cross-contract calls to the Farms contract for updating user stakes
//! when their positions change (deposit, withdraw, borrow, repay, liquidate).

use farms_interface::FarmsClient;
use soroban_sdk::{Address, BytesN, Env, Vec};

use crate::{
    error::MCError,
    events,
    obligation::{Obligation, ObligationKey},
    pool::Pool,
    storage,
};

/// Farm kind: supply (j-token) or debt (d-token).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FarmKind {
    Supply,
    Debt,
}

/// Syncs a user's stake with the farms contract for a specific pool.
/// No-op if no farm is configured for the pool/kind
/// Supply farms pay on j-tokens; debt farms on d-tokens, except debt under a bad-debt request, which no
/// longer earns.
fn farm_stake(obligation: &Obligation, pool_address: &Address, kind: FarmKind) -> i128 {
    match kind {
        FarmKind::Supply => {
            obligation.deposits.get(pool_address.clone()).map(|d| d.j_tokens).unwrap_or(0)
        }
        FarmKind::Debt => {
            let under_bad_debt = obligation
                .insurance_fund_requests_ids
                .keys()
                .iter()
                .any(|(pool, _)| pool == *pool_address);
            if under_bad_debt {
                0
            } else {
                obligation.borrows.get(pool_address.clone()).map(|b| b.d_tokens).unwrap_or(0)
            }
        }
    }
}

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

    let stake = farm_stake(obligation, &pool.pool_address, kind);

    // Fallible on purpose: a refusing farm would otherwise abort every deposit and withdraw on the
    // pool, trapping lenders' funds. A stale stake traps nothing.
    if FarmsClient::new(e, farms_contract)
        .try_set_stake_delegated(&obligation_key.into(), farm_id, &stake)
        .is_err()
    {
        events::farm_stake_push_failed(
            e,
            pool.pool_address.clone(),
            farm_id.clone(),
            obligation_key.clone(),
            stake,
        );
    }

    Ok(())
}

/// Re-pushes both farms of each listed pool; an absent obligation pushes zero.
pub fn refresh_farm_stakes(
    e: &Env,
    farms_contract: &Address,
    obligation_key: &ObligationKey,
    pools: &Vec<Address>,
) -> Result<(), MCError> {
    let obligation =
        storage::get_obligation(e, obligation_key).unwrap_or_else(|| Obligation::new(e));
    let mut num_supply_farms = 0u32;
    let mut num_debt_farms = 0u32;

    for pool_address in pools.iter() {
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

    Ok(())
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
