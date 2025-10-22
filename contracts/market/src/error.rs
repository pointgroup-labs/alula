use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MCError {
    // Core errors (0-9)
    InternalError = 0,
    OverOrUnderflow = 1,
    InvalidTimestamp = 2,
    MarketIsNotOwned = 3,
    DependencyContractError = 4,
    ForbiddenMarketOperation = 5,
    InvalidMarketUpdate = 6,
    ForbiddenPoolOperation = 7,

    // Pool-related errors (10-19)
    PoolAlreadyExists = 10,
    PoolDoesNotExist = 11,
    InvalidLoanPoolConfig = 12,
    NotEnoughPoolFunds = 13,
    DepositPoolDoesNotExist = 14,
    BorrowPoolDoesNotExist = 15,
    CollateralPoolDoesNotExist = 16,
    PoolAlreadyContainsEnqueuedConfigUpdate = 17,
    PoolDoesNotHaveQueuedInConfigUpdate = 18,
    PoolConfigUpdateIsNotSeasonedYet = 19,

    // Obligation-related errors (20-29)
    ObligationDoesNotExist = 20,
    DepositDoesNotExist = 21,
    BorrowDoesNotExist = 22,
    WithdrawScarcityOverLimit = 23, // TODO: Pool errors?
    ScarcityCooldownPeriod = 24,

    // Amount validation errors (30-39)
    NegativeAmount = 30,

    // Balance and limit errors (40-49)
    WithdrawOverBalance = 40,
    PoolSupplyLimitExceeded = 41,
    PoolUtilizationRatioCapExceeded = 42, /* TODO: Must be `Not Enough Available Funds` for Withdraw */
    CollateralRemovalOverbalance = 43,

    // Oracle-related errors (50-59)
    OracleDoesNotKnowAssetPrice = 50,
    OracleStalePrice = 51,

    // Health factor and liquidation errors (60-69)
    HealthFactorIsLowerThanRequiredThreshold = 60,
    InvalidLiquidationThreshold = 61,
    LiquidatedPositionIsHealthy = 62,
    LiquidationExceedsCloseFactor = 63,
    SelfLiquidation = 64,
    LiquidationWithEqualCollateralAndDepositPools = 65,
    PositionDoesNotHaveBadDebt = 66,
    BadDebtPosition = 67,

    // Leverage and swap errors (70-79)
    InvalidLeverageMultiplier = 70,
    InvalidSwapSlippage = 71,
    MultiplyPairAlreadyExists = 72,
    MultiplyPairDoesNotExist = 73,
}
