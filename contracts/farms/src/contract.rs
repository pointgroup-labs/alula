use farms_interface::Delegatee;
use soroban_sdk::{
    Address, Bytes, BytesN, Env, Vec, contract, contractclient, contractimpl, token,
};

use crate::{
    error::FCError,
    events,
    math::reward_curve::RewardScheduleCurve,
    processors,
    state::{DelegateeState, Farm, FarmConfig, FarmState, GlobalConfig, RewardConfig, RewardInfo},
    storage,
    utils::{MathUtils, require_nonnegative, require_positive, transfer_in},
};

#[contractclient(name = "FarmsClient")]
pub trait Farms {
    /// Initializes the Farms contract.
    ///
    /// # Arguments
    /// * `admin` - global admin.
    fn __constructor(e: Env, admin: Address) -> Result<(), FCError>;

    /// Upgrades farm's contract.
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), FCError>;

    /// Proposes a new global admin for a two-step admin transfer.
    fn propose_admin(e: Env, proposed_admin: Address) -> Result<(), FCError>;

    /// Accepts global admin role.
    fn accept_admin(e: Env) -> Result<(), FCError>;

    /// Initializes a new farm. Only the global admin can initialize farms;
    /// he becomes the farm's admin and may later hand the role over via
    /// `propose_farm_admin` / `accept_farm_admin`.
    ///
    /// # Arguments
    /// * `seed` - optional unique seed that indexes the created farm. When
    ///   omitted, an internal counter is hashed to derive the farm ID.
    /// * `config` - farm's config
    ///
    /// # Returns
    /// The new farm's ID.
    fn initialize_farm(
        e: Env,
        seed: Option<BytesN<32>>,
        config: FarmConfig,
    ) -> Result<BytesN<32>, FCError>;

    /// Updates farm configuration (full replacement).
    ///
    /// The farm's `token` and delegation kind (delegated vs non-delegated)
    /// are immutable. Locking parameters of non-delegated farms cannot change
    /// while there are active stakes.
    fn update_farm_config(
        e: Env,
        farm_id: BytesN<32>,
        config_update: FarmConfig,
    ) -> Result<(), FCError>;

    /// Freezes a farm (disables staking).
    fn freeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FCError>;

    /// Unfreezes a farm.
    fn unfreeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FCError>;

    /// Initializes a new reward token for a farm.
    ///
    /// # Returns
    /// The index of the new reward on the farm (used as `reward_index`).
    fn initialize_reward(
        e: Env,
        farm_id: BytesN<32>,
        reward_config: RewardConfig,
    ) -> Result<u32, FCError>;

    /// Replaces the reward emission schedule of an initialized reward.
    fn set_reward_schedule(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        curve: RewardScheduleCurve,
    ) -> Result<(), FCError>;

    /// Adds rewards to a farm's reward pool.
    fn add_rewards(
        e: Env,
        funder: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError>;

    /// Withdraws unused rewards from a farm.
    fn withdraw_unused_rewards(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
        recipient: Address,
    ) -> Result<(), FCError>;

    /// Withdraws slashed amounts from early withdrawal penalties (for
    /// non-delegated farms) to the farm's admin.
    ///
    /// Slashed amounts accumulate when users exit locked positions early.
    fn withdraw_slashed(e: Env, farm_id: BytesN<32>, amount: i128) -> Result<(), FCError>;

    /// Withdraws accumulated treasury fees of a farm's reward to the farm's admin.
    fn withdraw_fees(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError>;

    /// Proposes farm admin role.
    fn propose_farm_admin(
        e: Env,
        farm_id: BytesN<32>,
        proposed_admin: Address,
    ) -> Result<(), FCError>;

    /// Accepts farm admin role.
    fn accept_farm_admin(e: Env, farm_id: BytesN<32>) -> Result<(), FCError>;

    /// Awards a one-time reward directly to a delegatee (airdrop/bonus)
    ///
    /// This bypasses the normal RPS calculation and directly credits rewards
    /// to a specific delegatee.
    ///
    /// # Arguments
    /// * `delegatee` - delegatee to receive the reward
    /// * `farm_id` - farm's ID
    /// * `reward_index` - index of a reward token to use
    /// * `amount` - reward amount
    fn reward_once(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError>;

    // ═══════════════════════════════════════════════════════════════════════════════
    // User Operations
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Refreshes delegatee's state (activates pending stakes, updates rewards)
    fn refresh_delegatee_state(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<(), FCError>;

    /// Sets a delegatee's stake (delegated mode - called by delegate authority)
    ///
    /// Core push-model function. The delegate authority calls this to update a delegatee's
    /// stake whenever their position changes.
    ///
    /// # Arguments
    /// * `delegatee` - delegatee
    /// * `farm_id` - farm to update stake for
    /// * `new_stake` - new delegatee's total stake amount
    fn set_stake_delegated(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        new_stake: i128,
    ) -> Result<(), FCError>;

    /// Directly stakes an amount to the farm (non-delegated mode)
    fn stake(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), FCError>;

    /// Directly unstakes an amount from the farm (non-delegated mode)
    fn unstake(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<i128, FCError>;

    /// Withdraws unstaked tokens after cooldown period
    fn withdraw_unstaked(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<i128, FCError>;

    /// Harvests rewards for a specific reward token
    fn harvest(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        reward_index: u32,
    ) -> Result<i128, FCError>;

    // ═══════════════════════════════════════════════════════════════════════════════
    // Queries
    // ═══════════════════════════════════════════════════════════════════════════════

    /// Gets the global configuration
    fn get_global_config(e: Env) -> Result<GlobalConfig, FCError>;

    /// Gets a farm by its ID
    fn get_farm(e: Env, farm_id: BytesN<32>) -> Result<FarmState, FCError>;

    /// Gets all farms
    fn get_all_farms(e: Env) -> Vec<BytesN<32>>;

    /// Gets user state for a farm
    fn get_delegatee_state(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<DelegateeState, FCError>;

    /// Gets pending rewards for a user
    fn get_pending_rewards(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<Vec<(Address, i128)>, FCError>;
}

#[contract]
pub struct FarmsContract;

#[contractimpl]
impl Farms for FarmsContract {
    fn __constructor(e: Env, admin: Address) -> Result<(), FCError> {
        GlobalConfig::new(admin).set(&e);
        
        Ok(())
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let global_config = GlobalConfig::try_get(&e)?;
        global_config.require_admin();

        e.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    fn propose_admin(e: Env, proposed_admin: Address) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut global_config = GlobalConfig::try_get(&e)?;
        global_config.require_admin();

        global_config.propose_admin(&proposed_admin);
        global_config.set(&e);

        events::propose_admin(&e, proposed_admin);

        Ok(())
    }

    fn accept_admin(e: Env) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut global_config = GlobalConfig::try_get(&e)?;
        global_config.accept_admin()?;
        global_config.set(&e);

        events::accept_admin(&e);

        Ok(())
    }

    fn initialize_farm(
        e: Env,
        seed: Option<BytesN<32>>,
        config: FarmConfig,
    ) -> Result<BytesN<32>, FCError> {
        storage::extend_instance(&e);

        let mut global_config = GlobalConfig::try_get(&e)?;
        global_config.require_admin();

        config.require_valid()?;

        let farm_id = match seed {
            Some(seed) => seed,
            None => {
                let counter_bytes = Bytes::from_slice(&e, &global_config.num_farms.to_be_bytes());
                e.crypto().sha256(&counter_bytes).to_bytes()
            }
        };

        if storage::has_farm(&e, &farm_id) {
            return Err(FCError::FarmAlreadyExists);
        }

        global_config.num_farms = global_config.num_farms.checked_add(1).map_over_or_underflow()?;

        let admin = global_config.admin.clone();
        let token = config.token.clone();
        let farm = Farm::new(&e, farm_id.clone(), admin.clone(), config);

        farm.set(&e);
        storage::add_farm_to_registry(&e, &farm_id);
        global_config.set(&e);

        events::initialize_farm(&e, farm_id.clone(), admin, token);

        Ok(farm_id)
    }

    fn update_farm_config(
        e: Env,
        farm_id: BytesN<32>,
        config_update: FarmConfig,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();

        farm.update_config(config_update)?;
        farm.set(&e);

        events::update_farm_config(&e, farm_id);

        Ok(())
    }

    fn freeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();

        farm.is_frozen = true;
        farm.set(&e);

        events::freeze_farm(&e, farm_id);

        Ok(())
    }

    fn unfreeze_farm(e: Env, farm_id: BytesN<32>) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();

        farm.is_frozen = false;
        farm.set(&e);

        events::unfreeze_farm(&e, farm_id);

        Ok(())
    }

    fn initialize_reward(
        e: Env,
        farm_id: BytesN<32>,
        reward_config: RewardConfig,
    ) -> Result<u32, FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();

        let reward_index = farm.try_initialize_reward(&e, &reward_config)?;
        farm.set(&e);

        events::initialize_reward(
            &e,
            farm_id,
            reward_config.reward_token,
            reward_index,
            reward_config.reward_type,
        );

        Ok(reward_index)
    }

    fn set_reward_schedule(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        curve: RewardScheduleCurve,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();
        let reward_token = farm.reward_token(reward_index)?;

        let mut reward_info = RewardInfo::try_get(&e, &farm_id, &reward_token)?;
        reward_info.try_set_reward_schedule_curve(&e, &mut farm, &reward_token, &curve)?;

        farm.set(&e);
        reward_info.set(&e, &farm_id, &reward_token);

        events::update_rewards_schedule(&e, farm_id, reward_token, curve);

        Ok(())
    }

    fn add_rewards(
        e: Env,
        funder: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        funder.require_auth();
        require_positive(amount)?;

        let mut farm = Farm::try_get(&e, &farm_id)?;
        let reward_token = farm.reward_token(reward_index)?;
        farm.try_add_reward(&e, &reward_token, amount)?;
        farm.set(&e);

        transfer_in(&e, &reward_token, &funder, amount)?;

        events::add_rewards(&e, farm_id, funder, reward_token, amount);

        Ok(())
    }

    fn withdraw_unused_rewards(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
        recipient: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();
        let reward_token = farm.reward_token(reward_index)?;
        let mut reward_info = RewardInfo::try_get(&e, &farm_id, &reward_token)?;

        processors::withdraw_unused(&e, amount, &mut farm, &reward_token, &mut reward_info)?;

        farm.set(&e);
        reward_info.set(&e, &farm_id, &reward_token);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_unused(&e, farm_id, recipient, reward_token, amount);

        Ok(())
    }

    fn withdraw_slashed(e: Env, farm_id: BytesN<32>, amount: i128) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();
        let farm_token = farm.token();
        let recipient = farm.admin.clone();

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

    fn withdraw_fees(
        e: Env,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();
        let recipient = farm.admin.clone();
        let reward_token = farm.reward_token(reward_index)?;
        let mut reward_info = RewardInfo::try_get(&e, &farm_id, &reward_token)?;

        if amount > reward_info.accumulated_treasury_fees {
            return Err(FCError::InsufficientTreasuryFees);
        }

        reward_info.accumulated_treasury_fees =
            reward_info.accumulated_treasury_fees.checked_sub(amount).map_over_or_underflow()?;
        reward_info.set(&e, &farm_id, &reward_token);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_treasury_fees(&e, farm_id, recipient, reward_token, amount);

        Ok(())
    }

    fn propose_farm_admin(
        e: Env,
        farm_id: BytesN<32>,
        proposed_admin: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();

        farm.propose_admin(&proposed_admin);
        farm.set(&e);

        events::propose_farm_admin(&e, farm_id, proposed_admin);

        Ok(())
    }

    fn accept_farm_admin(e: Env, farm_id: BytesN<32>) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.accept_admin()?;
        farm.set(&e);

        events::accept_farm_admin(&e, farm_id);

        Ok(())
    }

    fn reward_once(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_admin();
        farm.require_can_reward_once()?;
        let reward_token = farm.reward_token(reward_index)?;

        farm.refresh_rewards(&e)?;

        let mut delegatee_state = DelegateeState::try_get(&e, &farm_id, &delegatee)?;
        let mut reward_info = RewardInfo::try_get(&e, &farm_id, &reward_token)?;

        reward_info.reward_once(amount)?;
        delegatee_state.reward_once(&reward_token, amount)?;

        farm.set(&e);
        delegatee_state.set(&e, &farm_id, &delegatee);
        reward_info.set(&e, &farm_id, &reward_token);

        events::reward_once(&e, farm_id, delegatee, reward_token, amount);

        Ok(())
    }

    fn refresh_delegatee_state(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        delegatee.owner.require_auth();

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_not_frozen()?;
        let mut delegatee_state = DelegateeState::try_get(&e, &farm_id, &delegatee)?;

        processors::activate_pending_stake(&e, &mut farm, &mut delegatee_state)?;

        farm.set(&e);
        delegatee_state.set(&e, &farm_id, &delegatee);

        events::refresh_delegatee_state(&e, farm_id, delegatee);

        Ok(())
    }

    fn set_stake_delegated(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        new_stake: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_nonnegative(new_stake)?;

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_delegate_authority()?;
        farm.require_not_frozen()?;

        let mut is_new_user = false;
        let mut delegatee_state =
            DelegateeState::try_get(&e, &farm_id, &delegatee).unwrap_or_else(|_| {
                is_new_user = true;
                DelegateeState::new(&e)
            });

        processors::set_stake_delegated(
            &e,
            new_stake,
            &mut farm,
            is_new_user,
            &mut delegatee_state,
        )?;

        farm.set(&e);
        delegatee_state.set(&e, &farm_id, &delegatee);

        events::set_stake_delegated(&e, farm_id, delegatee, new_stake);

        Ok(())
    }

    fn stake(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        delegatee.owner.require_auth();
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, &farm_id)?;
        farm.require_not_frozen()?;
        let farm_token = farm.token();
        let mut delegatee_state = DelegateeState::try_get(&e, &farm_id, &delegatee)
            .unwrap_or_else(|_| DelegateeState::new(&e));

        processors::stake(&e, &mut farm, &mut delegatee_state, amount)?;

        farm.set(&e);
        delegatee_state.set(&e, &farm_id, &delegatee);

        transfer_in(&e, &farm_token, &delegatee.owner, amount)?;

        events::stake(&e, farm_id, delegatee, amount);

        Ok(())
    }

    fn unstake(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        delegatee.owner.require_auth();
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e, &farm_id)?;
        let mut delegatee_state = DelegateeState::try_get(&e, &farm_id, &delegatee)?;

        let net = processors::unstake(&e, &mut farm, &mut delegatee_state, amount)?;

        farm.set(&e);
        delegatee_state.set(&e, &farm_id, &delegatee);

        events::unstake(&e, farm_id, delegatee, amount);

        Ok(net)
    }

    fn withdraw_unstaked(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        delegatee.owner.require_auth();

        let farm = Farm::try_get(&e, &farm_id)?;
        let mut delegatee_state = DelegateeState::try_get(&e, &farm_id, &delegatee)?;

        let farm_token = farm.token();
        let withdrawn_amount = delegatee_state.withdraw_unstaked(&e, &farm)?;

        delegatee_state.set(&e, &farm_id, &delegatee);

        token::Client::new(&e, &farm_token).transfer(
            &e.current_contract_address(),
            &delegatee.owner,
            &withdrawn_amount,
        );

        events::withdraw_unstaked(&e, farm_id, delegatee, withdrawn_amount);

        Ok(withdrawn_amount)
    }

    fn harvest(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        reward_index: u32,
    ) -> Result<i128, FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, &farm_id)?;
        if !farm.config.is_harvest_permissionless {
            delegatee.owner.require_auth();
        }
        let reward_token = farm.reward_token(reward_index)?;
        let mut reward_info = RewardInfo::try_get(&e, &farm_id, &reward_token)?;
        let mut delegatee_state = DelegateeState::try_get(&e, &farm_id, &delegatee)?;

        let harvested_amount = processors::harvest(
            &e,
            &mut farm,
            &reward_token,
            &mut reward_info,
            &mut delegatee_state,
        )?;

        farm.set(&e);
        reward_info.set(&e, &farm_id, &reward_token);
        delegatee_state.set(&e, &farm_id, &delegatee);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &delegatee.owner,
            &harvested_amount,
        );

        events::harvest(&e, farm_id, delegatee, reward_token, harvested_amount);

        Ok(harvested_amount)
    }

    fn get_global_config(e: Env) -> Result<GlobalConfig, FCError> {
        storage::extend_instance(&e);

        GlobalConfig::try_get(&e)
    }

    fn get_farm(e: Env, farm_id: BytesN<32>) -> Result<FarmState, FCError> {
        storage::extend_instance(&e);

        let farm = Farm::try_get(&e, &farm_id)?;
        FarmState::load(&e, farm)
    }

    fn get_all_farms(e: Env) -> Vec<BytesN<32>> {
        storage::extend_instance(&e);

        storage::get_farms(&e)
    }

    fn get_delegatee_state(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<DelegateeState, FCError> {
        storage::extend_instance(&e);

        DelegateeState::try_get(&e, &farm_id, &delegatee)
    }

    fn get_pending_rewards(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<Vec<(Address, i128)>, FCError> {
        storage::extend_instance(&e);

        let farm = Farm::try_get(&e, &farm_id)?;
        let delegatee_state = DelegateeState::try_get(&e, &farm_id, &delegatee)?;
        let rps_snapshot = farm.simulate_refresh_rewards(&e)?;

        delegatee_state.get_pending_rewards(&e, &farm, &rps_snapshot)
    }
}
