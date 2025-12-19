use soroban_sdk::{Address, BytesN, Env, contractevent};

// ═══════════════════════════════════════════════════════════════════════════════
// Event Definitions
// ═══════════════════════════════════════════════════════════════════════════════

#[contractevent]
struct InitializedEvent {
    #[topic]
    pub admin: Address,
}

#[contractevent]
struct PendingAdminSetEvent {
    #[topic]
    pub new_pending_admin: Address,
}

#[contractevent]
struct AdminAcceptedEvent {
    #[topic]
    pub new_admin: Address,
}

#[contractevent]
struct FarmCreatedEvent {
    #[topic]
    pub farm_id: BytesN<32>,
}

#[contractevent]
struct FarmFrozenEvent {
    #[topic]
    pub farm_id: BytesN<32>,
}

#[contractevent]
struct FarmUnfrozenEvent {
    #[topic]
    pub farm_id: BytesN<32>,
}

#[contractevent]
struct FarmConfigUpdatedEvent {
    #[topic]
    pub farm_id: BytesN<32>,
}

#[contractevent]
struct RewardInitializedEvent {
    #[topic]
    pub farm_id: BytesN<32>,
    #[topic]
    pub reward_token: Address,
    pub index: u32,
}

#[contractevent]
struct RewardsAddedEvent {
    #[topic]
    pub farm_id: BytesN<32>,
    pub reward_index: u32,
    pub amount: i128,
}

#[contractevent]
struct RewardScheduleUpdatedEvent {
    #[topic]
    pub farm_id: BytesN<32>,
    pub reward_index: u32,
}

#[contractevent]
struct RewardsWithdrawnEvent {
    #[topic]
    pub farm_id: BytesN<32>,
    pub reward_index: u32,
    pub amount: i128,
}

#[contractevent]
struct UserInitializedEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub farm_id: BytesN<32>,
}

#[contractevent]
struct StakeEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub farm_id: BytesN<32>,
    pub amount: i128,
}

#[contractevent]
struct UnstakeEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub farm_id: BytesN<32>,
    pub amount: i128,
    pub penalty: i128,
}

#[contractevent]
struct WithdrawUnstakedEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub farm_id: BytesN<32>,
    pub amount: i128,
}

#[contractevent]
struct HarvestEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub farm_id: BytesN<32>,
    pub reward_index: u32,
    pub amount: i128,
    pub fee: i128,
}

#[contractevent]
struct StakeDelegatedEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub farm_id: BytesN<32>,
    pub old_stake: i128,
    pub new_stake: i128,
}

#[contractevent]
struct RewardsAccruedEvent {
    #[topic]
    pub farm_id: BytesN<32>,
    pub reward_index: u32,
    pub amount: i128,
}

#[contractevent]
struct SlashedAmountWithdrawnEvent {
    #[topic]
    pub farm_id: BytesN<32>,
    #[topic]
    pub recipient: Address,
    pub amount: i128,
}

#[contractevent]
struct FarmAdminAcceptedEvent {
    #[topic]
    pub farm_id: BytesN<32>,
    #[topic]
    pub new_admin: Address,
}

#[contractevent]
struct RewardUserOnceEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub farm_id: BytesN<32>,
    pub reward_index: u32,
    pub amount: i128,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Event Publishing Functions
// ═══════════════════════════════════════════════════════════════════════════════

pub fn emit_initialized(e: &Env, admin: &Address) {
    InitializedEvent { admin: admin.clone() }.publish(e);
}

pub fn emit_pending_admin_set(e: &Env, new_pending_admin: &Address) {
    PendingAdminSetEvent { new_pending_admin: new_pending_admin.clone() }.publish(e);
}

pub fn emit_admin_accepted(e: &Env, new_admin: &Address) {
    AdminAcceptedEvent { new_admin: new_admin.clone() }.publish(e);
}

pub fn emit_farm_created(e: &Env, farm_id: &BytesN<32>) {
    FarmCreatedEvent { farm_id: farm_id.clone() }.publish(e);
}

pub fn emit_farm_frozen(e: &Env, farm_id: &BytesN<32>) {
    FarmFrozenEvent { farm_id: farm_id.clone() }.publish(e);
}

pub fn emit_farm_unfrozen(e: &Env, farm_id: &BytesN<32>) {
    FarmUnfrozenEvent { farm_id: farm_id.clone() }.publish(e);
}

pub fn emit_farm_config_updated(e: &Env, farm_id: &BytesN<32>) {
    FarmConfigUpdatedEvent { farm_id: farm_id.clone() }.publish(e);
}

pub fn emit_reward_initialized(e: &Env, farm_id: &BytesN<32>, reward_token: &Address, index: u32) {
    RewardInitializedEvent { farm_id: farm_id.clone(), reward_token: reward_token.clone(), index }
        .publish(e);
}

pub fn emit_rewards_added(e: &Env, farm_id: &BytesN<32>, reward_index: u32, amount: i128) {
    RewardsAddedEvent { farm_id: farm_id.clone(), reward_index, amount }.publish(e);
}

pub fn emit_reward_schedule_updated(e: &Env, farm_id: &BytesN<32>, reward_index: u32) {
    RewardScheduleUpdatedEvent { farm_id: farm_id.clone(), reward_index }.publish(e);
}

pub fn emit_rewards_withdrawn(e: &Env, farm_id: &BytesN<32>, reward_index: u32, amount: i128) {
    RewardsWithdrawnEvent { farm_id: farm_id.clone(), reward_index, amount }.publish(e);
}

pub fn emit_user_initialized(e: &Env, user: &Address, farm_id: &BytesN<32>) {
    UserInitializedEvent { user: user.clone(), farm_id: farm_id.clone() }.publish(e);
}

pub fn emit_stake(e: &Env, user: &Address, farm_id: &BytesN<32>, amount: i128) {
    StakeEvent { user: user.clone(), farm_id: farm_id.clone(), amount }.publish(e);
}

pub fn emit_unstake(e: &Env, user: &Address, farm_id: &BytesN<32>, amount: i128, penalty: i128) {
    UnstakeEvent { user: user.clone(), farm_id: farm_id.clone(), amount, penalty }.publish(e);
}

pub fn emit_withdraw_unstaked(e: &Env, user: &Address, farm_id: &BytesN<32>, amount: i128) {
    WithdrawUnstakedEvent { user: user.clone(), farm_id: farm_id.clone(), amount }.publish(e);
}

pub fn emit_harvest(
    e: &Env,
    user: &Address,
    farm_id: &BytesN<32>,
    reward_index: u32,
    amount: i128,
    fee: i128,
) {
    HarvestEvent { user: user.clone(), farm_id: farm_id.clone(), reward_index, amount, fee }
        .publish(e);
}

pub fn emit_stake_delegated(
    e: &Env,
    user: &Address,
    farm_id: &BytesN<32>,
    old_stake: i128,
    new_stake: i128,
) {
    StakeDelegatedEvent { user: user.clone(), farm_id: farm_id.clone(), old_stake, new_stake }
        .publish(e);
}

pub fn emit_rewards_accrued(e: &Env, farm_id: &BytesN<32>, reward_index: u32, amount: i128) {
    RewardsAccruedEvent { farm_id: farm_id.clone(), reward_index, amount }.publish(e);
}

pub fn emit_slashed_amount_withdrawn(
    e: &Env,
    farm_id: &BytesN<32>,
    recipient: &Address,
    amount: i128,
) {
    SlashedAmountWithdrawnEvent { farm_id: farm_id.clone(), recipient: recipient.clone(), amount }
        .publish(e);
}

pub fn emit_farm_admin_accepted(e: &Env, farm_id: &BytesN<32>, new_admin: &Address) {
    FarmAdminAcceptedEvent { farm_id: farm_id.clone(), new_admin: new_admin.clone() }.publish(e);
}

pub fn emit_reward_user_once(
    e: &Env,
    user: &Address,
    farm_id: &BytesN<32>,
    reward_index: u32,
    amount: i128,
) {
    RewardUserOnceEvent { user: user.clone(), farm_id: farm_id.clone(), reward_index, amount }
        .publish(e);
}
