use soroban_sdk::contracterror;

#[derive(Debug)]
#[contracterror]
pub enum AOCError {
    // Core errors (0-9)
    InternalError = 0,
    InvalidMaxAge = 1,
    OverOrUnderflow = 2,
    InvalidOraclesAmount = 3,
    AssetAlreadyRegistered = 4,
    OracleAlreadyRegistered = 5,
    NonUniqueOraclesRegistered = 6,
    InvalidOracleConfig = 7,
}
