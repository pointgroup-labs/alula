use farms_interface::FarmingKey;
use soroban_sdk::{Address, Env, contractevent};

use crate::{
    CommonFarmConfigUpdate, DelegatedFarmConfigUpdate, NonDelegatedFarmConfigUpdate,
    math::reward_curve::RewardScheduleCurve, state::RewardType,
};

#[contractevent]
struct UpdateCommonFarmConfig {
    config_update: CommonFarmConfigUpdate,
}

#[contractevent]
struct UpdateDelegatedFarmConfig {
    config_update: DelegatedFarmConfigUpdate,
}

#[contractevent]
struct UpdateNonDelegatedFarmConfig {
    config_update: NonDelegatedFarmConfigUpdate,
}

#[contractevent]
struct FreezeFarm {}

#[contractevent]
struct UnfreezeFarm {}

#[contractevent]
struct InitializeReward {
    reward_token: Address,
    reward_type: RewardType,
}

#[contractevent]
struct AddRewards {
    #[topic]
    funder: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct UpdateRewardsSchedule {
    #[topic]
    reward_token: Address,
    schedule: RewardScheduleCurve,
}

#[contractevent]
struct WithdrawUnused {
    #[topic]
    recipient: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct WithdrawSlashed {
    #[topic]
    recipient: Address,
    amount: i128,
}

#[contractevent]
struct ProposeAdmin {
    #[topic]
    proposed_admin: Address,
}

#[contractevent]
struct AcceptAdmin {}

#[contractevent]
struct RewardOnce {
    #[topic]
    farming_key: FarmingKey,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct CancelPendingDeposit {
    #[topic]
    farming_key: FarmingKey,
    amount: i128,
}

#[contractevent]
struct RefreshFarmingPosition {
    #[topic]
    farming_key: FarmingKey,
}

#[contractevent]
struct SetStakeDelegated {
    #[topic]
    farming_key: FarmingKey,
    new_stake: i128,
}

#[contractevent]
struct Stake {
    #[topic]
    farming_key: FarmingKey,
    amount: i128,
}

#[contractevent]
struct Unstake {
    #[topic]
    farming_key: FarmingKey,
    amount: i128,
}

#[contractevent]
struct WithdrawUnstaked {
    #[topic]
    farming_key: FarmingKey,
    amount: i128,
}

#[contractevent]
struct Harvest {
    #[topic]
    farming_key: FarmingKey,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct WithdrawTreasuryFees {
    #[topic]
    recipient: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

pub fn update_common_farm_config(e: &Env, config_update: CommonFarmConfigUpdate) {
    UpdateCommonFarmConfig { config_update }.publish(e);
}

pub fn update_delegated_farm_config(e: &Env, config_update: DelegatedFarmConfigUpdate) {
    UpdateDelegatedFarmConfig { config_update }.publish(e);
}

pub fn update_non_delegated_farm_config(e: &Env, config_update: NonDelegatedFarmConfigUpdate) {
    UpdateNonDelegatedFarmConfig { config_update }.publish(e);
}

pub fn freeze_farm(e: &Env) {
    FreezeFarm {}.publish(e);
}

pub fn unfreeze_farm(e: &Env) {
    UnfreezeFarm {}.publish(e);
}

pub fn initialize_reward(e: &Env, reward_token: Address, reward_type: RewardType) {
    InitializeReward { reward_token, reward_type }.publish(e);
}

pub fn add_rewards(e: &Env, funder: Address, reward_token: Address, amount: i128) {
    AddRewards { funder, reward_token, amount }.publish(e);
}

pub fn update_rewards_schedule(e: &Env, reward_token: Address, schedule: RewardScheduleCurve) {
    UpdateRewardsSchedule { reward_token, schedule }.publish(e);
}

pub fn withdraw_unused(e: &Env, recipient: Address, reward_token: Address, amount: i128) {
    WithdrawUnused { recipient, reward_token, amount }.publish(e);
}

pub fn withdraw_slashed(e: &Env, recipient: Address, amount: i128) {
    WithdrawSlashed { recipient, amount }.publish(e);
}

pub fn propose_admin(e: &Env, proposed_admin: Address) {
    ProposeAdmin { proposed_admin }.publish(e);
}

pub fn accept_admin(e: &Env) {
    AcceptAdmin {}.publish(e);
}

pub fn reward_once(e: &Env, farming_key: FarmingKey, reward_token: Address, amount: i128) {
    RewardOnce { farming_key, reward_token, amount }.publish(e);
}

pub fn cancel_pending_deposit(e: &Env, farming_key: FarmingKey, amount: i128) {
    CancelPendingDeposit { farming_key, amount }.publish(e);
}

pub fn refresh_farming_position(e: &Env, farming_key: FarmingKey) {
    RefreshFarmingPosition { farming_key }.publish(e);
}

pub fn set_stake_delegated(e: &Env, farming_key: FarmingKey, new_stake: i128) {
    SetStakeDelegated { farming_key, new_stake }.publish(e);
}

pub fn stake(e: &Env, farming_key: FarmingKey, amount: i128) {
    Stake { farming_key, amount }.publish(e);
}

pub fn unstake(e: &Env, farming_key: FarmingKey, amount: i128) {
    Unstake { farming_key, amount }.publish(e);
}

pub fn withdraw_unstaked(e: &Env, farming_key: FarmingKey, amount: i128) {
    WithdrawUnstaked { farming_key, amount }.publish(e);
}

pub fn harvest(e: &Env, farming_key: FarmingKey, reward_token: Address, amount: i128) {
    Harvest { farming_key, reward_token, amount }.publish(e);
}

pub fn withdraw_treasury_fees(e: &Env, recipient: Address, reward_token: Address, amount: i128) {
    WithdrawTreasuryFees { recipient, reward_token, amount }.publish(e);
}
