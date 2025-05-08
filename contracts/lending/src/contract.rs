use {
    crate::{
        error::LendingContractError,
        storage::{self, GlobalState, Obligation, Pool, PoolAddress},
    },
    soroban_sdk::{contract, contractimpl, token, Address, BytesN, Env},
};

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    pub fn __constructor(e: Env, admin: Address) {
        let global_state = GlobalState {
            admin,
            status: true,
        };
        storage::write_global_state(&e, &global_state);
    }

    pub fn initialize_pool(
        e: Env,
        token_address: Address,
        salt: Option<BytesN<32>>, // @TODO: Check how this looks with CLI
        liquidation_threshold: i128,
    ) -> Result<PoolAddress, LendingContractError> {
        let pool_address: PoolAddress = if let Some(salt) = salt {
            // @TODO: Check some other ways of deriving an address
            e.deployer()
                .with_address(token_address.clone(), salt)
                .deployed_address()
        } else {
            token_address.clone() // @TODO: think of clone()
        };

        if storage::pool_exists(&e, &pool_address) {
            return Err(LendingContractError::PoolAlreadyExists);
        }
        storage::set_pool(&e, &pool_address, &token_address, liquidation_threshold);

        Ok(pool_address)
    }

    pub fn deposit(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LendingContractError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LendingContractError::NonPositiveDeposit);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LendingContractError::PoolDoesNotExist);
        }
        let Pool { token_address, .. } = storage::get_pool_data(&e, &pool_address).unwrap(); // safe unwrap
        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);
        storage::adjust_pool_balance(&e, &pool_address, amount)?;
        storage::set_deposit(&e, &user, &pool_address, amount)?;
        // @TODO: add something with interest and with utilization rate

        Ok(())
    }

    pub fn withdraw(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LendingContractError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LendingContractError::NonPositiveWithdraw);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LendingContractError::PoolDoesNotExist);
        }

        if !storage::deposit_exists(&e, &user, &pool_address)? {
            return Err(LendingContractError::MissingDeposit);
        }

        let Pool {
            token_address,
            balance,
            ..
        } = storage::get_pool_data(&e, &pool_address).unwrap(); // safe `unwrap`

        if amount > balance {
            return Err(LendingContractError::NotEnoughPoolFunds);
        }
        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);
        storage::adjust_pool_balance(&e, &pool_address, -amount)?;
        // something like create_deposit???
        storage::set_deposit(&e, &user, &pool_address, -amount)?;

        Ok(())
    }

    pub fn get_user_obligation(e: Env, user: Address) -> Option<Obligation> {
        storage::get_obligation(&e, &user)
    }
}
