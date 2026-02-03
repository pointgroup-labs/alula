use soroban_fixed_point_math::FixedPoint;

use crate::{
    constants::BPS_FACTOR,
    error::FCError,
    state::{Delegation, Farm, FarmingPosition},
    utils::MathUtils,
};

pub fn calculate_early_withdrawal_penalty(
    farm: &Farm,
    farming_position: &FarmingPosition,
    amount: i128,
    current_ts: u64,
) -> Result<(i128, i128), FCError> {
    let Delegation::NonDelegated(delegation_config) = &farm.config.delegation else {
        return Err(FCError::DelegatedFarm);
    };

    if delegation_config.early_withdrawal_penalty_bps == 0 || farm.config.locking_duration == 0 {
        return Ok((amount, 0));
    }

    use crate::state::LockingMode;
    let (_, lock_end) = match farm.config.locking_mode {
        LockingMode::None => return Ok((amount, 0)),
        LockingMode::Continuous => {
            let start = farming_position.last_stake_ts;
            let end = start.checked_add(farm.config.locking_duration).map_over_or_underflow()?;

            (start, end)
        }
        LockingMode::WithExpiry => {
            let start = farm.locking_start;
            let end = start.checked_add(farm.config.locking_duration).map_over_or_underflow()?;

            (start, end)
        }
    };
    if current_ts >= lock_end {
        return Ok((amount, 0));
    }

    let time_remaining = lock_end - current_ts; // safe
    let total_duration = farm.config.locking_duration;

    let effective_penalty_bps = delegation_config
        .early_withdrawal_penalty_bps
        .fixed_mul_ceil(time_remaining as i128, total_duration as i128)
        .map_over_or_underflow()?;
    let penalty =
        amount.fixed_mul_ceil(effective_penalty_bps, BPS_FACTOR).map_over_or_underflow()?;

    let net = amount - penalty; // safe

    Ok((net, penalty))
}
