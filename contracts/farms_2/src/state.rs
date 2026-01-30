use farms_interface::FarmingKey;
use soroban_sdk::{Address, Env, Map, Vec, contracttype, vec as svec};

use crate::{constants::*, error::FCError, storage};

#[contracttype]
pub struct Farm {
    pub id: u64,
    pub num_users: u64,
    pub is_frozen: bool,
    pub rewards: Map<Address, ()>, // why not let it be a Map here?
    pub total_staked: i128,
    pub config: FarmConfig,
}

impl Farm {
    pub fn new(e: &Env, config: FarmConfig) -> Self {
        let id = storage::get_farms_counter(e).unwrap_or(0);

        Self { id, num_users: 0, rewards: Map::new(e), is_frozen: true, total_staked: 0, config }
    }

    pub fn try_get(e: &Env, farm_id: u64) -> Result<Farm, FCError> {
        storage::get_farm(e, farm_id).ok_or(FCError::FarmDoesNotExist)
    }

    pub fn set(self, e: &Env) {
        storage::set_farm(e, &self);
    }

    pub fn update_farm_config(&mut self, config_update: &FarmConfigUpdate) -> Result<(), FCError> {
        match config_update {
            FarmConfigUpdate::DepositCap(cap) => {
                self.config.deposit_cap = *cap;
            }
            FarmConfigUpdate::MaxNumRewards(max_num_rewards) => {
                todo!()
            }
            FarmConfigUpdate::DepositWarmupPeriod(period) => {
                if !(0..=MAX_DEPOSIT_WARMUP_PERIOD).contains(period) {
                    return Err(FCError::InvalidFarmConfigUpdate);
                }

                self.config.deposit_warmup_period = *period;
            }
            FarmConfigUpdate::WithdrawalCooldownPeriod(_) => todo!(),
            FarmConfigUpdate::LockingMode(locking_mode) => todo!(),
        }

        Ok(())
    }

    pub fn require_delegated_authority_auth(&self) -> Result<(), FCError> {
        let Some(authority) = &self.config.delegated_authority else {
            return Err(FCError::NotDelegatedFarm);
        };
        authority.require_auth();

        Ok(())
    }

    pub fn require_not_delegated(&self) -> Result<(), FCError> {
        if self.config.delegated_authority.is_some() {
            return Err(FCError::DelegatedFarm);
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
    /// Max allowed number of farm rewards
    pub max_num_rewards: u32,
    /// Delay before the new stake becomes active in seconds
    pub deposit_warmup_period: u64,
}

#[contracttype]
pub enum FarmConfigUpdate {
    DepositCap(i128),
    MaxNumRewards(u32),
    DepositWarmupPeriod(u64),
    WithdrawalCooldownPeriod(u64),
    LockingMode(LockingMode),
}

#[contracttype]
#[derive(Default)]
pub enum LockingMode {
    #[default]
    None,
    Continuous,
    WithExpiry,
}

#[contracttype]
pub struct RewardInfo {
    /// Remaining rewards available for distribution
    pub available: i128,
    pub rewards_per_share: i128, // TODO: Scaled?
    /// Minimum duration between claims
    pub min_claim_duration: u64,
    pub reward_schedule: RewardScheduleCurve,
    pub last_issuance_ts: u64,
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
    pub fn require_valid(&self, e: &Env) -> Result<(), FCError> {
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

    pub fn calculate_rewards(&self, from: u64, to: u64) -> Result<i128, FCError> {
        let mut total_rewards = 0_i128;

        for (p_a, p_b) in self.points.iter().zip(self.points.iter().skip(1)) {
            let segment_end = p_b.ts_start;

            let (overlap_start, overlap_end) = (from.max(p_a.ts_start), to.min(segment_end));
            if overlap_start < overlap_end {
                let duration = overlap_end - overlap_start; // safe
                let duration_rewards =
                    duration.checked_mul(p_a.reward_per_segment_bps as u64).unwrap();
                total_rewards = total_rewards.checked_add(duration_rewards as i128).unwrap();
            }
        }

        Ok(total_rewards)
    }
}

#[contracttype]
pub struct User {
    pub active_stake: i128,

    pub pending_deposit_stake: i128,
    pub pending_deposit_ts: u64,

    pub pending_withdrawal_stake: i128,
    pub pending_withdrawal_ts: u64,

    // This all goes per reward token, ok

    // See MasterChef algorithm
    pub reward_debts: Vec<i128>,
    pub rewards_unclaimed: Vec<i128>,
    pub last_claim_ts: Vec<u64>,

    pub last_stake_ts: u64,
}

#[contractimpl]
impl User {
    pub fn new(e: &Env) -> Self {
        Self {
            active_stake: 0,
            pending_deposit_stake: 0,
            pending_deposit_ts: 0,
            pending_withdrawal_stake: 0,
            pending_withdrawal_ts: 0,
            rewards_tally_scaled: svec![e],
            rewards_unclaimed: svec![e],
            last_claim_ts: 0,
            last_stake_ts: 0,
        }
    }

    pub fn try_get(e: &Env, delegatee: &Delegatee) -> Result<Self, FCError> {
        storage::get_user(e, delegatee).ok_or(FCError::UserDoesNotExist)
    }

    pub fn refresh_rewards(&mut self) -> Result<(), FCError> {
        // for i in 0..farm.reward_infos.len() {
        //     if let Some(reward_info) = farm.reward_infos.get(i) {
        //         let user_tally = user_state.rewards_tally_scaled.get(i).unwrap_or(0);

        //         let pending = calculate_pending_reward(
        //             user_state.active_stake,
        //             reward_info.reward_per_share_scaled,
        //             user_tally,
        //         )?;

        //         if pending > 0 {
        //             let current_unclaimed = user_state.rewards_unclaimed.get(i).unwrap_or(0);
        //             let new_unclaimed =
        //                 current_unclaimed.checked_add(pending).ok_or(FarmsError::Overflow)?;

        //             user_state.rewards_unclaimed.set(i, new_unclaimed);

        //             // Update tally
        //             let new_tally = update_user_rewards_tally(
        //                 user_state.active_stake,
        //                 reward_info.reward_per_share_scaled,
        //             );
        //             user_state.rewards_tally_scaled.set(i, new_tally);
        //         }
        //     }
        // }

        todo!()
    }

    pub fn set(self, e: &Env) {}
}

#[contracttype]
pub struct GlobalConfig {
    pub admin: Address,
    pub pending_admin: Option<Address>,
    pub fee_bps: i128,
}
