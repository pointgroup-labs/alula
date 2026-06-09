use farms_interface::Delegatee;
use soroban_sdk::{Address, BytesN, Env, contractevent};

use crate::{math::reward_curve::RewardScheduleCurve, state::RewardType};

#[contractevent]
struct InitializeFarm {
    #[topic]
    farm_id: BytesN<32>,
    admin: Address,
    token: Address,
}

#[contractevent]
struct UpdateFarmConfig {
    #[topic]
    farm_id: BytesN<32>,
}

#[contractevent]
struct FreezeFarm {
    #[topic]
    farm_id: BytesN<32>,
}

#[contractevent]
struct UnfreezeFarm {
    #[topic]
    farm_id: BytesN<32>,
}

#[contractevent]
struct InitializeReward {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    reward_token: Address,
    reward_index: u32,
    reward_type: RewardType,
}

#[contractevent]
struct AddRewards {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    funder: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct UpdateRewardsSchedule {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    reward_token: Address,
    schedule: RewardScheduleCurve,
}

#[contractevent]
struct WithdrawUnused {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    recipient: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct WithdrawSlashed {
    #[topic]
    farm_id: BytesN<32>,
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
struct ProposeFarmAdmin {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    proposed_admin: Address,
}

#[contractevent]
struct AcceptFarmAdmin {
    #[topic]
    farm_id: BytesN<32>,
}

#[contractevent]
struct RewardOnce {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    delegatee: Delegatee,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct RefreshDelegateeState {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    delegatee: Delegatee,
}

#[contractevent]
struct SetStakeDelegated {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    delegatee: Delegatee,
    new_stake: i128,
}

#[contractevent]
struct Stake {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    delegatee: Delegatee,
    amount: i128,
}

#[contractevent]
struct Unstake {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    delegatee: Delegatee,
    amount: i128,
}

#[contractevent]
struct WithdrawUnstaked {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    delegatee: Delegatee,
    amount: i128,
}

#[contractevent]
struct Harvest {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    delegatee: Delegatee,
    #[topic]
    reward_token: Address,
    amount: i128,
}

#[contractevent]
struct WithdrawTreasuryFees {
    #[topic]
    farm_id: BytesN<32>,
    #[topic]
    recipient: Address,
    #[topic]
    reward_token: Address,
    amount: i128,
}

pub fn initialize_farm(e: &Env, farm_id: BytesN<32>, admin: Address, token: Address) {
    InitializeFarm { farm_id, admin, token }.publish(e);
}

pub fn update_farm_config(e: &Env, farm_id: BytesN<32>) {
    UpdateFarmConfig { farm_id }.publish(e);
}

pub fn freeze_farm(e: &Env, farm_id: BytesN<32>) {
    FreezeFarm { farm_id }.publish(e);
}

pub fn unfreeze_farm(e: &Env, farm_id: BytesN<32>) {
    UnfreezeFarm { farm_id }.publish(e);
}

pub fn initialize_reward(
    e: &Env,
    farm_id: BytesN<32>,
    reward_token: Address,
    reward_index: u32,
    reward_type: RewardType,
) {
    InitializeReward { farm_id, reward_token, reward_index, reward_type }.publish(e);
}

pub fn add_rewards(
    e: &Env,
    farm_id: BytesN<32>,
    funder: Address,
    reward_token: Address,
    amount: i128,
) {
    AddRewards { farm_id, funder, reward_token, amount }.publish(e);
}

pub fn update_rewards_schedule(
    e: &Env,
    farm_id: BytesN<32>,
    reward_token: Address,
    schedule: RewardScheduleCurve,
) {
    UpdateRewardsSchedule { farm_id, reward_token, schedule }.publish(e);
}

pub fn withdraw_unused(
    e: &Env,
    farm_id: BytesN<32>,
    recipient: Address,
    reward_token: Address,
    amount: i128,
) {
    WithdrawUnused { farm_id, recipient, reward_token, amount }.publish(e);
}

pub fn withdraw_slashed(e: &Env, farm_id: BytesN<32>, recipient: Address, amount: i128) {
    WithdrawSlashed { farm_id, recipient, amount }.publish(e);
}

pub fn propose_admin(e: &Env, proposed_admin: Address) {
    ProposeAdmin { proposed_admin }.publish(e);
}

pub fn accept_admin(e: &Env) {
    AcceptAdmin {}.publish(e);
}

pub fn propose_farm_admin(e: &Env, farm_id: BytesN<32>, proposed_admin: Address) {
    ProposeFarmAdmin { farm_id, proposed_admin }.publish(e);
}

pub fn accept_farm_admin(e: &Env, farm_id: BytesN<32>) {
    AcceptFarmAdmin { farm_id }.publish(e);
}

pub fn reward_once(
    e: &Env,
    farm_id: BytesN<32>,
    delegatee: Delegatee,
    reward_token: Address,
    amount: i128,
) {
    RewardOnce { farm_id, delegatee, reward_token, amount }.publish(e);
}

pub fn refresh_delegatee_state(e: &Env, farm_id: BytesN<32>, delegatee: Delegatee) {
    RefreshDelegateeState { farm_id, delegatee }.publish(e);
}

pub fn set_stake_delegated(e: &Env, farm_id: BytesN<32>, delegatee: Delegatee, new_stake: i128) {
    SetStakeDelegated { farm_id, delegatee, new_stake }.publish(e);
}

pub fn stake(e: &Env, farm_id: BytesN<32>, delegatee: Delegatee, amount: i128) {
    Stake { farm_id, delegatee, amount }.publish(e);
}

pub fn unstake(e: &Env, farm_id: BytesN<32>, delegatee: Delegatee, amount: i128) {
    Unstake { farm_id, delegatee, amount }.publish(e);
}

pub fn withdraw_unstaked(e: &Env, farm_id: BytesN<32>, delegatee: Delegatee, amount: i128) {
    WithdrawUnstaked { farm_id, delegatee, amount }.publish(e);
}

pub fn harvest(
    e: &Env,
    farm_id: BytesN<32>,
    delegatee: Delegatee,
    reward_token: Address,
    amount: i128,
) {
    Harvest { farm_id, delegatee, reward_token, amount }.publish(e);
}

pub fn withdraw_treasury_fees(
    e: &Env,
    farm_id: BytesN<32>,
    recipient: Address,
    reward_token: Address,
    amount: i128,
) {
    WithdrawTreasuryFees { farm_id, recipient, reward_token, amount }.publish(e);
}
