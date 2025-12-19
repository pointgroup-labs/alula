#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    Address, BytesN, Env, Vec, contract, contractclient, contractimpl, vec, xdr::ToXdr,
};

use crate::{
    constants::{MAX_REWARD_TOKENS, MAX_TREASURY_FEE_BPS},
    error::FarmsError,
    events,
    operations::{farm_ops, reward_ops, stake_ops},
    state::{
        FarmConfig, FarmConfigUpdate, FarmState, GlobalConfig, GlobalConfigUpdate,
        RewardScheduleCurve, UserState,
    },
    storage,
};

#[contractclient(name = "FarmsClient")]
pub trait Farms {
    // ═══════════════════════════════════════════════════════════════════════════════
    // Initialization
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Initializes the Farms contract
    ///
    /// # Arguments
    /// * `admin` - Administrator address
    /// * `treasury_vault` - Treasury vault for collecting fees
    fn __constructor(e: Env, admin: Address, treasury_vault: Address) -> Result<(), FarmsError>;

    // ═══════════════════════════════════════════════════════════════════════════════
    // Admin: Global Config
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Updates global configuration
    fn update_global_config(e: Env, update: GlobalConfigUpdate) -> Result<(), FarmsError>;

    /// Sets a pending admin for two-step admin transfer
    fn set_pending_admin(e: Env, new_admin: Address) -> Result<(), FarmsError>;

    /// Accepts admin role (must be called by pending admin)
    fn accept_admin(e: Env) -> Result<(), FarmsError>;

    // ═══════════════════════════════════════════════════════════════════════════════
    // Admin: Farm Management
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Initializes a new farm
    ///
    /// # Arguments
    /// * `config` - Farm configuration
    ///
    /// # Returns
    /// * The new farm's unique ID
    fn initialize_farm(e: Env, config: FarmConfig) -> Result<BytesN<32>, FarmsError>;

    /// Updates farm configuration
    fn update_farm_config(
        e: Env,
        farm_id: BytesN<32>,
        update: FarmConfigUpdate,
    ) -> Result<(), FarmsError>;

    /// Freezes a farm (disables staking)
    fn freeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FarmsError>;

    /// Unfreezes a farm
    fn unfreeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FarmsError>;

    // ═══════════════════════════════════════════════════════════════════════════════
    // Admin: Rewards
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Initializes a new reward token for a farm
    fn initialize_reward(
        e: Env,
        farm_id: BytesN<32>,
        reward_token: Address,
        rewards_vault: Address,
    ) -> Result<u32, FarmsError>;

    /// Adds rewards to a farm's reward pool
    fn add_rewards(
        e: Env,
        funder: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FarmsError>;

    /// Updates the reward emission schedule
    fn update_reward_schedule(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FarmsError>;

    /// Withdraws unused rewards from a farm
    fn withdraw_unused_rewards(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
        recipient: Address,
    ) -> Result<(), FarmsError>;

    /// Withdraws slashed amounts from early withdrawal penalties
    ///
    /// Slashed amounts accumulate when users exit locked positions early.
    /// Admin can withdraw these to the configured spill address.
    fn withdraw_slashed_amount(e: Env, farm_id: BytesN<32>, amount: i128)
    -> Result<(), FarmsError>;

    /// Accepts farm admin role (must be called by pending farm admin)
    ///
    /// Two-step farm admin transfer: set pending via update_farm_config, then accept
    fn accept_farm_admin(e: Env, farm_id: BytesN<32>) -> Result<(), FarmsError>;

    /// Awards a one-time reward directly to a user (airdrop/bonus)
    ///
    /// This bypasses the normal RPS calculation and directly credits rewards
    /// to a specific user. Only callable by delegate authority when
    /// `is_reward_user_once_enabled` is true.
    ///
    /// # Arguments
    /// * `user` - User to receive the reward
    /// * `farm_id` - Farm ID
    /// * `reward_index` - Which reward token to use
    /// * `amount` - Amount to credit to the user
    ///
    /// # Use Cases
    /// - Airdrops to specific users
    /// - Bonus rewards for special events
    /// - Retroactive reward corrections
    fn reward_user_once(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FarmsError>;

    // ═══════════════════════════════════════════════════════════════════════════════
    // User Operations
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Initializes user state for a farm (must be called before staking)
    fn initialize_user(e: Env, user: Address, farm_id: BytesN<32>) -> Result<(), FarmsError>;

    /// Refreshes user state (activates pending stakes, updates rewards)
    fn refresh_user_state(e: Env, user: Address, farm_id: BytesN<32>) -> Result<(), FarmsError>;

    /// Sets a user's stake (delegated mode - called by delegate authority)
    ///
    /// This is the core push-model function. The delegate authority (e.g., Market contract)
    /// calls this to update a user's stake whenever their position changes (deposit, withdraw,
    /// borrow, repay).
    ///
    /// # Arguments
    /// * `user` - User whose stake to update
    /// * `farm_id` - Farm to update stake for
    /// * `new_stake` - The user's new total stake amount
    ///
    /// # Authorization
    /// * Only the farm's delegate_authority can call this function
    /// * Fails with `NotDelegateAuthority` if called by any other address
    ///
    /// # Use Cases
    /// - Lending protocols: Call after deposit/withdraw/borrow/repay
    /// - AMM integrations: Call after add/remove liquidity
    /// - Any system that tracks user positions externally
    fn set_stake_delegated(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        new_stake: i128,
    ) -> Result<(), FarmsError>;

    /// Directly stakes an amount in the farm (non-delegated mode)
    ///
    /// Only works on farms without a delegate_authority set.
    /// Updates the user's stake by the specified amount.
    ///
    /// # Errors
    /// * `FarmIsDelegated` - If the farm has a delegate_authority set
    fn stake(e: Env, user: Address, farm_id: BytesN<32>, amount: i128) -> Result<(), FarmsError>;

    /// Directly unstakes an amount from the farm (non-delegated mode)
    ///
    /// Only works on farms without a delegate_authority set.
    /// If locking is enabled, early withdrawal penalties may apply.
    ///
    /// # Errors
    /// * `FarmIsDelegated` - If the farm has a delegate_authority set
    ///
    /// # Returns
    /// * The net amount after any early withdrawal penalty
    fn unstake(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<i128, FarmsError>;

    /// Withdraws unstaked tokens after cooldown period
    fn withdraw_unstaked(e: Env, user: Address, farm_id: BytesN<32>) -> Result<i128, FarmsError>;

    /// Harvests rewards for a specific reward token
    fn harvest(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
    ) -> Result<i128, FarmsError>;

    /// Harvests all available rewards
    fn harvest_all(e: Env, user: Address, farm_id: BytesN<32>) -> Result<i128, FarmsError>;

    // ═══════════════════════════════════════════════════════════════════════════════
    // Queries
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Gets the global configuration
    fn get_global_config(e: Env) -> Result<GlobalConfig, FarmsError>;

    /// Gets a farm by its ID
    fn get_farm(e: Env, farm_id: BytesN<32>) -> Result<FarmState, FarmsError>;

    /// Gets all farms
    fn get_all_farms(e: Env) -> Vec<BytesN<32>>;

    /// Gets user state for a farm
    fn get_user_state(e: Env, user: Address, farm_id: BytesN<32>) -> Result<UserState, FarmsError>;

    /// Gets pending rewards for a user
    fn get_pending_rewards(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
    ) -> Result<Vec<i128>, FarmsError>;
}

#[contract]
pub struct FarmsContract;

#[contractimpl]
impl Farms for FarmsContract {
    fn __constructor(e: Env, admin: Address, treasury_vault: Address) -> Result<(), FarmsError> {
        if storage::has_global_config(&e) {
            return Err(FarmsError::AlreadyInitialized);
        }

        let config = GlobalConfig {
            admin: admin.clone(),
            treasury_vault,
            treasury_fee_bps: 0,
            pending_admin: None,
        };

        storage::set_global_config(&e, &config);
        events::emit_initialized(&e, &admin);

        Ok(())
    }

    fn update_global_config(e: Env, update: GlobalConfigUpdate) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let mut config = require_initialized(&e)?;
        require_admin(&e, &config)?;

        match update {
            GlobalConfigUpdate::TreasuryVault(vault) => {
                config.treasury_vault = vault;
            }
            GlobalConfigUpdate::TreasuryFeeBps(fee_bps) => {
                if !(0..=MAX_TREASURY_FEE_BPS).contains(&fee_bps) {
                    return Err(FarmsError::InvalidConfig);
                }
                config.treasury_fee_bps = fee_bps;
            }
        }

        storage::set_global_config(&e, &config);
        Ok(())
    }

    fn set_pending_admin(e: Env, new_admin: Address) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let mut config = require_initialized(&e)?;
        require_admin(&e, &config)?;

        config.pending_admin = Some(new_admin.clone());
        storage::set_global_config(&e, &config);

        events::emit_pending_admin_set(&e, &new_admin);
        Ok(())
    }

    fn accept_admin(e: Env) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let mut config = require_initialized(&e)?;

        let pending = config.pending_admin.ok_or(FarmsError::NoPendingAdmin)?;
        pending.require_auth();

        config.admin = pending.clone();
        config.pending_admin = None;
        storage::set_global_config(&e, &config);

        events::emit_admin_accepted(&e, &pending);
        Ok(())
    }

    fn initialize_farm(e: Env, config: FarmConfig) -> Result<BytesN<32>, FarmsError> {
        storage::extend_instance_storage(&e);
        let global_config = require_initialized(&e)?;
        require_admin(&e, &global_config)?;

        // Generate unique farm ID
        let counter = storage::increment_farm_counter(&e);
        let farm_id = generate_farm_id(&e, counter);

        let farm = FarmState {
            farm_id: farm_id.clone(),
            farm_admin: None,
            pending_farm_admin: None,
            delegate_authority: config.delegate_authority,
            total_staked: 0,
            num_users: 0,
            time_unit: config.time_unit,
            deposit_warmup_period: config.deposit_warmup_period,
            withdrawal_cooldown_period: config.withdrawal_cooldown_period,
            locking_mode: config.locking_mode,
            locking_start_ts: config.locking_start_ts,
            locking_duration: config.locking_duration,
            early_withdrawal_penalty_bps: config.early_withdrawal_penalty_bps,
            deposit_cap: config.deposit_cap,
            reward_infos: vec![&e],
            num_reward_tokens: 0,
            is_frozen: false,
            is_reward_user_once_enabled: false,
            slashed_amount_current: 0,
            slashed_amount_cumulative: 0,
            // Slashed amounts go to treasury by default
            slashed_amount_spill_address: global_config.treasury_vault.clone(),
        };

        storage::set_farm(&e, &farm_id, &farm);
        storage::register_farm(&e, &farm_id);

        events::emit_farm_created(&e, &farm_id);

        Ok(farm_id)
    }

    fn update_farm_config(
        e: Env,
        farm_id: BytesN<32>,
        update: FarmConfigUpdate,
    ) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let global_config = require_initialized(&e)?;

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        // Require farm admin or global admin authorization
        require_farm_admin(&e, &global_config, &farm)?;

        match update {
            FarmConfigUpdate::DepositWarmupPeriod(period) => {
                farm.deposit_warmup_period = period;
            }
            FarmConfigUpdate::WithdrawalCooldownPeriod(period) => {
                farm.withdrawal_cooldown_period = period;
            }
            FarmConfigUpdate::LockingMode(mode) => {
                farm.locking_mode = mode;
            }
            FarmConfigUpdate::LockingStartTs(ts) => {
                farm.locking_start_ts = ts;
            }
            FarmConfigUpdate::LockingDuration(duration) => {
                farm.locking_duration = duration;
            }
            FarmConfigUpdate::EarlyWithdrawalPenalty(penalty_bps) => {
                use crate::constants::MAX_EARLY_WITHDRAWAL_PENALTY_BPS;
                if !(0..=MAX_EARLY_WITHDRAWAL_PENALTY_BPS).contains(&penalty_bps) {
                    return Err(FarmsError::InvalidConfig);
                }
                farm.early_withdrawal_penalty_bps = penalty_bps;
            }
            FarmConfigUpdate::DepositCap(cap) => {
                farm.deposit_cap = cap;
            }
            FarmConfigUpdate::MinClaimDuration(duration) => {
                // Update min_claim_duration for all reward tokens
                for i in 0..farm.reward_infos.len() {
                    if let Some(mut info) = farm.reward_infos.get(i) {
                        info.min_claim_duration = duration;
                        farm.reward_infos.set(i, info);
                    }
                }
            }
            FarmConfigUpdate::DelegateAuthority(authority) => {
                farm.delegate_authority = authority;
            }
            FarmConfigUpdate::SlashedAmountSpillAddress(address) => {
                farm.slashed_amount_spill_address = address;
            }
            FarmConfigUpdate::PendingFarmAdmin(new_admin) => {
                farm.pending_farm_admin = Some(new_admin);
            }
            FarmConfigUpdate::RewardUserOnceEnabled(enabled) => {
                // reward_user_once requires a delegated farm
                if enabled && farm.delegate_authority.is_none() {
                    return Err(FarmsError::NotDelegateAuthority);
                }
                farm.is_reward_user_once_enabled = enabled;
            }
            FarmConfigUpdate::RewardType(reward_index, reward_type) => {
                if reward_index >= farm.reward_infos.len() {
                    return Err(FarmsError::RewardNotFound);
                }
                let mut reward_info =
                    farm.reward_infos.get(reward_index).ok_or(FarmsError::InternalError)?;
                reward_info.reward_type = reward_type;
                farm.reward_infos.set(reward_index, reward_info);
            }
        }

        storage::set_farm(&e, &farm_id, &farm);
        events::emit_farm_config_updated(&e, &farm_id);

        Ok(())
    }

    fn freeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let global_config = require_initialized(&e)?;

        let farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        require_farm_admin(&e, &global_config, &farm)?;

        let mut farm = farm;
        farm.is_frozen = true;
        storage::set_farm(&e, &farm_id, &farm);

        events::emit_farm_frozen(&e, &farm_id);
        Ok(())
    }

    fn unfreeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let global_config = require_initialized(&e)?;

        let farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        require_farm_admin(&e, &global_config, &farm)?;

        let mut farm = farm;
        if !farm.is_frozen {
            return Err(FarmsError::FarmNotFrozen);
        }
        farm.is_frozen = false;
        storage::set_farm(&e, &farm_id, &farm);

        events::emit_farm_unfrozen(&e, &farm_id);
        Ok(())
    }

    fn initialize_reward(
        e: Env,
        farm_id: BytesN<32>,
        reward_token: Address,
        rewards_vault: Address,
    ) -> Result<u32, FarmsError> {
        storage::extend_instance_storage(&e);
        let config = require_initialized(&e)?;
        require_admin(&e, &config)?;

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        if farm.num_reward_tokens >= MAX_REWARD_TOKENS {
            return Err(FarmsError::MaxRewardTokensReached);
        }

        // Check if reward token already exists
        for i in 0..farm.reward_infos.len() {
            if let Some(info) = farm.reward_infos.get(i)
                && info.token == reward_token
            {
                return Err(FarmsError::RewardTokenAlreadyExists);
            }
        }

        let reward_info = farm_ops::initialize_reward_info(&e, &reward_token, &rewards_vault);
        let index = farm.reward_infos.len();
        farm.reward_infos.push_back(reward_info);
        farm.num_reward_tokens += 1;

        storage::set_farm(&e, &farm_id, &farm);
        events::emit_reward_initialized(&e, &farm_id, &reward_token, index);

        Ok(index)
    }

    fn add_rewards(
        e: Env,
        funder: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FarmsError> {
        funder.require_auth();
        storage::extend_instance_storage(&e);

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        reward_ops::add_rewards(&e, &mut farm, reward_index, amount, &funder)
    }

    fn update_reward_schedule(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let config = require_initialized(&e)?;
        require_admin(&e, &config)?;

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        reward_ops::update_reward_schedule(&e, &mut farm, reward_index, schedule)
    }

    fn withdraw_unused_rewards(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
        recipient: Address,
    ) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let config = require_initialized(&e)?;
        require_admin(&e, &config)?;

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        reward_ops::withdraw_unused_rewards(&e, &mut farm, reward_index, amount, &recipient)
    }

    fn withdraw_slashed_amount(
        e: Env,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        let config = require_initialized(&e)?;
        require_admin(&e, &config)?;

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        if amount <= 0 {
            return Err(FarmsError::InvalidAmount);
        }

        if farm.slashed_amount_current < amount {
            return Err(FarmsError::InsufficientSlashedAmount);
        }

        // Reduce tracked slashed amount
        farm.slashed_amount_current =
            farm.slashed_amount_current.checked_sub(amount).ok_or(FarmsError::Underflow)?;

        storage::set_farm(&e, &farm_id, &farm);
        events::emit_slashed_amount_withdrawn(
            &e,
            &farm_id,
            &farm.slashed_amount_spill_address,
            amount,
        );

        Ok(())
    }

    fn accept_farm_admin(e: Env, farm_id: BytesN<32>) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);
        require_initialized(&e)?;

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        let pending = farm.pending_farm_admin.clone().ok_or(FarmsError::NoPendingAdmin)?;
        pending.require_auth();

        farm.farm_admin = Some(pending.clone());
        farm.pending_farm_admin = None;
        storage::set_farm(&e, &farm_id, &farm);

        events::emit_farm_admin_accepted(&e, &farm_id, &pending);
        Ok(())
    }

    fn reward_user_once(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        // Must be a delegated farm with reward_user_once enabled
        if !farm.is_reward_user_once_enabled {
            return Err(FarmsError::RewardUserOnceDisabled);
        }

        // Only delegate authority can call this
        let delegate = farm.delegate_authority.as_ref().ok_or(FarmsError::NotDelegateAuthority)?;
        delegate.require_auth();

        // Validate inputs
        if amount <= 0 {
            return Err(FarmsError::InvalidAmount);
        }

        if reward_index >= farm.reward_infos.len() {
            return Err(FarmsError::RewardNotFound);
        }

        let mut user_state =
            storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        // Update reward info - credit to issued unclaimed
        let mut reward_info =
            farm.reward_infos.get(reward_index).ok_or(FarmsError::InternalError)?;
        reward_info.rewards_issued_unclaimed =
            reward_info.rewards_issued_unclaimed.checked_add(amount).ok_or(FarmsError::Overflow)?;
        reward_info.rewards_issued_cumulative = reward_info
            .rewards_issued_cumulative
            .checked_add(amount)
            .ok_or(FarmsError::Overflow)?;
        farm.reward_infos.set(reward_index, reward_info);

        // Credit to user's unclaimed rewards
        let current_unclaimed = user_state.rewards_unclaimed.get(reward_index).unwrap_or(0);
        user_state
            .rewards_unclaimed
            .set(reward_index, current_unclaimed.checked_add(amount).ok_or(FarmsError::Overflow)?);

        // Save states
        storage::set_farm(&e, &farm_id, &farm);
        storage::set_user(&e, &user, &farm_id, &user_state);

        events::emit_reward_user_once(&e, &user, &farm_id, reward_index, amount);

        Ok(())
    }

    fn initialize_user(e: Env, user: Address, farm_id: BytesN<32>) -> Result<(), FarmsError> {
        user.require_auth();
        storage::extend_instance_storage(&e);

        if storage::has_user(&e, &user, &farm_id) {
            return Err(FarmsError::UserAlreadyExists);
        }

        let farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        // Initialize user with empty rewards vectors matching farm's reward tokens
        let num_rewards = farm.num_reward_tokens as u32;
        let mut rewards_tally = vec![&e];
        let mut rewards_unclaimed = vec![&e];
        let mut last_claim_ts = vec![&e];

        for _ in 0..num_rewards {
            rewards_tally.push_back(0i128);
            rewards_unclaimed.push_back(0i128);
            last_claim_ts.push_back(0u64);
        }

        let user_state = UserState {
            user: user.clone(),
            farm_id: farm_id.clone(),
            active_stake: 0,
            pending_deposit_stake: 0,
            pending_deposit_ts: 0,
            pending_withdrawal_stake: 0,
            pending_withdrawal_ts: 0,
            rewards_tally_scaled: rewards_tally,
            rewards_unclaimed,
            last_claim_ts,
            last_stake_ts: 0,
        };

        storage::set_user(&e, &user, &farm_id, &user_state);
        events::emit_user_initialized(&e, &user, &farm_id);

        Ok(())
    }

    fn refresh_user_state(e: Env, user: Address, farm_id: BytesN<32>) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        let mut user_state =
            storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        // Refresh global rewards (updates reward_per_share for all tokens)
        farm_ops::refresh_global_rewards(&e, &mut farm)?;

        // Try to activate pending stake if warmup complete
        if user_state.pending_deposit_stake > 0 {
            let _ = stake_ops::activate_pending_stake(&e, &mut farm, &mut user_state);
        }

        // CRITICAL: Save farm state after refresh_global_rewards updated it
        storage::set_farm(&e, &farm_id, &farm);
        // Also save user state in case pending stake was activated
        storage::set_user(&e, &user, &farm_id, &user_state);

        Ok(())
    }

    fn set_stake_delegated(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        new_stake: i128,
    ) -> Result<(), FarmsError> {
        storage::extend_instance_storage(&e);

        // Validate new_stake is non-negative
        if new_stake < 0 {
            return Err(FarmsError::InvalidAmount);
        }

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        // Verify caller is the delegate authority
        let delegate = farm.delegate_authority.as_ref().ok_or(FarmsError::NotDelegateAuthority)?;
        delegate.require_auth();

        // Check farm is not frozen (delegated farms shouldn't update stakes when frozen)
        if farm.is_frozen {
            return Err(FarmsError::FarmFrozen);
        }

        // Get or auto-initialize user state
        let is_new_user = storage::get_user(&e, &user, &farm_id).is_none();
        let mut user_state = if is_new_user {
            // Auto-initialize user for delegated farms
            let num_rewards = farm.num_reward_tokens as u32;
            let mut rewards_tally = soroban_sdk::vec![&e];
            let mut rewards_unclaimed = soroban_sdk::vec![&e];
            let mut last_claim_ts = soroban_sdk::vec![&e];

            for _ in 0..num_rewards {
                rewards_tally.push_back(0i128);
                rewards_unclaimed.push_back(0i128);
                last_claim_ts.push_back(0u64);
            }

            UserState {
                user: user.clone(),
                farm_id: farm_id.clone(),
                active_stake: 0,
                pending_deposit_stake: 0,
                pending_deposit_ts: 0,
                pending_withdrawal_stake: 0,
                pending_withdrawal_ts: 0,
                rewards_tally_scaled: rewards_tally,
                rewards_unclaimed,
                last_claim_ts,
                last_stake_ts: 0,
            }
        } else {
            storage::get_user(&e, &user, &farm_id).unwrap()
        };

        // Refresh global rewards first
        farm_ops::refresh_global_rewards(&e, &mut farm)?;

        // Refresh user rewards before changing stake
        stake_ops::refresh_user_rewards(&farm, &mut user_state)?;

        // Calculate the stake difference
        let current_stake = user_state.active_stake;

        // Early return if no change (but still save to update reward timestamps)
        if current_stake == new_stake {
            // Save states to persist reward refresh updates
            storage::set_farm(&e, &farm_id, &farm);
            storage::set_user(&e, &user, &farm_id, &user_state);
            return Ok(());
        }

        // Calculate delta (can be positive or negative)
        let delta = new_stake.checked_sub(current_stake).ok_or(FarmsError::Overflow)?;

        if delta > 0 {
            // Stake is increasing - check deposit cap
            if farm.deposit_cap > 0 {
                let new_total = farm.total_staked.checked_add(delta).ok_or(FarmsError::Overflow)?;
                if new_total > farm.deposit_cap {
                    return Err(FarmsError::DepositCapExceeded);
                }
            }

            farm.total_staked = farm.total_staked.checked_add(delta).ok_or(FarmsError::Overflow)?;

            // Increment user count if this is first stake
            if current_stake == 0 {
                farm.num_users = farm.num_users.saturating_add(1);
            }

            // Update last stake timestamp only on increases (for locking calculation)
            user_state.last_stake_ts = e.ledger().timestamp();
        } else {
            // Stake is decreasing (delta < 0)
            farm.total_staked = farm
                .total_staked
                .checked_add(delta) // delta is negative, so this subtracts
                .ok_or(FarmsError::Underflow)?;

            // Decrement user count if user has fully unstaked
            if new_stake == 0 && !is_new_user {
                farm.num_users = farm.num_users.saturating_sub(1);
            }
        }

        // Update user's stake
        user_state.active_stake = new_stake;

        // Update user's reward tally to match new stake
        // tally = new_stake × reward_per_share_scaled
        for i in 0..farm.reward_infos.len() {
            if let Some(reward_info) = farm.reward_infos.get(i) {
                let new_tally = new_stake.saturating_mul(reward_info.reward_per_share_scaled);
                user_state.rewards_tally_scaled.set(i, new_tally);
            }
        }

        // Save updated states
        storage::set_farm(&e, &farm_id, &farm);
        storage::set_user(&e, &user, &farm_id, &user_state);

        events::emit_stake_delegated(&e, &user, &farm_id, current_stake, new_stake);

        Ok(())
    }

    fn stake(e: Env, user: Address, farm_id: BytesN<32>, amount: i128) -> Result<(), FarmsError> {
        user.require_auth();
        storage::extend_instance_storage(&e);

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        // Ensure farm is not delegated
        if farm.delegate_authority.is_some() {
            return Err(FarmsError::FarmIsDelegated);
        }

        let mut user_state =
            storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        stake_ops::process_stake(&e, &mut farm, &mut user_state, amount)
    }

    fn unstake(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<i128, FarmsError> {
        user.require_auth();
        storage::extend_instance_storage(&e);

        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;

        // Ensure farm is not delegated - users cannot unstake directly from delegated farms
        if farm.delegate_authority.is_some() {
            return Err(FarmsError::FarmIsDelegated);
        }

        let mut user_state =
            storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        stake_ops::process_unstake(&e, &mut farm, &mut user_state, amount)
    }

    fn withdraw_unstaked(e: Env, user: Address, farm_id: BytesN<32>) -> Result<i128, FarmsError> {
        user.require_auth();
        storage::extend_instance_storage(&e);

        let farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        let mut user_state =
            storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        stake_ops::process_withdraw_unstaked(&e, &farm, &mut user_state)
    }

    fn harvest(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
    ) -> Result<i128, FarmsError> {
        user.require_auth();
        storage::extend_instance_storage(&e);

        let config = require_initialized(&e)?;
        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        let mut user_state =
            storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        reward_ops::harvest_single(&e, &config, &mut farm, &mut user_state, reward_index)
    }

    fn harvest_all(e: Env, user: Address, farm_id: BytesN<32>) -> Result<i128, FarmsError> {
        user.require_auth();
        storage::extend_instance_storage(&e);

        let config = require_initialized(&e)?;
        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        let mut user_state =
            storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        reward_ops::harvest_all(&e, &config, &mut farm, &mut user_state)
    }

    fn get_global_config(e: Env) -> Result<GlobalConfig, FarmsError> {
        storage::get_global_config(&e).ok_or(FarmsError::NotInitialized)
    }

    fn get_farm(e: Env, farm_id: BytesN<32>) -> Result<FarmState, FarmsError> {
        storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)
    }

    fn get_all_farms(e: Env) -> Vec<BytesN<32>> {
        storage::get_all_farms(&e)
    }

    fn get_user_state(e: Env, user: Address, farm_id: BytesN<32>) -> Result<UserState, FarmsError> {
        storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)
    }

    fn get_pending_rewards(
        e: Env,
        user: Address,
        farm_id: BytesN<32>,
    ) -> Result<Vec<i128>, FarmsError> {
        // Clone farm state for simulation - we don't persist changes
        let mut farm = storage::get_farm(&e, &farm_id).ok_or(FarmsError::FarmNotFound)?;
        let user_state = storage::get_user(&e, &user, &farm_id).ok_or(FarmsError::UserNotFound)?;

        // Simulate refresh to get accurate pending rewards
        // This updates reward_per_share_scaled in-memory but doesn't persist
        farm_ops::refresh_global_rewards(&e, &mut farm)?;

        reward_ops::get_pending_rewards_with_env(&e, &farm, &user_state)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

fn require_initialized(e: &Env) -> Result<GlobalConfig, FarmsError> {
    storage::get_global_config(e).ok_or(FarmsError::NotInitialized)
}

fn require_admin(_e: &Env, config: &GlobalConfig) -> Result<(), FarmsError> {
    config.admin.require_auth();
    Ok(())
}

/// Requires farm admin or global admin authorization.
/// If the farm has a `farm_admin` set, that address must authorize.
/// Otherwise, the global admin must authorize.
fn require_farm_admin(
    _e: &Env,
    global_config: &GlobalConfig,
    farm: &FarmState,
) -> Result<(), FarmsError> {
    if let Some(farm_admin) = &farm.farm_admin {
        farm_admin.require_auth();
    } else {
        global_config.admin.require_auth();
    }
    Ok(())
}

fn generate_farm_id(e: &Env, counter: u64) -> BytesN<32> {
    use soroban_sdk::crypto::Hash;

    // Create a unique hash from contract address and counter
    let mut preimage = soroban_sdk::Bytes::new(e);
    preimage.extend_from_array(&counter.to_be_bytes());

    // Add current contract address XDR bytes
    let addr_xdr = e.current_contract_address().to_xdr(e);
    for i in 0..addr_xdr.len() {
        if let Some(byte) = addr_xdr.get(i) {
            preimage.push_back(byte);
        }
    }

    let hash: Hash<32> = e.crypto().sha256(&preimage);
    hash.into()
}
