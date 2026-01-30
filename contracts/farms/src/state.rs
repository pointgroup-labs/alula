// Re-export Delegatee from farms_interface to ensure type compatibility
pub use farms_interface::Delegatee;
use soroban_sdk::{Address, BytesN, Vec, contracttype};

/// Global configuration for the Farms contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GlobalConfig {
    /// Administrator address
    pub admin: Address,
    /// Treasury vault authority for fee collection
    pub treasury_vault: Address,
    /// Fee taken from rewards in basis points
    pub treasury_fee_bps: i128,
    /// Pending admin for two-step admin transfer
    pub pending_admin: Option<Address>,
}

/// Time unit for reward calculations
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[repr(u32)]
pub enum TimeUnit {
    /// Use ledger timestamps (seconds)
    #[default]
    Seconds = 0,
    /// Use ledger sequence numbers
    Slot = 1,
}

/// Locking mode for farm stakes
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[repr(u32)]
pub enum LockingMode {
    /// No locking - users can unstake freely
    #[default]
    None = 0,
    /// Continuous locking - lock duration restarts from user's last stake
    Continuous = 1,
    /// Global expiry - all stakes unlock at a fixed timestamp
    WithExpiry = 2,
    // TODO: Some other?
}

/// Reward distribution type
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, Default)]
#[repr(u32)]
pub enum RewardType {
    /// Proportional - rewards distributed proportionally to stake (default)
    /// reward = (user_stake / total_staked) × rewards_issued
    #[default]
    Proportional = 0,
    // Yup, but what to do when somebody 'add_rewards's?
    /// Constant - same reward amount per user regardless of stake
    /// reward = rewards_per_second × total_staked (multiplied by user count)
    /// Useful for participation-based incentives
    Constant = 1,
    // So, we are about to accrue a 'per second' rewards

    // This has few issues when talking about the fixed point arithmetic, so must sure that BTC and FOGO both
    // make sense in this scenario
}

/// A point on the reward emission curve
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardCurvePoint {
    /// Timestamp when this rate starts
    pub ts_start: u64,
    /// Reward amount per time unit
    pub reward_per_time_unit: i128,
}

/// Reward emission schedule defined as a curve
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardScheduleCurve {
    /// Points defining the curve (up to MAX_CURVE_POINTS)
    pub points: Vec<RewardCurvePoint>,
}

/// Information about a single reward token for a farm
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RewardInfo {
    /// Reward token address
    pub token: Address,
    /// Vault holding reward tokens
    pub rewards_vault: Address,
    /// Remaining rewards available for distribution
    pub rewards_available: i128,

    /// Reward distribution type (Proportional or Constant)
    pub reward_type: RewardType,
    /// Emission schedule
    pub reward_schedule: RewardScheduleCurve,
    /// Last timestamp when rewards were issued
    pub last_issuance_ts: u64,

    /// Accumulated reward per share (scaled by SCALE_FACTOR)
    pub reward_per_share_scaled: i128,
    /// Total rewards issued but not yet claimed
    pub rewards_issued_unclaimed: i128,
    /// Total rewards issued cumulatively
    pub rewards_issued_cumulative: i128,

    /// Minimum duration between claims (to prevent spam)
    pub min_claim_duration: u64,
}

/// Farm state
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FarmState {
    /// Unique farm identifier
    pub farm_id: BytesN<32>,

    /// Farm-specific admin (if None, global admin controls this farm)
    /// Allows delegation of farm management to a separate address
    pub farm_admin: Option<Address>,
    /// Pending farm admin for two-step transfer
    pub pending_farm_admin: Option<Address>,

    /// Delegate authority - contract authorized to update stakes via set_stake_delegated
    /// When Some: only this address can update stakes (push model from Market/AMM)
    /// When None: users can call stake()/unstake() directly (pull model)
    pub delegate_authority: Option<Address>,

    /// Total staked amount across all users
    pub total_staked: i128,
    /// Number of users with active stakes
    pub num_users: u64,

    /// Time unit for calculations
    pub time_unit: TimeUnit,
    /// Delay before new stakes become active (warmup)
    pub deposit_warmup_period: u64, // This is also the case for the delegated stake, right?
    /// Delay after unstake before withdrawal (cooldown)
    pub withdrawal_cooldown_period: u64,

    /// Locking mode
    pub locking_mode: LockingMode,
    /// When locking started (for WithExpiry mode)
    pub locking_start_ts: u64,
    /// Lock duration
    pub locking_duration: u64,
    /// Penalty for early withdrawal in basis points (max penalty, decays linearly)
    pub early_withdrawal_penalty_bps: i128,

    /// Maximum total stake allowed (0 = unlimited)
    pub deposit_cap: i128,

    /// Reward configurations (up to MAX_REWARD_TOKENS)
    pub reward_infos: Vec<RewardInfo>,
    /// Number of active reward tokens
    pub num_reward_tokens: u32,

    /// Whether the farm is frozen
    pub is_frozen: bool,

    /// Whether reward_user_once is enabled (for airdrops via delegate)
    pub is_reward_user_once_enabled: bool,

    /// Current slashed amount from early withdrawals (available for admin to withdraw)
    pub slashed_amount_current: i128,
    /// Cumulative slashed amount (for tracking purposes)
    pub slashed_amount_cumulative: i128, // Shouldn't this be per reserve?
    /// Address to receive slashed amounts
    pub slashed_amount_spill_address: Address,
}

/// User state for a specific farm
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserState {
    /// The owner's address (for rewards and events)
    pub owner: Address, // Owner?
    /// Farm this state belongs to
    pub farm_id: BytesN<32>, // By the way, how do we represent this relation?

    /// Active stake currently earning rewards
    pub active_stake: i128,

    /// Stake in warmup period (not yet active)
    pub pending_deposit_stake: i128, // Who will refresh this?
    /// When pending deposit was initiated
    pub pending_deposit_ts: u64,

    /// Stake in cooldown period (unstaked, waiting for withdrawal)
    pub pending_withdrawal_stake: i128,
    /// When pending withdrawal was initiated
    pub pending_withdrawal_ts: u64,

    /// Rewards tally per reward token (for RPS calculation)
    /// user_reward = (reward_per_share * stake) - rewards_tally
    pub rewards_tally_scaled: Vec<i128>,
    /// Unclaimed rewards per reward token
    pub rewards_unclaimed: Vec<i128>,
    /// Last claim timestamp per reward token
    pub last_claim_ts: Vec<u64>,

    /// Timestamp of user's last stake (for continuous locking)
    pub last_stake_ts: u64,
}

/// Configuration for initializing a new farm
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FarmConfig {
    /// Delegate authority address (optional)
    /// When Some: only this address can update stakes via set_stake_delegated (push model)
    /// When None: users can call stake()/unstake() directly
    pub delegate_authority: Option<Address>,
    pub time_unit: TimeUnit,
    pub deposit_warmup_period: u64,
    pub withdrawal_cooldown_period: u64,
    pub locking_mode: LockingMode,
    pub locking_start_ts: u64, // Should this coincide with the first
    pub locking_duration: u64, // and the last point on the curve?
    pub early_withdrawal_penalty_bps: i128, // Also, I don't think this must apply to the delegated stake scenario
    pub deposit_cap: i128,
}

impl Default for FarmConfig {
    fn default() -> Self {
        Self {
            delegate_authority: None,
            time_unit: TimeUnit::Seconds,
            deposit_warmup_period: 0,
            withdrawal_cooldown_period: 0,
            locking_mode: LockingMode::None,
            locking_start_ts: 0,
            locking_duration: 0,
            early_withdrawal_penalty_bps: 0,
            deposit_cap: 0,
        }
    }
}

/// Global configuration update - each variant carries its typed value
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GlobalConfigUpdate {
    /// Update the treasury vault address
    TreasuryVault(Address),
    /// Update the treasury fee in basis points (max 1000 = 10%)
    TreasuryFeeBps(i128),
}

/// Farm configuration update - each variant carries its typed value
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FarmConfigUpdate {
    /// Update the deposit warmup period (seconds or slots)
    DepositWarmupPeriod(u64),
    /// Update the withdrawal cooldown period (seconds or slots)
    WithdrawalCooldownPeriod(u64),
    /// Update the locking mode
    LockingMode(LockingMode),
    /// Update the locking start timestamp (for WithExpiry mode)
    LockingStartTs(u64),
    /// Update the lock duration
    LockingDuration(u64),
    /// Update the early withdrawal penalty in basis points (max 10000 = 100%)
    EarlyWithdrawalPenalty(i128),
    /// Update the deposit cap (0 = unlimited)
    DepositCap(i128),
    /// Update the minimum claim duration for all reward tokens
    MinClaimDuration(u64),
    /// Update or clear the delegate authority (None = enable direct staking)
    DelegateAuthority(Option<Address>),
    /// Update the slashed amount spill address
    SlashedAmountSpillAddress(Address),
    /// Set pending farm admin (for two-step transfer)
    PendingFarmAdmin(Address),
    /// Enable/disable reward_user_once feature (requires delegated farm)
    RewardUserOnceEnabled(bool),
    /// Update reward type for a specific reward token (reward_index, RewardType)
    RewardType(u32, RewardType),
}
