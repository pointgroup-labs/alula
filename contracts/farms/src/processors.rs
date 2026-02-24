use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, Vec, vec as svec};

use crate::{
    constants::{BPS_FACTOR, SCALE_FACTOR},
    error::FCError,
    math::penalty::calculate_early_withdrawal_penalty,
    oracle,
    state::{Delegation, Farm, FarmingPosition, RewardInfo, RewardType},
    utils::MathUtils,
};

/// Adds stake to a non-delegated farm.
///
/// If a deposit warmup period is configured, the stake enters a pending state
/// and must be activated later via [`activate_pending_stake`].
/// Otherwise, the stake becomes active immediately.
///
/// Refreshes reward accumulators before modifying position state.
pub fn stake(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
    amount: i128,
) -> Result<(), FCError> {
    if amount <= 0 || amount < farm.config.min_stake_amount {
        return Err(FCError::InvalidAmount);
    }

    check_deposit_cap(e, farm, amount)?;

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

/// Initiates withdrawal from a non-delegated farm.
///
/// Applies an early-withdrawal penalty (linear decay) if the position is still
/// within its locking period. The net amount (after penalty) enters a pending
/// withdrawal state subject to a cooldown period before it can be claimed via
/// `withdraw_unstaked`.
///
/// Only one pending withdrawal is allowed at a time.
pub fn unstake(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
    amount: i128,
) -> Result<(), FCError> {
    let is_full_withdrawal = amount == farming_position.active_stake;
    if amount <= 0 || (!is_full_withdrawal && amount < farm.config.min_stake_amount) {
        return Err(FCError::InvalidAmount);
    }
    if farming_position.active_stake < amount {
        return Err(FCError::InsufficientStake);
    }
    if farming_position.pending_withdrawal_stake.is_positive() {
        return Err(FCError::PendingWithdrawalExists);
    }

    farm.non_delegated_config()?;

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
        farm.num_users = farm.num_users.checked_sub(1).map_over_or_underflow()?;
    }

    update_tallies(e, farm, farming_position)?;

    farm.total_staked = farm.total_staked.checked_sub(amount).map_over_or_underflow()?;

    farming_position.pending_withdrawal_stake =
        farming_position.pending_withdrawal_stake.checked_add(net).map_over_or_underflow()?;
    farming_position.pending_withdrawal_ts = current_ts;

    Ok(())
}

/// Claims accrued rewards for a single reward token.
///
/// Computes pending rewards from the rewards-per-share accumulator, adds any
/// unclaimed balance, deducts the treasury fee, and returns the net amount to
/// be transferred to the user. Resets the user's tally and unclaimed balance
/// for this reward token.
///
/// Respects `min_harvest_delay` — reverts with `ClaimTooSoon` if called before
/// the cooldown has elapsed since the last harvest.
pub fn harvest(
    e: &Env,
    farm: &mut Farm,
    reward_token: &Address,
    reward_info: &mut RewardInfo,
    farming_position: &mut FarmingPosition,
) -> Result<i128, FCError> {
    farm.refresh_rewards(e)?;
    *reward_info = RewardInfo::try_get(e, reward_token)?;
    harvest_after_refresh(e, farm, reward_token, reward_info, farming_position)
}

/// Claims accrued rewards across all reward tokens on the farm.
///
/// Iterates each configured reward token and calls [`harvest_after_refresh`].
/// Skips tokens with no pending rewards or that are still within their harvest
/// delay. Returns a vec of `(reward_token, net_amount)` pairs for each
/// successful harvest; the caller is responsible for executing the token
/// transfers.
pub fn harvest_all(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
) -> Result<Vec<(Address, i128)>, FCError> {
    let mut res: Vec<(Address, i128)> = svec![e];

    farm.refresh_rewards(e)?;

    for reward_token in farm.rewards.keys() {
        let mut reward_info = RewardInfo::try_get(e, &reward_token)?;

        match harvest_after_refresh(e, farm, &reward_token, &mut reward_info, farming_position) {
            Ok(amount) => {
                res.push_back((reward_token.clone(), amount));
            }
            Err(FCError::NoRewardsToHarvest) | Err(FCError::ClaimTooSoon) => continue,
            Err(err) => return Err(err),
        }

        reward_info.set(e, &reward_token);
    }

    Ok(res)
}

/// Inner harvest logic that assumes `farm.refresh_rewards` has already been
/// called and `reward_info` is up-to-date from storage.
fn harvest_after_refresh(
    e: &Env,
    farm: &Farm,
    reward_token: &Address,
    reward_info: &mut RewardInfo,
    farming_position: &mut FarmingPosition,
) -> Result<i128, FCError> {
    if !farm.rewards.contains_key(reward_token.clone()) {
        return Err(FCError::RewardDoesNotExistOnFarm);
    }

    let current_ts = e.ledger().timestamp();
    let last_claim = farming_position.last_claim_ts.get(reward_token.clone()).unwrap_or(0);

    if farm.config.min_harvest_delay > 0 {
        let next_claim_ts =
            last_claim.checked_add(farm.config.min_harvest_delay).map_over_or_underflow()?;
        if current_ts < next_claim_ts {
            return Err(FCError::ClaimTooSoon);
        }
    }

    let user_tally = farming_position.rewards_tallies.get(reward_token.clone()).unwrap_or(0);

    let stake = effective_stake(farming_position.active_stake, reward_info.reward_type);
    let pending_from_rps =
        calculate_pending_reward(stake, reward_info.accum_rewards_per_share_sc, user_tally)?;

    let unclaimed = farming_position.rewards_unclaimed.get(reward_token.clone()).unwrap_or(0);
    let total_pending = pending_from_rps.checked_add(unclaimed).map_over_or_underflow()?;

    if total_pending == 0 {
        return Err(FCError::NoRewardsToHarvest);
    }

    let fee = if farm.config.treasury_fee_bps > 0 {
        total_pending
            .fixed_mul_ceil(farm.config.treasury_fee_bps, BPS_FACTOR)
            .map_over_or_underflow()?
    } else {
        0
    };
    let net = total_pending.checked_sub(fee).map_over_or_underflow()?;

    if net == 0 {
        return Err(FCError::NoRewardsToHarvest);
    }

    let new_tally = stake
        .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
        .map_over_or_underflow()?;

    reward_info.rewards_issued_unclaimed =
        reward_info.rewards_issued_unclaimed.checked_sub(total_pending).map_over_or_underflow()?;

    if fee > 0 {
        reward_info.accumulated_treasury_fees =
            reward_info.accumulated_treasury_fees.checked_add(fee).map_over_or_underflow()?;
    }

    farming_position.rewards_tallies.set(reward_token.clone(), new_tally);
    farming_position.last_claim_ts.set(reward_token.clone(), current_ts);
    farming_position.rewards_unclaimed.set(reward_token.clone(), 0);

    Ok(net)
}

/// Cancels a pending deposit and returns the amount to be refunded.
///
/// Clears the pending deposit state without activating it.
/// The caller is responsible for transferring the tokens back to the user.
pub fn cancel_pending_deposit(
    farming_position: &mut FarmingPosition,
) -> Result<i128, FCError> {
    let amount = farming_position.pending_deposit_stake;
    if amount == 0 {
        return Err(FCError::NoPendingDeposit);
    }

    farming_position.pending_deposit_stake = 0;
    farming_position.pending_deposit_ts = 0;

    Ok(amount)
}

/// Converts a pending deposit into an active stake after the warmup period.
///
/// No-op if no pending deposit exists. Fails with `WarmupNotComplete` if
/// called before the warmup period has elapsed.
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
    farming_position.pending_deposit_ts = 0;

    check_deposit_cap(e, farm, amount)?;

    farming_position.refresh_rewards(e, farm)?;
    activate_stake(e, farm, farming_position, amount)?;

    Ok(())
}

/// Sets a user's stake in a delegated farm to an absolute value.
///
/// Called by the delegate authority (e.g. a lending market) to sync a user's
/// farm stake with their actual position. Handles both increases and decreases:
///
/// - **Increase** (`new_stake > active_stake`): enforces deposit cap, increments
///   `num_users` if this is the user's first active stake.
/// - **Decrease to zero**: decrements `num_users`.
/// - **No change**: returns early without modifying state.
///
/// No-op penalty or cooldown logic — delegated farms trust the delegate
/// authority to manage position lifecycle.
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

    if diff > 0 {
        check_deposit_cap(e, farm, diff)?;

        if is_new_user || farming_position.active_stake == 0 {
            farm.num_users = farm.num_users.checked_add(1).map_over_or_underflow()?;
        }

        farming_position.last_stake_ts = e.ledger().timestamp();
    } else if new_stake == 0 {
        farm.num_users = farm.num_users.checked_sub(1).map_over_or_underflow()?;
    }

    let new_total = farm.total_staked.checked_add(diff).map_over_or_underflow()?;
    if new_total < 0 {
        return Err(FCError::InternalError);
    }
    farm.total_staked = new_total;
    farming_position.active_stake = new_stake;

    update_tallies(e, farm, farming_position)?;

    Ok(())
}

/// Withdraws unissued reward tokens from the farm's available reward pool.
///
/// Refreshes reward accumulators first to ensure `rewards_available` reflects
/// all emissions up to the current ledger timestamp before subtracting.
pub fn withdraw_unused(
    e: &Env,
    amount: i128,
    farm: &mut Farm,
    reward_token: &Address,
    reward_info: &mut RewardInfo,
) -> Result<(), FCError> {
    farm.refresh_rewards(e)?;
    *reward_info = RewardInfo::try_get(e, reward_token)?;

    if amount > reward_info.rewards_available {
        return Err(FCError::InsufficientAvailableRewards);
    }
    reward_info.rewards_available =
        reward_info.rewards_available.checked_sub(amount).map_over_or_underflow()?;

    Ok(())
}

fn check_deposit_cap(e: &Env, farm: &Farm, additional_amount: i128) -> Result<(), FCError> {
    if farm.config.deposit_cap <= 0 {
        return Ok(());
    }

    let new_total = farm.total_staked.checked_add(additional_amount).map_over_or_underflow()?;

    let cap_value = if let Some(oracle_config) = farm.config.oracle.as_ref() {
        let price = oracle::get_oracle_price(e, &farm.config.token, oracle_config)?;
        let decimals = oracle::get_oracle_decimals(e, oracle_config)?;
        new_total
            .checked_mul(price)
            .map_over_or_underflow()?
            .checked_div(10_i128.pow(decimals))
            .map_over_or_underflow()?
    } else {
        new_total
    };

    if cap_value > farm.config.deposit_cap {
        return Err(FCError::DepositCapExceeded);
    }

    Ok(())
}

/// Moves tokens from pending deposit into active stake, updating farm totals
/// and reward tallies. Increments `num_users` if this is the position's first
/// active stake.
fn activate_stake(
    e: &Env,
    farm: &mut Farm,
    farming_position: &mut FarmingPosition,
    amount: i128,
) -> Result<(), FCError> {
    let is_first_stake = farming_position.active_stake == 0;

    farming_position.active_stake =
        farming_position.active_stake.checked_add(amount).map_over_or_underflow()?;
    farm.total_staked = farm.total_staked.checked_add(amount).map_over_or_underflow()?;

    update_tallies(e, farm, farming_position)?;

    if is_first_stake {
        farm.num_users = farm.num_users.checked_add(1).map_over_or_underflow()?;
    }

    Ok(())
}

/// Recalculates the user's reward tallies for all reward tokens.
///
/// Sets each tally to `effective_stake × accum_rewards_per_share` so that
/// subsequent reward calculations only count rewards accrued after this point.
fn update_tallies(
    e: &Env,
    farm: &Farm,
    farming_position: &mut FarmingPosition,
) -> Result<(), FCError> {
    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, &reward_token)?;
        let stake = effective_stake(farming_position.active_stake, reward_info.reward_type);
        let new_tally = stake
            .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
            .map_over_or_underflow()?;
        farming_position.rewards_tallies.set(reward_token, new_tally);
    }
    Ok(())
}

/// Computes pending (unharvested) rewards for a user.
///
/// `entitled = stake × reward_per_share` (floor), then `pending = entitled - tally`.
/// Returns 0 if the user has no stake or if the tally exceeds entitled (rounding).
pub fn calculate_pending_reward(
    user_stake: i128,
    reward_per_share_scaled: i128,
    user_tally: i128,
) -> Result<i128, FCError> {
    if user_stake == 0 {
        return Ok(0);
    }

    let entitled = user_stake
        .fixed_mul_floor(reward_per_share_scaled, SCALE_FACTOR)
        .map_over_or_underflow()?;

    let pending = entitled.checked_sub(user_tally).map_over_or_underflow()?;

    Ok(pending.max(0))
}

/// Determines how many reward tokens should be emitted between the last
/// issuance timestamp and `current_ts`.
///
/// Integrates the reward schedule curve over the elapsed period, then caps the
/// result at `rewards_available` to prevent over-emission.
/// Returns 0 if there are no stakers or no time has elapsed.
pub fn calculate_rewards_to_issue(
    farm: &Farm,
    reward_info: &RewardInfo,
    current_ts: u64,
) -> Result<i128, FCError> {
    let has_stakers = match reward_info.reward_type {
        RewardType::Proportional => farm.total_staked > 0,
        RewardType::Constant => farm.num_users > 0,
    };
    if !has_stakers {
        return Ok(0);
    }

    let from_ts = reward_info.last_issuance_ts;
    if from_ts >= current_ts {
        return Ok(0);
    }

    let rewards_from_curve =
        reward_info.reward_schedule_curve.calculate_rewards(from_ts, current_ts)?;
    let rewards_to_issue = rewards_from_curve.min(reward_info.rewards_available);

    Ok(rewards_to_issue)
}

/// Returns the effective stake for reward calculation purposes.
///
/// - `Proportional`: rewards scale with stake amount (standard).
/// - `Constant`: every staker with `active_stake > 0` counts as 1 (equal share).
pub fn effective_stake(active_stake: i128, reward_type: RewardType) -> i128 {
    match reward_type {
        RewardType::Proportional => active_stake,
        RewardType::Constant => i128::from(active_stake > 0),
    }
}
