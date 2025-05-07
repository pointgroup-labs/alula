use soroban_sdk::contracterror;

#[contracterror]
#[repr(u32)]
pub enum LendingContractError {
    PoolAlreadyExists = 1,
    PoolDoesNotExist = 2,
    NonPositiveDeposit = 3,
}
