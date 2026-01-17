use soroban_sdk::contracterror;

#[contracterror]
pub enum PSCError {
    InternalError = 0,
    UnregisteredProviderAddress = 1,
}
