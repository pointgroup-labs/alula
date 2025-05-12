use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug)]
#[repr(u32)]
pub enum LendingContractError {
    PoolAlreadyExists = 1,
    PoolDoesNotExist = 2,
    NonPositiveDeposit = 3,
    NonPositiveWithdraw = 4,
    MisslingObligation = 5,
    WithdrawOverBalance = 6,
    NotEnoughPoolFunds = 7,
    OverOrUnderflow = 8,
    MissingDeposit = 9,
    InvalidLoanPoolConfig = 10,
    InconsistentPoolState = 11,
}
