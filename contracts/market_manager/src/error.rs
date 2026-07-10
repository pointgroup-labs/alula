use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
/// Market Manager Contract Error
pub enum MMCError {
    // Common Core errors (0-99)
    InvalidInputAmount = 1,
    OverOrUnderflow = 9,

    // Market Manager errors (1000+)
    MarketAlreadyExists = 1000,
    UpgradeAlreadyExists = 1001,
    UpgradeDoesNotExist = 1002,
    UpgradeIsNotYetApplicable = 1003,
    NoPendingAdmin = 1004,
    MarketNotDeployedByManager = 1005,
    BadUpgradeInQueuePeriod = 1006,
}
