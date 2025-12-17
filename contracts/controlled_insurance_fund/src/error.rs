use soroban_sdk::contracterror;

#[contracterror]
pub enum ContractError {
    InternalError = 0,
    RequestDoesNotExist = 1,
    RequestIsNotReady = 2,
    MustClaimCoverage = 3,
    InsufficientContractBalance = 4,
}
