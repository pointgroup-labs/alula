use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)]
/// Market Manager Contract Error
pub enum MMCError {
    // Core errors
    InternalError = 0,
    MarketAlreadyExists = 1,
}
