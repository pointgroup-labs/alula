use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
pub enum MCError {
    // Core errors (0-99)
    InternalError = 0,
    NegativeInputAmount = 1,
    DependencyContractError = 2,
    MarketIsNotOwned = 3,
    BorrowForbiddenOnMarket = 4,
    DepositForbiddenOnMarket = 5,
    MarketIsFrozen = 6,
    InvalidMarketUpdate = 7,
    InvalidMarketStatusUpdate = 8,
    IncorrectRequestType = 9,
    OverOrUnderflow = 10,
    TooManyPositions = 11,
    MinCollateralValueIsNotMet = 12,

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
    BorrowForbiddenOnPool = 110,
    DepositForbiddenOnPool = 111,
    PoolIsFrozen = 112,
    InvalidBootstrapPeriod = 113,

    // Obligation-related errors (200-299)
    ObligationDoesNotExist = 200,
    DepositDoesNotExist = 201,
    BorrowDoesNotExist = 202,
    WithdrawScarcityOverLimit = 203,
    ScarcityCooldownPeriod = 204,
    BorrowPositionForAssetExists = 205,
    DepositPositionForAssetExists = 206,

    // Balance and limit errors (400-499)
    PoolSupplyLimitExceeded = 400,
    PoolUtilizationRatioCapExceeded = 401,

    // Oracle-related errors (500-599)
    OracleDoesNotKnowAssetPrice = 500,
    OracleStalePrice = 501,

    // Health factor and liquidation errors (600-699)
    InvalidLiquidationInputs = 600,
    ObligationIsHealthy = 601,
    LiquidationExceedsCloseFactor = 602,
    PositionDoesNotHaveBadDebt = 603,
    AssetCannotBeUsedAsCollateral = 604,
    LiquidationExcessiveDemandedCollateral = 605,

    // Leverage and swap errors (700-799)
    InvalidLeverageMultiplier = 700,
    InvalidSwapSlippage = 701,
    MultiplyPairAlreadyExists = 702,
    MultiplyPairDoesNotExist = 703,
    LeveragePositionContainsBadDebt = 704,
    InconsistentDepositWithLeverage = 705,
}
