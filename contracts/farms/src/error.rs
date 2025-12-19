use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum FarmsError {
    // Initialization Errors (1-9)
    /// Contract already initialized
    AlreadyInitialized = 1,
    /// Contract not initialized
    NotInitialized = 2,
    /// No pending admin to accept
    NoPendingAdmin = 3,

    // Entity Not Found Errors (10-19)
    /// Farm does not exist
    FarmNotFound = 10,
    /// User state does not exist
    UserNotFound = 11,
    /// Reward token not found
    RewardNotFound = 12,

    // Entity Already Exists Errors (20-29)
    /// User already initialized for this farm
    UserAlreadyExists = 20,
    /// Reward token already exists
    RewardTokenAlreadyExists = 21,

    // Farm State Errors (30-39)
    /// Farm is frozen
    FarmFrozen = 30,
    /// Farm is not frozen (when trying to unfreeze)
    FarmNotFrozen = 31,
    /// Farm has a delegate authority set (direct stake/unstake not allowed)
    FarmIsDelegated = 32,
    /// Caller is not the delegate authority
    NotDelegateAuthority = 33,
    /// Maximum reward tokens reached
    MaxRewardTokensReached = 34,
    /// reward_user_once feature is disabled for this farm
    RewardUserOnceDisabled = 35,

    // Stake/Unstake Errors (40-49)
    /// Insufficient stake balance
    InsufficientStake = 40,
    /// Insufficient pending withdrawal
    InsufficientPendingWithdrawal = 41,
    /// Pending withdrawal exists (must claim before new unstake)
    PendingWithdrawalExists = 42,
    /// Deposit cap exceeded
    DepositCapExceeded = 43,

    // Timing Errors (50-59)
    /// Warmup period not complete
    WarmupNotComplete = 50,
    /// Cooldown period not complete
    CooldownNotComplete = 51,
    /// Claim too soon (min_claim_duration not elapsed)
    ClaimTooSoon = 52,

    // Reward Errors (60-69)
    /// Insufficient rewards available
    InsufficientRewards = 60,
    /// No rewards to harvest
    NoRewardsToHarvest = 61,
    /// Insufficient slashed amount available
    InsufficientSlashedAmount = 62,

    // Validation Errors (70-79)
    /// Invalid amount (zero or negative)
    InvalidAmount = 70,
    /// Invalid configuration parameter
    InvalidConfig = 71,
    /// Invalid reward schedule (curve points)
    InvalidRewardSchedule = 72,

    // Math Errors (80-89)
    /// Arithmetic overflow
    Overflow = 80,
    /// Arithmetic underflow
    Underflow = 81,
    /// Division by zero
    DivisionByZero = 82,

    // Misc
    /// Internal error
    InternalError = 100,
}
