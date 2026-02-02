use farms_interface::FarmingKey;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, BytesN, Env, Vec, contract, contractclient, contractimpl, panic_with_error, token,
    vec, xdr::ToXdr,
};

use crate::{
    constants::*,
    error::FCError,
    events, processors,
    state::{
        CommonFarmConfigUpdate, DelegatedFarmConfig, DelegatedFarmConfigUpdate, Farm, FarmConfig,
        GlobalConfig, NonDelegatedFarmConfig, NonDelegatedFarmConfigUpdate, RewardInfo,
        RewardScheduleCurve, User,
    },
    storage::{self, extend_instance},
    utils::{self, require_admin, require_nonnegative},
};

#[contractclient(name = "FarmsClient")]
pub trait Farms {
    // ---- Admin ----

    /// # Arguments
    /// * `admin` - administration address
    /// * `treasury_fee_bps` - contract's fee from reward distribution in basis points
    fn __constructor(e: Env, admin: Address, treasury_fee_bps: i128);

    fn propose_admin(e: Env, proposed_admin: Address);

    fn accept_admin(e: Env) -> Result<(), FCError>;

    fn update_treasury_fee(e: Env, new_fee_bps: i128) -> Result<(), FCError>;

    // --- Farms ---

    /// Initializes a new farm
    ///
    /// # Arguments
    /// * `config` - farm's configuration
    ///
    /// # Returns
    /// Farm's unique ID as [`u64`]
    fn initialize_farm(e: Env, farm_config: FarmConfig) -> Result<u64, FCError>;

    /// Updates the common farm's configuration
    ///
    /// # Arguments
    /// * `farm_id` - farm's ID
    /// * `config_update` - update to the current farm's config
    fn update_common_farm_config(
        e: Env,
        farm_id: u64,
        config_update: CommonFarmConfigUpdate,
    ) -> Result<(), FCError>;

    /// Updates the delegated farm's configuration
    ///
    /// # Arguments
    /// * `farm_id` - farm's ID
    /// * `config_update` - update to the current farm's config
    fn update_delegated_farm_config(
        e: Env,
        farm_id: u64,
        config_update: DelegatedFarmConfigUpdate,
    ) -> Result<(), FCError>;

    /// Updates the non-delegated farm's configuration
    ///
    /// # Arguments
    /// * `farm_id` - farm's ID
    /// * `config_update` - update to the current farm's config
    fn update_non_delegated_farm_config(
        e: Env,
        farm_id: u64,
        config_update: NonDelegatedFarmConfigUpdate,
    ) -> Result<(), FCError>;

    /// Freezes the farm(disables staking)
    fn freeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    /// Unfreezes the farm
    fn unfreeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    // --- Rewards ---

    /// Initializes a new reward token pool for the farm
    ///
    /// # Arguments
    /// * `farm_id` - farm's ID
    /// * `reward_token` - new reward token
    fn initialize_reward(e: Env, farm_id: u64, reward_token: Address) -> Result<(), FCError>;

    /// Adds rewards to an existing reward's available pool
    fn add_rewards(
        e: Env,
        farm_id: u64,
        amount: i128,
        funder: Address,
        reward_token: Address,
    ) -> Result<(), FCError>;

    /// Updates a reward schedule for the reserve pool on a farm
    fn update_reward_schedule(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError>;

    /// Withdraws unused rewards to the farm's admin account
    fn withdraw_unused(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<i128, FCError>;

    /// Withdraws slashed amounts from early withdrawal penalties
    ///
    /// Slashed amounts accumulate when users exit locked positions early
    fn withdraw_slashed(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
    ) -> Result<i128, FCError>;

    /// Proposes a new farm admin. Must be called in order for a new admin to accept
    fn propose_farm_admin(e: Env, farm_id: u64, proposed_admin: Address) -> Result<(), FCError>;

    /// Accepts a farm admin proposal
    fn accept_farm_admin(e: Env, farm_id: u64) -> Result<(), FCError>;

    /// Rewards a farming key's account once
    ///
    /// This bypasses the normal RPS calculation and directly credits rewards
    /// from the available pool to a given farming key
    ///
    /// # Arguments
    /// * `farming_key` - farming key to receive the reward
    /// * `farm_id` - farm's ID
    /// * `reward_token` - reward pool's token address
    /// * `amount` - amount to credit the farming key
    ///
    /// # Use Cases
    /// - Airdrops to specific users
    /// - Bonus rewards for special events
    /// - Retroactive reward corrections
    fn reward_once(
        e: Env,
        farm_id: u64,
        amount: i128,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<(), FCError>;

    // -- User Operations --

    /// Refreshes the farming key state (activates pending stakes, updates rewards)
    fn refresh_farming_key_state(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<(), FCError>;

    /// Sets a farming key's stake by the delegate authority.
    ///
    /// This is the core push-model function. The delegate authority (e.g., lending contract)
    /// calls this to update a farming key's stake whenever their position changes (deposit, withdraw,
    /// borrow, repay).
    ///
    /// # Arguments
    /// * `new_stake` - the farming key updated the new stake
    /// * `farm_id` - farm's ID
    /// * `farming_key` - farming key whose stake is updated/set
    ///
    /// # Authorization
    /// * Only callable via the farm's `delegate_authority`(hence, cannot be called for non-delegated farms)
    ///
    /// # Use Cases
    /// - Lending protocols: Call after deposit/withdraw/borrow/repay with obligation seed
    /// - AMM integrations: Call after add/remove liquidity
    /// - Any system that tracks user positions externally
    fn set_stake_delegated(
        e: Env,
        farm_id: u64,
        new_stake: i128,
        farming_key: FarmingKey,
    ) -> Result<(), FCError>;

    /// Stakes tokens for the non-delegated farm.
    ///
    /// # WARNING
    /// Must be used carefully, since non-delegated farms can slash the stake
    /// upon the early withdrawal
    fn stake(e: Env, farming_key: FarmingKey, farm_id: u64, amount: i128) -> Result<(), FCError>;

    /// Unstakes the tokens from the non-delegated farm. **MUST** be invoked before withdrawing
    ///
    /// # WARNING
    /// If locking is enabled, early withdrawal penalties may apply
    ///
    /// # Returns
    /// * The net amount after any early withdrawal penalty
    fn unstake(
        e: Env,
        farm_id: u64,
        amount: i128,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError>;

    /// Withdraws unstaked tokens after the cooldown period(if present)
    fn withdraw_unstaked(e: Env, farm_id: u64, farming_key: FarmingKey) -> Result<i128, FCError>;

    /// Harvests available given token rewards from the farm
    fn harvest(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError>;

    /// Harvests all available rewards from the farm for the given key
    fn harvest_all(e: Env, farming_key: FarmingKey, farm_id: u64) -> Result<i128, FCError>;

    // ---- Queries ----

    fn get_global_config(e: Env) -> GlobalConfig;

    fn get_farm(e: Env, farm_id: u64) -> Result<Farm, FCError>;

    fn get_farms_ids(e: Env) -> Result<Vec<u64>, FCError>;

    fn get_farming_key_position(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<User, FCError>;

    fn get_pending_rewards(
        e: Env,
        farm_id: u64,
        farming_key: Address,
    ) -> Result<Vec<(Address, i128)>, FCError>;
}

#[contract]
struct FarmsContract;
// TODO: Add events

#[contractimpl]
impl Farms for FarmsContract {
    fn __constructor(e: Env, admin: Address, treasury_fee_bps: i128) {
        if !(0..=MAX_TREASURY_FEE_BPS).contains(&treasury_fee_bps) {
            panic!("Invalid farms treasury fee bps");
        } else {
            storage::set_admin(&e, &admin);
            storage::set_treasury_fee_bps(&e, treasury_fee_bps);
        }
    }

    fn propose_admin(e: Env, proposed_admin: Address) {
        storage::extend_instance(&e);
        require_admin(&e);

        storage::set_proposed_admin(&e, &proposed_admin);
    }

    fn accept_admin(e: Env) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let proposed_admin =
            storage::get_proposed_admin(&e).ok_or(FCError::ProposedAdminDoesNotExist)?;
        proposed_admin.require_auth();

        storage::remove_proposed_admin(&e);
        storage::set_admin(&e, &proposed_admin);

        Ok(())
    }

    fn update_treasury_fee(e: Env, new_fee_bps: i128) -> Result<(), FCError> {
        storage::extend_instance(&e);
        utils::require_admin(&e);

        if !(0..=MAX_TREASURY_FEE_BPS).contains(&new_fee_bps) {
            return Err(FCError::InvalidConfigUpdate);
        }
        storage::set_treasury_fee_bps(&e, new_fee_bps);

        Ok(())
    }

    fn initialize_farm(e: Env, farm_config: FarmConfig) -> Result<u64, FCError> {
        storage::extend_instance(&e);
        utils::require_admin(&e);

        let farm = Farm::new(&e, farm_config);
        let farm_id = farm.id;

        storage::increment_farms_counter(&e);
        storage::register_farm(&e, farm_id);
        storage::set_farm(&e, &farm);

        events::initialize_farm(&e, farm);

        Ok(farm_id)
    }

    fn update_common_farm_config(
        e: Env,
        farm_id: u64,
        config_update: CommonFarmConfigUpdate,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.update_common_config(&config_update)?;
        farm.set(&e);

        Ok(())
    }

    fn update_delegated_farm_config(
        e: Env,
        farm_id: u64,
        config_update: DelegatedFarmConfigUpdate,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.update_delegated_config(&config_update)?;
        farm.set(&e);

        Ok(())
    }

    fn update_non_delegated_farm_config(
        e: Env,
        farm_id: u64,
        config_update: NonDelegatedFarmConfigUpdate,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.update_non_delegated_config(&config_update)?;
        farm.set(&e);

        Ok(())
    }

    fn freeze_farm(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.is_frozen = true;
        farm.set(&e);

        Ok(())
    }

    fn unfreeze_farm(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.is_frozen = false;
        farm.set(&e);

        Ok(())
    }

    fn initialize_reward(e: Env, farm_id: u64, reward_token: Address) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.try_initialize_reward(&e, &reward_token)?;
        farm.set(&e);

        Ok(())
    }

    fn add_rewards(
        e: Env,
        farm_id: u64,
        amount: i128,
        funder: Address,
        reward_token: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        funder.require_auth();
        utils::require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.try_add_reward(&e, &reward_token, amount)?;

        token::Client::new(&e, &reward_token).transfer(
            &funder,
            &e.current_contract_address(),
            &amount,
        );

        Ok(())
    }

    fn update_reward_schedule(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;
        reward_info.try_set_reward_schedule_curve(&e, farm_id, &schedule)?;

        Ok(())
    }

    fn withdraw_unused(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<i128, FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        let withdrawn_amount = farm.try_withdraw_unused(&e, &reward_token, amount)?;
        farm.set(&e);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        Ok(withdrawn_amount)
    }

    fn withdraw_slashed(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
    ) -> Result<i128, FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();
        farm.require_not_delegated()?;

        let (farm_token, withdrawn_amount) = farm.try_withdraw_slashed(&e, amount)?;
        farm.set(&e);

        token::Client::new(&e, &farm_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        Ok(withdrawn_amount)
    }

    fn propose_farm_admin(e: Env, farm_id: u64, proposed_admin: Address) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.propose_admin(&proposed_admin);
        farm.set(&e);

        Ok(())
    }

    fn accept_farm_admin(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.accept_admin()?;

        farm.set(&e);

        Ok(())
    }

    fn reward_once(
        e: Env,
        farm_id: u64,
        amount: i128,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<(), FCError> {
        storage::extend_instance(e);
        utils::require_nonnegative(amount)?;

        let farm = Farm::try_get(&e, farm_id)?;
        farm.require_can_reward_once()?;

        let mut user = User::try_get(&e, &farming_key, farm_id)?;
        let mut reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;

        reward_info.reward_once(amount);
        user.reward_once(&reward_token, amount);

        user.set(&e);
        reward_info.set(&e, farm_id, &reward_token);

        Ok(())
    }

    fn refresh_farming_key_state(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        let mut user = User::try_get(&e, &farming_key, farm_id)?;

        farm.refresh_rewards(&e)?;

        if user.pending_deposit_stake.is_positive() {
            processors::activate_pending_stake(&e, &farming_key, &mut farm, &mut user);
        }

        farm.set(&e);
        user.set(&e, farm_id, &farming_key);

        Ok(())
    }

    fn set_stake_delegated(
        e: Env,
        farm_id: u64,
        new_stake: i128,
        farming_key: FarmingKey,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_nonnegative(new_stake);

        let mut farm = Farm::try_get(&e, farm_id)?;
        // farm.require_can_stake()?;

        let mut is_new_user = false;
        let mut user = User::try_get(&e, &farming_key, farm_id).unwrap_or_else(|_| {
            is_new_user = true;

            User::new(&e)
        });

        processors::refresh_farm_rewards(&e, &mut farm)?;

        farm.refresh_rewards(&e)?;
        user.refresh_rewards(&e, &farm)?;

        if new_stake == user.active_stake {
            farm.set(&e);
            user.set(&e, farm_id, &farming_key);

            return Ok(());
        }

        let diff = new_stake.checked_sub(user.active_stake).unwrap();

        if diff.is_positive() {
            if farm.config.deposit_cap.is_positive() {
                let new_total = farm.total_staked.checked_add(diff).unwrap();
                if new_total > farm.config.deposit_cap {
                    return Err(FCError::DepositCapExceeded);
                }
            }

            user.last_stake_ts = e.ledger().timestamp();
        } else {
            if new_stake == 0 && !is_new_user {
                farm.num_users = farm.num_users.saturating_sub(1);
            }
        }

        farm.total_staked = farm.total_staked.checked_add(diff).unwrap();
        user.active_stake = new_stake;

        for reward_token in farm.rewards.keys() {
            let reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;

            let new_reward_debt = new_stake
                .fixed_mul_ceil(reward_info.accum_rewards_per_share_sc, SCALE_FACTOR)
                .unwrap();
            user.debts_per_rewards_sc.set(reward_token, new_reward_debt);
        }

        farm.set(&e);
        user.set(&e, farm_id, &farming_key);

        Ok(())
    }

    fn stake(e: Env, farming_key: FarmingKey, farm_id: u64, amount: i128) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();
        utils::require_nonnegative(amount);

        let mut farm = Farm::try_get(&e, farm_id)?;
        let mut user = User::try_get(&e, &farming_key, farm_id)?;

        processors::stake(&e, &farming_key, &mut farm, &mut user, amount)?;

        farm.set(&e);
        user.set(&e, farm_id, &farming_key);

        Ok(())
    }

    fn unstake(
        e: Env,
        farm_id: u64,
        amount: i128,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();
        utils::require_nonnegative(amount);

        let mut farm = Farm::try_get(&e, farm_id)?;
        let mut user = User::try_get(&e, &farming_key, farm_id)?;

        processors::unstake(&e, &farming_key, &mut farm, &mut user, amount)?;

        farm.set(&e);
        user.set(&e, farm_id, &farming_key);

        Ok(())
    }

    fn withdraw_unstaked(e: Env, farm_id: u64, farming_key: FarmingKey) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let farm = Farm::try_get(&e, farm_id)?;
        let mut user = User::try_get(&e, &farming_key, farm_id)?;

        let (token, withdrawn_amount) = user.withdraw_unstaked(&e, &farm)?;
        user.set(&e, farm_id, &farming_key);

        token::Client::new(&e, &token).transfer(
            &e.current_contract_address(),
            &farming_key.owner,
            &withdrawn_amount,
        );

        Ok(withdrawn_amount)
    }

    fn harvest(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError> {
        todo!()
    }

    fn harvest_all(e: Env, farming_key: FarmingKey, farm_id: u64) -> Result<i128, FCError> {
        todo!()
    }

    fn get_global_config(e: Env) -> GlobalConfig {
        storage::extend_instance(&e);

        GlobalConfig {
            admin: storage::get_admin(&e).expect("Admin must be set"),
            proposed_admin: storage::get_proposed_admin(&e),
        }
    }

    fn get_farm(e: Env, farm_id: u64) -> Result<Farm, FCError> {
        todo!()
    }

    fn get_farms_ids(e: Env) -> Result<Vec<u64>, FCError> {
        todo!()
    }

    fn get_farming_key_position(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<User, FCError> {
        todo!()
    }

    fn get_pending_rewards(
        e: Env,
        farm_id: u64,
        farming_key: Address,
    ) -> Result<Vec<(Address, i128)>, FCError> {
        todo!()
    }
}
