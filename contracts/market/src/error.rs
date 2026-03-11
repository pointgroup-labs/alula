use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
pub enum MCError {
    // Core errors (0-99)
    InternalError = 0,
    InvalidInputAmount = 1,
    DependencyContractError = 2,
    MarketIsNotOwned = 3,
    BorrowForbiddenOnMarket = 4,
    DepositForbiddenOnMarket = 5,
    MarketIsFrozen = 6,
    InvalidMarketConfigOrUpdate = 7,
    IncorrectRequestType = 8,
    OverOrUnderflow = 9,
    TooManyPositions = 10,
    MinCollateralValueIsNotMet = 11,
    NonPositiveSharesAmount = 12,

    // Pool-related errors (100-199)
    InvalidInitialization = 100,
    PoolDoesNotExist = 101,
    InvalidLoanPoolConfig = 102,
    NotEnoughPoolFunds = 103,
    DepositPoolDoesNotExist = 104,
    BorrowPoolDoesNotExist = 105,
    CollateralPoolDoesNotExist = 106,
    PoolAlreadyContainsQueuedPoolSet = 107,
    PoolDoesNotHaveQueuedPoolSet = 108,
    PoolSetIsNotYetApplicable = 109,
    OperationForbiddenOnPool = 110,
    MarketAlreadyContainsQueuedInConfigUpdate = 111,
    MarketDoesNotHaveQueuedInConfigUpdate = 112,
    MarketConfigUpdateIsNotYetApplicable = 113,

    // Obligation-related errors (200-299)
    ObligationDoesNotExist = 200,
    DepositPositionDoesNotExist = 201,
    BorrowPositionDoesNotExist = 202,
    WithdrawScarcityOverLimit = 203,
    ScarcityCooldownPeriod = 204,
    BorrowPositionForAssetExists = 205,
    DepositPositionForAssetExists = 206,
    UnhealthyOperation = 207,

    // Balance and limit errors (400-499)
    PoolSupplyLimitExceeded = 400,
    PoolUtilizationRatioCapExceeded = 401,

    // Oracle-related errors (500-599)
    OracleDoesNotKnowAssetPrice = 500,
    OracleStalePrice = 501,
    NonPositiveOraclePrice = 502,

    // Health factor and liquidation errors (600-699)
    InvalidLiquidationInputs = 600,
    ObligationIsHealthy = 601,
    ObligationContainsOpenCoverBadDebtRequests = 602,
    BadDebtCoverageCriterionIsNotMet = 603,
    AssetCannotBeUsedAsCollateral = 604,
    LiquidationExcessiveDemandedCollateral = 605,

    // Requests batching & swap errors (700-799)
    InvalidSwap = 701,
    FlashBorrowAlreadyRegistered = 702,
    SwapSlippageExceeded = 703,
}
