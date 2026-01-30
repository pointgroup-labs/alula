use farms_interface::FarmingKey;
use soroban_sdk::{contract, contractclient, contractimpl, panic_with_error, vec, xdr::ToXdr, Address, BytesN, Env, Vec};

use crate::{
    error::FCError,
    events, processors,
    state::{Farm, FarmConfig, FarmConfigUpdate, GlobalConfig, RewardInfo, RewardScheduleCurve, User},
    storage,
    utils::{self, require_admin, require_nonnegative},
};

#[contractclient(name = "FarmsClient")]
pub trait Farms {
    // ---- Admin ----

    fn __constructor(e: Env, admin: Address, treasury_fee_bps: i128);

    fn propose_admin(e: Env, proposed_admin: Address);

    fn accept_admin(e: Env) -> Result<(), FCError>;
    
    // --- Farms ---

    /// Initializes a new farm
    ///
    /// # Arguments
    /// * `config` - farm's configuration
    ///
    /// # Returns
    /// Farm's unique ID as [`u64`]
    fn initialize_farm(e: Env, farm_config: FarmConfig) -> Result<u64, FCError>;

    /// Updates farm's configuration
    /// 
    /// # Arguments
    /// * `farm_id` - farm's ID
    /// * `config_update` - update to the current farm's config
    /// 
    /// # Panics
    /// If the farm's admin hasn't authorized the call or if the new farm's config is invalid
    fn update_farm_config(e: Env, farm_id: u64, config_update: FarmConfigUpdate) -> Result<(), FCError>;

    // --- Rewards ---
    
    /// Initializes a new reward token pool for a farm
    /// 
    /// # Arguments
    /// * `farm_id` - farm's ID
    /// * `reward_token` - new reward token
    fn initialize_reward(e: Env, farm_id: u64, reward_token: Address) -> Result<u64, FCError>;
    
    /// Adds rewards to an existing farm's reward pool
    /// 
    /// # Panics
    /// If a reward isn't initialized on the farm
    fn add_rewards(
        e: Env,
        farm_id: u64,
        amount: i128,
        funder: Address,
        reward_token: Address,
 ) -> Result<(), FCError>;

    // WARN: Should we even allow to update this schedule here???

    /// Sets a reward schedule for the reserve pool on a farm
    /// 
    /// # Arguments
    /// * `farm_id` - farm's ID
    /// * `reward_token` - reward pool's token address
    /// * `reward_schedule` - reward distribution schedule 
    fn set_reward_schedule(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError>;


    /// Withdraws unused rewards to the farm's admin
    fn withdraw_unused_rewards(
        e: Env,
        farm_id: u64,
    ) -> Result<i128, FCError>;

    /// Withdraws slashed amounts from early withdrawal penalties
    ///
    /// Slashed amounts accumulate when users exit locked positions early
    fn withdraw_slashed_amount(
        e: Env,
        farm_id: u64,
    ) -> Result<i128, FCError>;

    /// Proposes a new farm admin. Must be called in order for a new admin to accept
    fn propose_farm_admin(e: Env, farm_id: u64, proposed_admin: Address) -> Result<(), FCError>;

    /// Accepts a farm admin proposal
    fn accept_farm_admin(e: Env, farm_id: u64) -> Result<(), FCError>;


    /// Rewards a farming key's account once
    ///
    /// This bypasses the normal RPS calculation and directly credits rewards
    /// to a specific farming key
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
    
    /// Simulates the updated user state per a farm. Designed to be used
    /// for simulation purposes only
    fn simulate_get_user_state(e: Env, farming_key: FarmingKey, farm_id: u64) -> Result<User, FCError>;

        
    /// Sets a farming key's stake by the delegate authority.
    ///
    /// This is the core push-model function. The delegate authority (e.g., lending contract)
    /// calls this to update a farming key's stake whenever their position changes (deposit, withdraw,
    /// borrow, repay).
    ///
    /// # Arguments
    /// * `farming_key` - farming key whose stake is updated/set
    /// * `farm_id` - farm's ID
    /// * `new_stake` - the farming key updated the new stake
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

    /// Withdraws unstaked tokens after cooldown period(if present)
    fn withdraw_unstaked(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError>;

    /// Harvests available rewards for a specific reward token
    fn harvest(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<i128, FCError>;

    /// Harvests all available rewards for a specific reward token
    fn harvest_all(e: Env, farming_key: FarmingKey, farm_id: u64) -> Result<i128, FCError>;

    /// Freezes the farm by the farm admin
    fn freeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    /// Unfreezes the farm by the farm admin
    fn unfreeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    // ---- Queries ----

    fn get_global_config(e: Env) -> Result<GlobalConfig, FCError>;

    fn get_farm(e: Env, farm_id: u64) -> Result<Farm, FCError>;

    fn get_farms_ids(e: Env) -> Result<Vec<u64>, FCError>;

    fn get_farming_key_farm_position(
        e: Env,
        farm_id: u64,
        farming_key: FarmingKey,
    ) -> Result<User, FCError>;
}

#[contract]
struct FarmsContract;

// TODO: Add events

#[contractimpl]
impl Farms for FarmsContract {
    // ---- Admin ----

    fn __constructor(e: Env, admin: Address, treasury_fee_bps: i128) {
        if !(0..=MAX_TREASURY_FEE_BPS).contains(treasury_fee_bps) {
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

    // ---- Farms -----

    fn initialize_farm(e: Env, farm_config: FarmConfig) -> Result<u64, FCError> {
        storage::extend_instance(&e);
        utils::require_admin(&e);

        let farm = Farm::new(&e, farm_config);

        storage::increment_farms_counter(&e);
        storage::register_farm(&e, farm.id);
        storage::set_farm(&e, &farm);
        
        events::initialize_farm(&e, farm);

        Ok(farm_id)
    }

    fn update_farm_config(e: Env, farm_id: u64, config_update: FarmConfigUpdate) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_farm_admin();

        farm.update_config(&config_update)?;

        farm.set(&e);

        Ok(())
    }

    fn freeze_farm(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_farm_admin();

        farm.is_frozen = true;

        farm.set(&e);

        Ok(())
    }

    fn unfreeze_farm(e: Env, farm_id: u64) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_farm_admin();

        farm.is_frozen = false;

        farm.set(&e);

        Ok(())
    }

    fn initialize_reward(e: Env, farm_id: u64, reward_token: Address) -> Result<u64, FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_farm_admin();

        // farm.try_add_reward();

        Ok(1)
    }

    fn update_reward_schedule(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_farm_admin();

        // farm.update_reward_schedule()

        farm.set(&e);

        Ok(())
    }

    fn set_stake_delegated(
        e: Env,
        farming_key: FarmingKey,
        farm_id: u64,
        new_stake: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        require_nonnegative(new_stake);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_can_stake()?;

        // TODO: Add missing initialization here, I guess..
        let mut user = User::try_get(&e, &farming_key).unwrap_or_else(|| User::new(&e));

        // Ми маємо перерахувати X, які ми нараховуємо, еге ж?
        // Reward per second....

        processors::refresh_farm_rewards(&e, &mut farm)?;

        user.refresh_rewards()?;
        // Refresh global rewards...

        if new_stake == user.active_stake {
            todo!();
        }
        let diff = new_stake.checked_sub(user.active_stake).unwrap();

        // Update the total stake per the farm's reward
        if diff > 0 {
            if farm.config.deposit_cap > 0 {
                let new_total = farm.total_staked.checked_add(diff).unwrap();
                if new_total > farm.config.deposit_cap {
                    panic!();
                }
            }

            farm.total_staked = farm.total_staked.checked_add(diff).unwrap();
            user.last_stake_ts = e.ledger().timestamp();
        } else {
            farm.total_staked = farm.total_staked.checked_add(diff).unwrap();

            if new_stake == 0 {
                farm.num_users = farm.num_users.saturating_sub(1);
            }
        }

        user.active_stake = new_stake;

        for (reward_idx, (reward_token, _)) in farm.rewards.iter().enumerate() {
            let reward_info = RewardInfo::try_get(&e, farm, reward_token)?;

            // Tally is like a debt, right?
            let new_reward_debt = new_stake.checked_mul(reward_info.rewards_per_share).unwrap();
            user.reward_debts.set(&reward_token, new_reward_debt);
        }

        farm.set(&e);
        // user.set

        // for i in 0..farm.reward_infos.len() {
        //     if let Some(reward_info) = farm.reward_infos.get(i) {
        //         let new_tally = new_stake.saturating_mul(reward_info.reward_per_share_scaled);
        //         // user.rewards_tally_scaled.set(i, new_tally);
        //     }
        // }

        Ok(())
    }

    fn stake(e: Env, farming_key: FarmingKey, farm_id: u64, amount: i128) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_not_delegated();

        let mut user = User::try_get(&e, &farming_key)?;

        user.pro

        Ok(())
    }

    fn add_rewards(
        e: Env,
        donor: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError> {
        todo!()

        // storage::extend_instance(&e);

        // let mut farm = Farm::try_get(&e, farm_id)?;
        // farm.require_not_delegated();

        // Ok(())
    }
}
