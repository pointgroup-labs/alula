use soroban_sdk::contracterror;

#[contracterror]
pub enum FCError {
    // Common
    InternalError = 0,
    NegativeInputAmount = 1,
    // Farms
    FarmDoesNotExist = 10,
    ProposedAdminDoesNotExist = 11,
    MaxNumRewardsReached = 12,
    TokenIsAlreadyAReward = 13,
    InvalidRewardScheduleCurve = 14,
    RewardDoesNotExistOnFarm = 15,
    DelegatedFarm = 16,
    NotDelegatedFarm = 19,
    FarmIsFrozen = 17,
    UserDoesNotExist = 18,
    InvalidTreasuryFeeBps = 19,
    MaxAllowedFarmsReached = 20,
}
