use soroban_sdk::contracterror;

#[derive(Debug)]
#[contracterror]
pub enum AOCError {
    // Aggregated oracle's errors (1000-1100)
    InvalidMaxAge = 1000,
    InvalidOraclesAmount = 1001,
    AssetAlreadyRegistered = 1002,
    NonUniqueOraclesWhileDeploying = 1003,
    InvalidOracleConfig = 1004,
    ProposedAdminIsNotSet = 1005,
}
