use soroban_fixed_point_math::FixedPoint;

use crate::{
    constants::BPS_FACTOR,
    error::FCError,
    state::{DelegateeState, Delegation, Farm, LockingMode},
    utils::MathUtils,
};

/// Calculates the early withdrawal penalty using linear decay.
///
/// Penalty decreases linearly from full penalty at lock start to zero at lock end.
/// Formula: `effective_penalty_bps = penalty_bps × time_remaining / total_duration`
///
/// Rounding uses ceil (protocol-favorable) to prevent dust-amount penalty evasion.
pub fn calculate_early_withdrawal_penalty(
    farm: &Farm,
    delegatee_state: &DelegateeState,
    amount: i128,
    current_ts: u64,
) -> Result<(i128, i128), FCError> {
    let Delegation::NonDelegated(cfg) = &farm.config.delegation else {
        return Err(FCError::DelegatedFarm);
    };

    if cfg.early_withdrawal_penalty_bps == 0 || cfg.locking_duration == 0 {
        return Ok((amount, 0));
    }

    let lock_start = match cfg.locking_mode {
        LockingMode::None => return Ok((amount, 0)),
        LockingMode::Continuous => delegatee_state.last_stake_ts,
        LockingMode::WithExpiry => cfg.locking_ts,
    };

    let lock_end = lock_start.checked_add(cfg.locking_duration).map_over_or_underflow()?;

    if current_ts >= lock_end {
        return Ok((amount, 0));
    }

    if current_ts < lock_start {
        return Ok((amount, 0));
    }

    let time_remaining = lock_end - current_ts;
    let total_duration = cfg.locking_duration;

    let effective_penalty_bps = cfg
        .early_withdrawal_penalty_bps
        .fixed_mul_ceil(time_remaining as i128, total_duration as i128)
        .map_over_or_underflow()?;

    let penalty =
        amount.fixed_mul_ceil(effective_penalty_bps, BPS_FACTOR).map_over_or_underflow()?;

    let net_amount = amount.checked_sub(penalty).map_over_or_underflow()?;

    Ok((net_amount, penalty))
}
