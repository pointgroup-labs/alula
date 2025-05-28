use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug)]
#[repr(u32)]
pub enum LendingContractError {
    PoolAlreadyExists = 1,
    PoolDoesNotExist = 2,
    NonPositiveDeposit = 3,
    NonPositiveWithdraw = 4,
    NonPositiveRepay = 5,
    ObligationDoesNotExist = 6,
    WithdrawOverBalance = 7,
    NotEnoughPoolFunds = 8,
    OverOrUnderflow = 9,
    DepositDoesNotExist = 10,
    InvalidLoanPoolConfig = 11,
    InvalidLiquidationThreshold = 12,
    OracleDoesNotKnowAssetPrice = 13,
    HealthFactorIsLowerThanRequiredThreshold = 14,
    BorrowPositionDoesNotExistForUserInPool = 15,
    LiquidatedPositionIsHealthy = 16,
    LiquidationExceedsCloseFactor = 17,
}
