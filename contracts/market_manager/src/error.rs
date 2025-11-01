use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
/// Market Manager Contract Error
pub enum MMCError {
    // Core errors
    InternalError = 0,
    MarketAlreadyExists = 1,
}
