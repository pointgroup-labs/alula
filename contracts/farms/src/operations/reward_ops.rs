use soroban_sdk::{Env, token};

use crate::{
    constants::BPS_FACTOR,
    error::FarmsError,
    events,
    math::utils::fixed_mul_floor,
    operations::farm_ops::{
        calculate_pending_reward, get_current_ts, refresh_global_rewards, update_user_rewards_tally,
    },
    state::{Delegatee, FarmState, GlobalConfig, RewardScheduleCurve, UserState},
    storage,
};

/// Adds rewards to a farm's reward pool
///
/// # Arguments
/// * `e` - The environment
/// * `farm` - Mutable reference to the farm state
/// * `reward_index` - Index of the reward token
/// * `amount` - Amount of rewards to add
/// * `funder` - Address funding the rewards
///
/// # Returns
/// * `Ok(())` on success
pub fn add_rewards(
    e: &Env,
    farm: &mut FarmState,
    reward_index: u32,
    amount: i128,
    funder: &soroban_sdk::Address,
) -> Result<(), FarmsError> {
    if amount <= 0 {
        return Err(FarmsError::InvalidAmount);
    }

    if reward_index >= farm.reward_infos.len() {
        return Err(FarmsError::RewardNotFound);
    }

    let mut reward_info = farm.reward_infos.get(reward_index).ok_or(FarmsError::InternalError)?;

    // Transfer tokens from funder to vault
    let token_client = token::Client::new(e, &reward_info.token);
    token_client.transfer(funder, &reward_info.rewards_vault, &amount);

    // But how are they distributed?

    // Update available rewards
    reward_info.rewards_available =
        reward_info.rewards_available.checked_add(amount).ok_or(FarmsError::Overflow)?;

    farm.reward_infos.set(reward_index, reward_info);

    // Persist changes
    storage::set_farm(e, &farm.farm_id, farm);

    events::emit_rewards_added(e, &farm.farm_id, reward_index, amount);

    Ok(())
}

/// Updates the reward schedule for a reward token
///
/// # Arguments
/// * `e` - The environment
/// * `farm` - Mutable reference to the farm state
/// * `reward_index` - Index of the reward token
/// * `schedule` - New reward schedule curve
///
/// # Returns
/// * `Ok(())` on success
pub fn update_reward_schedule(
    e: &Env,
    farm: &mut FarmState,
    reward_index: u32,
    schedule: RewardScheduleCurve,
) -> Result<(), FarmsError> {
    // Validate schedule
    schedule.validate()?; // How do we validate this...

    if reward_index >= farm.reward_infos.len() {
        return Err(FarmsError::RewardNotFound);
    }

    refresh_global_rewards(e, farm)?; // So, this 
    // increases the accumulative reward per share value

    let mut reward_info = farm.reward_infos.get(reward_index).ok_or(FarmsError::InternalError)?;
    reward_info.reward_schedule = schedule; // I don't like the idea of updating the schedule here...
    farm.reward_infos.set(reward_index, reward_info);

    // Persist changes
    storage::set_farm(e, &farm.farm_id, farm);

    events::emit_reward_schedule_updated(e, &farm.farm_id, reward_index);

    Ok(())
}

/// Withdraws unused rewards from a farm
///
/// # Arguments
/// * `e` - The environment
/// * `farm` - Mutable reference to the farm state
/// * `reward_index` - Index of the reward token
/// * `amount` - Amount to withdraw
/// * `recipient` - Address to receive the rewards
///
/// # Returns
/// * `Ok(())` on success
pub fn withdraw_unused_rewards(
    e: &Env,
    farm: &mut FarmState,
    reward_index: u32,
    amount: i128,
    recipient: &soroban_sdk::Address,
) -> Result<(), FarmsError> {
    if amount <= 0 {
        return Err(FarmsError::InvalidAmount);
    }

    if reward_index >= farm.reward_infos.len() {
        return Err(FarmsError::RewardNotFound);
    }

    // Refresh rewards first to get accurate available amount
    refresh_global_rewards(e, farm)?;

    let mut reward_info = farm.reward_infos.get(reward_index).ok_or(FarmsError::InternalError)?;

    if amount > reward_info.rewards_available {
        // Simply decreases the available....
        return Err(FarmsError::InsufficientRewards);
    }

    // Transfer from vault to recipient
    let token_client = token::Client::new(e, &reward_info.token);
    token_client.transfer(&reward_info.rewards_vault, recipient, &amount);

    // Update available rewards
    reward_info.rewards_available =
        reward_info.rewards_available.checked_sub(amount).ok_or(FarmsError::Underflow)?;

    farm.reward_infos.set(reward_index, reward_info);

    // Persist changes
    storage::set_farm(e, &farm.farm_id, farm);

    events::emit_rewards_withdrawn(e, &farm.farm_id, reward_index, amount);

    Ok(())
}

/// Harvests rewards for a user for a specific reward token
///
/// # Arguments
/// * `e` - The environment
/// * `delegatee` - The delegatee identifier (for storage key)
/// * `config` - Global config (for treasury fee)
/// * `farm` - Mutable reference to the farm state
/// * `user_state` - Mutable reference to the user state
/// * `reward_index` - Index of the reward token to harvest
///
/// # Returns
/// * `Ok(net_amount)` - The net amount harvested after fees
pub fn harvest_single(
    // Shouldn't this just decrease the pending?
    e: &Env,
    delegatee: &Delegatee,
    config: &GlobalConfig,
    farm: &mut FarmState,
    user_state: &mut UserState,
    reward_index: u32,
) -> Result<i128, FarmsError> {
    if reward_index >= farm.reward_infos.len() {
        return Err(FarmsError::RewardNotFound);
    }

    // Refresh global rewards
    refresh_global_rewards(e, farm)?;

    let current_ts = get_current_ts(e, farm);

    // Check min claim duration
    let last_claim = user_state.last_claim_ts.get(reward_index).unwrap_or(0);
    let reward_info = farm.reward_infos.get(reward_index).ok_or(FarmsError::InternalError)?;

    if reward_info.min_claim_duration > 0 {
        let next_claim_ts =
            last_claim.checked_add(reward_info.min_claim_duration).ok_or(FarmsError::Overflow)?;
        if current_ts < next_claim_ts {
            return Err(FarmsError::ClaimTooSoon);
        }
    }

    // Calculate pending rewards
    let user_tally = user_state.rewards_tally_scaled.get(reward_index).unwrap_or(0);
    let pending_from_rps = calculate_pending_reward(
        user_state.active_stake,
        reward_info.reward_per_share_scaled,
        user_tally,
    )?;

    // This pending must be added to the issued, right?

    let unclaimed = user_state.rewards_unclaimed.get(reward_index).unwrap_or(0);
    let total_pending = pending_from_rps.checked_add(unclaimed).ok_or(FarmsError::Overflow)?;

    if total_pending == 0 {
        return Err(FarmsError::NoRewardsToHarvest);
    }

    // Calculate treasury fee using fixed-point multiplication
    // fee = (total_pending * treasury_fee_bps) / BPS_FACTOR
    let fee = if config.treasury_fee_bps > 0 {
        fixed_mul_floor(total_pending, config.treasury_fee_bps, BPS_FACTOR)?
    } else {
        0
    };

    let net_amount = total_pending.checked_sub(fee).ok_or(FarmsError::Underflow)?;

    // Transfer rewards
    let token_client = token::Client::new(e, &reward_info.token);

    // Transfer net amount to owner
    if net_amount > 0 {
        token_client.transfer(&reward_info.rewards_vault, &user_state.owner, &net_amount);
    }

    // Transfer fee to treasury
    if fee > 0 {
        token_client.transfer(&reward_info.rewards_vault, &config.treasury_vault, &fee);
    }

    // Update reward info
    let mut reward_info = farm.reward_infos.get(reward_index).ok_or(FarmsError::InternalError)?;
    reward_info.rewards_issued_unclaimed = // this is fine, because of refresh previously
        reward_info.rewards_issued_unclaimed.checked_sub(total_pending).unwrap_or(0); // Use unwrap_or(0) as safety

    // Update tally
    let new_tally = // stake is unchanged
        update_user_rewards_tally(user_state.active_stake, reward_info.reward_per_share_scaled);

    farm.reward_infos.set(reward_index, reward_info);

    // Update user state
    user_state.rewards_unclaimed.set(reward_index, 0); // must be this per `User`
    user_state.last_claim_ts.set(reward_index, current_ts);
    user_state.rewards_tally_scaled.set(reward_index, new_tally); // for now we can make it per reward

    // Persist changes
    storage::set_farm(e, &farm.farm_id, farm);
    storage::set_user(e, delegatee, &farm.farm_id, user_state);

    events::emit_harvest(e, &user_state.owner, &farm.farm_id, reward_index, net_amount, fee);

    Ok(net_amount)
}

/// Harvests all rewards for a user
///
/// # Arguments
/// * `e` - The environment
/// * `delegatee` - The delegatee identifier (for storage key)
/// * `config` - Global config
/// * `farm` - Mutable reference to the farm state
/// * `user_state` - Mutable reference to the user state
///
/// # Returns
/// * `Ok(total_harvested)` - Total amount harvested across all reward tokens
pub fn harvest_all(
    e: &Env,
    delegatee: &Delegatee,
    config: &GlobalConfig,
    farm: &mut FarmState,
    user_state: &mut UserState,
) -> Result<i128, FarmsError> {
    let mut total_harvested: i128 = 0;

    for i in 0..farm.num_reward_tokens {
        match harvest_single(e, delegatee, config, farm, user_state, i) {
            Ok(amount) => {
                total_harvested =
                    total_harvested.checked_add(amount).ok_or(FarmsError::Overflow)?;
            }
            Err(FarmsError::NoRewardsToHarvest) => continue, // Skip if no rewards
            Err(FarmsError::ClaimTooSoon) => continue,       // Skip if too soon
            Err(e) => return Err(e),
        }
    }

    Ok(total_harvested)
}

/// Gets the pending rewards for a user across all reward tokens
///
/// # Arguments
/// * `e` - The environment
/// * `farm` - Reference to the farm state
/// * `user_state` - Reference to the user state
///
/// # Returns
/// * Vector of pending reward amounts
pub fn get_pending_rewards_with_env(
    //with env?
    e: &Env,
    farm: &FarmState,
    user_state: &UserState,
) -> Result<soroban_sdk::Vec<i128>, FarmsError> {
    use soroban_sdk::vec;

    let mut pending_rewards = vec![e];

    for i in 0..farm.reward_infos.len() {
        if let Some(reward_info) = farm.reward_infos.get(i) {
            let user_tally = user_state.rewards_tally_scaled.get(i).unwrap_or(0);
            let pending_from_rps = calculate_pending_reward(
                user_state.active_stake,
                reward_info.reward_per_share_scaled,
                user_tally,
            )?;

            let unclaimed = user_state.rewards_unclaimed.get(i).unwrap_or(0);
            let total = pending_from_rps.checked_add(unclaimed).ok_or(FarmsError::Overflow)?;

            pending_rewards.push_back(total);
        }
    }

    Ok(pending_rewards)
}
