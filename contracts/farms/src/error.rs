use soroban_sdk::contracterror;
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum FCError {
    // Common
    InternalError = 0,
    NegativeInputAmount = 1,
    OverOrUnderflow = 2,

    // Farms
    InvalidRewardScheduleCurve = 10,
    InvalidConfigUpdate = 11,
    InvalidFarmConfigUpdate = 12,
    FarmDoesNotExist = 13,
    FarmingPositionDoesNotExist = 14,
    RewardDoesNotExistOnFarm = 15,
    TokenIsAlreadyAReward = 16,
    FarmIsFrozen = 17,
    DelegatedFarm = 18,
    NotDelegatedFarm = 19,
    MaxFarmNumRewardsReached = 20,
    RewardUserOnceIsDisabled = 21,
    InsufficientStake = 22,
    InsufficientPendingWithdrawal = 23,
    PendingWithdrawalExists = 24,
    DepositCapExceeded = 25,
    WarmupNotComplete = 26,
    CooldownNotComplete = 27,
    ClaimTooSoon = 28,
    InsufficientAvailableRewards = 29,
    NoRewardsToHarvest = 30,
    InsufficientCurrentSlashedAmount = 31,
    InvalidAmount = 32,
    InvalidConfig = 33,
    ProposedFarmAdminDoesNotExist = 34,
}
