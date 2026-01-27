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
    DelegatedAuthorityIsNotSetForFarm = 16,
    FarmIsFrozen = 17,
}
