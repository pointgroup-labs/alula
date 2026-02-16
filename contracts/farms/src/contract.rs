use farms_interface::FarmingKey;
use soroban_sdk::{
    Address, Env, Map, Vec, contract, contractclient, contractimpl, map as smap, token,
};

use crate::{
    constants::*,
    error::FCError,
    events,
    math::reward_curve::RewardScheduleCurve,
    processors,
    state::{
        CommonFarmConfigUpdate, DelegatedFarmConfigUpdate, Farm, FarmConfig, FarmingPosition,
        GlobalConfig, NonDelegatedFarmConfigUpdate, RewardInfo,
    },
    storage,
    utils::{require_admin, require_nonnegative, transfer_in},
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

    // ---- Farms ----

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

    /// Freezes the farm (disables staking)
    fn freeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    /// Unfreezes the farm
    fn unfreeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    // ---- Rewards ----

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

    /// Withdraws unused rewards
    fn withdraw_unused(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError>;

    /// Withdraws slashed amounts from early unstaking penalties.
    /// Slashed amounts accumulate when users unstake from the non-delegated farms early
    fn withdraw_slashed(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
    ) -> Result<(), FCError>;

    /// Withdraws accumulated treasury fees for a reward token
    fn withdraw_treasury_fees(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError>;

    /// Proposes a new farm admin. Must be called for a new admin to accept
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

    // ---- FarmingPosition Operations ----

    /// Refreshes the farming position (activates pending stakes, updates rewards)
    fn refresh_farming_position(
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
    fn unstake(e: Env, farm_id: u64, amount: i128, farming_key: FarmingKey) -> Result<(), FCError>;

    /// Withdraws unstaked tokens after the cooldown period(if present)
    fn withdraw_unstaked(e: Env, farm_id: u64, farming_key: FarmingKey) -> Result<i128, FCError>;

    /// Harvests available token rewards from the farm
    fn harvest(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError>;

    /// Harvests all available rewards from the farm for the given key
    fn harvest_all(e: Env, farming_key: FarmingKey, farm_id: u64) -> Result<(), FCError>;

    // ---- Queries ----

    fn get_global_config(e: Env) -> GlobalConfig;

    fn get_farm(e: Env, farm_id: u64) -> Result<Farm, FCError>;

    fn get_farms_ids(e: Env) -> Map<u64, ()>;

    fn get_farming_position(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<FarmingPosition, FCError>;

    fn get_pending_rewards(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
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
        require_admin(&e);

        if !(0..=MAX_TREASURY_FEE_BPS).contains(&new_fee_bps) {
            return Err(FCError::InvalidConfigUpdate);
        }
        storage::set_treasury_fee_bps(&e, new_fee_bps);

        Ok(())
    }

    fn initialize_farm(e: Env, farm_config: FarmConfig) -> Result<u64, FCError> {
        storage::extend_instance(&e);
        require_admin(&e);
        farm_config.require_valid()?;

        let farm = Farm::new(&e, farm_config.clone());
        let farm_id = farm.id;

        storage::increment_farms_counter(&e)?;
        storage::register_farm(&e, farm_id)?;
        storage::set_farm(&e, &farm);

        events::initialize_farm(&e, farm_id, farm_config);

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

        events::update_common_farm_config(&e, farm_id, config_update);

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

        events::update_delegated_farm_config(&e, farm_id, config_update);

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

        events::update_non_delegated_farm_config(&e, farm_id, config_update);

        Ok(())
    }

    fn freeze_farm(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.is_frozen = true;
        farm.set(&e);

        events::freeze_farm(&e, farm_id);

        Ok(())
    }

    fn unfreeze_farm(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.is_frozen = false;
        farm.set(&e);

        events::unfreeze_farm(&e, farm_id);

        Ok(())
    }

    fn initialize_reward(e: Env, farm_id: u64, reward_token: Address) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.try_initialize_reward(&e, &reward_token)?;
        farm.set(&e);

        events::initialize_reward(&e, farm_id, reward_token);

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
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.try_add_reward(&e, &reward_token, amount)?;
        farm.set(&e);

        transfer_in(&e, &reward_token, &funder, amount)?;

        events::add_rewards(&e, farm_id, funder, reward_token, amount);

        Ok(())
    }

    // TODO: Implement constant distribution as well

    fn update_reward_schedule(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        let mut reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;
        reward_info.try_set_reward_schedule_curve(&e, &mut farm, &reward_token, &schedule)?;

        farm.set(&e);
        reward_info.set(&e, farm_id, &reward_token);

        events::update_rewards_schedule(&e, farm_id, reward_token, schedule);

        Ok(())
    }

    fn withdraw_unused(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();
        let mut reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;

        processors::withdraw_unused(&e, amount, &mut farm, &reward_token, &mut reward_info)?;

        farm.set(&e);
        reward_info.set(&e, farm_id, &reward_token);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_unused(&e, farm_id, recipient, reward_token, amount);

        Ok(())
    }

    fn withdraw_slashed(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_admin(&e);
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, farm_id)?;
        let farm_token = farm.token();

        farm.try_withdraw_slashed(amount)?;
        farm.set(&e);

        token::Client::new(&e, &farm_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_slashed(&e, farm_id, recipient, amount);

        Ok(())
    }

    fn withdraw_treasury_fees(
        e: Env,
        farm_id: u64,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_admin(&e);
        require_nonnegative(amount)?;

        let _farm = Farm::try_get(&e, farm_id)?;
        let mut reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;

        if amount > reward_info.accumulated_treasury_fees {
            return Err(FCError::InsufficientTreasuryFees);
        }

        reward_info.accumulated_treasury_fees -= amount; // safe
        reward_info.set(&e, farm_id, &reward_token);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_treasury_fees(&e, farm_id, recipient, reward_token, amount);

        Ok(())
    }

    fn propose_farm_admin(e: Env, farm_id: u64, proposed_admin: Address) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();

        farm.propose_admin(&proposed_admin);
        farm.set(&e);

        events::propose_farm_admin(&e, farm_id, proposed_admin);

        Ok(())
    }

    fn accept_farm_admin(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.accept_admin()?;
        farm.set(&e);

        events::accept_farm_admin(&e, farm_id);

        Ok(())
    }

    fn reward_once(
        e: Env,
        farm_id: u64,
        amount: i128,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_nonnegative(amount)?;

        let farm = Farm::try_get(&e, farm_id)?;
        farm.require_admin();
        farm.require_can_reward_once()?;

        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)?;
        let mut reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;

        reward_info.reward_once(amount)?;
        farming_position.reward_once(&reward_token, amount);

        farming_position.set(&e, farm_id, &farming_key);
        reward_info.set(&e, farm_id, &reward_token);

        events::reward_once(&e, farm_id, farming_key, reward_token, amount);

        Ok(())
    }

    fn refresh_farming_position(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)?;

        processors::activate_pending_stake(&e, &mut farm, &mut farming_position)?;

        farm.set(&e);
        farming_position.set(&e, farm_id, &farming_key);

        events::refresh_farming_position(&e, farm_id, farming_key);

        Ok(())
    }

    fn set_stake_delegated(
        e: Env,
        farm_id: u64,
        new_stake: i128,
        farming_key: FarmingKey,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_nonnegative(new_stake)?;

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_delegate_authority()?;
        farm.require_not_frozen()?;

        let mut is_new_user = false;
        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)
            .unwrap_or_else(|_| {
                is_new_user = true;
                FarmingPosition::new(&e)
            });

        processors::set_stake_delegated(
            &e,
            new_stake,
            &mut farm,
            is_new_user,
            &mut farming_position,
        )?;

        farm.set(&e);
        farming_position.set(&e, farm_id, &farming_key);

        events::set_stake_delegated(&e, farm_id, farming_key, new_stake);

        Ok(())
    }

    fn stake(e: Env, farming_key: FarmingKey, farm_id: u64, amount: i128) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_not_frozen()?;
        let farm_token = farm.token();
        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)
            .unwrap_or_else(|_| FarmingPosition::new(&e));

        processors::stake(&e, &mut farm, &mut farming_position, amount)?;

        farm.set(&e);
        farming_position.set(&e, farm_id, &farming_key);

        transfer_in(&e, &farm_token, &farming_key.owner, amount)?;

        events::stake(&e, farm_id, farming_key, amount);

        Ok(())
    }

    fn unstake(e: Env, farm_id: u64, amount: i128, farming_key: FarmingKey) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, farm_id)?;
        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)?;

        processors::unstake(&e, &mut farm, &mut farming_position, amount)?;

        farm.set(&e);
        farming_position.set(&e, farm_id, &farming_key);

        events::unstake(&e, farm_id, farming_key, amount);

        Ok(())
    }

    fn withdraw_unstaked(e: Env, farm_id: u64, farming_key: FarmingKey) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let farm = Farm::try_get(&e, farm_id)?;
        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)?;

        let farm_token = farm.token();
        let withdrawn_amount = farming_position.withdraw_unstaked(&e, &farm)?;

        farming_position.set(&e, farm_id, &farming_key);

        token::Client::new(&e, &farm_token).transfer(
            &e.current_contract_address(),
            &farming_key.owner,
            &withdrawn_amount,
        );

        events::withdraw_unstaked(&e, farm_id, farming_key);

        Ok(withdrawn_amount)
    }

    fn harvest(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let mut farm = Farm::try_get(&e, farm_id)?;
        let mut reward_info = RewardInfo::try_get(&e, farm_id, &reward_token)?;
        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)?;

        let harvested_amount = processors::harvest(
            &e,
            &mut farm,
            &reward_token,
            &mut reward_info,
            &mut farming_position,
        )?;

        farm.set(&e);
        reward_info.set(&e, farm_id, &reward_token);
        farming_position.set(&e, farm_id, &farming_key);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &farming_key.owner,
            &harvested_amount,
        );

        events::harvest(&e, farm_id, farming_key, reward_token);

        Ok(harvested_amount)
    }

    fn harvest_all(e: Env, farming_key: FarmingKey, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let mut farm = Farm::try_get(&e, farm_id)?;
        let mut farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)?;

        let harvest_results = processors::harvest_all(&e, &mut farm, &mut farming_position)?;

        farm.set(&e);
        farming_position.set(&e, farm_id, &farming_key);

        for (reward_token, harvested_amount) in harvest_results {
            token::Client::new(&e, &reward_token).transfer(
                &e.current_contract_address(),
                &farming_key.owner,
                &harvested_amount,
            );
        }

        events::harvest_all(&e, farm_id, farming_key);

        Ok(())
    }

    fn get_global_config(e: Env) -> GlobalConfig {
        storage::extend_instance(&e);

        GlobalConfig {
            admin: storage::get_admin(&e).expect("Admin must be set"),
            proposed_admin: storage::get_proposed_admin(&e),
        }
    }

    fn get_farm(e: Env, farm_id: u64) -> Result<Farm, FCError> {
        storage::extend_instance(&e);

        Farm::try_get(&e, farm_id)
    }

    fn get_farms_ids(e: Env) -> Map<u64, ()> {
        storage::extend_instance(&e);

        storage::get_all_farms(&e).unwrap_or(smap![&e])
    }

    fn get_farming_position(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<FarmingPosition, FCError> {
        storage::extend_instance(&e);

        FarmingPosition::try_get(&e, farm_id, &farming_key)
    }

    fn get_pending_rewards(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<Vec<(Address, i128)>, FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        let farming_position = FarmingPosition::try_get(&e, farm_id, &farming_key)?;
        farm.refresh_rewards(&e)?;

        farming_position.get_pending_rewards(&e, &farm)
    }
}
