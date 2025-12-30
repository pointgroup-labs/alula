use soroban_sdk::Env;

use crate::{
    error::FarmsError,
    events,
    math::penalty::calculate_early_withdrawal_penalty,
    operations::farm_ops::{get_current_ts, refresh_global_rewards, update_user_rewards_tally},
    state::{Delegatee, FarmState, UserState},
    storage,
};

/// Processes a stake operation
///
/// # Arguments
/// * `e` - The environment
/// * `delegatee` - The delegatee identifier (for storage key)
/// * `farm` - Mutable reference to the farm state
/// * `user_state` - Mutable reference to the user state
/// * `amount` - Amount to stake
///
/// # Returns
/// * `Ok(())` on success
pub fn process_stake(
    e: &Env,
    delegatee: &Delegatee,
    farm: &mut FarmState,
    user_state: &mut UserState,
    amount: i128,
) -> Result<(), FarmsError> {
    use crate::constants::MIN_STAKE_AMOUNT;

    if amount < MIN_STAKE_AMOUNT {
        return Err(FarmsError::InvalidAmount);
    }

    if farm.is_frozen {
        return Err(FarmsError::FarmFrozen);
    }

    // Check deposit cap
    if farm.deposit_cap > 0 {
        let new_total = farm.total_staked.checked_add(amount).ok_or(FarmsError::Overflow)?;
        if new_total > farm.deposit_cap {
            return Err(FarmsError::DepositCapExceeded);
        }
    }

    // Refresh global rewards first
    refresh_global_rewards(e, farm)?;

    // Refresh user's pending rewards before changing stake
    refresh_user_rewards(farm, user_state)?;

    let current_ts = get_current_ts(e, farm);

    if farm.deposit_warmup_period > 0 {
        // Stake goes to pending first
        // If there's already pending stake, add to it and reset timer
        user_state.pending_deposit_stake =
            user_state.pending_deposit_stake.checked_add(amount).ok_or(FarmsError::Overflow)?;
        user_state.pending_deposit_ts = current_ts;
    } else {
        // Immediate stake
        activate_stake(farm, user_state, amount)?;
    }

    // Update last stake timestamp for locking
    user_state.last_stake_ts = current_ts;

    // Persist changes
    storage::set_farm(e, &farm.farm_id, farm);
    storage::set_user(e, delegatee, &farm.farm_id, user_state);

    events::emit_stake(e, &user_state.owner, &farm.farm_id, amount);

    Ok(())
}

/// Processes an unstake operation
///
/// # Arguments
/// * `e` - The environment
/// * `delegatee` - The delegatee identifier (for storage key)
/// * `farm` - Mutable reference to the farm state
/// * `user_state` - Mutable reference to the user state
/// * `amount` - Amount to unstake
///
/// # Returns
/// * `Ok(net_amount)` - The net amount after any early withdrawal penalty
pub fn process_unstake(
    e: &Env,
    delegatee: &Delegatee,
    farm: &mut FarmState,
    user_state: &mut UserState,
    amount: i128,
) -> Result<i128, FarmsError> {
    use crate::constants::MIN_STAKE_AMOUNT;

    if amount < MIN_STAKE_AMOUNT {
        return Err(FarmsError::InvalidAmount);
    }

    if user_state.active_stake < amount {
        return Err(FarmsError::InsufficientStake);
    }

    // CRITICAL: Validate that user has no pending withdrawal waiting
    // Users must claim their pending withdrawal before initiating a new unstake
    // This prevents gaming the cooldown system and simplifies accounting
    if user_state.pending_withdrawal_stake > 0 {
        return Err(FarmsError::PendingWithdrawalExists);
    }

    // Refresh global rewards first
    refresh_global_rewards(e, farm)?;

    // Refresh user's pending rewards before changing stake
    refresh_user_rewards(farm, user_state)?;

    let current_ts = get_current_ts(e, farm);

    // Calculate penalty if locked (linear decay)
    let (net_amount, penalty) =
        calculate_early_withdrawal_penalty(farm, user_state, current_ts, amount)?;

    // Track slashed amounts for admin withdrawal
    if penalty > 0 {
        farm.slashed_amount_current =
            farm.slashed_amount_current.checked_add(penalty).ok_or(FarmsError::Overflow)?;
        farm.slashed_amount_cumulative =
            farm.slashed_amount_cumulative.checked_add(penalty).ok_or(FarmsError::Overflow)?;
    }

    // Reduce active stake
    user_state.active_stake =
        user_state.active_stake.checked_sub(amount).ok_or(FarmsError::Underflow)?;

    // Decrement user count if user has fully unstaked
    if user_state.active_stake == 0 {
        farm.num_users = farm.num_users.saturating_sub(1);
    }

    // Update user's rewards tally for the new stake amount
    for i in 0..farm.reward_infos.len() {
        if let Some(reward_info) = farm.reward_infos.get(i)
            && user_state.rewards_tally_scaled.get(i).is_some()
        {
            let new_tally = update_user_rewards_tally(
                user_state.active_stake,
                reward_info.reward_per_share_scaled,
            );
            user_state.rewards_tally_scaled.set(i, new_tally);
        }
    }

    // Update farm total
    farm.total_staked = farm.total_staked.checked_sub(amount).ok_or(FarmsError::Underflow)?;

    if farm.withdrawal_cooldown_period > 0 {
        // Net amount goes to pending withdrawal
        user_state.pending_withdrawal_stake = user_state
            .pending_withdrawal_stake
            .checked_add(net_amount)
            .ok_or(FarmsError::Overflow)?;
        user_state.pending_withdrawal_ts = current_ts;
    }

    // Persist changes
    storage::set_farm(e, &farm.farm_id, farm);
    storage::set_user(e, delegatee, &farm.farm_id, user_state);

    events::emit_unstake(e, &user_state.owner, &farm.farm_id, amount, penalty);

    Ok(net_amount)
}

/// Processes a withdraw of unstaked tokens (after cooldown)
///
/// # Arguments
/// * `e` - The environment
/// * `delegatee` - The delegatee identifier (for storage key)
/// * `farm` - Reference to the farm state
/// * `user_state` - Mutable reference to the user state
///
/// # Returns
/// * `Ok(amount)` - The amount available for withdrawal
pub fn process_withdraw_unstaked(
    e: &Env,
    delegatee: &Delegatee,
    farm: &FarmState,
    user_state: &mut UserState,
) -> Result<i128, FarmsError> {
    if user_state.pending_withdrawal_stake == 0 {
        return Err(FarmsError::InsufficientPendingWithdrawal);
    }

    let current_ts = get_current_ts(e, farm);
    let cooldown_end = user_state
        .pending_withdrawal_ts
        .checked_add(farm.withdrawal_cooldown_period)
        .ok_or(FarmsError::Overflow)?;

    if current_ts < cooldown_end {
        return Err(FarmsError::CooldownNotComplete);
    }

    let amount = user_state.pending_withdrawal_stake;
    user_state.pending_withdrawal_stake = 0;
    user_state.pending_withdrawal_ts = 0;

    // Persist changes
    storage::set_user(e, delegatee, &farm.farm_id, user_state);

    events::emit_withdraw_unstaked(e, &user_state.owner, &farm.farm_id, amount);

    Ok(amount)
}

/// Activates pending deposit stake (after warmup period)
pub fn activate_pending_stake(
    e: &Env,
    delegatee: &Delegatee,
    farm: &mut FarmState,
    user_state: &mut UserState,
) -> Result<(), FarmsError> {
    if user_state.pending_deposit_stake == 0 {
        return Ok(());
    }

    let current_ts = get_current_ts(e, farm);
    let warmup_end = user_state
        .pending_deposit_ts
        .checked_add(farm.deposit_warmup_period)
        .ok_or(FarmsError::Overflow)?;

    if current_ts < warmup_end {
        return Err(FarmsError::WarmupNotComplete);
    }

    // Refresh global rewards before activation so reward_per_share is up-to-date
    // This ensures the user's tally is set correctly for the current rps
    refresh_global_rewards(e, farm)?;

    let amount = user_state.pending_deposit_stake;
    user_state.pending_deposit_stake = 0;
    user_state.pending_deposit_ts = 0;

    activate_stake(farm, user_state, amount)?;

    // Persist changes
    storage::set_farm(e, &farm.farm_id, farm);
    storage::set_user(e, delegatee, &farm.farm_id, user_state);

    Ok(())
}

/// Internal: Activates stake (moves to active_stake and updates totals)
fn activate_stake(
    farm: &mut FarmState,
    user_state: &mut UserState,
    amount: i128,
) -> Result<(), FarmsError> {
    // Add to active stake
    user_state.active_stake =
        user_state.active_stake.checked_add(amount).ok_or(FarmsError::Overflow)?;

    // Update farm total
    farm.total_staked = farm.total_staked.checked_add(amount).ok_or(FarmsError::Overflow)?;

    // Ensure user vectors are sized correctly
    extend_user_reward_vectors(farm, user_state);

    // Update user's rewards tally so they don't receive rewards for period before staking
    for i in 0..farm.reward_infos.len() {
        if let Some(reward_info) = farm.reward_infos.get(i) {
            let new_tally = update_user_rewards_tally(
                user_state.active_stake,
                reward_info.reward_per_share_scaled,
            );
            // Safe to use set() now because extend_user_reward_vectors ensures length matches
            user_state.rewards_tally_scaled.set(i, new_tally);
        }
    }

    // Increment user count if this is first stake
    if user_state.active_stake == amount {
        farm.num_users = farm.num_users.saturating_add(1);
    }

    Ok(())
}

/// Refreshes user's pending rewards (before stake changes)
pub fn refresh_user_rewards(
    farm: &FarmState,
    user_state: &mut UserState,
) -> Result<(), FarmsError> {
    use crate::operations::farm_ops::calculate_pending_reward;

    // Ensure user vectors are sized to match farm's reward tokens
    extend_user_reward_vectors(farm, user_state);

    for i in 0..farm.reward_infos.len() {
        if let Some(reward_info) = farm.reward_infos.get(i) {
            let user_tally = user_state.rewards_tally_scaled.get(i).unwrap_or(0);

            let pending = calculate_pending_reward(
                user_state.active_stake,
                reward_info.reward_per_share_scaled,
                user_tally,
            )?;

            if pending > 0 {
                let current_unclaimed = user_state.rewards_unclaimed.get(i).unwrap_or(0);
                let new_unclaimed =
                    current_unclaimed.checked_add(pending).ok_or(FarmsError::Overflow)?;

                user_state.rewards_unclaimed.set(i, new_unclaimed);

                // Update tally
                let new_tally = update_user_rewards_tally(
                    user_state.active_stake,
                    reward_info.reward_per_share_scaled,
                );
                user_state.rewards_tally_scaled.set(i, new_tally);
            }
        }
    }

    Ok(())
}

/// Extends user reward vectors to match the number of reward tokens in the farm.
///
/// Called when a farm adds new reward tokens after a user has already initialized.
/// New entries are initialized with:
/// - rewards_tally_scaled: current stake × current rps (so user starts fresh for new token)
/// - rewards_unclaimed: 0
/// - last_claim_ts: 0
fn extend_user_reward_vectors(farm: &FarmState, user_state: &mut UserState) {
    let num_rewards = farm.reward_infos.len();
    let current_len = user_state.rewards_tally_scaled.len();

    // Extend vectors if farm has more reward tokens than user vectors
    for i in current_len..num_rewards {
        // For new reward tokens, set tally to current stake × rps
        // This ensures user doesn't get retroactive rewards for the new token
        let initial_tally = if let Some(reward_info) = farm.reward_infos.get(i) {
            update_user_rewards_tally(user_state.active_stake, reward_info.reward_per_share_scaled)
        } else {
            0
        };

        user_state.rewards_tally_scaled.push_back(initial_tally);
        user_state.rewards_unclaimed.push_back(0);
        user_state.last_claim_ts.push_back(0);
    }
}
