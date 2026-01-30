use soroban_sdk::Env;

use crate::{
    error::FCError,
    events,
    state::{Farm, RewardInfo, User},
};

pub fn refresh_farm_rewards(e: &Env, farm: &mut Farm) -> Result<(), FCError> {
    let current_ts = e.ledger().timestamp();

    for reward_token in farm.rewards.keys() {
        let reward_info = RewardInfo::try_get(e, farm.id, reward_token)?;

        let rewards_to_issue = todo!();
    }

    Ok(())
}

pub fn stake(e: &Env, user: &mut User, amount: i128, stake: i128) -> Result<(), FCError> {
    Ok(())
}

/// Calculates rewards to issue for a single staked token
fn calculate_rewards_to_issue(
    farm: &Farm,
    reward_info: &RewardInfo,
    current_ts: u64,
) -> Result<i128, FCError> {
    if farm.total_staked == 0 {
        return Ok(0);
    }

    let from_ts = reward_info.last_issuance_ts;
    // assert!(from_ts > current_ts)?

    let rewards_from_curve = reward_info.reward_schedule.calculate_rewards(from_ts, current_ts)?;
    let rewards_to_issue = rewards_from_curve.min(reward_info.available);

    Ok(rewards_to_issue)
}
