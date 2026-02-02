use farms_interface::FarmingKey;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, Map, Vec, contracttype, map as smap, vec as svec};

use crate::{constants::*, error::FCError, processors, storage, utils::require_nonnegative};

#[contracttype]
pub struct Farm {
    pub id: u64,
    pub num_users: u64,
    pub is_frozen: bool,
    pub total_staked: i128,
    pub config: FarmConfig,
    pub locking_start: u64,
    pub rewards: Map<Address, ()>,
    pub current_slashed_amount: i128,
    pub cumulative_slashed_amount: i128,
}

impl Farm {
    pub fn new(e: &Env, config: FarmConfig) -> Self {
        let id = storage::get_farms_counter(e).unwrap_or(0);

        Self { id, num_users: 0, rewards: Map::new(e), is_frozen: true, total_staked: 0, config }
    }

    pub fn try_get(e: &Env, farm_id: u64) -> Result<Farm, FCError> {
        storage::get_farm(e, farm_id).ok_or(FCError::FarmDoesNotExist)
    }

    pub fn require_can_reward_once(&self) -> Result<(), FCError> {
        self.require_delegated_authority_auth()?;

        if !self.config.is_reward_once_enabled {
            return Err(FCError::RewardUserOnceDisabled);
        }

        Ok(())
    }

    /// Refreshes rewards:
    /// TODO...
    pub fn refresh_rewards(&mut self, e: &Env) -> Result<(), FCError> {
        let current_ts = e.ledger().timestamp();

        for reward_token in self.rewards.keys() {
            let mut reward_info = RewardInfo::try_get(e, self.id, &reward_token)?; // TODO: internal error

            let rewards_to_issue =
                processors::calculate_rewards_to_issue(self, &reward_info, current_ts)?;

            if rewards_to_issue.is_positive() {
                if self.total_staked.is_positive() {
                    let rps_delta =
                        rewards_to_issue.fixed_div_floor(self.total_staked, SCALE_FACTOR).unwrap();
                    reward_info.accum_rewards_per_share_sc =
                        reward_info.accum_rewards_per_share_sc.checked_add(rps_delta).unwrap();
                }

                reward_info.rewards_available =
                    reward_info.rewards_available.checked_sub(rewards_to_issue).unwrap();
                reward_info.rewards_issued_unclaimed =
                    reward_info.rewards_issued_unclaimed.checked_add(rewards_to_issue).unwrap();
                reward_info.rewards_issued_cumulative =
                    reward_info.rewards_issued_cumulative.checked_add(rewards_to_issue).unwrap();
            }

            reward_info.last_issuance_ts = current_ts;
            reward_info.set(e, self.id, &reward_token);
        }

        Ok(())
    }

    // pub fn require_reward_token_exists(&self, reward_token: &Address) -> Result<(), FCError> {
    //     if !self.rewards.contains_key(reward_token) {
    //         return Err(FCError::RewardDoesNotExistOnFarm);
    //     }

    //     Ok(())
    // }


    pub fn set(self, e: &Env) {
        storage::set_farm(e, &self);
    }

    pub fn update_common_config(
        &mut self,
        config_update: &CommonFarmConfigUpdate,
    ) -> Result<(), FCError> {
        match config_update {
            CommonFarmConfigUpdate::DepositCap(cap) => {
                require_nonnegative(*cap)?;
                self.config.deposit_cap = *cap;
            }
            CommonFarmConfigUpdate::MinHarvestDelay(delay) => {
                if !(0..=MAX_HARVEST_DELAY).contains(delay) {
                    return Err(FCError::InvalidFarmConfigUpdate);
                }
                self.config.min_harvest_delay = *delay;
            }
        }

        Ok(())
    }

    pub fn update_delegated_config(
        &mut self,
        config_update: &DelegatedFarmConfigUpdate,
    ) -> Result<(), FCError> {
        let Delegation::Delegated(config) = &mut self.config.delegation_type else {
            return Err(FCError::InvalidFarmConfigUpdate);
        };

        match config_update {
            DelegatedFarmConfigUpdate::DelegateAuthority(authority) => {
                config.delegate_authority = authority.clone();
            }
        }

        Ok(())
    }

    pub fn update_non_delegated_config(
        &mut self,
        config_update: &NonDelegatedFarmConfigUpdate,
    ) -> Result<(), FCError> {
        let Delegation::NonDelegated(config) = &mut self.config.delegation_type else {
            return Err(FCError::InvalidFarmConfigUpdate);
        };

        match config_update {
            NonDelegatedFarmConfigUpdate::LockingTs(ts) => {
                config.locking_ts = *ts;
            }
            NonDelegatedFarmConfigUpdate::LockingDuration(duration) => {
                if *duration > MAX_LOCKING_DURATION {
                    return Err(FCError::InvalidFarmConfigUpdate);
                }
                config.locking_duration = *duration;
            }
            NonDelegatedFarmConfigUpdate::LockingMode(mode) => {
                // TODO: Check if rewards started already
                config.locking_mode = *mode;
            }
            NonDelegatedFarmConfigUpdate::DepositWarmupPeriod(period) => {
                if !(0..=MAX_WITHDRAWAL_COOLDOWN_PERIOD).contains(period) {
                    return Err(FCError::InvalidFarmConfigUpdate);
                }
                config.deposit_warmup_period = *period;
            }
            NonDelegatedFarmConfigUpdate::EarlyWithdrawalPenaltyBps(penalty_bps) => {
                if !(0..=BPS_FACTOR).contains(penalty_bps) {
                    return Err(FCError::InvalidFarmConfigUpdate);
                }
                config.early_withdrawal_penalty_bps = *penalty_bps;
            }
            NonDelegatedFarmConfigUpdate::WithdrawalCooldownPeriod(period) => {
                if !(0..=MAX_WITHDRAWAL_COOLDOWN_PERIOD).contains(period) {
                    return Err(FCError::InvalidFarmConfigUpdate);
                }
                config.withdrawal_cooldown_period = *period;
            }
        }

        Ok(())
    }

    pub fn require_delegated_authority_auth(&self) -> Result<(), FCError> {
        let Some(authority) = &self.config.delegate_authority else {
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

    pub fn try_withdraw_unused(
        &mut self,
        e: &Env,
        reward_token: &Address,
        amount: i128,
    ) -> Result<i128, FCError> {
        todo!()
    }

    pub fn try_withdraw_slashed(
        &mut self,
        e: &Env,
        amount: i128,
    ) -> Result<(Address, i128), FCError> {
        todo!()
    }

    pub fn require_admin(&self) {
        self.config.admin.require_auth();
    }

    pub fn accept_admin(&mut self) -> Result<(), FCError> {
        let Some(proposed_farm_admin) = self.config.proposed_admin.clone() else {
            return Err(FCError::ProposedFarmAdminDoesNotExist);
        };
        proposed_farm_admin.require_auth();

        self.config.admin = proposed_farm_admin;
        self.config.proposed_admin = None;

        Ok(())
    }

    pub fn propose_admin(&mut self, admin: &Address) {
        self.config.proposed_admin = Some(admin.clone());
    }

    fn require_can_add_token_to_rewards(&self, reward_token: &Address) -> Result<(), FCError> {
        if self.rewards.len() >= MAX_FARM_NUM_REWARDS {
            return Err(FCError::MaxFarmNumRewardsReached);
        }
        if self.rewards.contains_key(reward_token.clone()) {
            return Err(FCError::TokenIsAlreadyAReward);
        }

        Ok(())
    }

    pub fn try_initialize_reward(
        &mut self,
        e: &Env,
        reward_token: &Address,
    ) -> Result<(), FCError> {
        self.require_can_add_token_to_rewards(reward_token)?;
        self.rewards.set(reward_token.clone(), ());

        let reward_info = RewardInfo::new(&e);
        reward_info.set(e, self.id, reward_token);

        Ok(())
    }

    pub fn try_add_reward(
        &mut self,
        e: &Env,
        reward_token: &Address,
        amount: i128,
    ) -> Result<(), FCError> {
        if !self.rewards.contains_key(reward_token.clone()) {
            return Err(FCError::RewardDoesNotExistOnFarm);
        }

        let mut reward_info = RewardInfo::try_get(e, self.id, reward_token)?;
        reward_info.require_is_set();

        reward_info.rewards_available = reward_info.rewards_available.checked_add(amount).unwrap();
        reward_info.set(e, self.id, reward_token);

        Ok(())
    }
}

#[contracttype]
pub struct FarmConfig {
    pub admin: Address,
    pub deposit_cap: i128,
    pub locking_duration: u64,
    pub treasury_fee_bps: i128,
    pub min_harvest_delay: u64,
    pub delegation: Delegation,
    pub locking_mode: LockingMode,
    pub is_reward_once_enabled: bool,
    pub proposed_admin: Option<Address>,
    pub withdrawal_cooldown_period: u64,
}

#[contracttype]
pub enum Delegation {
    Delegated(DelegatedFarmConfig),
    NonDelegated(NonDelegatedFarmConfig),
}

#[contracttype]
pub enum DelegatedFarmConfigUpdate {
    DelegateAuthority(Address),
}

#[contracttype]
pub enum NonDelegatedFarmConfigUpdate {
    LockingTs(u64),
    LockingDuration(u64),
    LockingMode(LockingMode),
    DepositWarmupPeriod(u64),
    WithdrawalCooldownPeriod(u64),
    EarlyWithdrawalPenaltyBps(i128),
}

#[contracttype]
pub enum CommonFarmConfigUpdate {
    DepositCap(i128),
    MinHarvestDelay(u64),
}

#[contracttype]
pub struct DelegatedFarmConfig {
    pub delegate_authority: Address,
}

#[contracttype]
pub struct NonDelegatedFarmConfig {
    pub token: Address,
    pub locking_ts: u64,
    pub locking_duration: u64,
    pub locking_mode: LockingMode,
    pub deposit_warmup_period: u64,
    pub withdrawal_cooldown_period: u64,
    pub early_withdrawal_penalty_bps: i128,
}

#[contracttype]
#[derive(Default, Copy, Clone)]
pub enum LockingMode {
    #[default]
    None,
    Continuous,
    WithExpiry,
}

#[contracttype]
#[derive(Default, Copy, Clone)]
pub enum RewardType {
    #[default]
    Proportional,
    Constant,
}

#[contracttype]
pub struct RewardInfo {
    pub last_issuance_ts: u64,
    pub rewards_available: i128,
    pub reward_type: RewardType,
    pub rewards_issued_unclaimed: i128,
    pub rewards_issued_cumulative: i128,
    pub cumulative_issued_rewards: i128,
    pub accum_rewards_per_share_sc: i128,
    pub reward_schedule_curve: RewardScheduleCurve,
}

impl RewardInfo {
    pub fn new(e: &Env) -> Self {
        Self {
            last_issuance_ts: 0,
            rewards_available: 0,
            rewards_issued_unclaimed: 0,
            rewards_issued_cumulative: 0,
            cumulative_issued_rewards: 0,
            accum_rewards_per_share_sc: 0,
            reward_type: RewardType::Proportional,
            reward_schedule_curve: RewardScheduleCurve { points: svec![e] },
        }
    }

    pub fn reward_once(&mut self, amount: i128) -> Result<(), FCError> {
        self.rewards_issued_unclaimed = self.rewards_issued_unclaimed.checked_add(amount).unwrap();
        self.rewards_issued_cumulative =
            self.rewards_issued_cumulative.checked_add(amount).unwrap();
        self.rewards_available = self.rewards_available.checked_sub(amount).unwrap();

        Ok(())
    }

    pub fn try_get(e: &Env, farm_id: u64, reward_token: &Address) -> Result<Self, FCError> {
        storage::get_reward_info(e, farm_id, &reward_token).ok_or(FCError::RewardDoesNotExistOnFarm)
    }

    pub fn set(self, e: &Env, farm_id: u64, reward_token: &Address) {
        storage::set_reward_info(e, farm_id, &reward_token, &self);
    }

    pub fn try_set_reward_schedule_curve(
        &mut self,
        e: &Env,
        farm_id: u64,
        curve: &RewardScheduleCurve,
    ) -> Result<(), FCError> {
        curve.require_valid(e)?;

        self.refresh(e, farm_id)?;

        Ok(())
    }

    fn refresh(&mut self, e: &Env, farm_id: u64) -> Result<(), FCError> {
        let rewards_to_issue = self.calculate_rewards_to_issue(&e, farm_id)?;
    }

    fn calculate_rewards_to_issue(&self, e: &Env, farm_id: u64) -> Result<i128, FCError> {
        let current_ts = e.ledger().timestamp();
        let farm = Farm::try_get(e, farm_id).unwrap(); // TODO: Event

        if farm.total_staked == 0 {
            return Ok(0);
        }

        let from_ts = self.last_issuance_ts;
        if from_ts >= current_ts {
            return Ok(0);
        }
        let rewards_from_curve =
            self.reward_schedule_curve.calculate_rewards(from_ts, current_ts)?;

        Ok(rewards_from_curve.min(self.rewards_available))
    }

    // pub fn require_valid(&self, e: &Env) -> Result<(), FCError> {
    //     // let Self {
    //     //     available: _, rewards_per_share: _, min_claim_duration: _, reward_schedule, ..
    //     // } = &self;

    //     reward_schedule.require_valid(e)?;

    //     Ok(())
    // }

    pub fn increase_accum_reward_per_share_sc(&mut self, stake_diff: i128) -> Result<(), FCError> {
        // Oh, yeah. That's why they have this 'refresh functionality'

        // Do something with the curve, right?

        todo!()
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
#[derive(Clone)]
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
        if from >= to {
            return Ok(0);
        }

        if self.points.is_empty() {
            return Ok(0);
        }

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
    pub debts_per_rewards_sc: Map<Address, i128>,
    pub rewards_unclaimed: Map<Address, i128>,

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
            rewards_unclaimed: smap![e],
            last_claim_ts: 0,
            last_stake_ts: 0,
            debts_per_rewards_sc: smap![e],
        }
    }

    pub fn withdraw_unstaked(&mut self, e: &Env, farm: &Farm) -> Result<(Address, i128), FCError> {
        if self.pending_withdrawal_stake == 0 {
            return Err(FCError::InsufficientPendingWithdrawal);
        }

        let Delegation::NonDelegated(delegation_config) = farm.config.delegation else {
            return Err(FCError::DelegatedFarm);
        };

        let current_ts = e.ledger().timestamp();
        let cooldown_end = self.pending_withdrawal_ts.checked_add(delegation_config.withdrawal_cooldown_period).unwrap();

        if current_ts < cooldown_end {
            return Err(FCError::CooldownNotComplete);
        }

        let amount = self.pending_withdrawal_stake;
        self.pending_withdrawal_stake = 0;
        self.pending_withdrawal_ts = 0; // I am not sure...

        let token = delegation_config.token.clone();

        Ok((token, amount))
    }

    pub fn reward_once(&mut self, reward_token: &Address, amount: i128) -> Result<(), FCError> {
        let current_unclaimed = self.rewards_unclaimed.get(reward_token.clone()).unwrap();
        let new_unclaimed = current_unclaimed.checked_add(amount).unwrap();

        self.rewards_unclaimed.set(reward_token.clone(), new_unclaimed);

        Ok(())
    }

    pub fn try_get(e: &Env, farming_key: &FarmingKey, farm_id: u64) -> Result<Self, FCError> {
        storage::get_user(e, farming_key).ok_or(FCError::UserDoesNotExist)
    }

    pub fn stake(&mut self, e: &Env, farm: &mut Farm, amount: i128) -> Result<(), FCError> {
        // TODO: Min STAKE amount
        // if amount < farm.config.de

        // TODO: Farm is frozen check

        if farm.config.deposit_cap > 0 {
            let new_total = farm.total_staked.checked_add(amount).unwrap();
            if new_total > farm.config.deposit_cap {
                return Err(FCError::InternalError); // TODO Error
            }
        }

        self.refresh_user_rewards(farm, amount)?;

        Ok(())
    }

    pub fn refresh_rewards(
        &mut self,
        e: &Env,
        farm: &Farm,
    ) -> Result<(), FCError> {
        for reward_token in farm.rewards.keys() {
            let mut reward_info =
                RewardInfo::try_get(&e, farm.id, &reward_token).map_err(|_| {
                    FCError::InternalError // TODO: Event
                })?;

            let pending =
                self.calculate_pending_reward(&reward_token, &reward_info)?;

            if pending.is_positive() {
                let current_unclaimed = self.rewards_unclaimed.get(reward_token).unwrap_or(0);
                let new_unclaimed = current_unclaimed.checked_add(pending).unwrap();

                self.rewards_unclaimed.set(reward_token, new_unclaimed);
                let new_tally = self.active_stake.fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR).unwrap();
                self.debts_per_rewards_sc.set(reward_token, new_tally);
            }

        }

        Ok(())
    }

    fn adjust_pending(&mut self, reward_token: &Address, diff: i128) -> Result<(), FCError> {
        let old_pending = self.pending_rewards_unclaimed.get(reward_token.clone()).unwrap();
        let new_pending = old_pending.checked_add(diff).unwrap();

        self.pending_rewards_unclaimed.set(reward_token.clone(), new_pending);

        Ok(())
    }

    fn refresh_stake(
        &mut self,
        reward_token: &Address,
        reward_info: &RewardInfo,
        stake_diff: i128,
    ) -> Result<(), FCError> {
        let new_stake = self.active_stake.checked_add(stake_diff).unwrap();

        // let new_debt = new_stake.fixed_mul_
    }

    // So, refresh makes sense if you want to increase your pending...
    fn calculate_pending_reward(
        &self,
        reward_token: &Address,
        reward_info: &RewardInfo,
    ) -> Result<i128, FCError> {
        if self.active_stake == 0 {
            return Ok(0);
        }

        let unadjusted_reward = self
            .active_stake
            .fixed_mul_floor(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR).unwrap();
            .unwrap();
        let pending_increased = unadjusted_reward
            .checked_sub(self.debts_per_rewards_sc.get(reward_token.clone()).unwrap())
            .unwrap();

        Ok(pending_increased)
    }

    pub fn set(self, e: &Env, farm_id: u64, farming_key: &FarmingKey) {
        storage::set_user(e, farming_key, self);
    }
}

#[contracttype]
pub struct GlobalConfig {
    pub admin: Address,
    pub proposed_admin: Option<Address>,
}
