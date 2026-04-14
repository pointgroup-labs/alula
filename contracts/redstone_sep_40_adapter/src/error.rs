use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
/// RedStone `SEP-40` Adapter Error
pub enum RS40ACError {
    InternalError = 0,
    OverOrUnderflow = 1,
    Unimplemented = 2,
}
