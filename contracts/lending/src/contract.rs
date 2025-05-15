use crate::constants::{BPS_IN_PERCENT, DEFAULT_LIQUIDATION_THRESHOLD, REFLECTOR_TESTNET_ADDRESS};
use {
    crate::{
        error::LendingContractError,
        interest_rate::InterestRates,
        oracle,
        storage::{self, GlobalState, Obligation, Pool, PoolAddress, PoolConfig},
    },
    soroban_sdk::{
        contract, contractimpl, symbol_short, token, Address, BytesN, Env, String, Symbol,
    },
};

#[contract]
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    pub fn __constructor(
        e: Env,
        admin: Address,
        liquidation_threshold: Option<i128>,
    ) -> Result<(), LendingContractError> {
        let liquidation_threshold = if let Some(lt) = liquidation_threshold {
            if lt <= 0 || lt > 100 {
                return Err(LendingContractError::InvalidLiquidationThreshold);
            }
            lt
        } else {
            DEFAULT_LIQUIDATION_THRESHOLD
        };
        let liquidation_threshold_bps = liquidation_threshold * BPS_IN_PERCENT;
        let global_state = GlobalState {
            admin,
            status: true,
            liquidation_threshold_bps,
        };
        storage::write_global_state(&e, &global_state);

        Ok(())
    }

    // TODO: Experiment more with oracle
    // We must be sure that the price is relevant and is updated as frequent as possible
    pub fn test_oracle_price(e: Env) -> i128 {
        let reflector_address =
            Address::from_string(&String::from_str(&e, REFLECTOR_TESTNET_ADDRESS));
        let _reflector_contract = oracle::Client::new(&e, &reflector_address);
        let eurc_ticker = symbol_short!("EURC");
        let _eurc_asset = oracle::Asset::Other(eurc_ticker);
        // let all_assets = reflector_contract.assets();

        todo!()
    }

    pub fn initialize_pool(
        e: Env,
        token_address: Address,
        token_ticker: Symbol, // NB: Token Interface contains a `.symbol()` endpoint, which can be used for retrieving a token's ticker
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>,
    ) -> Result<PoolAddress, LendingContractError> {
        let pool_address: PoolAddress = if let Some(salt) = salt {
            // TODO: Check some other ways of deriving an address
            e.deployer()
                .with_address(token_address.clone(), salt)
                .deployed_address()
        } else {
            token_address.clone()
        };

        if storage::pool_exists(&e, &pool_address) {
            return Err(LendingContractError::PoolAlreadyExists);
        }

        let pool_config = if let Some(config) = pool_config {
            if !config.is_valid() {
                return Err(LendingContractError::InvalidLoanPoolConfig);
            }

            config
        } else {
            Default::default()
        };
        storage::set_pool(
            &e,
            &pool_address,
            &token_address,
            &token_ticker,
            pool_config,
        )?;

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

        let Pool { token_address, .. } =
            storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        storage::adjust_pool_supply(&e, &pool_address, amount)?;
        storage::adjust_deposit(&e, &user, &pool_address, amount)?;
        // TODO: add interest rate accrual

        Ok(())
    }

    pub fn borrow(
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

        // TODO: Rename this, since misleading..
        storage::adjust_borrow(&e, &user, &pool_address, amount)?;
        storage::adjust_pool_borrowed(&e, &pool_address, amount)?;

        const HEALTH_FACTOR_THRESHOLD: i128 = 100 * BPS_IN_PERCENT;

        let health_factor: i128 = Self::compute_health_factor(&e, &user)?;
        if health_factor < HEALTH_FACTOR_THRESHOLD {
            return Err(LendingContractError::HealthFactorIsLowerThanRequiredThreshold);
        }

        let Pool { token_address, .. } =
            storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    fn compute_health_factor(e: &Env, user: &Address) -> Result<i128, LendingContractError> {
        let Obligation {
            deposits, borrows, ..
        } = storage::get_obligation(e, user).ok_or(LendingContractError::ObligationDoesNotExist)?;
        let GlobalState {
            liquidation_threshold_bps,
            ..
        } = storage::read_global_state(e);
        // HF = ((Collateral_Value1 + ... + Collateral_ValueN) * LT) / (Borrow_Value1 + ... + BorrowValueM)
        let reflector_address =
            Address::from_string(&String::from_str(e, REFLECTOR_TESTNET_ADDRESS));
        let reflector_contract = oracle::Client::new(e, &reflector_address);
        let (mut collateral_sum_value, mut borrow_sum_value) = (0i128, 0i128);

        for (pool_address, amount) in deposits {
            // TODO: Maybe, get it from the token client?
            // This will introduce an additional cross contract call, but will decrease the amount of errors,
            // which can happen.
            let ticker =
                storage::get_pool_ticker(e, &pool_address).expect("Pool must exist at this point");
            let asset = oracle::Asset::Other(ticker); // TODO: What about XLM?
            let lastprice = reflector_contract
                .lastprice(&asset)
                .ok_or(LendingContractError::OracleDoesNotKnowAssetPrice)?;
            collateral_sum_value = collateral_sum_value
                .checked_add(
                    lastprice
                        .price
                        .checked_mul(amount)
                        .ok_or(LendingContractError::OverOrUnderflow)?,
                )
                .ok_or(LendingContractError::OverOrUnderflow)?;
        }

        for (pool_address, amount) in borrows {
            let ticker =
                storage::get_pool_ticker(e, &pool_address).expect("Pool must exist at this point");
            let asset = oracle::Asset::Other(ticker);
            let lastprice = reflector_contract
                .lastprice(&asset)
                .ok_or(LendingContractError::OracleDoesNotKnowAssetPrice)?;
            borrow_sum_value = borrow_sum_value
                .checked_add(
                    lastprice
                        .price
                        .checked_mul(amount)
                        .ok_or(LendingContractError::OverOrUnderflow)?,
                )
                .ok_or(LendingContractError::OverOrUnderflow)?;
        }

        let numerator = collateral_sum_value
            .checked_mul(liquidation_threshold_bps)
            .ok_or(LendingContractError::OverOrUnderflow)?;
        let health_factor = numerator
            .checked_div(borrow_sum_value)
            .ok_or(LendingContractError::OverOrUnderflow)?;

        Ok(health_factor)
    }

    // TODO: Would also be good to separate `health_factor` into a separate method
    // and write unit test for it..

    #[allow(unused)]
    pub fn accrue_interest(e: Env, pool_address: Address) -> Result<(), LendingContractError> {
        // How should this look?
        todo!()
    }

    #[allow(unused)]
    pub fn repay(e: Env, pool_address: Address) -> Result<(), LendingContractError> {
        // How should this look?
        todo!()
    }

    #[allow(unused)]
    pub fn deposit_collateral() {
        todo!()
    }

    #[allow(unused)]
    pub fn withdraw_collateral() {
        todo!()
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
            return Err(LendingContractError::DepositDoesNotExist);
        }

        let Pool {
            token_address,
            supply,
            borrowed,
            ..
        } = storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        if amount > (supply - borrowed) {
            return Err(LendingContractError::NotEnoughPoolFunds);
        }

        storage::adjust_pool_supply(&e, &pool_address, -amount)?;
        storage::adjust_deposit(&e, &user, &pool_address, -amount)?;

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    pub fn get_user_obligation(e: Env, user: Address) -> Option<Obligation> {
        storage::get_obligation(&e, &user)
    }

    pub fn get_pool(e: Env, pool_address: Address) -> Option<Pool> {
        storage::get_pool(&e, &pool_address)
    }

    pub fn get_interest_rates(
        e: Env,
        pool_address: Address,
    ) -> Result<InterestRates, LendingContractError> {
        let pool =
            storage::get_pool(&e, &pool_address).ok_or(LendingContractError::PoolDoesNotExist)?;

        pool.get_interest_rates()
    }
}
