use farms_interface::Delegatee;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, BytesN, Env, Map, Vec, contracttype, map as smap, vec as svec};

use crate::{
    constants::*,
    error::FCError,
    math::reward_curve::RewardScheduleCurve,
    processors::{self, calculate_pending_reward, effective_stake},
    storage,
    utils::MathUtils,
};

// X1

/// Contract-wide configuration: the global admin who can initialize farms
/// and upgrade the contract.
#[contracttype]
#[derive(Clone)]
pub struct GlobalConfig {
    pub admin: Address,
    pub proposed_admin: Option<Address>,
    /// Internal counter used to derive farm IDs when no seed is provided.
    pub num_farms: u64, // X2(we can delete farms must check this)
}

impl GlobalConfig {
    pub fn new(admin: Address) -> Self {
        Self { admin, proposed_admin: None, num_farms: 0 }
    }

    pub fn try_get(e: &Env) -> Result<Self, FCError> {
        storage::get_global_config(e).ok_or(FCError::NotInitialized) // X4(this will never fail)
    }

    pub fn set(self, e: &Env) {
        storage::set_global_config(e, &self);
    }

    pub fn require_admin(&self) {
        self.admin.require_auth();
    }

    pub fn propose_admin(&mut self, admin: &Address) {
        self.proposed_admin = Some(admin.clone());
    }

    pub fn accept_admin(&mut self) -> Result<(), FCError> {
        let Some(proposed_admin) = self.proposed_admin.clone() else {
            return Err(FCError::ProposedAdminDoesNotExist);
        };
        proposed_admin.require_auth();

        self.admin = proposed_admin;
        self.proposed_admin = None;

        Ok(())
    }
}

#[contracttype]
pub struct Farm {
    pub id: BytesN<32>,
    pub admin: Address,
    pub proposed_admin: Option<Address>,
    pub num_users: u64,
    pub is_frozen: bool,
    pub total_staked: i128,
    pub config: FarmConfig,
    /// Reward tokens in initialization order; `reward_index` points here.
    pub rewards: Vec<Address>,
    pub current_slashed_amount: i128,
    pub cumulative_slashed_amount: i128, // X5(what the hell is this)
}

impl Farm {
    pub fn new(e: &Env, id: BytesN<32>, admin: Address, config: FarmConfig) -> Self {
        Self {
            id,
            admin,
            config,
            num_users: 0,
            total_staked: 0,
            is_frozen: true,
            rewards: svec![e],
            proposed_admin: None,
            current_slashed_amount: 0,
            cumulative_slashed_amount: 0,
        }
    }

    pub fn try_get(e: &Env, farm_id: &BytesN<32>) -> Result<Farm, FCError> {
        storage::get_farm(e, farm_id).ok_or(FCError::FarmDoesNotExist)
    }

    pub fn set(self, e: &Env) {
        storage::set_farm(e, &self);
    }

    pub fn require_admin(&self) {
        self.admin.require_auth();
    }

    pub fn require_delegate_authority(&self) -> Result<(), FCError> {
        let Delegation::Delegated(config) = &self.config.delegation else {
            return Err(FCError::NotDelegatedFarm);
        };
        config.delegate_authority.require_auth();

        Ok(())
    }

    pub fn require_not_frozen(&self) -> Result<(), FCError> {
        if self.is_frozen {
            return Err(FCError::FarmIsFrozen);
        }

        Ok(())
    }

    pub fn require_can_reward_once(&self) -> Result<(), FCError> {
        if !self.config.is_reward_once_enabled {
            return Err(FCError::RewardUserOnceIsDisabled);
        }

        Ok(())
    }

    pub fn reward_token(&self, reward_index: u32) -> Result<Address, FCError> {
        self.rewards.get(reward_index).ok_or(FCError::RewardDoesNotExistOnFarm)
    }

    pub fn refresh_rewards(&mut self, e: &Env) -> Result<(), FCError> {
        let current_ts = e.ledger().timestamp();

        for reward_token in self.rewards.iter() {
            let mut reward_info = RewardInfo::try_get(e, &self.id, &reward_token)?;

            let rewards_to_issue =
                processors::calculate_rewards_to_issue(self, &reward_info, current_ts)?;

            if rewards_to_issue.is_positive() {
                let denominator = match reward_info.reward_type {
                    RewardType::Proportional => self.total_staked,
                    RewardType::Constant => self.num_users as i128,
                };

                if denominator > 0 {
                    let rps_delta = rewards_to_issue
                        .fixed_div_floor(denominator, SCALE_FACTOR)
                        .map_over_or_underflow()?;
                    reward_info.accum_rewards_per_share_sc = reward_info
                        .accum_rewards_per_share_sc
                        .checked_add(rps_delta)
                        .map_over_or_underflow()?;
                }

                reward_info.rewards_available = reward_info
                    .rewards_available
                    .checked_sub(rewards_to_issue)
                    .map_over_or_underflow()?;
                reward_info.rewards_issued_unclaimed = reward_info
                    .rewards_issued_unclaimed
                    .checked_add(rewards_to_issue)
                    .map_over_or_underflow()?;
                reward_info.rewards_issued_cumulative = reward_info
                    .rewards_issued_cumulative
                    .checked_add(rewards_to_issue)
                    .map_over_or_underflow()?;
            }

            reward_info.last_issuance_ts = current_ts;
            reward_info.set(e, &self.id, &reward_token);
        }

        Ok(())
    }

    /// Computes a snapshot of each reward's accumulated RPS as if `refresh_rewards`
    /// had been called, but without persisting any changes to storage.
    pub fn simulate_refresh_rewards(&self, e: &Env) -> Result<Map<Address, i128>, FCError> {
        let current_ts = e.ledger().timestamp();
        let mut rps_snapshot: Map<Address, i128> = smap![e];

        for reward_token in self.rewards.iter() {
            let reward_info = RewardInfo::try_get(e, &self.id, &reward_token)?;

            let rewards_to_issue =
                processors::calculate_rewards_to_issue(self, &reward_info, current_ts)?;

            let mut accum_rps = reward_info.accum_rewards_per_share_sc;

            if rewards_to_issue.is_positive() {
                let denominator = match reward_info.reward_type {
                    RewardType::Proportional => self.total_staked,
                    RewardType::Constant => self.num_users as i128,
                };

                if denominator > 0 {
                    let rps_delta = rewards_to_issue
                        .fixed_div_floor(denominator, SCALE_FACTOR)
                        .map_over_or_underflow()?;
                    accum_rps = accum_rps.checked_add(rps_delta).map_over_or_underflow()?;
                }
            }

            rps_snapshot.set(reward_token, accum_rps);
        }

        Ok(rps_snapshot)
    }

    /// Applies a full-config update, enforcing immutability and update rules:
    ///
    /// - `token` is immutable.
    /// - The delegation *variant* (delegated vs non-delegated) is immutable.
    /// - Non-delegated locking parameters (`locking_ts`, `locking_duration`,
    ///   `locking_mode`, `early_withdrawal_penalty_bps`,
    ///   `withdrawal_cooldown_period`) cannot change while there are active stakes.
    pub fn update_config(&mut self, new_config: FarmConfig) -> Result<(), FCError> {
        new_config.require_valid()?;

        if new_config.token != self.config.token {
            return Err(FCError::InvalidFarmConfigUpdate);
        }

        match (&self.config.delegation, &new_config.delegation) {
            (Delegation::Delegated(_), Delegation::Delegated(_)) => {}
            (Delegation::NonDelegated(old), Delegation::NonDelegated(new)) => {
                let has_active_stakes = self.total_staked != 0;
                let locking_params_changed = old.locking_ts != new.locking_ts
                    || old.locking_duration != new.locking_duration
                    || !matches!(
                        (old.locking_mode, new.locking_mode),
                        (LockingMode::None, LockingMode::None)
                            | (LockingMode::Continuous, LockingMode::Continuous)
                            | (LockingMode::WithExpiry, LockingMode::WithExpiry)
                    )
                    || old.early_withdrawal_penalty_bps != new.early_withdrawal_penalty_bps
                    || old.withdrawal_cooldown_period != new.withdrawal_cooldown_period;

                if has_active_stakes && locking_params_changed {
                    return Err(FCError::InvalidFarmConfigUpdate);
                }
            }
            _ => return Err(FCError::InvalidFarmConfigUpdate),
        }

        self.config = new_config;

        Ok(())
    }

    pub fn try_withdraw_slashed(&mut self, amount: i128) -> Result<(), FCError> {
        if amount > self.current_slashed_amount {
            return Err(FCError::InsufficientCurrentSlashedAmount);
        }
        self.current_slashed_amount =
            self.current_slashed_amount.checked_sub(amount).map_over_or_underflow()?;

        Ok(())
    }

    pub fn try_initialize_reward(
        &mut self,
        e: &Env,
        reward_config: &RewardConfig,
    ) -> Result<u32, FCError> {
        self.require_can_initialize_reward(&reward_config.reward_token)?;

        let reward_index = self.rewards.len();
        self.rewards.push_back(reward_config.reward_token.clone());

        let reward_info = RewardInfo::new(e, reward_config)?;
        reward_info.set(e, &self.id, &reward_config.reward_token);

        Ok(reward_index)
    }

    fn require_can_initialize_reward(&self, reward_token: &Address) -> Result<(), FCError> {
        if self.rewards.len() >= MAX_FARM_NUM_REWARDS {
            return Err(FCError::MaxFarmNumRewardsReached);
        }
        if self.rewards.contains(reward_token) {
            return Err(FCError::TokenIsAlreadyAReward);
        }
        Ok(())
    }

    pub fn try_add_reward(
        &mut self,
        e: &Env,
        reward_token: &Address,
        amount: i128,
    ) -> Result<(), FCError> {
        let mut reward_info = RewardInfo::try_get(e, &self.id, reward_token)?;

        reward_info.rewards_available =
            reward_info.rewards_available.checked_add(amount).map_over_or_underflow()?;
        reward_info.set(e, &self.id, reward_token);

        Ok(())
    }

    pub fn accept_admin(&mut self) -> Result<(), FCError> {
        let Some(proposed_farm_admin) = self.proposed_admin.clone() else {
            return Err(FCError::ProposedAdminDoesNotExist);
        };
        proposed_farm_admin.require_auth();

        self.admin = proposed_farm_admin;
        self.proposed_admin = None;

        Ok(())
    }

    pub fn propose_admin(&mut self, admin: &Address) {
        self.proposed_admin = Some(admin.clone());
    }

    pub fn token(&self) -> Address {
        self.config.token.clone()
    }

    pub fn non_delegated_config(&self) -> Result<&NonDelegatedFarmConfig, FCError> {
        let Delegation::NonDelegated(config) = &self.config.delegation else {
            return Err(FCError::DelegatedFarm);
        };
        Ok(config)
    }
}

#[contracttype]
#[derive(Clone)]
pub struct OracleConfig {
    pub oracle_address: Address,
    pub oracle_max_age: u64,
}

#[contracttype]
#[derive(Clone)]
pub enum OptionalOracle {
    None,
    Some(OracleConfig),
}

impl OptionalOracle {
    pub fn as_ref(&self) -> Option<&OracleConfig> {
        match self {
            OptionalOracle::None => None,
            OptionalOracle::Some(config) => Some(config),
        }
    }
}

#[contracttype]
#[derive(Clone)]
pub struct FarmConfig {
    pub token: Address, // X5 this is a token for delegation
    pub deposit_cap: i128,
    pub treasury_fee_bps: i128,
    pub min_harvest_delay: u64,
    pub min_stake_amount: i128,
    pub delegation: Delegation,
    pub is_reward_once_enabled: bool,
    pub is_harvest_permissionless: bool,
    pub oracle: OptionalOracle,
}

impl FarmConfig {
    pub fn require_valid(&self) -> Result<(), FCError> {
        if !(0..=MAX_TREASURY_FEE_BPS).contains(&self.treasury_fee_bps) {
            return Err(FCError::InvalidConfig);
        }
        if self.min_stake_amount < 0 {
            return Err(FCError::InvalidConfig);
        }
        if self.deposit_cap < 0 {
            return Err(FCError::InvalidConfig);
        }
        if self.min_harvest_delay > MAX_HARVEST_DELAY {
            return Err(FCError::InvalidConfig);
        }

        if let OptionalOracle::Some(ref oracle) = self.oracle
            && oracle.oracle_max_age == 0
        {
            return Err(FCError::InvalidConfig);
        }

        if let Delegation::NonDelegated(nd) = &self.delegation {
            if !(0..=BPS_FACTOR).contains(&nd.early_withdrawal_penalty_bps) {
                return Err(FCError::InvalidConfig);
            }
            if nd.locking_duration > MAX_LOCKING_DURATION {
                return Err(FCError::InvalidConfig);
            }
            if nd.deposit_warmup_period > MAX_DEPOSIT_WARMUP_PERIOD {
                return Err(FCError::InvalidConfig);
            }
            if nd.withdrawal_cooldown_period > MAX_WITHDRAWAL_COOLDOWN_PERIOD {
                return Err(FCError::InvalidConfig);
            }
        }

        Ok(())
    }
}

#[contracttype]
#[derive(Clone)]
pub enum Delegation {
    Delegated(DelegatedFarmConfig),
    NonDelegated(NonDelegatedFarmConfig),
}

#[contracttype]
#[derive(Clone)]
pub struct DelegatedFarmConfig {
    /// The platform (e.g. a lending market) authorized to push stake updates
    /// via `set_stake_delegated`.
    pub delegate_authority: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct NonDelegatedFarmConfig {
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

/// Configuration for initializing a reward on a farm.
#[contracttype]
#[derive(Clone)]
pub struct RewardConfig {
    pub reward_token: Address,
    pub reward_type: RewardType,
    pub reward_schedule_curve: RewardScheduleCurve,
}

#[contracttype]
pub struct RewardInfo {
    pub reward_token: Address,
    pub last_issuance_ts: u64,
    pub rewards_available: i128,
    pub reward_type: RewardType,
    pub rewards_issued_unclaimed: i128,
    pub rewards_issued_cumulative: i128,
    pub accum_rewards_per_share_sc: i128,
    pub accumulated_treasury_fees: i128,
    pub reward_schedule_curve: RewardScheduleCurve,
}

impl RewardInfo {
    pub fn new(e: &Env, reward_config: &RewardConfig) -> Result<Self, FCError> {
        reward_config.reward_schedule_curve.require_valid()?;

        Ok(Self {
            reward_token: reward_config.reward_token.clone(),
            last_issuance_ts: e.ledger().timestamp(),
            rewards_available: 0,
            rewards_issued_unclaimed: 0,
            rewards_issued_cumulative: 0,
            accum_rewards_per_share_sc: 0,
            accumulated_treasury_fees: 0,
            reward_type: reward_config.reward_type,
            reward_schedule_curve: reward_config.reward_schedule_curve.clone(),
        })
    }

    pub fn try_get(e: &Env, farm_id: &BytesN<32>, reward_token: &Address) -> Result<Self, FCError> {
        storage::get_reward_info(e, farm_id, reward_token).ok_or(FCError::RewardDoesNotExistOnFarm)
    }

    pub fn set(self, e: &Env, farm_id: &BytesN<32>, reward_token: &Address) {
        storage::set_reward_info(e, farm_id, reward_token, &self);
    }

    pub fn reward_once(&mut self, amount: i128) -> Result<(), FCError> {
        if amount > self.rewards_available {
            return Err(FCError::InsufficientAvailableRewards);
        }

        self.rewards_issued_unclaimed =
            self.rewards_issued_unclaimed.checked_add(amount).map_over_or_underflow()?;
        self.rewards_issued_cumulative =
            self.rewards_issued_cumulative.checked_add(amount).map_over_or_underflow()?;
        self.rewards_available =
            self.rewards_available.checked_sub(amount).map_over_or_underflow()?;

        Ok(())
    }

    pub fn try_set_reward_schedule_curve(
        &mut self,
        e: &Env,
        farm: &mut Farm,
        reward_token: &Address,
        curve: &RewardScheduleCurve,
    ) -> Result<(), FCError> {
        curve.require_valid()?;

        farm.refresh_rewards(e)?;
        *self = RewardInfo::try_get(e, &farm.id, reward_token)?;
        self.reward_schedule_curve = curve.clone();
        self.last_issuance_ts = e.ledger().timestamp();

        Ok(())
    }
}

/// Full farm view returned by queries: the farm itself plus the state of
/// every initialized reward (ordered by `reward_index`).
#[contracttype]
pub struct FarmState {
    pub farm: Farm,
    pub rewards: Vec<RewardInfo>,
}

impl FarmState {
    pub fn load(e: &Env, farm: Farm) -> Result<Self, FCError> {
        let mut rewards = svec![e];
        for reward_token in farm.rewards.iter() {
            rewards.push_back(RewardInfo::try_get(e, &farm.id, &reward_token)?);
        }
        Ok(Self { farm, rewards })
    }
}

#[contracttype]
pub struct DelegateeState {
    pub active_stake: i128,

    pub pending_deposit_ts: u64,
    pub pending_deposit_stake: i128,

    pub pending_withdrawal_ts: u64,
    pub pending_withdrawal_stake: i128,
    pub rewards_tallies: Map<Address, i128>,
    pub rewards_unclaimed: Map<Address, i128>,

    pub last_stake_ts: u64,
    pub last_claim_ts: Map<Address, u64>,
}

impl DelegateeState {
    pub fn new(e: &Env) -> Self {
        Self {
            active_stake: 0,
            last_stake_ts: 0,
            pending_deposit_ts: 0,
            last_claim_ts: smap![e],
            pending_withdrawal_ts: 0,
            pending_deposit_stake: 0,
            rewards_tallies: smap![e],
            rewards_unclaimed: smap![e],
            pending_withdrawal_stake: 0,
        }
    }

    pub fn try_get(e: &Env, farm_id: &BytesN<32>, delegatee: &Delegatee) -> Result<Self, FCError> {
        storage::get_delegatee_state(e, farm_id, delegatee)
            .ok_or(FCError::FarmingPositionDoesNotExist)
    }

    pub fn set(self, e: &Env, farm_id: &BytesN<32>, delegatee: &Delegatee) {
        storage::set_delegatee_state(e, farm_id, delegatee, &self);
    }

    pub fn withdraw_unstaked(&mut self, e: &Env, farm: &Farm) -> Result<i128, FCError> {
        if self.pending_withdrawal_stake == 0 {
            return Err(FCError::InsufficientPendingWithdrawal);
        }

        let delegation_config = farm.non_delegated_config()?;

        let current_ts = e.ledger().timestamp();
        let cooldown_end = self
            .pending_withdrawal_ts
            .checked_add(delegation_config.withdrawal_cooldown_period)
            .map_over_or_underflow()?;

        if current_ts < cooldown_end {
            return Err(FCError::CooldownNotComplete);
        }

        let amount = self.pending_withdrawal_stake;
        self.pending_withdrawal_stake = 0;
        self.pending_withdrawal_ts = 0;

        Ok(amount)
    }

    pub fn reward_once(&mut self, reward_token: &Address, amount: i128) -> Result<(), FCError> {
        let current_unclaimed = self.rewards_unclaimed.get(reward_token.clone()).unwrap_or(0);
        let new_unclaimed = current_unclaimed.checked_add(amount).map_over_or_underflow()?;
        self.rewards_unclaimed.set(reward_token.clone(), new_unclaimed);
        
        Ok(())
    }

    pub fn get_pending_rewards(
        &self,
        e: &Env,
        farm: &Farm,
        rps_snapshot: &Map<Address, i128>,
    ) -> Result<Vec<(Address, i128)>, FCError> {
        let mut res = svec![e];

        for reward_token in farm.rewards.iter() {
            let reward_info = RewardInfo::try_get(e, &farm.id, &reward_token)?;
            let user_tally = self.rewards_tallies.get(reward_token.clone()).unwrap_or(0);

            let accum_rps = rps_snapshot
                .get(reward_token.clone())
                .unwrap_or(reward_info.accum_rewards_per_share_sc);

            let stake = effective_stake(self.active_stake, reward_info.reward_type);
            let pending_from_rps = calculate_pending_reward(stake, accum_rps, user_tally)?;

            let unclaimed = self.rewards_unclaimed.get(reward_token.clone()).unwrap_or(0);
            let total = pending_from_rps.checked_add(unclaimed).map_over_or_underflow()?;

            let net = if farm.config.treasury_fee_bps > 0 && total > 0 {
                let fee = total
                    .fixed_mul_ceil(farm.config.treasury_fee_bps, BPS_FACTOR)
                    .map_over_or_underflow()?;
                total.checked_sub(fee).map_over_or_underflow()?
            } else {
                total
            };

            res.push_back((reward_token, net));
        }

        Ok(res)
    }

    pub fn refresh_rewards(&mut self, e: &Env, farm: &Farm) -> Result<(), FCError> {
        for reward_token in farm.rewards.iter() {
            let reward_info = RewardInfo::try_get(e, &farm.id, &reward_token)?;

            let stake = effective_stake(self.active_stake, reward_info.reward_type);
            let user_tally = self.rewards_tallies.get(reward_token.clone()).unwrap_or(0);
            let pending = calculate_pending_reward(
                stake,
                reward_info.accum_rewards_per_share_sc,
                user_tally,
            )?;

            if pending > 0 {
                let current_unclaimed =
                    self.rewards_unclaimed.get(reward_token.clone()).unwrap_or(0);
                let new_unclaimed =
                    current_unclaimed.checked_add(pending).map_over_or_underflow()?;

                self.rewards_unclaimed.set(reward_token.clone(), new_unclaimed);
            }

            let new_tally = stake
                .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
                .map_over_or_underflow()?;
            self.rewards_tallies.set(reward_token, new_tally);
        }

        Ok(())
    }
}
