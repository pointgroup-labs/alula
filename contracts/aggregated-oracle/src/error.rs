use soroban_sdk::contracterror;

#[derive(Debug)]
#[contracterror]
pub enum AOCError {
    // Core errors (0-9)
    InvalidMaxAge = 0,
    InvalidOraclesAmount = 2,
    AssetAlreadyRegistered = 3,
    NonUniqueOraclesWhileDeploying = 4,
    InvalidOracleConfig = 5,
}
