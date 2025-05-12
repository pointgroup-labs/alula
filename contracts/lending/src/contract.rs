use {
    crate::{
        error::LendingContractError,
        oracle,
        storage::{self, GlobalState, Obligation, PoolAddress, PoolConfig, PoolData},
    },
    soroban_sdk::{contract, contractimpl, symbol_short, token, Address, BytesN, Env, String},
};

const REFLECTOR_TESTNET_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

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

    // @TODO: Experiment more with oracle
    // We must be sure that the price is relevant and is updated as frequent as possible
    pub fn test_oracle_price(e: Env) -> i128 {
        let reflector_address =
            Address::from_string(&String::from_str(&e, REFLECTOR_TESTNET_ADDRESS));
        let reflector_contract = oracle::Client::new(&e, &reflector_address);
        let eurc_ticker = symbol_short!("USDC");
        let eurc_asset = oracle::Asset::Other(eurc_ticker);
        let lastprice = reflector_contract.lastprice(&eurc_asset).unwrap();

        lastprice.price
    }

    pub fn initialize_pool(
        e: Env,
        token_address: Address,
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>, // @NB: Can this be more convenient?
    ) -> Result<PoolAddress, LendingContractError> {
        let pool_address: PoolAddress = if let Some(salt) = salt {
            // @TODO: Check some other ways of deriving an address
            e.deployer()
                .with_address(token_address.clone(), salt)
                .deployed_address()
        } else {
            token_address.clone()
        };

        if storage::pool_exists(&e, &pool_address) {
            return Err(LendingContractError::PoolAlreadyExists);
        }
        storage::set_pool(&e, &pool_address, &token_address, pool_config)?;

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
        let PoolData { token_address, .. } =
            storage::get_pool_data(&e, &pool_address).expect("Pool must exist at this point");
        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);
        storage::adjust_pool_supply(&e, &pool_address, amount)?;
        storage::adjust_deposit(&e, &user, &pool_address, amount)?;
        // @TODO: add interest rate accrual

        Ok(())
    }

    // @TODO: pub fn deposit_collateral() {}

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
        let PoolData {
            token_address,
            supply,
            borrowed,
            ..
        } = storage::get_pool_data(&e, &pool_address).expect("Pool must exist at this point");

        if amount > (supply - borrowed) {
            return Err(LendingContractError::NotEnoughPoolFunds);
        }
        storage::adjust_pool_supply(&e, &pool_address, -amount)?;
        storage::adjust_deposit(&e, &user, &pool_address, -amount)?;
        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    // @TODO: pub fn withdraw_collateral() {}

    pub fn get_user_obligation(e: Env, user: Address) -> Option<Obligation> {
        storage::get_obligation(&e, &user)
    }

    pub fn get_pool(e: Env, pool_address: Address) -> Option<PoolData> {
        storage::get_pool_data(&e, &pool_address)
    }
}
