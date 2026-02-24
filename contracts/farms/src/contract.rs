use farms_interface::FarmingKey;
use soroban_sdk::{Address, BytesN, Env, Vec, contract, contractclient, contractimpl, token};

use crate::{
    error::FCError,
    events,
    math::reward_curve::RewardScheduleCurve,
    processors,
    state::{
        CommonFarmConfigUpdate, DelegatedFarmConfigUpdate, Farm, FarmConfig, FarmingPosition,
        NonDelegatedFarmConfigUpdate, RewardInfo, RewardType,
    },
    storage,
    utils::{MathUtils, require_nonnegative, require_positive, transfer_in},
};

#[contractclient(name = "FarmsClient")]
pub trait Farms {
    fn __constructor(e: Env, config: FarmConfig);

    fn propose_admin(e: Env, proposed_admin: Address) -> Result<(), FCError>;

    fn accept_admin(e: Env) -> Result<(), FCError>;

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), FCError>;

    fn update_common_farm_config(
        e: Env,
        config_update: CommonFarmConfigUpdate,
    ) -> Result<(), FCError>;

    fn update_delegated_farm_config(
        e: Env,
        config_update: DelegatedFarmConfigUpdate,
    ) -> Result<(), FCError>;

    fn update_non_delegated_farm_config(
        e: Env,
        config_update: NonDelegatedFarmConfigUpdate,
    ) -> Result<(), FCError>;

    fn freeze_farm(e: Env) -> Result<(), FCError>;

    fn unfreeze_farm(e: Env) -> Result<(), FCError>;

    fn initialize_reward(
        e: Env,
        reward_token: Address,
        reward_type: RewardType,
    ) -> Result<(), FCError>;

    fn add_rewards(
        e: Env,
        amount: i128,
        funder: Address,
        reward_token: Address,
    ) -> Result<(), FCError>;

    fn update_reward_schedule(
        e: Env,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError>;

    fn withdraw_unused(
        e: Env,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError>;

    fn withdraw_slashed(e: Env, amount: i128, recipient: Address) -> Result<(), FCError>;

    fn withdraw_treasury_fees(
        e: Env,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError>;

    fn reward_once(
        e: Env,
        amount: i128,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<(), FCError>;

    fn refresh_farming_position(e: Env, farming_key: FarmingKey) -> Result<(), FCError>;

    fn cancel_pending_deposit(e: Env, farming_key: FarmingKey) -> Result<i128, FCError>;

    fn set_stake_delegated(
        e: Env,
        caller: Address,
        farming_key: FarmingKey,
        new_stake: i128,
    ) -> Result<(), FCError>;

    fn stake(e: Env, farming_key: FarmingKey, amount: i128) -> Result<(), FCError>;

    fn unstake(e: Env, amount: i128, farming_key: FarmingKey) -> Result<(), FCError>;

    fn withdraw_unstaked(e: Env, farming_key: FarmingKey) -> Result<i128, FCError>;

    fn harvest(e: Env, reward_token: Address, farming_key: FarmingKey) -> Result<i128, FCError>;

    fn harvest_all(e: Env, farming_key: FarmingKey) -> Result<(), FCError>;

    fn get_farm(e: Env) -> Result<Farm, FCError>;

    fn get_farming_position(e: Env, farming_key: FarmingKey) -> Result<FarmingPosition, FCError>;

    fn get_pending_rewards(
        e: Env,
        farming_key: FarmingKey,
    ) -> Result<Vec<(Address, i128)>, FCError>;
}

#[contract]
pub struct FarmsContract;

#[contractimpl]
impl Farms for FarmsContract {
    fn __constructor(e: Env, config: FarmConfig) {
        config.require_valid().expect("Invalid farm config");
        let farm = Farm::new(&e, config);
        farm.set(&e);
    }

    fn propose_admin(e: Env, proposed_admin: Address) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        farm.propose_admin(&proposed_admin);
        farm.set(&e);

        events::propose_admin(&e, proposed_admin);

        Ok(())
    }

    fn accept_admin(e: Env) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.accept_admin()?;
        farm.set(&e);

        events::accept_admin(&e);

        Ok(())
    }

    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let farm = Farm::try_get(&e)?;
        farm.require_admin();

        e.deployer().update_current_contract_wasm(new_wasm_hash);

        Ok(())
    }

    fn update_common_farm_config(
        e: Env,
        config_update: CommonFarmConfigUpdate,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        farm.update_common_config(&config_update)?;
        farm.set(&e);

        events::update_common_farm_config(&e, config_update);

        Ok(())
    }

    fn update_delegated_farm_config(
        e: Env,
        config_update: DelegatedFarmConfigUpdate,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        farm.update_delegated_config(&config_update)?;
        farm.set(&e);

        events::update_delegated_farm_config(&e, config_update);

        Ok(())
    }

    fn update_non_delegated_farm_config(
        e: Env,
        config_update: NonDelegatedFarmConfigUpdate,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        farm.update_non_delegated_config(&config_update)?;
        farm.set(&e);

        events::update_non_delegated_farm_config(&e, config_update);

        Ok(())
    }

    fn freeze_farm(e: Env) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        farm.is_frozen = true;
        farm.set(&e);

        events::freeze_farm(&e);

        Ok(())
    }

    fn unfreeze_farm(e: Env) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        farm.is_frozen = false;
        farm.set(&e);

        events::unfreeze_farm(&e);

        Ok(())
    }

    fn initialize_reward(
        e: Env,
        reward_token: Address,
        reward_type: RewardType,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        farm.try_initialize_reward(&e, &reward_token, reward_type)?;
        farm.set(&e);

        events::initialize_reward(&e, reward_token, reward_type);

        Ok(())
    }

    fn add_rewards(
        e: Env,
        amount: i128,
        funder: Address,
        reward_token: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        funder.require_auth();
        if amount <= 0 {
            return Err(FCError::InvalidAmount);
        }

        let mut farm = Farm::try_get(&e)?;
        farm.try_add_reward(&e, &reward_token, amount)?;
        farm.set(&e);

        transfer_in(&e, &reward_token, &funder, amount)?;

        events::add_rewards(&e, funder, reward_token, amount);

        Ok(())
    }

    fn update_reward_schedule(
        e: Env,
        reward_token: Address,
        schedule: RewardScheduleCurve,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();

        let mut reward_info = RewardInfo::try_get(&e, &reward_token)?;
        reward_info.try_set_reward_schedule_curve(&e, &mut farm, &reward_token, &schedule)?;

        farm.set(&e);
        reward_info.set(&e, &reward_token);

        events::update_rewards_schedule(&e, reward_token, schedule);

        Ok(())
    }

    fn withdraw_unused(
        e: Env,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();
        let mut reward_info = RewardInfo::try_get(&e, &reward_token)?;

        processors::withdraw_unused(&e, amount, &mut farm, &reward_token, &mut reward_info)?;

        farm.set(&e);
        reward_info.set(&e, &reward_token);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_unused(&e, recipient, reward_token, amount);

        Ok(())
    }

    fn withdraw_slashed(e: Env, amount: i128, recipient: Address) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();
        let farm_token = farm.token();

        farm.try_withdraw_slashed(amount)?;
        farm.set(&e);

        token::Client::new(&e, &farm_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_slashed(&e, recipient, amount);

        Ok(())
    }

    fn withdraw_treasury_fees(
        e: Env,
        amount: i128,
        recipient: Address,
        reward_token: Address,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let farm = Farm::try_get(&e)?;
        farm.require_admin();
        let mut reward_info = RewardInfo::try_get(&e, &reward_token)?;

        if amount > reward_info.accumulated_treasury_fees {
            return Err(FCError::InsufficientTreasuryFees);
        }

        reward_info.accumulated_treasury_fees =
            reward_info.accumulated_treasury_fees.checked_sub(amount).map_over_or_underflow()?;
        reward_info.set(&e, &reward_token);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &recipient,
            &amount,
        );

        events::withdraw_treasury_fees(&e, recipient, reward_token, amount);

        Ok(())
    }

    fn reward_once(
        e: Env,
        amount: i128,
        reward_token: Address,
        farming_key: FarmingKey,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_positive(amount)?;

        let mut farm = Farm::try_get(&e)?;
        farm.require_admin();
        farm.require_can_reward_once()?;

        farm.refresh_rewards(&e)?;

        let mut farming_position = FarmingPosition::try_get(&e, &farming_key)?;
        let mut reward_info = RewardInfo::try_get(&e, &reward_token)?;

        reward_info.reward_once(amount)?;
        farming_position.reward_once(&reward_token, amount)?;

        farm.set(&e);
        farming_position.set(&e, &farming_key);
        reward_info.set(&e, &reward_token);

        events::reward_once(&e, farming_key, reward_token, amount);

        Ok(())
    }

    fn refresh_farming_position(e: Env, farming_key: FarmingKey) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let mut farm = Farm::try_get(&e)?;
        farm.require_not_frozen()?;
        let mut farming_position = FarmingPosition::try_get(&e, &farming_key)?;

        processors::activate_pending_stake(&e, &mut farm, &mut farming_position)?;

        farm.set(&e);
        farming_position.set(&e, &farming_key);

        events::refresh_farming_position(&e, farming_key);

        Ok(())
    }

    fn cancel_pending_deposit(e: Env, farming_key: FarmingKey) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let farm = Farm::try_get(&e)?;
        let farm_token = farm.token();
        let mut farming_position = FarmingPosition::try_get(&e, &farming_key)?;

        let refund = processors::cancel_pending_deposit(&mut farming_position)?;

        farming_position.set(&e, &farming_key);

        token::Client::new(&e, &farm_token).transfer(
            &e.current_contract_address(),
            &farming_key.owner,
            &refund,
        );

        events::cancel_pending_deposit(&e, farming_key, refund);

        Ok(refund)
    }

    fn set_stake_delegated(
        e: Env,
        caller: Address,
        farming_key: FarmingKey,
        new_stake: i128,
    ) -> Result<(), FCError> {
        storage::extend_instance(&e);
        require_nonnegative(new_stake)?;

        let mut farm = Farm::try_get(&e)?;
        farm.require_delegate_authority(&caller)?;
        farm.require_not_frozen()?;

        let mut is_new_user = false;
        let mut farming_position =
            FarmingPosition::try_get(&e, &farming_key).unwrap_or_else(|_| {
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
        farming_position.set(&e, &farming_key);

        events::set_stake_delegated(&e, farming_key, new_stake);

        Ok(())
    }

    fn stake(e: Env, farming_key: FarmingKey, amount: i128) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e)?;
        farm.require_not_frozen()?;
        let farm_token = farm.token();
        let mut farming_position =
            FarmingPosition::try_get(&e, &farming_key).unwrap_or_else(|_| FarmingPosition::new(&e));

        processors::stake(&e, &mut farm, &mut farming_position, amount)?;

        farm.set(&e);
        farming_position.set(&e, &farming_key);

        transfer_in(&e, &farm_token, &farming_key.owner, amount)?;

        events::stake(&e, farming_key, amount);

        Ok(())
    }

    fn unstake(e: Env, amount: i128, farming_key: FarmingKey) -> Result<(), FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();
        require_nonnegative(amount)?;

        let mut farm = Farm::try_get(&e)?;
        let mut farming_position = FarmingPosition::try_get(&e, &farming_key)?;

        processors::unstake(&e, &mut farm, &mut farming_position, amount)?;

        farm.set(&e);
        farming_position.set(&e, &farming_key);

        events::unstake(&e, farming_key, amount);

        Ok(())
    }

    fn withdraw_unstaked(e: Env, farming_key: FarmingKey) -> Result<i128, FCError> {
        storage::extend_instance(&e);
        farming_key.owner.require_auth();

        let farm = Farm::try_get(&e)?;
        let mut farming_position = FarmingPosition::try_get(&e, &farming_key)?;

        let farm_token = farm.token();
        let withdrawn_amount = farming_position.withdraw_unstaked(&e, &farm)?;

        farming_position.set(&e, &farming_key);

        token::Client::new(&e, &farm_token).transfer(
            &e.current_contract_address(),
            &farming_key.owner,
            &withdrawn_amount,
        );

        events::withdraw_unstaked(&e, farming_key, withdrawn_amount);

        Ok(withdrawn_amount)
    }

    fn harvest(e: Env, reward_token: Address, farming_key: FarmingKey) -> Result<i128, FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        if !farm.config.is_harvest_permissionless {
            farming_key.owner.require_auth();
        }
        let mut reward_info = RewardInfo::try_get(&e, &reward_token)?;
        let mut farming_position = FarmingPosition::try_get(&e, &farming_key)?;

        let harvested_amount = processors::harvest(
            &e,
            &mut farm,
            &reward_token,
            &mut reward_info,
            &mut farming_position,
        )?;

        farm.set(&e);
        reward_info.set(&e, &reward_token);
        farming_position.set(&e, &farming_key);

        token::Client::new(&e, &reward_token).transfer(
            &e.current_contract_address(),
            &farming_key.owner,
            &harvested_amount,
        );

        events::harvest(&e, farming_key, reward_token, harvested_amount);

        Ok(harvested_amount)
    }

    fn harvest_all(e: Env, farming_key: FarmingKey) -> Result<(), FCError> {
        storage::extend_instance(&e);

        let mut farm = Farm::try_get(&e)?;
        if !farm.config.is_harvest_permissionless {
            farming_key.owner.require_auth();
        }
        let mut farming_position = FarmingPosition::try_get(&e, &farming_key)?;

        let harvest_results = processors::harvest_all(&e, &mut farm, &mut farming_position)?;

        farm.set(&e);
        farming_position.set(&e, &farming_key);

        for (reward_token, harvested_amount) in harvest_results {
            token::Client::new(&e, &reward_token).transfer(
                &e.current_contract_address(),
                &farming_key.owner,
                &harvested_amount,
            );

            events::harvest(&e, farming_key.clone(), reward_token, harvested_amount);
        }

        Ok(())
    }

    fn get_farm(e: Env) -> Result<Farm, FCError> {
        storage::extend_instance(&e);

        Farm::try_get(&e)
    }

    fn get_farming_position(e: Env, farming_key: FarmingKey) -> Result<FarmingPosition, FCError> {
        storage::extend_instance(&e);

        FarmingPosition::try_get(&e, &farming_key)
    }

    fn get_pending_rewards(
        e: Env,
        farming_key: FarmingKey,
    ) -> Result<Vec<(Address, i128)>, FCError> {
        storage::extend_instance(&e);

        let farm = Farm::try_get(&e)?;
        let farming_position = FarmingPosition::try_get(&e, &farming_key)?;
        let rps_snapshot = farm.simulate_refresh_rewards(&e)?;

        farming_position.get_pending_rewards(&e, &farm, &rps_snapshot)
    }
}
