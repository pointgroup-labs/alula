use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum LendingContractError {
    InternalError = 0,
    PoolAlreadyExists = 1,
    PoolDoesNotExist = 2,
    InvalidLoanPoolConfig = 3,
    NotEnoughPoolFunds = 4,
    ObligationDoesNotExist = 5,
    DepositDoesNotExist = 6,
    NonPositiveDeposit = 7,
    NonPositiveWithdraw = 8,
    WithdrawOverBalance = 9,
    NonPositiveRepay = 10,
    OverOrUnderflow = 11,
    OracleDoesNotKnowAssetPrice = 12,
    BorrowDoesNotExist = 13,
    HealthFactorIsLowerThanRequiredThreshold = 14,
    InvalidLiquidationThreshold = 15,
    LiquidatedPositionIsHealthy = 16,
    LiquidationExceedsCloseFactor = 17,
    NonPositiveLiquidation = 18,
    NonPositiveBorrow = 19,
    CollateralPoolDoesNotExist = 20,
    NonPositiveFlashLoan = 21,
    InvalidTimestamp = 23,
    SelfLiquidation = 24,
    DepositPoolDoesNotExist = 27,
    BorrowPoolDoesNotExist = 28,
    InvalidLeverageMultiplier = 29,
    InvalidSwapSlippage = 30,
    DependencyContractError = 31,
}
