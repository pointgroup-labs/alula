use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum AggregatedOracleContractError {
    // Core errors (0-9)
    InternalError = 0,
    NotAnAdmin = 1,
    OverOrUnderflow = 2,
    Unimplemented = 3,
    UnknownAsset = 43,
}

pub type AOCError = AggregatedOracleContractError;
