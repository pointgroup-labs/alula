use soroban_sdk::contracterror;

// TODO: Standardize/stabilize error codes(check how others do it)

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MCError {
    // Core errors (0-99)
    InternalError = 0,
    OverOrUnderflow = 1,
    InvalidTimestamp = 2,
    DependencyContractError = 3,
    MarketIsNotOwned = 4,
    ForbiddenMarketOperation = 5,
    InvalidMarketUpdate = 6,

    // Pool-related errors (100-199)
    PoolAlreadyExists = 100,
    PoolDoesNotExist = 101,
    InvalidLoanPoolConfig = 102,
    NotEnoughPoolFunds = 103,
    DepositPoolDoesNotExist = 104,
    BorrowPoolDoesNotExist = 105,
    CollateralPoolDoesNotExist = 106,
    PoolAlreadyContainsQueuedInConfigUpdate = 107,
    PoolDoesNotHaveQueuedInConfigUpdate = 108,
    PoolConfigUpdateIsNotYetApplicable = 109,
    ForbiddenPoolOperation = 110,

    // Obligation-related errors (200-299)
    ObligationDoesNotExist = 200,
    DepositDoesNotExist = 201,
    CollateralDoesNotExist = 202,
    BorrowDoesNotExist = 203,
    WithdrawScarcityOverLimit = 205,
    ScarcityCooldownPeriod = 206,

    // Amount validation errors (300-399)
    NegativeAmount = 300,

    // Balance and limit errors (400-499)
    WithdrawOverBalance = 400,
    PoolSupplyLimitExceeded = 401,
    PoolUtilizationRatioCapExceeded = 402,
    CollateralRemovalOverbalance = 403,

    // Oracle-related errors (500-599)
    OracleDoesNotKnowAssetPrice = 500,
    OracleStalePrice = 501,

    // Health factor and liquidation errors (600-699)
    HealthFactorIsLowerThanRequiredThreshold = 600,
    InvalidLiquidationThreshold = 601,
    LiquidatedPositionIsHealthy = 602,
    LiquidationExceedsCloseFactor = 603,
    SelfLiquidation = 604,
    LiquidationWithEqualCollateralAndDepositPools = 605,
    PositionDoesNotHaveBadDebt = 606,
    BadDebtPosition = 607,

    // Leverage and swap errors (700-799)
    InvalidLeverageMultiplier = 700,
    InvalidSwapSlippage = 701,
    MultiplyPairAlreadyExists = 702,
    MultiplyPairDoesNotExist = 703,
}
