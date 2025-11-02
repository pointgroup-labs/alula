use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
/// Market Manager Contract Error
pub enum MMCError {
    // Common Core errors (0-99)
    NegativeInputAmount = 1,

    // Market Manager errors (1000+)
    MarketAlreadyExists = 1000,
    InvalidMaxPositions = 1001,
}
