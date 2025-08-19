use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AggregatedOracleContractError {
    // Core errors (0-9)
    InternalError = 0,
}
