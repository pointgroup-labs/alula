use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
/// RedStone `SEP-40` Adapter Error
pub enum RS40ACError {
    InternalError = 0,
    BadConfig = 10,
    Unimplemented = 11,
    NoPendingAdmin = 12,
    FeedForTokenAlreadyInUse = 13,
    TokenForFeedAlreadyInUse = 14,
}
