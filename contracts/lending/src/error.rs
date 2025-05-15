use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug)]
#[repr(u32)]
pub enum LendingContractError {
    PoolAlreadyExists = 1,
    PoolDoesNotExist = 2,
    NonPositiveDeposit = 3,
    NonPositiveWithdraw = 4,
    ObligationDoesNotExist = 5,
    WithdrawOverBalance = 6,
    NotEnoughPoolFunds = 7,
    OverOrUnderflow = 8,
    DepositDoesNotExist = 9,
    InvalidLoanPoolConfig = 10,
    InvalidLiquidationThreshold = 11,
    OracleDoesNotKnowAssetPrice = 12,
    HealthFactorIsLowerThanRequiredThreshold = 13,
}
