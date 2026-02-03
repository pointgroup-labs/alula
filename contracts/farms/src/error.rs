use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug)]
pub enum FCError {
    // Common
    InternalError = 0,
    NegativeInputAmount = 1,
    OverOrUnderflow = 2,

    // Farms
    InvalidConfigUpdate = 10,

    FarmDoesNotExist = 1001,
    ProposedAdminDoesNotExist = 11,
    MaxFarmNumRewardsReached = 12,
    TokenIsAlreadyAReward = 13,
    InvalidRewardScheduleCurve = 14,
    RewardDoesNotExistOnFarm = 15,
    DelegatedFarm = 16,
    NotDelegatedFarm = 19,
    FarmIsFrozen = 17,
    UserDoesNotExist = 18,
    MaxAllowedFarmsReached = 20,
    InvalidTreasuryFeeBps = 21,
    InvalidFarmConfigUpdate = 22,
    RewardIsNotSet = 23,
    ProposedFarmAdminDoesNotExist = 26,
    RewardUserOnceDisabled = 27,
    WarmupNotComplete = 28,
    DepositCapExceeded = 29,

    InsufficientStake = 30,

    PendingWithdrawalExists = 31,
    InsufficientPendingWithdrawal = 32,

    CooldownNotComplete = 33,
    ClaimTooSoon = 34,

    NoRewardsToHarvest = 35,

    InsufficientAvailableRewards = 36,

    InsufficientCurrentSlashedAmount = 37,
    InvalidAmount = 38,
}
