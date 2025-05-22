use {
    crate::{
        constants::{
            LCError, ACCRUAL_INIT_VALUE, BPS_IN_PERCENT, DEFAULT_LIQUIDATION_THRESHOLD,
            REFLECTOR_TESTNET_ADDRESS,
        },
        interest_rate::CompoundRates,
        oracle,
        storage::{
            self, Accrual, GlobalState, Obligation, ObligationPosition, Pool, PoolAddress,
            PoolConfig,
        },
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
    ) -> Result<(), LCError> {
        let liquidation_threshold = if let Some(lt) = liquidation_threshold {
            if lt <= 0 || lt > 100 {
                return Err(LCError::InvalidLiquidationThreshold);
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

        storage::set_global_state(&e, &global_state);
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

        todo!()
    }

    pub fn initialize_pool(
        e: Env,
        token_address: Address,
        token_ticker: Symbol, // NB: Token Interface contains a `.symbol()` endpoint, which can be used for retrieving a token's ticker
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>,
    ) -> Result<PoolAddress, LCError> {
        let pool_address: PoolAddress = if let Some(salt) = salt {
            // TODO: Check some other ways of deriving an address
            e.deployer()
                .with_address(token_address.clone(), salt)
                .deployed_address()
        } else {
            token_address.clone()
        };

        if storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolAlreadyExists);
        }

        let config = if let Some(config) = pool_config {
            if !config.is_valid() {
                return Err(LCError::InvalidLoanPoolConfig);
            }

            config
        } else {
            Default::default()
        };

        // Accrual increase depends on the interest rate, so it must be stored and updated separately for each loan pool
        let accrual = Accrual {
            timestamp: e.ledger().timestamp(),
            borrow_accrual: ACCRUAL_INIT_VALUE,
            supply_accrual: ACCRUAL_INIT_VALUE,
        };

        let pool = Pool {
            token_address,
            token_ticker,
            borrowed: 0,
            supply: 0,
            config,
            accrual,
        };

        storage::set_pool(&e, &pool_address, &pool)?;

        Ok(pool_address)
    }

    pub fn deposit(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveDeposit);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        let Pool { token_address, .. } =
            storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");
        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&user, &e.current_contract_address(), &amount);

        storage::set_pool_supply(&e, &pool_address, amount)?;
        storage::set_obligation_deposit(&e, &user, &pool_address, amount)?;
        // TODO: add interest rate accrual

        Ok(())
    }

    pub fn borrow(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveDeposit);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        let Pool {
            token_address,
            borrowed,
            supply,
            ..
        } = storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        if amount >= (supply - borrowed) {
            return Err(LCError::NotEnoughPoolFunds);
        }

        // TODO: Rename this, since misleading..
        storage::set_obligation_borrow(&e, &user, &pool_address, amount)?;
        storage::set_pool_borrowed(&e, &pool_address, amount)?;

        const HEALTH_FACTOR_THRESHOLD: i128 = 100 * BPS_IN_PERCENT;

        let health_factor: i128 = Self::compute_health_factor(&e, &user)?;
        if health_factor < HEALTH_FACTOR_THRESHOLD {
            return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
        }

        let token_client = token::Client::new(&e, &token_address);
        token_client.transfer(&e.current_contract_address(), &user, &amount);

        Ok(())
    }

    fn compute_health_factor(e: &Env, user: &Address) -> Result<i128, LCError> {
        let Obligation {
            deposits, borrows, ..
        } = storage::get_obligation(e, user).ok_or(LCError::ObligationDoesNotExist)?;
        let GlobalState {
            liquidation_threshold_bps,
            ..
        } = storage::get_global_state(e);
        // HF = ((Collateral_Value1 + ... + Collateral_ValueN) * LT) / (Borrow_Value1 + ... + BorrowValueM)
        let reflector_address =
            Address::from_string(&String::from_str(e, REFLECTOR_TESTNET_ADDRESS));
        let reflector_contract = oracle::Client::new(e, &reflector_address);
        let (mut collateral_sum_value, mut borrow_sum_value) = (0i128, 0i128);

        for (pool_address, deposit_position) in deposits {
            let ObligationPosition { amount, .. } = deposit_position;

            // TODO: Maybe, get it from the token client?
            // This will introduce an additional cross contract call, but will decrease the amount of errors,
            // which can happen.
            let ticker =
                storage::get_pool_ticker(e, &pool_address).expect("Pool must exist at this point");
            let asset = oracle::Asset::Other(ticker); // TODO: What about XLM?
            let lastprice = reflector_contract
                .lastprice(&asset)
                .ok_or(LCError::OracleDoesNotKnowAssetPrice)?;
            collateral_sum_value = collateral_sum_value
                .checked_add(
                    lastprice
                        .price
                        .checked_mul(amount)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;
        }

        for (pool_address, borrow_position) in borrows {
            let ObligationPosition { amount, .. } = borrow_position;

            let ticker =
                storage::get_pool_ticker(e, &pool_address).expect("Pool must exist at this point");
            let asset = oracle::Asset::Other(ticker);
            let lastprice = reflector_contract
                .lastprice(&asset)
                .ok_or(LCError::OracleDoesNotKnowAssetPrice)?;
            borrow_sum_value = borrow_sum_value
                .checked_add(
                    lastprice
                        .price
                        .checked_mul(amount)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;
        }

        let numerator = collateral_sum_value
            .checked_mul(liquidation_threshold_bps)
            .ok_or(LCError::OverOrUnderflow)?;
        let health_factor = numerator
            .checked_div(borrow_sum_value)
            .ok_or(LCError::OverOrUnderflow)?;

        Ok(health_factor)
    }

    // TODO: Would also be good to separate `health_factor` into a separate method
    // and write unit test for it...

    #[allow(unused)]
    pub fn repay(e: Env, pool_address: Address) -> Result<(), LCError> {
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
    ) -> Result<(), LCError> {
        user.require_auth();

        if amount <= 0 {
            return Err(LCError::NonPositiveWithdraw);
        }

        if !storage::pool_exists(&e, &pool_address) {
            return Err(LCError::PoolDoesNotExist);
        }

        if !storage::deposit_exists(&e, &user, &pool_address)? {
            return Err(LCError::DepositDoesNotExist);
        }

        let Pool {
            token_address,
            supply,
            borrowed,
            ..
        } = storage::get_pool(&e, &pool_address).expect("Pool must exist at this point");

        if amount >= (supply - borrowed) {
            return Err(LCError::NotEnoughPoolFunds);
        }

        storage::set_pool_supply(&e, &pool_address, -amount)?;
        storage::set_obligation_deposit(&e, &user, &pool_address, -amount)?;

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

    pub fn get_apy(e: Env, pool_address: Address) -> Result<CompoundRates, LCError> {
        let pool = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        pool.get_apy()
    }

    pub fn accrue_interest(e: Env, pool_address: Address) -> Result<(), LCError> {
        // TODO: check for admin's signature
        let mut pool = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        pool.accrue_interest(&e)?;
        storage::set_pool(&e, &pool_address, &pool)?;

        Ok(())
    }

    pub fn add_interest_to_user_obligations(e: Env, user: Address) -> Result<(), LCError> {
        let _obligation =
            storage::get_obligation(&e, &user).ok_or(LCError::ObligationDoesNotExist)?;

        todo!()
    }
}
