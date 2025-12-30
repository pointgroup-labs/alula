use crate::{
    constants::BPS_FACTOR,
    error::FarmsError,
    math::utils::fixed_mul_floor,
    state::{FarmState, LockingMode, UserState},
};

/// Calculates the early withdrawal penalty for unstaking from a locked farm
///
/// Uses linear penalty decay: the penalty decreases linearly as the lock period
/// progresses. At lock start, the full penalty applies. At lock end, no penalty applies.
///
/// Formula: `effective_penalty_bps = penalty_bps * time_remaining / total_duration`
///
/// # Arguments
/// * `farm` - The farm state
/// * `user` - The user state
/// * `current_ts` - Current timestamp
/// * `amount` - Amount being unstaked
///
/// # Returns
/// * `Ok((net_amount, penalty_amount))` - The net amount user receives and penalty deducted
pub fn calculate_early_withdrawal_penalty(
    farm: &FarmState,
    user: &UserState,
    current_ts: u64,
    amount: i128,
) -> Result<(i128, i128), FarmsError> {
    if farm.early_withdrawal_penalty_bps == 0 || farm.locking_duration == 0 {
        return Ok((amount, 0));
    }

    // Calculate lock start and end times based on locking mode
    let (_lock_start, lock_end) = match farm.locking_mode {
        LockingMode::None => return Ok((amount, 0)),
        LockingMode::Continuous => {
            // Lock expires `locking_duration` after user's last stake
            let start = user.last_stake_ts;
            let end = start.checked_add(farm.locking_duration).ok_or(FarmsError::Overflow)?;
            (start, end)
        }
        LockingMode::WithExpiry => {
            // Global lock expires at locking_start_ts + locking_duration
            let start = farm.locking_start_ts;
            let end = start.checked_add(farm.locking_duration).ok_or(FarmsError::Overflow)?;
            (start, end)
        }
    };

    // Check if lock has expired
    if current_ts >= lock_end {
        return Ok((amount, 0));
    }

    // Calculate time remaining in lock period
    let time_remaining = lock_end.saturating_sub(current_ts);
    let total_duration = farm.locking_duration;

    // Linear decay: penalty = base_penalty * time_remaining / total_duration
    // This ensures penalty decreases linearly as lock progresses
    // Using fixed-point multiplication: (penalty_bps * time_remaining) / total_duration
    let effective_penalty_bps = fixed_mul_floor(
        farm.early_withdrawal_penalty_bps,
        time_remaining as i128,
        total_duration as i128,
    )?;

    // Calculate penalty amount: (amount * effective_penalty_bps) / BPS_FACTOR
    let penalty = fixed_mul_floor(amount, effective_penalty_bps, BPS_FACTOR)?;

    let net_amount = amount.checked_sub(penalty).ok_or(FarmsError::Underflow)?;

    Ok((net_amount, penalty))
}

/// Checks if the user's lock has expired
///
/// # Arguments
/// * `farm` - The farm state
/// * `user` - The user state
/// * `current_ts` - Current timestamp
///
/// # Returns
/// * `true` if lock has expired or no lock exists
#[allow(dead_code)]
pub fn is_lock_expired(farm: &FarmState, user: &UserState, current_ts: u64) -> bool {
    match farm.locking_mode {
        LockingMode::None => true,
        LockingMode::Continuous => {
            if let Some(lock_end) = user.last_stake_ts.checked_add(farm.locking_duration) {
                current_ts >= lock_end
            } else {
                false // Overflow means very far future, so not expired
            }
        }
        LockingMode::WithExpiry => {
            if let Some(lock_end) = farm.locking_start_ts.checked_add(farm.locking_duration) {
                current_ts >= lock_end
            } else {
                false
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{BytesN, Env, testutils::Address, vec};

    use super::*;

    fn create_test_user(env: &Env, last_stake_ts: u64) -> UserState {
        UserState {
            owner: soroban_sdk::Address::generate(env),
            farm_id: BytesN::from_array(env, &[0u8; 32]),
            active_stake: 1000,
            pending_deposit_stake: 0,
            pending_deposit_ts: 0,
            pending_withdrawal_stake: 0,
            pending_withdrawal_ts: 0,
            rewards_tally_scaled: vec![env],
            rewards_unclaimed: vec![env],
            last_claim_ts: vec![env],
            last_stake_ts,
        }
    }

    fn create_test_farm(
        env: &Env,
        locking_mode: LockingMode,
        locking_duration: u64,
        penalty_bps: i128,
    ) -> FarmState {
        use crate::state::TimeUnit;
        FarmState {
            farm_id: BytesN::from_array(env, &[0u8; 32]),
            farm_admin: None,
            pending_farm_admin: None,
            delegate_authority: None,
            total_staked: 1000,
            num_users: 1,
            time_unit: TimeUnit::Seconds,
            deposit_warmup_period: 0,
            withdrawal_cooldown_period: 0,
            locking_mode,
            locking_start_ts: 0,
            locking_duration,
            early_withdrawal_penalty_bps: penalty_bps,
            deposit_cap: 0,
            reward_infos: vec![env],
            num_reward_tokens: 0,
            is_frozen: false,
            is_reward_user_once_enabled: false,
            slashed_amount_current: 0,
            slashed_amount_cumulative: 0,
            slashed_amount_spill_address: soroban_sdk::Address::generate(env),
        }
    }

    #[test]
    fn test_no_penalty_when_not_locked() {
        let env = Env::default();
        let farm = create_test_farm(&env, LockingMode::None, 0, 1000); // 10% penalty
        let user = create_test_user(&env, 0);

        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 100, 1000).unwrap();
        assert_eq!(net, 1000);
        assert_eq!(penalty, 0);
    }

    #[test]
    fn test_linear_penalty_decay_continuous_lock() {
        let env = Env::default();
        // 10% max penalty, 1000s lock duration
        let farm = create_test_farm(&env, LockingMode::Continuous, 1000, 1000);
        let user = create_test_user(&env, 0); // Staked at t=0, lock ends at t=1000

        // At t=0 (start of lock), full 10% penalty
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 0, 1000).unwrap();
        assert_eq!(penalty, 100); // 10% of 1000
        assert_eq!(net, 900);

        // At t=500 (halfway), 5% penalty (linear decay)
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 500, 1000).unwrap();
        assert_eq!(penalty, 50); // 5% of 1000 (half of 10%)
        assert_eq!(net, 950);

        // At t=750 (75% through), 2.5% penalty
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 750, 1000).unwrap();
        assert_eq!(penalty, 25); // 2.5% of 1000
        assert_eq!(net, 975);

        // At t=1000 (lock expired), no penalty
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 1000, 1000).unwrap();
        assert_eq!(penalty, 0);
        assert_eq!(net, 1000);

        // After lock expires
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 1200, 1000).unwrap();
        assert_eq!(penalty, 0);
        assert_eq!(net, 1000);
    }

    #[test]
    fn test_linear_penalty_decay_with_expiry() {
        let env = Env::default();
        // 20% max penalty, global lock from t=100 to t=1100 (1000s duration)
        let mut farm = create_test_farm(&env, LockingMode::WithExpiry, 1000, 2000);
        farm.locking_start_ts = 100;
        let user = create_test_user(&env, 50); // User stake time doesn't matter for WithExpiry

        // At t=100 (start of global lock), full 20% penalty
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 100, 1000).unwrap();
        assert_eq!(penalty, 200); // 20% of 1000
        assert_eq!(net, 800);

        // At t=600 (halfway through global lock), 10% penalty
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 600, 1000).unwrap();
        assert_eq!(penalty, 100); // 10% of 1000
        assert_eq!(net, 900);

        // At t=1100 (global lock expired), no penalty
        let (net, penalty) = calculate_early_withdrawal_penalty(&farm, &user, 1100, 1000).unwrap();
        assert_eq!(penalty, 0);
        assert_eq!(net, 1000);
    }

    #[test]
    fn test_is_lock_expired() {
        let env = Env::default();
        let farm = create_test_farm(&env, LockingMode::Continuous, 1000, 0);
        let user = create_test_user(&env, 100);

        assert!(!is_lock_expired(&farm, &user, 500)); // Before expiry
        assert!(is_lock_expired(&farm, &user, 1100)); // At expiry
        assert!(is_lock_expired(&farm, &user, 2000)); // After expiry
    }
}
