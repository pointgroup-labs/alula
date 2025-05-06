use {
    crate::{
        error::LendingContractError,
        storage::{self},
    },
    soroban_sdk::{contract, contractimpl, Address, Env, Symbol},
};

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    pub fn __constructor(e: Env, admin: Address) {
        storage::write_admin(&e, &admin);
    }

    pub fn initialize_pool(
        e: Env,
        pool_name: Symbol,
        pool_admin: Address,
        token_address: Address,
        liquidation_threshold: i128,
    ) -> Result<(), LendingContractError> {
        pool_admin.require_auth();

        if storage::pool_exists(&e, token_address.clone(), pool_name.clone()) {
            return Err(LendingContractError::PoolAlreadyExists);
        }
        storage::initialize_pool(
            &e,
            token_address,
            pool_name,
            pool_admin,
            liquidation_threshold,
        );

        Ok(())
    }

    pub fn deposit(
        e: Env,
        user: Address,
        token_address: Address,
        pool_name: Symbol,
        _amount: i128,
    ) -> Result<(), LendingContractError> {
        user.require_auth();

        if !storage::pool_exists(&e, token_address, pool_name) {
            return Err(LendingContractError::PoolDoesNotExist);
        }

        todo!();
    }
}
