use farms_interface::Delegatee;
use soroban_sdk::{Address, BytesN, Env, contract, contractclient, contractimpl, vec, xdr::ToXdr};

use crate::{
    error::FCError,
    events,
    state::{Farm, FarmConfig, RewardInfo, RewardScheduleCurve},
    storage,
    utils::{self, Numeric, require_admin, require_nonnegative},
};

#[contractclient(name = "FarmsClient")]
pub trait Farms {
    // --- Farm init ---

    fn initialize_farm(e: Env, farm_config: FarmConfig) -> Result<u64, FCError>;

    fn update_farm_config(e: Env, farm_id: BytesN<32> /* TODO */) -> Result<(), FCError>;

    // --- Rewards ---

    fn initialize_reward(e: Env, farm_id: u64, reward_token: Address) -> Result<u64, FCError>;

    /// Adds rewards to a farm's reward pool
    fn add_rewards(
        e: Env,
        donor: Address,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError>;

    fn update_reward_schedule(
        e: Env,
        farm_id: u64,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError>;

    fn propose_farm_admin(e: Env, farm_id: BytesN<32>) -> Result<(), FCError>;

    fn reward_user_once(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        reward_index: u32,
        amount: i128,
    ) -> Result<(), FCError>;

    // TODO: Withdraw unused rewards

    // -- User ops --

    fn freeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    fn unfreeze_farm(e: Env, farm_id: u64) -> Result<(), FCError>;

    // Arguably, shouldn't exist
    fn initialize_user(e: Env, delegatee: Delegatee, farm_id: BytesN<32>) -> Result<(), FCError>;

    fn refresh_user_state(e: Env, delegatee: Delegatee, farm_id: BytesN<32>)
    -> Result<(), FCError>;

    fn set_stake_delegated(
        e: Env,
        delegatee: Delegatee,
        farm_id: u64,
        new_stake: i128,
    ) -> Result<(), FCError>;

    fn stake(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<(), FCError>;

    fn unstake(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        amount: i128,
    ) -> Result<i128, FCError>;

    fn withdraw_unstaked(
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
    ) -> Result<i128, FCError>;

    fn harvest(
        // TODO: Claim?
        e: Env,
        delegatee: Delegatee,
        farm_id: BytesN<32>,
        reward_index: u32,
    ) -> Result<i128, FCError>;

    fn harvest_all(e: Env, delegatee: Delegatee, farm_id: BytesN<32>) -> Result<i128, FCError>;

    // -- TODO: Queries --
}

#[contract]
struct FarmsContract;

#[contractimpl]
impl FarmsContract {
    pub fn __constructor(e: Env, admin: Address) {
        storage::set_admin(&e, &admin);
    }

    fn propose_admin(e: Env, proposed_admin: Address) {
        storage::extend_instance(&e);
        require_admin(&e);

        storage::set_proposed_admin(&e, &proposed_admin);

        // TODO: Should we add events here additionally?
    }

    fn accept_admin(e: Env) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let proposed_admin =
            storage::get_proposed_admin(&e).ok_or(FCError::ProposedAdminDoesNotExist)?;
        proposed_admin.require_auth();

        storage::reset_proposed_admin(&e);
        storage::set_admin(&e, &proposed_admin);

        Ok(())
    }
}

#[contractimpl]
impl Farms for FarmsContract {
    fn initialize_farm(e: Env, farm_config: FarmConfig) -> Result<u64, FCError> {
        storage::extend_instance(&e);
        utils::require_admin(&e);

        let farm = Farm::new_and_increment_farms_counter(&e, farm_config);
        let farm_id = farm.id;

        storage::set_farm(&e, &farm);
        events::initialize_farm(&e, farm);

        Ok(farm_id)
    }

    fn update_farm_config(e: Env, farm_id: BytesN<32> /* TODO */) -> Result<(), FCError> {
        todo!()
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
        delegatee: Delegatee,
        farm_id: u64,
        new_stake: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        require_nonnegative(new_stake);

        let mut farm = Farm::try_get(&e, farm_id)?;
        farm.require_can_stake()?;

        // Get or auto-initialize new user state

        Ok(())
    }
}
