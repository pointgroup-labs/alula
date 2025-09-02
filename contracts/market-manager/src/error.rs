use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum MarketManagerError {
    // Core errors
    InternalError = 0,
    MarketAlreadyExists = 1,
}
