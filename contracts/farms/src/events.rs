use crate::{
    CommonFarmConfigUpdate, DelegatedFarmConfigUpdate, NonDelegatedFarmConfigUpdate,
    math::reward_curve::RewardScheduleCurve, state::FarmConfig,
};
use farms_interface::FarmingKey;
use soroban_sdk::{Address, Env, contractevent};

// -- Event structs --

#[contractevent]
struct InitializeFarm {
    #[topic]
    id: u64,
    farm_config: FarmConfig,
}

#[contractevent]
struct UpdateCommonFarmConfig {
    #[topic]
    farm_id: u64,
    config_update: CommonFarmConfigUpdate,
}

#[contractevent]
struct UpdateDelegatedFarmConfig {
    #[topic]
    farm_id: u64,
    config_update: DelegatedFarmConfigUpdate,
}

#[contractevent]
struct UpdateNonDelegatedFarmConfig {
    #[topic]
    farm_id: u64,
    config_update: NonDelegatedFarmConfigUpdate,
}

#[contractevent]
struct FreezeFarm {
    #[topic]
    farm_id: u64,
}

#[contractevent]
struct UnfreezeFarm {
    #[topic]
    farm_id: u64,
}

#[contractevent]
struct InitializeReward {
    #[topic]
    farm_id: u64,
    reward_token: Address,
}

#[contractevent]
struct AddRewards {
    #[topic]
    farm_id: u64,
    #[topic]
    funder: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct UpdateRewardsSchedule {
    #[topic]
    farm_id: u64,
    #[topic]
    reward_token: Address,
    schedule: RewardScheduleCurve,
}

#[contractevent]
struct WithdrawUnused {
    #[topic]
    farm_id: u64,
    #[topic]
    recipient: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct WithdrawSlashed {
    #[topic]
    farm_id: u64,
    #[topic]
    recipient: Address,
    amount: i128,
}

#[contractevent]
struct ProposeFarmAdmin {
    #[topic]
    farm_id: u64,
    #[topic]
    proposed_admin: Address,
}

#[contractevent]
struct AcceptFarmAdmin {
    #[topic]
    farm_id: u64,
}

#[contractevent]
struct RewardOnce {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct RefreshFarmingPosition {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
}

#[contractevent]
struct SetStakeDelegated {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
    new_stake: i128,
}

#[contractevent]
struct Stake {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
    amount: i128,
}

#[contractevent]
struct Unstake {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
    amount: i128,
}

#[contractevent]
struct WithdrawUnstaked {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
}

#[contractevent]
struct Harvest {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
    #[topic]
    reward_token: Address,
}

#[contractevent]
struct HarvestAll {
    #[topic]
    farm_id: u64,
    #[topic]
    farming_key: FarmingKey,
}

// -- Emitting functions --

pub fn initialize_farm(e: &Env, id: u64, farm_config: FarmConfig) {
    InitializeFarm { id, farm_config }.publish(e);
}

pub fn update_common_farm_config(e: &Env, farm_id: u64, config_update: CommonFarmConfigUpdate) {
    UpdateCommonFarmConfig { farm_id, config_update }.publish(e);
}

pub fn update_delegated_farm_config(
    e: &Env,
    farm_id: u64,
    config_update: DelegatedFarmConfigUpdate,
) {
    UpdateDelegatedFarmConfig { farm_id, config_update }.publish(e);
}

pub fn update_non_delegated_farm_config(
    e: &Env,
    farm_id: u64,
    config_update: NonDelegatedFarmConfigUpdate,
) {
    UpdateNonDelegatedFarmConfig { farm_id, config_update }.publish(e);
}

pub fn freeze_farm(e: &Env, farm_id: u64) {
    FreezeFarm { farm_id }.publish(e);
}

pub fn unfreeze_farm(e: &Env, farm_id: u64) {
    UnfreezeFarm { farm_id }.publish(e);
}

pub fn initialize_reward(e: &Env, farm_id: u64, reward_token: Address) {
    InitializeReward { farm_id, reward_token }.publish(e);
}

pub fn add_rewards(e: &Env, farm_id: u64, funder: Address, reward_token: Address, amount: i128) {
    AddRewards { farm_id, funder, reward_token, amount }.publish(e);
}

pub fn update_rewards_schedule(
    e: &Env,
    farm_id: u64,
    reward_token: Address,
    schedule: RewardScheduleCurve,
) {
    UpdateRewardsSchedule { farm_id, reward_token, schedule }.publish(e);
}

pub fn withdraw_unused(
    e: &Env,
    farm_id: u64,
    recipient: Address,
    reward_token: Address,
    amount: i128,
) {
    WithdrawUnused { farm_id, recipient, reward_token, amount }.publish(e);
}

pub fn withdraw_slashed(e: &Env, farm_id: u64, recipient: Address, amount: i128) {
    WithdrawSlashed { farm_id, recipient, amount }.publish(e);
}

pub fn propose_farm_admin(e: &Env, farm_id: u64, proposed_admin: Address) {
    ProposeFarmAdmin { farm_id, proposed_admin }.publish(e);
}

pub fn accept_farm_admin(e: &Env, farm_id: u64) {
    AcceptFarmAdmin { farm_id }.publish(e);
}

pub fn reward_once(
    e: &Env,
    farm_id: u64,
    farming_key: FarmingKey,
    reward_token: Address,
    amount: i128,
) {
    RewardOnce { farm_id, farming_key, reward_token, amount }.publish(e);
}

pub fn refresh_farming_position(e: &Env, farm_id: u64, farming_key: FarmingKey) {
    RefreshFarmingPosition { farm_id, farming_key }.publish(e);
}

pub fn set_stake_delegated(e: &Env, farm_id: u64, farming_key: FarmingKey, new_stake: i128) {
    SetStakeDelegated { farm_id, farming_key, new_stake }.publish(e);
}

pub fn stake(e: &Env, farm_id: u64, farming_key: FarmingKey, amount: i128) {
    Stake { farm_id, farming_key, amount }.publish(e);
}

pub fn unstake(e: &Env, farm_id: u64, farming_key: FarmingKey, amount: i128) {
    Unstake { farm_id, farming_key, amount }.publish(e);
}

pub fn withdraw_unstaked(e: &Env, farm_id: u64, farming_key: FarmingKey) {
    WithdrawUnstaked { farm_id, farming_key }.publish(e);
}

pub fn harvest(e: &Env, farm_id: u64, farming_key: FarmingKey, reward_token: Address) {
    Harvest { farm_id, farming_key, reward_token }.publish(e);
}

pub fn harvest_all(e: &Env, farm_id: u64, farming_key: FarmingKey) {
    HarvestAll { farm_id, farming_key }.publish(e);
}

#[contractevent]
struct WithdrawTreasuryFees {
    #[topic]
    farm_id: u64,
    #[topic]
    recipient: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

pub fn withdraw_treasury_fees(
    e: &Env,
    farm_id: u64,
    recipient: Address,
    reward_token: Address,
    amount: i128,
) {
    WithdrawTreasuryFees { farm_id, recipient, reward_token, amount }.publish(e);
}
