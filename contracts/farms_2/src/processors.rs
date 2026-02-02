use farms_interface::FarmingKey;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::Env;

use crate::{
    constants::{BPS_FACTOR, SCALE_FACTOR},
    error::FCError,
    events,
    state::{Delegation, Farm, RewardInfo, User},
};

pub fn refresh_farm_rewards(e: &Env, farm: &mut Farm) -> Result<(), FCError> {
    let current_ts = e.ledger().timestamp();

    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, farm.id, reward_token)?;

        let rewards_to_issue = todo!();
    }

    Ok(())
}

pub fn stake(
    e: &Env,
    farming_key: &FarmingKey,
    farm: &mut Farm,
    user: &mut User,
    amount: i128,
) -> Result<(), FCError> {
    if farm.config.deposit_cap.is_positive() {
        let new_total = farm.total_staked.checked_add(amount).unwrap();
        if new_total > farm.config.deposit_cap {
            return Err(FCError::DepositCapExceeded);
        }
    }

    farm.refresh_rewards(e)?;
    user.refresh_rewards(e, farm)?;

    let current_ts = e.ledger().timestamp();
    let Delegation::NonDelegated(delegation_config) = farm.config.delegation else {
        return Err(FCError::DelegatedFarm);
    };

    if delegation_config.deposit_warmup_period > 0 {
        user.pending_deposit_stake = user.pending_deposit_stake.checked_add(amount).unwrap();
        user.pending_deposit_ts = current_ts;
    } else {
        activate_stake(e, farm, user, amount)?;
    }

    user.last_stake_ts = current_ts;

    Ok(())
}

pub fn unstake(
    e: &Env,
    farming_key: &FarmingKey,
    farm: &mut Farm,
    user: &mut User,
    amount: i128,
) -> Result<(), FCError> {
    // if user.active_stake < MIN_STAKE_AMOUNT {
    //     return Err(FCError::InsufficientStake);
    // }

    if user.active_stake < amount {
        return Err(FCError::InsufficientStake);
    }

    if user.pending_withdrawal_stake.is_positive() {
        return Err(FCError::PendingWithdrawalExists);
    }

    farm.refresh_rewards(e)?;
    user.refresh_rewards(e, farm)?;

    let current_ts = e.ledger().timestamp();

    let (net, penalty) = calculate_early_withdrawal_penalty(farm, user, amount, current_ts)?;

    if penalty.is_positive() {
        farm.current_slashed_amount = farm.current_slashed_amount.checked_add(penalty).unwrap();
        farm.cumulative_slashed_amount =
            farm.cumulative_slashed_amount.checked_add(penalty).unwrap();
    }

    user.active_stake = user.active_stake.checked_sub(amount).unwrap();
    if user.active_stake == 0 {
        farm.num_users = farm.num_users.checked_sub(1).unwrap();
    }

    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, farm.id, &reward_token)?;
        let new_tally = user
            .active_stake
            .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
            .unwrap();
        user.debts_per_rewards_sc.set(reward_token.clone(), new_tally);
    }

    farm.total_staked = farm.total_staked.checked_sub(amount).unwrap();

    if farm.config.withdrawal_cooldown_period > 0 {
        user.pending_withdrawal_stake = user.pending_withdrawal_stake.checked_add(net).unwrap();
        user.pending_withdrawal_ts = current_ts;
    }

    Ok(())
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

pub fn calculate_early_withdrawal_penalty(
    farm: &Farm,
    user: &User,
    amount: i128,
    current_ts: u64,
) -> Result<(i128, i128), FCError> {
    let Delegation::NonDelegated(delegation_config) = farm.config.delegation else {
        return Err(FCError::DelegatedFarm);
    };
    if delegation_config.early_withdrawal_penalty_bps == 0 || farm.config.locking_duration == 0 {
        return Ok((amount, 0));
    }

    use crate::state::LockingMode;
    let (_, lock_end) = match farm.config.locking_mode {
        LockingMode::None => return Ok((amount, 0)),
        LockingMode::Continuous => {
            let start = user.last_stake_ts;
            let end = start.checked_add(farm.config.locking_duration).unwrap();

            (start, end)
        }
        LockingMode::WithExpiry => {
            let start = farm.locking_start;
            let end = start.checked_add(farm.config.locking_duration).unwrap();

            (start, end)
        }
    };

    if current_ts >= lock_end {
        return Ok((amount, 0));
    }

    let time_remaining = lock_end.checked_sub(current_ts).unwrap();
    let total_duration = farm.config.locking_duration;

    let effective_penalty_bps = delegation_config
        .early_withdrawal_penalty_bps
        .fixed_mul_ceil(time_remaining as i128, total_duration as i128)
        .unwrap();

    let penalty = amount.fixed_mul_ceil(effective_penalty_bps, BPS_FACTOR)?;
    let net = amount.checked_sub(penalty).unwrap();

    Ok((net, penalty))
}

pub fn activate_pending_stake(
    e: &Env,
    farming_key: &FarmingKey,
    farm: &mut Farm,
    user: &mut User,
) -> Result<(), FCError> {
    if user.pending_deposit_stake == 0 {
        return Ok(());
    }
    let Delegation::NonDelegated(delegation_config) = farm.config.delegation else {
        return Err(FCError::DelegatedFarm);
    };

    let current_ts = e.ledger().timestamp();
    let warmup_end =
        user.pending_deposit_ts.checked_add(delegation_config.deposit_warmup_period).unwrap();

    if current_ts < warmup_end {
        return Err(FCError::WarmupNotComplete);
    }

    farm.refresh_rewards(e)?;

    let amount = user.pending_deposit_stake;
    user.pending_deposit_stake = 0;
    user.pending_deposit_ts = 0;

    activate_stake(e, farm, user, amount)?;

    Ok(())
}

fn activate_stake(e: &Env, farm: &mut Farm, user: &mut User, amount: i128) -> Result<(), FCError> {
    user.active_stake = user.active_stake.checked_add(amount).unwrap();
    farm.total_staked = farm.total_staked.checked_add(amount).unwrap();

    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, farm.id, &reward_token)?;

        let new_tally =
            user.active_stake.checked_mul(reward_info.accum_rewards_per_share_sc).unwrap();
        user.debts_per_rewards_sc.set(reward_token.clone(), new_tally);
    }

    if user.active_stake == amount {
        farm.num_users = farm.num_users.checked_sub(1).unwrap();
    }

    Ok(())
}
