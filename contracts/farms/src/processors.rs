use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, Vec, vec as svec};

use crate::{
    constants::{BPS_FACTOR, SCALE_FACTOR},
    error::FCError,
    state::{Delegation, Farm, FarmingPosition, RewardInfo},
    utils::MathUtils,
};

pub fn withdraw_unused(
    e: &Env,
    amount: i128,
    farm: &mut Farm,
    reward_info: &mut RewardInfo,
) -> Result<(), FCError> {
    farm.refresh_rewards(e)?;

    if amount > reward_info.rewards_available {
        return Err(FCError::InsufficientAvailableRewards);
    }
    reward_info.rewards_available -= amount; // safe 

    Ok(())
}

pub fn stake(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
    amount: i128,
) -> Result<(), FCError> {
    if amount < farm.config.min_stake_amount {
        return Err(FCError::InvalidAmount);
    }

    if farm.config.deposit_cap.is_positive() {
        let new_total = farm.total_staked.checked_add(amount).map_over_or_underflow()?;

        if new_total > farm.config.deposit_cap {
            return Err(FCError::DepositCapExceeded);
        }
    }

    farm.refresh_rewards(e)?;
    farming_position.refresh_rewards(e, farm)?;

    let current_ts = e.ledger().timestamp();
    let Delegation::NonDelegated(delegation_config) = &farm.config.delegation else {
        return Err(FCError::DelegatedFarm);
    };

    if delegation_config.deposit_warmup_period > 0 {
        farming_position.pending_deposit_stake =
            farming_position.pending_deposit_stake.checked_add(amount).map_over_or_underflow()?;
        farming_position.pending_deposit_ts = current_ts;
    } else {
        activate_stake(e, farm, farming_position, amount)?;
    }

    farming_position.last_stake_ts = current_ts;

    Ok(())
}

pub fn unstake(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
    amount: i128,
) -> Result<(), FCError> {
    if amount < farm.config.min_stake_amount {
        return Err(FCError::InvalidAmount);
    }
    if farming_position.active_stake < amount {
        return Err(FCError::InsufficientStake);
    }
    if farming_position.pending_withdrawal_stake.is_positive() {
        return Err(FCError::PendingWithdrawalExists);
    }

    farm.refresh_rewards(e)?;
    farming_position.refresh_rewards(e, farm)?;

    let current_ts = e.ledger().timestamp();
    let (net, penalty) =
        calculate_early_withdrawal_penalty(farm, farming_position, amount, current_ts)?;

    if penalty.is_positive() {
        farm.current_slashed_amount =
            farm.current_slashed_amount.checked_add(penalty).map_over_or_underflow()?;
        farm.cumulative_slashed_amount =
            farm.cumulative_slashed_amount.checked_add(penalty).map_over_or_underflow()?;
    }

    farming_position.active_stake =
        farming_position.active_stake.checked_sub(amount).map_over_or_underflow()?;
    if farming_position.active_stake == 0 {
        farm.num_users -= 1; // safe?
    }

    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, farm.id, &reward_token)?;
        let new_tally = farming_position
            .active_stake
            .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
            .map_over_or_underflow()?;
        farming_position.rewards_tallies.set(reward_token.clone(), new_tally);
    }

    farm.total_staked = farm.total_staked.checked_sub(amount).map_over_or_underflow()?;

    if farm.config.withdrawal_cooldown_period > 0 {
        farming_position.pending_withdrawal_stake =
            farming_position.pending_withdrawal_stake.checked_add(net).map_over_or_underflow()?;
        farming_position.pending_withdrawal_ts = current_ts;
    }

    Ok(())
}

pub fn harvest(
    e: &Env,
    farm: &mut Farm,
    reward_token: &Address,
    reward_info: &mut RewardInfo,
    farming_position: &mut FarmingPosition,
) -> Result<i128, FCError> {
    if !farm.rewards.contains_key(reward_token.clone()) {
        return Err(FCError::RewardDoesNotExistOnFarm);
    }

    farm.refresh_rewards(e)?;
    let current_ts = e.ledger().timestamp();
    let last_claim = farming_position.last_claim_ts.get(reward_token.clone()).unwrap_or(0);

    if reward_info.min_claim_duration > 0 {
        let next_claim_ts =
            last_claim.checked_add(reward_info.min_claim_duration).map_over_or_underflow()?;
        if current_ts < next_claim_ts {
            return Err(FCError::ClaimTooSoon);
        }
    }

    let user_tally = farming_position.rewards_tallies.get(reward_token.clone()).unwrap_or(0);
    let pending_from_rps = calculate_pending_reward(
        farming_position.active_stake,
        reward_info.accum_rewards_per_share_sc,
        user_tally,
    )?;

    let unclaimed = farming_position
        .rewards_unclaimed
        .get(reward_token.clone())
        .ok_or(FCError::InternalError)?; // TODO: Event?
    let total_pending = pending_from_rps.checked_add(unclaimed).map_over_or_underflow()?;

    if total_pending == 0 {
        return Err(FCError::NoRewardsToHarvest);
    }

    let fee = if farm.config.treasury_fee_bps.is_positive() {
        total_pending
            .fixed_mul_ceil(farm.config.treasury_fee_bps, BPS_FACTOR)
            .map_over_or_underflow()?
    } else {
        0
    };
    let net = total_pending - fee; // safe

    let new_tally = farming_position
        .active_stake
        .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
        .map_over_or_underflow()?;

    reward_info.rewards_issued_unclaimed =
        reward_info.rewards_issued_unclaimed.checked_add(total_pending).map_over_or_underflow()?;

    farming_position.rewards_tallies.set(reward_token.clone(), new_tally);
    farming_position.last_claim_ts.set(reward_token.clone(), current_ts);
    farming_position.rewards_unclaimed.set(reward_token.clone(), 0);

    Ok(net)
}

pub fn harvest_all(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
) -> Result<Vec<(Address, i128)>, FCError> {
    let mut res: Vec<(Address, i128)> = svec![e];

    for reward_token in farm.rewards.keys() {
        let mut reward_info = RewardInfo::try_get(e, farm.id, &reward_token)?;

        match harvest(e, farm, &reward_token, &mut reward_info, farming_position) {
            Ok(amount) => {
                res.push_back((reward_token.clone(), amount));
            }
            Err(FCError::NoRewardsToHarvest) | Err(FCError::ClaimTooSoon) => continue,
            Err(err) => return Err(err),
        }

        reward_info.set(e, farm.id, &reward_token);
    }

    Ok(res)
}

pub fn calculate_pending_reward(
    user_stake: i128,
    reward_per_share_scaled: i128,
    user_rewards_tally_scaled: i128,
) -> Result<i128, FCError> {
    if user_stake == 0 {
        return Ok(0);
    }

    let entitled_scaled =
        user_stake.checked_mul(reward_per_share_scaled).map_over_or_underflow()?;
    let pending_scaled = entitled_scaled.saturating_sub(user_rewards_tally_scaled);

    let pending = pending_scaled.checked_div(SCALE_FACTOR).map_over_or_underflow()?;

    Ok(pending.max(0)) // is this fine, though?
}

/// Calculates the amount of reward token to issue in total
pub fn calculate_rewards_to_issue(
    farm: &Farm,
    reward_info: &RewardInfo,
    current_ts: u64,
) -> Result<i128, FCError> {
    if farm.total_staked == 0 {
        return Ok(0);
    }

    let from_ts = reward_info.last_issuance_ts;
    if from_ts >= current_ts {
        return Ok(0); // TODO: Internal Error??
    }

    let rewards_from_curve =
        reward_info.reward_schedule_curve.calculate_rewards(from_ts, current_ts)?;
    let rewards_to_issue = rewards_from_curve.min(reward_info.rewards_available);

    Ok(rewards_to_issue)
}

// TODO: Move to math
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

/// Activates pending deposit stake (after warmup period)
pub fn activate_pending_stake(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
) -> Result<(), FCError> {
    farm.refresh_rewards(e)?;

    if farming_position.pending_deposit_stake == 0 {
        return Ok(());
    }
    let Delegation::NonDelegated(delegation_config) = &farm.config.delegation else {
        return Err(FCError::DelegatedFarm);
    };

    let current_ts = e.ledger().timestamp();

    let warmup_end = farming_position
        .pending_deposit_ts
        .checked_add(delegation_config.deposit_warmup_period)
        .map_over_or_underflow()?;
    if current_ts < warmup_end {
        return Err(FCError::WarmupNotComplete);
    }

    let amount = farming_position.pending_deposit_stake;
    farming_position.pending_deposit_stake = 0;
    farming_position.pending_deposit_ts = 0; // TODO: Is this ok?

    activate_stake(e, farm, farming_position, amount)?;

    Ok(())
}

pub fn set_stake_delegated(
    e: &Env,
    new_stake: i128,
    farm: &mut Farm,
    is_new_user: bool,
    farming_position: &mut FarmingPosition,
) -> Result<(), FCError> {
    farm.refresh_rewards(e)?;
    farming_position.refresh_rewards(e, farm)?;

    if new_stake == farming_position.active_stake {
        return Ok(());
    }

    let diff = new_stake.checked_sub(farming_position.active_stake).map_over_or_underflow()?;
    if diff.is_positive() {
        if farm.config.deposit_cap.is_positive() {
            let new_total = farm.total_staked.checked_add(diff).map_over_or_underflow()?;

            if new_total > farm.config.deposit_cap {
                return Err(FCError::DepositCapExceeded);
            }
        }

        farming_position.last_stake_ts = e.ledger().timestamp();
    } else if new_stake == 0 && !is_new_user {
        farm.num_users -= 1; // safe?
    }

    farm.total_staked = farm.total_staked.checked_add(diff).map_over_or_underflow()?; // safe?
    farming_position.active_stake = new_stake;

    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, farm.id, &reward_token)?;

        let new_tally = new_stake
            .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
            .map_over_or_underflow()?;
        farming_position.rewards_tallies.set(reward_token, new_tally);
    }

    Ok(())
}

fn activate_stake(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
    amount: i128,
) -> Result<(), FCError> {
    farming_position.active_stake =
        farming_position.active_stake.checked_add(amount).map_over_or_underflow()?;
    farm.total_staked = farm.total_staked.checked_add(amount).map_over_or_underflow()?;

    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, farm.id, &reward_token)?;

        // TODO: 'ceil' or 'floor'?
        let new_tally = farming_position
            .active_stake
            .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
            .map_over_or_underflow()?;
        farming_position.rewards_tallies.set(reward_token.clone(), new_tally);
    }

    if farming_position.active_stake == amount {
        farm.num_users -= 1; // WARN: safe?
    }

    Ok(())
}
