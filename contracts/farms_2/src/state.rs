use core::ops::AddAssign;

use soroban_sdk::{Address, Env, Map, Vec, contracttype};

use crate::{constants::*, error::FCError, storage};

#[contracttype]
pub struct Farm {
    pub id: u64,
    pub num_users: u64,
    pub is_frozen: bool,
    pub rewards: Map<Address, ()>,
    pub total_staked: i128,
    pub config: FarmConfig,
}

impl Farm {
    pub fn new_and_increment_farms_counter(e: &Env, config: FarmConfig) -> Self {
        let id = storage::get_farms_counter(e).unwrap_or(0);
        storage::increment_farms_counter(e);

        Self { id, num_users: 0, rewards: Map::new(e), is_frozen: true, total_staked: 0, config }
    }

    pub fn try_get(e: &Env, farm_id: u64) -> Result<Farm, FCError> {
        storage::get_farm(e, farm_id).ok_or(FCError::FarmDoesNotExist)
    }

    pub fn set(self, e: &Env) {
        storage::set_farm(e, &self);
    }

    pub fn require_can_stake(&self) -> Result<(), FCError> {
        let Some(authority) = &self.config.delegated_authority else {
            return Err(FCError::DelegatedAuthorityIsNotSetForFarm);
        };
        authority.require_auth();

        if self.is_frozen {
            return Err(FCError::FarmIsFrozen);
        }

        Ok(())
    }

    pub fn require_farm_admin(&self) {
        self.config.admin.require_auth();
    }

    fn require_can_add_token_to_rewards(
        &self,
        e: &Env,
        reward_token: &Address,
        reward_info: &RewardInfo,
    ) -> Result<(), FCError> {
        if self.rewards.len() >= self.config.max_num_rewards {
            return Err(FCError::MaxNumRewardsReached);
        }
        if self.rewards.contains_key(reward_token.clone()) {
            return Err(FCError::TokenIsAlreadyAReward);
        }

        reward_info.require_valid(e)?;

        Ok(())
    }

    pub fn try_add_reward(
        &mut self,
        e: &Env,
        reward_token: &Address,
        reward_info: &RewardInfo,
    ) -> Result<(), FCError> {
        self.require_can_add_token_to_rewards(e, reward_token, reward_info)?;
        self.rewards.set(reward_token.clone(), ());

        storage::set_reward_info(e, self.id, reward_token, reward_info);

        Ok(())
    }
}

#[contracttype]
pub struct FarmConfig {
    pub admin: Address,
    /// Delegate authority address (optional)
    /// When Some: only this address can update stakes via set_stake_delegated (push model)
    /// When None: users can call stake()/unstake() directly
    pub delegated_authority: Option<Address>,
    /// Max deposited reward amount
    pub deposit_cap: i128,
    /// Max allowed number of active farm rewards
    pub max_num_rewards: u32,
}

#[contracttype]
pub struct RewardInfo {
    /// Remaining rewards available for distribution
    pub available: i128,
    pub rewards_per_share: i128, // TODO: Scaled?
    /// Minimum duration between claims
    pub min_claim_duration: u64,
    pub reward_schedule: RewardScheduleCurve,
}

impl RewardInfo {
    pub fn try_get(e: &Env, farm_id: u64, reward_token: Address) -> Result<Self, FCError> {
        storage::get_reward_info(e, farm_id, &reward_token).ok_or(FCError::RewardDoesNotExistOnFarm)
    }

    pub fn set(self, e: &Env, farm_id: u64, reward_token: &Address) {
        storage::set_reward_info(e, farm_id, &reward_token, &self);
    }

    pub fn require_valid(&self, e: &Env) -> Result<(), FCError> {
        let Self { available: _, rewards_per_share: _, min_claim_duration: _, reward_schedule } =
            &self;

        reward_schedule.require_valid(e)?;

        Ok(())
    }
}

#[contracttype]
#[derive(Clone)]
pub struct RewardCurvePoint {
    /// Timestamp when the current rate starts applying
    pub ts_start: u64,
    /// Basis points of a reward allocated over a segment of time, beginning from this point.
    /// Ignored for the last point in the curve
    pub reward_per_segment_bps: u32,
}

#[contracttype]
pub struct RewardScheduleCurve {
    /// Points defining the curve (up to `[MAX_CURVE_POINTS]` and in ascending order)
    pub points: Vec<RewardCurvePoint>,
}

impl RewardScheduleCurve {
    fn require_valid(&self, e: &Env) -> Result<(), FCError> {
        let mut reward_per_segments_sum_bps = 0_u32;

        if self.points.is_empty() {
            return Err(FCError::InvalidRewardScheduleCurve);
        }
        // TODO: Introduce some necessary unfreeze period before activating rewards?
        if self.points.first().unwrap().ts_start <= e.ledger().timestamp() {
            return Err(FCError::InvalidRewardScheduleCurve);
        }

        for (p_current, p_next) in self.points.iter().zip(self.points.iter().skip(1)) {
            if p_current.ts_start >= p_next.ts_start {
                return Err(FCError::InvalidRewardScheduleCurve);
            }

            reward_per_segments_sum_bps =
                reward_per_segments_sum_bps.checked_add(p_current.reward_per_segment_bps).unwrap(); // TODO
        }
        if reward_per_segments_sum_bps != BPS_FACTOR {
            return Err(FCError::InvalidRewardScheduleCurve);
        }

        Ok(())
    }
}
