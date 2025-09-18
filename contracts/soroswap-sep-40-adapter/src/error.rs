use soroban_sdk::contracterror;

#[contracterror]
#[derive(Debug, Eq, PartialEq)]
#[repr(u32)] // NB: Is this really required w #[contracterror]?
/// Soroswap `SEP-40` Adapter Error
pub enum SS40ACError {
    // Core errors (0-9)
    InternalError = 0,
    OverOrUnderflow = 1,
    Unimplemented = 2,
    AssetAlreadyRegistered = 3,
}
