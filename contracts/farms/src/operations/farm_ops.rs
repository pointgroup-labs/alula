use soroban_sdk::Env;

use crate::{
    constants::SCALE_FACTOR,
    error::FarmsError,
    events,
    math::utils::fixed_div_floor,
    state::{FarmState, RewardInfo, TimeUnit},
};

/// Gets the current timestamp based on the farm's time unit
pub fn get_current_ts(e: &Env, farm: &FarmState) -> u64 {
    match farm.time_unit {
        TimeUnit::Seconds => e.ledger().timestamp(),
        TimeUnit::Slot => e.ledger().sequence() as u64,
    }
}

/// Refreshes global rewards for a farm, updating reward_per_share for all reward tokens.
///
/// This is the core of the Reward Per Share (RPS) algorithm:
/// 1. Calculate time elapsed since last issuance
/// 2. For each reward token, calculate new rewards based on the schedule curve
/// 3. Update reward_per_share: rps += new_rewards / total_staked
///
/// # Arguments
/// * `e` - The environment
/// * `farm` - Mutable reference to the farm state
///
/// # Returns
/// * `Ok(())` on success
pub fn refresh_global_rewards(e: &Env, farm: &mut FarmState) -> Result<(), FarmsError> {
    let current_ts = get_current_ts(e, farm);

    for i in 0..farm.reward_infos.len() {
        let mut reward_info = farm.reward_infos.get(i).ok_or(FarmsError::InternalError)?;

        let rewards_to_issue = calculate_rewards_to_issue(farm, &reward_info, current_ts)?;

        if rewards_to_issue > 0 {
            // Update reward_per_share (scaled)
            // reward_per_share += (rewards_to_issue * SCALE_FACTOR) / total_staked
            if farm.total_staked > 0 {
                // Using fixed-point division: (rewards * SCALE_FACTOR) / total_staked
                let rps_delta = fixed_div_floor(rewards_to_issue, SCALE_FACTOR, farm.total_staked)?;

                reward_info.reward_per_share_scaled = reward_info
                    .reward_per_share_scaled
                    .checked_add(rps_delta)
                    .ok_or(FarmsError::Overflow)?;
            }

            // Update accounting
            reward_info.rewards_available = reward_info
                .rewards_available
                .checked_sub(rewards_to_issue)
                .ok_or(FarmsError::Underflow)?;
            reward_info.rewards_issued_unclaimed = reward_info
                .rewards_issued_unclaimed
                .checked_add(rewards_to_issue)
                .ok_or(FarmsError::Overflow)?;
            reward_info.rewards_issued_cumulative = reward_info
                .rewards_issued_cumulative
                .checked_add(rewards_to_issue)
                .ok_or(FarmsError::Overflow)?;

            events::emit_rewards_accrued(e, &farm.farm_id, i, rewards_to_issue);
        }

        reward_info.last_issuance_ts = current_ts;
        farm.reward_infos.set(i, reward_info);
    }

    Ok(())
}

/// Calculates rewards to issue for a single reward token
fn calculate_rewards_to_issue(
    farm: &FarmState,
    reward_info: &RewardInfo,
    current_ts: u64,
) -> Result<i128, FarmsError> {
    if farm.total_staked == 0 {
        return Ok(0);
    }

    let from_ts = reward_info.last_issuance_ts;
    if from_ts >= current_ts {
        return Ok(0);
    }

    // Calculate rewards based on the emission curve
    let rewards_from_curve = reward_info.reward_schedule.calculate_rewards(from_ts, current_ts)?;

    // Cap at available rewards
    let rewards_to_issue = rewards_from_curve.min(reward_info.rewards_available);

    Ok(rewards_to_issue)
}

/// Initializes a new reward token for a farm
pub fn initialize_reward_info(
    e: &Env,
    reward_token: &soroban_sdk::Address,
    rewards_vault: &soroban_sdk::Address,
) -> RewardInfo {
    use soroban_sdk::vec;

    use crate::state::{RewardScheduleCurve, RewardType};

    RewardInfo {
        token: reward_token.clone(),
        rewards_vault: rewards_vault.clone(),
        rewards_available: 0,
        reward_schedule: RewardScheduleCurve { points: vec![e] },
        last_issuance_ts: 0,
        reward_per_share_scaled: 0,
        rewards_issued_unclaimed: 0,
        rewards_issued_cumulative: 0,
        min_claim_duration: 0,
        reward_type: RewardType::Proportional,
    }
}

/// Calculates the pending rewards for a user for a specific reward token.
///
/// Formula:
/// ```
/// entitled_scaled = user_stake × reward_per_share_scaled
/// pending_scaled = entitled_scaled - user_rewards_tally_scaled
/// pending = pending_scaled / SCALE_FACTOR
/// ```
///
/// # Arguments
/// * `user_stake` - User's active stake
/// * `reward_per_share_scaled` - Current scaled reward per share
/// * `user_rewards_tally_scaled` - User's rewards tally (scaled: stake × rps at last update)
///
/// # Returns
/// * Pending reward amount (unscaled tokens)
pub fn calculate_pending_reward(
    user_stake: i128,
    reward_per_share_scaled: i128,
    user_rewards_tally_scaled: i128,
) -> Result<i128, FarmsError> {
    if user_stake == 0 {
        return Ok(0);
    }

    // entitled_scaled = user_stake × reward_per_share_scaled
    // Both are in scaled space: stake × (rewards × SCALE / total_staked)
    let entitled_scaled =
        user_stake.checked_mul(reward_per_share_scaled).ok_or(FarmsError::Overflow)?;

    // pending_scaled = entitled_scaled - tally_scaled
    // Tally was set to stake × rps at user's last interaction, so same scale
    // Use saturating_sub to handle edge cases where tally might slightly exceed entitled
    let pending_scaled = entitled_scaled.saturating_sub(user_rewards_tally_scaled);

    // Unscale once to get actual token amount
    let pending = pending_scaled.checked_div(SCALE_FACTOR).ok_or(FarmsError::DivisionByZero)?;

    Ok(pending.max(0))
}

/// Updates user's rewards tally to match current reward_per_share
///
/// Called after claiming or when stake changes
pub fn update_user_rewards_tally(user_stake: i128, reward_per_share_scaled: i128) -> i128 {
    // tally = user_stake * reward_per_share_scaled (keeps scaled)
    user_stake.saturating_mul(reward_per_share_scaled)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test the core RPS (Reward Per Share) algorithm
    ///
    /// Scenario:
    /// - Total staked: 1000 tokens
    /// - User A stakes: 100 tokens (10% of pool)
    /// - User B stakes: 900 tokens (90% of pool)
    /// - Rewards issued: 500 tokens
    ///
    /// Expected:
    /// - rps_delta = (500 × SCALE) / 1000 = 0.5 × SCALE
    /// - User A pending = (100 × 0.5 × SCALE) / SCALE = 50 tokens
    /// - User B pending = (900 × 0.5 × SCALE) / SCALE = 450 tokens
    #[test]
    fn test_rps_algorithm_basic() {
        let total_staked: i128 = 1000;
        let rewards_issued: i128 = 500;

        // Calculate rps_delta using the same formula as refresh_global_rewards
        let rps_delta = fixed_div_floor(rewards_issued, SCALE_FACTOR, total_staked).unwrap();

        // User A: 10% stake
        let user_a_stake: i128 = 100;
        let user_a_tally: i128 = 0; // Fresh user, no previous tally

        let user_a_pending =
            calculate_pending_reward(user_a_stake, rps_delta, user_a_tally).unwrap();

        assert_eq!(user_a_pending, 50, "User A should receive 10% of 500 = 50 tokens");

        // User B: 90% stake
        let user_b_stake: i128 = 900;
        let user_b_tally: i128 = 0;

        let user_b_pending =
            calculate_pending_reward(user_b_stake, rps_delta, user_b_tally).unwrap();

        assert_eq!(user_b_pending, 450, "User B should receive 90% of 500 = 450 tokens");

        // Verify total matches
        assert_eq!(user_a_pending + user_b_pending, rewards_issued);
    }

    /// Test that tally correctly prevents double-claiming
    ///
    /// Scenario:
    /// - User stakes 100 tokens
    /// - First reward distribution: 100 tokens → user gets 100
    /// - User claims, tally updated
    /// - Second reward distribution: 50 tokens → user gets 50
    #[test]
    fn test_tally_prevents_double_claiming() {
        let user_stake: i128 = 100;
        let total_staked: i128 = 100; // User is 100% of pool

        // First distribution: 100 tokens
        let rps_after_first = fixed_div_floor(100, SCALE_FACTOR, total_staked).unwrap();

        let pending_after_first = calculate_pending_reward(user_stake, rps_after_first, 0).unwrap();
        assert_eq!(pending_after_first, 100);

        // User claims → tally updated to current rps × stake
        let tally_after_claim = update_user_rewards_tally(user_stake, rps_after_first);

        // Second distribution: 50 more tokens
        let rps_delta_second = fixed_div_floor(50, SCALE_FACTOR, total_staked).unwrap();
        let rps_after_second = rps_after_first + rps_delta_second;

        // User should only get 50, not 150
        let pending_after_second =
            calculate_pending_reward(user_stake, rps_after_second, tally_after_claim).unwrap();
        assert_eq!(
            pending_after_second, 50,
            "User should only get new rewards, not re-claim old ones"
        );
    }

    /// Test stake change mid-distribution
    ///
    /// Scenario:
    /// - User starts with 100 stake
    /// - 100 rewards distributed → user entitled to 100
    /// - User increases stake to 200
    /// - 100 more rewards distributed → user entitled to 100 (50% of 200 / 200 total)
    #[test]
    fn test_stake_increase_mid_distribution() {
        let initial_stake: i128 = 100;
        let total_staked: i128 = 100;

        // First distribution
        let rps_1 = fixed_div_floor(100, SCALE_FACTOR, total_staked).unwrap();
        let pending_1 = calculate_pending_reward(initial_stake, rps_1, 0).unwrap();
        assert_eq!(pending_1, 100);

        // User's pending rewards are "banked" in unclaimed, tally updated
        let _tally_after_bank = update_user_rewards_tally(initial_stake, rps_1);

        // User increases stake to 200 (total pool now 200)
        let new_stake: i128 = 200;
        let new_total: i128 = 200;

        // Tally must be recalculated for new stake at current rps
        let tally_after_restake = update_user_rewards_tally(new_stake, rps_1);

        // Second distribution: 100 tokens to pool of 200
        let rps_delta_2 = fixed_div_floor(100, SCALE_FACTOR, new_total).unwrap();
        let rps_2 = rps_1 + rps_delta_2;

        // User should get 100 (their 200 / 200 total × 100 rewards)
        let pending_2 = calculate_pending_reward(new_stake, rps_2, tally_after_restake).unwrap();
        assert_eq!(pending_2, 100, "User with 100% of pool should get 100% of new rewards");
    }

    /// Test zero stake returns zero pending
    #[test]
    fn test_zero_stake_returns_zero() {
        let pending = calculate_pending_reward(0, SCALE_FACTOR, 0).unwrap();
        assert_eq!(pending, 0);
    }

    /// Test precision with small amounts
    #[test]
    fn test_precision_small_amounts() {
        // Very small rewards relative to stake
        let total_staked: i128 = 1_000_000_000; // 1 billion
        let rewards: i128 = 1; // 1 token

        let rps = fixed_div_floor(rewards, SCALE_FACTOR, total_staked).unwrap();

        // User with 1 million stake (0.1% of pool)
        let user_stake: i128 = 1_000_000;
        let pending = calculate_pending_reward(user_stake, rps, 0).unwrap();

        // 0.1% of 1 token = 0.001 tokens, rounds down to 0
        assert_eq!(pending, 0, "Very small reward should round down to 0");

        // User with 100 million stake (10% of pool)
        let large_stake: i128 = 100_000_000;
        let pending_large = calculate_pending_reward(large_stake, rps, 0).unwrap();

        // 10% of 1 token = 0.1 tokens, still rounds to 0
        assert_eq!(pending_large, 0);

        // Only at 100% stake do we get the full token
        let pending_full = calculate_pending_reward(total_staked, rps, 0).unwrap();
        assert_eq!(pending_full, 1);
    }

    /// Test that pending never goes negative (saturating_sub)
    #[test]
    fn test_pending_never_negative() {
        // Edge case: tally slightly exceeds entitled due to rounding
        let user_stake: i128 = 100;
        let rps: i128 = SCALE_FACTOR; // 1.0 per share

        // Tally is slightly higher than it should be (simulating rounding edge case)
        let inflated_tally: i128 = user_stake * rps + 1;

        let pending = calculate_pending_reward(user_stake, rps, inflated_tally).unwrap();
        assert_eq!(pending, 0, "Pending should be 0, not negative");
    }
}
