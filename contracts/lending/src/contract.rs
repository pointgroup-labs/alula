use {
    crate::{
        constants::{
            LCError, ACCRUAL_INIT, BPS_FACTOR, BPS_IN_PERCENT, DEFAULT_FLASH_LOAN_FEE_BPS,
            DEFAULT_LIQUIDATION_THRESHOLD, DEFAULT_MAX_SLIPPAGE_BPS, REFLECTOR_TESTNET_ADDRESS,
        },
        interest_rate::CompoundRates,
        math_utils::MathUtils,
        obligation::{LiquidationValues, Obligation},
        oracle,
        pool::{Pool, PoolAddress, PoolConfig},
        storage::{self, GlobalState},
    },
    moderc3156::FlashLoanClient,
    soroban_sdk::{
        contract, contractimpl, log,
        token::{self, StellarAssetClient, TokenClient},
        Address, BytesN, Env, Symbol, Vec,
    },
};

#[contract]
/// Lending Smart Contract. Allows users to lend and borrow other users' assets
pub struct LendingContract;

#[contractimpl]
impl LendingContract {
    /// Constructs the lending contract
    ///
    /// ### Arguments
    /// * `admin` - contract's administrator
    /// * `liquidation_threshold_percent` - threshold percentage used for health factor calculation
    pub fn __constructor(
        e: Env,
        admin: Address,
        liquidation_threshold_percent: Option<i128>,
    ) -> Result<(), LCError> {
        let liquidation_threshold_percent = if let Some(lt) = liquidation_threshold_percent {
            if lt <= 0 || lt > 100 {
                return Err(LCError::InvalidLiquidationThreshold);
            }
            lt
        } else {
            DEFAULT_LIQUIDATION_THRESHOLD
        };
        let liquidation_threshold_bps = liquidation_threshold_percent * BPS_IN_PERCENT;

        let global_state = GlobalState {
            admin,
            status: true,
            liquidation_threshold_bps,
        };

        storage::set_global_state(&e, &global_state);

        Ok(())
    }

    /// Gets the contract's global state
    pub fn get_global_state(e: Env) -> GlobalState {
        storage::get_global_state(&e)
    }

    /// Initializes a loan pool for a specific asset
    ///
    /// ### Arguments
    /// * `token_address` - address of a corresponding Soroban Asset Contract
    /// * `token_symbol` - symbol which represents a pool's token
    /// * `salt` - optional salt data, which when provided is used along with `token_address` to derive a deterministic pool address
    /// * `pool_config` - optional `PoolConfig` data. If not provided - a default pool config is used
    pub fn initialize_pool(
        e: Env,
        token_address: Address,
        token_ticker: Symbol, // NB: Token Interface contains a `.symbol()` endpoint, which can be used for retrieving a token's ticker
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>,
    ) -> Result<PoolAddress, LCError> {
        process_initialize_pool(&e, &token_address, &token_ticker, &salt, &pool_config)
    }

    /// Deposits tokens into the loan pool
    ///
    /// ### Arguments
    /// * `user` - user which deposits a token
    /// * `pool_address` - address of a pool to which the deposit happens
    /// * `amount` - amount of tokens which are going to be deposited
    pub fn deposit(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_deposit(&e, &user, &pool_address, amount)
    }

    /// Borrows tokens from the loan pool
    ///
    /// ### Arguments
    /// * `user` - user which borrows a token
    /// * `pool_address` - address of a pool from which the borrow happens
    /// * `amount` - amount of tokens which are going to be borrowed
    pub fn borrow(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_borrow(&e, &user, &pool_address, amount)
    }

    /// Adds tokens into the loan pool as collateral only.
    /// This implies that they are always available for a healthy withdrawal for the
    /// cost of not accruing an interest rate
    ///
    /// ### Arguments
    /// * `user` - user which deposits a token
    /// * `pool_address` - address of a pool to which the collateral deposit happens
    /// * `amount` - amount of tokens which are being deposited as a collateral
    pub fn add_collateral(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_add_collateral(&e, &user, &pool_address, amount)
    }

    /// Repays borrowed tokens
    ///
    /// ### Arguments
    /// * `user` - user which repays borrowed tokens
    /// * `pool_address` - address of a pool from which the borrow happened
    /// * `amount` - amount of repaid tokens
    pub fn repay(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_repay(&e, &user, &pool_address, amount)
    }

    /// Liquidates borrower's position if position's health factor criterion isn't met
    ///
    /// ### Arguments
    /// * `liquidator` - agent which liquidates the borrower's position
    /// * `borrow_pool_address` - address of a pool whose borrowed tokens are repaid by the liquidator
    /// * `collateral_pool_address` - address of a pool whose tokens are sold to the liquidator with a discount
    /// * `amount` - amount of repaid tokens
    pub fn liquidate(
        e: Env,
        liquidator: Address,
        borrower: Address,
        borrow_pool_address: Address,
        collateral_pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        liquidator.require_auth();

        process_liquidate(
            &e,
            &liquidator,
            &borrower,
            &borrow_pool_address,
            &collateral_pool_address,
            amount,
        )
    }
    /// Removes collateral tokens from the loan pool to the user
    ///
    /// ### Arguments
    /// * `user` - user which withdraws collateral tokens
    /// * `pool_address` - address of a pool from which the withdrawal happens
    /// * `amount` - amount of withdrawn tokens
    pub fn remove_collateral(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_remove_collateral(&e, &user, &pool_address, amount)
    }

    /// Withdraws deposited tokens from the loan pool to the user
    ///
    /// ### Arguments
    /// * `user` - user which withdraws deposited tokens
    /// * `pool_address` - address of a pool from which the withdrawal happens
    /// * `amount` - amount of withdrawn tokens
    pub fn withdraw(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_withdraw(&e, &user, &pool_address, amount)
    }

    /// Creates a flash loan
    ///
    /// ### Arguments
    /// * `contract` - contract's address which leverages the flash loaned amount and adheres to `erc3156` standard
    /// * `pool_address` - address of a pool from which the flash loan happens
    /// * `amount` - amount of lent tokens
    pub fn flash_loan(
        e: Env,
        contract: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        contract.require_auth();

        process_flash_loan(&e, &contract, &pool_address, amount)
    }

    pub fn swap(
        e: Env,
        user: Address,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_swap(
            &e,
            &user,
            &token_in,
            &token_out,
            amount_in,
            DEFAULT_MAX_SLIPPAGE_BPS,
        )?;

        Ok(())
    }

    pub fn deposit_multiply(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        collateral_pool_address: Address,
        collateral_amount: i128,
        flash_borrow_amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_deposit_multiply(
            &e,
            &user,
            &deposit_pool_address,
            &collateral_pool_address,
            collateral_amount,
            flash_borrow_amount,
        )
    }

    pub fn withdraw_multiply(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        collateral_pool_address: Address,
        collateral_amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_withdraw_multiply(
            &e,
            &user,
            &deposit_pool_address,
            &collateral_pool_address,
            collateral_amount,
        )
    }

    pub fn get_user_obligation(e: Env, user: Address) -> Result<Obligation, LCError> {
        if let Some(mut obligation) = storage::get_obligation(&e, &user) {
            obligation.accrue_interest(&e)?;
            storage::set_obligation(&e, &user, &obligation);

            Ok(obligation)
        } else {
            Err(LCError::ObligationDoesNotExist)
        }
    }

    pub fn get_pool(e: Env, pool_address: Address) -> Option<Pool> {
        storage::get_pool(&e, &pool_address)
    }

    /// Returns a list of all pool addresses in the protocol
    pub fn get_all_pools(e: Env) -> Vec<PoolAddress> {
        storage::get_all_pools(&e)
    }

    pub fn get_apy(e: Env, pool_address: Address) -> Result<CompoundRates, LCError> {
        let pool = storage::get_pool(&e, &pool_address).ok_or(LCError::PoolDoesNotExist)?;

        pool.get_apy()
    }
}

fn process_initialize_pool(
    e: &Env,
    token_address: &Address,
    token_ticker: &Symbol,
    salt: &Option<BytesN<32>>,
    pool_config: &Option<PoolConfig>,
) -> Result<PoolAddress, LCError> {
    let pool_address: PoolAddress = if let Some(salt) = salt {
        // TODO: Check some other ways of deriving an address
        e.deployer()
            .with_address(token_address.clone(), salt.clone())
            .deployed_address()
    } else {
        token_address.clone()
    };

    if storage::pool_exists(&e, &pool_address) {
        return Err(LCError::PoolAlreadyExists);
    }

    let config = if let Some(config) = pool_config {
        if let Err(err) = config.validate() {
            log!(&e, "pool config error", err);
            return Err(LCError::InvalidLoanPoolConfig);
        }

        config.clone()
    } else {
        Default::default()
    };

    let pool = Pool {
        config,
        token_ticker: token_ticker.clone(),
        token_address: token_address.clone(),
        available: 0,
        total_shares: 0,
        total_borrowed: 0,
        total_collateral: 0,
        last_accrual: ACCRUAL_INIT,
        last_accrual_timestamp: e.ledger().timestamp(),
    };

    storage::set_pool(&e, &pool_address, &pool);
    storage::register_pool(&e, &pool_address);

    Ok(pool_address)
}

pub fn process_deposit(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveDeposit);
    }

    let Some(mut pool) = storage::get_pool(&e, &pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };
    pool.accrue_interest(&e)?;

    let shares_to_issue = pool.compute_shares_from_tokens(amount)?;

    let mut obligation = storage::get_obligation(&e, &user).unwrap_or(Obligation::new(&e));
    obligation.deposit(&pool_address, shares_to_issue)?;

    // NB: Should the depositor accrue interest on his obligation in this place?
    // obligation.accrue_interest(&e);

    pool.adjust_total_shares(shares_to_issue)?;
    pool.adjust_available(amount)?;

    storage::set_obligation(&e, &user, &obligation);
    storage::set_pool(&e, &pool_address, &pool);

    let token_client = token::Client::new(&e, &pool.token_address);
    token_client.transfer(&user, &e.current_contract_address(), &amount);

    Ok(())
}

fn process_borrow(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveBorrow);
    }

    let Some(mut obligation) = storage::get_obligation(&e, &user) else {
        return Err(LCError::ObligationDoesNotExist);
    };
    obligation.accrue_interest(&e)?;

    let Some(mut pool) = storage::get_pool(&e, &pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };

    if amount > pool.available {
        return Err(LCError::NotEnoughPoolFunds);
    }

    obligation.borrow(&pool_address, amount)?;

    pool.adjust_total_borrowed(amount)?;
    pool.adjust_available(-amount)?;

    if !obligation.is_healthy(&e)? {
        return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
    }

    storage::set_obligation(&e, &user, &obligation);
    storage::set_pool(&e, &pool_address, &pool);

    let token_client = token::Client::new(&e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), &user, &amount);

    Ok(())
}

fn process_add_collateral(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveDeposit);
    }

    let Some(mut pool) = storage::get_pool(&e, &pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };

    let mut obligation = storage::get_obligation(&e, &user).unwrap_or(Obligation::new(&e));
    obligation.accrue_interest(&e)?;

    obligation.add_collateral(&pool_address, amount)?;
    pool.adjust_total_collateral(amount)?;

    storage::set_obligation(&e, &user, &obligation);
    storage::set_pool(&e, &pool_address, &pool);

    let token_client = token::Client::new(&e, &pool.token_address);
    token_client.transfer(&user, &e.current_contract_address(), &amount);

    Ok(())
}

fn process_repay(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveRepay);
    }

    let Some(mut obligation) = storage::get_obligation(&e, &user) else {
        return Err(LCError::ObligationDoesNotExist);
    };
    obligation.accrue_interest(&e)?;

    let Some(mut pool) = storage::get_pool(&e, &pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };

    let repaid_amount = obligation.repay(&pool_address, amount)?;

    pool.adjust_total_borrowed(-repaid_amount)?;
    pool.adjust_available(repaid_amount)?;

    if obligation.is_empty() {
        // NB: This will never be hit because of the collateral required?
        storage::remove_obligation(&e, &user);
    } else {
        storage::set_obligation(&e, &user, &obligation);
    }
    storage::set_pool(&e, &pool_address, &pool);

    let token_client = token::Client::new(&e, &pool.token_address);
    token_client.transfer(&user, &e.current_contract_address(), &amount);

    Ok(())
}

fn process_liquidate(
    e: &Env,
    liquidator: &Address,
    borrower: &Address,
    borrow_pool_address: &Address,
    collateral_pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveLiquidation);
    }

    if liquidator == borrower {
        return Err(LCError::SelfLiquidation);
    }

    if borrow_pool_address == collateral_pool_address {
        // TODO: replace InternalError
        return Err(LCError::InternalError);
    }

    let Some(mut obligation) = storage::get_obligation(&e, &borrower) else {
        return Err(LCError::ObligationDoesNotExist);
    };

    obligation.accrue_interest(&e)?;
    if obligation.is_healthy(&e)? {
        return Err(LCError::LiquidatedPositionIsHealthy);
    }

    let Some(mut borrow_pool) = storage::get_pool(&e, &borrow_pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };
    let Some(mut collateral_pool) = storage::get_pool(&e, &collateral_pool_address) else {
        return Err(LCError::CollateralPoolDoesNotExist);
    };

    let LiquidationValues {
        liquidated_amount,
        collateral_amount_sold,
        shares_amount_sold,
        tokens_from_sold_shares,
    } = obligation.liquidate(
        &e,
        &borrow_pool_address,
        &collateral_pool_address,
        &borrow_pool,
        &collateral_pool,
        amount,
    )?;

    let total_collateral_tokens_received_by_the_liquidator = tokens_from_sold_shares
        .checked_add(collateral_amount_sold)
        .map_over_or_underflow()?;

    borrow_pool.adjust_total_borrowed(-liquidated_amount)?;
    borrow_pool.adjust_available(liquidated_amount)?;

    collateral_pool.adjust_total_shares(-shares_amount_sold)?;
    collateral_pool.adjust_available(-tokens_from_sold_shares)?;
    collateral_pool.adjust_total_collateral(-collateral_amount_sold)?;

    storage::set_obligation(&e, &borrower, &obligation);

    storage::set_pool(&e, &borrow_pool_address, &borrow_pool);
    storage::set_pool(&e, &collateral_pool_address, &collateral_pool);

    let borrowed_token_client = token::Client::new(&e, &borrow_pool.token_address);
    borrowed_token_client.transfer(
        &liquidator,
        &e.current_contract_address(),
        &liquidated_amount,
    );

    let collateral_token_client = token::Client::new(&e, &collateral_pool.token_address);
    collateral_token_client.transfer(
        &e.current_contract_address(),
        &liquidator,
        &total_collateral_tokens_received_by_the_liquidator,
    );

    Ok(())
}

fn process_remove_collateral(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveWithdraw);
    }

    let Some(mut obligation) = storage::get_obligation(&e, &user) else {
        return Err(LCError::ObligationDoesNotExist);
    };
    obligation.accrue_interest(&e)?;

    let Some(mut pool) = storage::get_pool(&e, &pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };

    if amount > pool.total_collateral {
        return Err(LCError::NotEnoughPoolFunds);
    }

    obligation.remove_collateral(&pool_address, amount)?;
    pool.adjust_total_collateral(-amount)?;

    if !obligation.is_healthy(&e)? {
        return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
    }

    if obligation.is_empty() {
        storage::remove_obligation(&e, &user);
    } else {
        storage::set_obligation(&e, &user, &obligation);
    }
    storage::set_pool(&e, &pool_address, &pool);

    let token_client = token::Client::new(&e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), &user, &amount);

    Ok(())
}

fn process_withdraw(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveWithdraw);
    }

    let Some(mut obligation) = storage::get_obligation(&e, &user) else {
        return Err(LCError::ObligationDoesNotExist);
    };
    obligation.accrue_interest(&e)?;

    let Some(mut pool) = storage::get_pool(&e, &pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };

    if amount > pool.available {
        return Err(LCError::NotEnoughPoolFunds);
    }

    let shares_to_burn = pool.compute_shares_from_tokens(amount)?;

    obligation.withdraw(&pool_address, shares_to_burn)?;

    pool.adjust_total_shares(-shares_to_burn)?;
    pool.adjust_available(-amount)?;

    if !obligation.is_healthy(&e)? {
        return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
    }

    if obligation.is_empty() {
        storage::remove_obligation(&e, &user);
    } else {
        storage::set_obligation(&e, &user, &obligation);
    }
    storage::set_pool(&e, &pool_address, &pool);

    let token_client = token::Client::new(&e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), &user, &amount);

    Ok(())
}

fn process_flash_loan(
    e: &Env,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    const TEST_FLASH_LOAN_FEE_BP: i128 = 100; // 1%
    const BPS_FACTOR: i128 = 10_000;

    if amount <= 0 {
        return Err(LCError::NonPositiveFlashLoan);
    }

    let Some(pool) = storage::get_pool(&e, &pool_address) else {
        return Err(LCError::PoolDoesNotExist);
    };

    if pool.available < amount {
        return Err(LCError::NotEnoughPoolFunds);
    }

    let token_client = token::Client::new(&e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), &contract, &amount);

    let flash_loan_taker_client = FlashLoanClient::new(&e, &contract);
    flash_loan_taker_client.exec_op(
        &e.current_contract_address(),
        &pool.token_address,
        &amount,
        &TEST_FLASH_LOAN_FEE_BP,
    );

    // WARN: Does this have enough precision?
    let fees = amount
        .checked_mul(TEST_FLASH_LOAN_FEE_BP)
        .map_over_or_underflow()?
        .checked_div(BPS_FACTOR)
        .map_over_or_underflow()?;
    let amount_to_repay = amount.checked_add(fees).map_over_or_underflow()?;

    token_client.transfer(&contract, &e.current_contract_address(), &amount_to_repay);

    Ok(())
}

fn process_deposit_multiply(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    amount: i128,
    flash_borrow_amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveDeposit);
    }

    let Some(mut borrow_pool) = storage::get_pool(&e, &borrow_pool_address) else {
        return Err(LCError::CollateralPoolDoesNotExist);
    };

    let Some(deposit_pool) = storage::get_pool(&e, &deposit_pool_address) else {
        return Err(LCError::DepositDoesNotExist);
    };

    // ----- Flash Borrow -----
    if borrow_pool.available < flash_borrow_amount {
        return Err(LCError::NotEnoughPoolFunds);
    }

    let flash_borrowed_token_client = token::Client::new(&e, &borrow_pool_address);
    flash_borrowed_token_client.transfer(
        &e.current_contract_address(),
        &user,
        &flash_borrow_amount,
    );

    // ----- Swap -----
    let amount_in = amount
        .checked_add(flash_borrow_amount)
        .map_over_or_underflow()?;
    let amount_out = process_swap(
        &e,
        &user,
        &borrow_pool.token_address,
        &deposit_pool.token_address,
        amount_in,
        DEFAULT_MAX_SLIPPAGE_BPS,
    )?;

    // ----- Deposit -----
    process_deposit(&e, &user, &deposit_pool.token_address, amount_out)?;

    // ----- Borrow -----
    process_borrow(&e, &user, &borrow_pool_address, flash_borrow_amount)?;

    // ----- Flash Repay -----
    let flash_loan_fee = flash_borrow_amount
        .checked_mul(DEFAULT_FLASH_LOAN_FEE_BPS)
        .map_over_or_underflow()?
        .checked_div(BPS_FACTOR)
        .map_over_or_underflow()?;
    let flash_repay_amount = flash_loan_fee
        .checked_add(flash_borrow_amount)
        .map_over_or_underflow()?;

    flash_borrowed_token_client.transfer(&user, &e.current_contract_address(), &flash_repay_amount);
    borrow_pool.adjust_available(flash_loan_fee)?;

    storage::set_pool(&e, &borrow_pool_address, &borrow_pool);

    Ok(())
}

// TODO: The `multiply` position must be somehow isolated, I think...
pub fn process_withdraw_multiply(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount <= 0 {
        return Err(LCError::NonPositiveWithdraw);
    }

    let Some(mut borrow_pool) = storage::get_pool(&e, &borrow_pool_address) else {
        return Err(LCError::CollateralPoolDoesNotExist);
    };

    let Some(deposit_pool) = storage::get_pool(&e, &deposit_pool_address) else {
        return Err(LCError::DepositDoesNotExist);
    };

    // ---- Flash Borrow ----
    let flash_borrow_amount = get_amount_out(
        e,
        &deposit_pool.token_address,
        &borrow_pool.token_address,
        amount,
    )?;

    if borrow_pool.available < flash_borrow_amount {
        return Err(LCError::NotEnoughPoolFunds);
    }

    let flash_borrowed_token_client = token::Client::new(&e, &borrow_pool_address);
    flash_borrowed_token_client.transfer(
        &e.current_contract_address(),
        &user,
        &flash_borrow_amount,
    );

    // ---- Repay Debt ----
    process_repay(&e, &user, &borrow_pool_address, flash_borrow_amount)?;

    // ---- Withdraw ----
    process_withdraw(&e, &user, &deposit_pool.token_address, amount)?;

    // ---- Swap ----
    process_swap(
        e,
        user,
        &deposit_pool.token_address,
        &borrow_pool.token_address,
        amount,
        DEFAULT_MAX_SLIPPAGE_BPS,
    )?;

    // ---- Flash Repay ----
    let flash_loan_fee = flash_borrow_amount
        .checked_mul(DEFAULT_FLASH_LOAN_FEE_BPS)
        .map_over_or_underflow()?
        .checked_div(BPS_FACTOR)
        .map_over_or_underflow()?;
    let flash_repay_amount = flash_loan_fee
        .checked_add(flash_borrow_amount)
        .map_over_or_underflow()?;

    flash_borrowed_token_client.transfer(&user, &e.current_contract_address(), &flash_repay_amount);
    borrow_pool.adjust_available(flash_loan_fee)?;

    storage::set_pool(&e, &borrow_pool_address, &borrow_pool);

    Ok(())
}

pub fn get_asset_price(e: &Env, ticker: &Symbol) -> Result<i128, LCError> {
    let reflector_address = Address::from_str(e, REFLECTOR_TESTNET_ADDRESS);
    let reflector_contract = oracle::Client::new(e, &reflector_address);

    let asset = oracle::Asset::Other(ticker.clone());

    let last_price = reflector_contract
        .lastprice(&asset)
        .ok_or(LCError::OracleDoesNotKnowAssetPrice)?;

    Ok(last_price.price)
}

pub fn get_amount_out(
    _e: &Env,
    _token_a: &Address,
    _token_b: &Address,
    amount_in: i128,
) -> Result<i128, LCError> {
    // TODO: This must be encapsulated..
    Ok(amount_in)
}

pub fn process_swap(
    e: &Env,
    user: &Address,
    token_a: &Address,
    token_b: &Address,
    amount_in: i128,
    max_slippage_bps: i128,
) -> Result<i128, LCError> {
    // TODO: `Swap` logic must be encapsulated in a module
    // so that whenever we want to change the way we do our swap - it's not tedious
    // Mint and burn for now...
    let sac_client = StellarAssetClient::new(&e, &token_b);
    let token_client = TokenClient::new(&e, &token_a);

    let max_slippage_amount = amount_in
        .checked_mul(max_slippage_bps)
        .map_over_or_underflow()?
        .checked_div(BPS_FACTOR)
        .map_over_or_underflow()?;

    let amount_out = amount_in
        .checked_sub(max_slippage_amount)
        .map_over_or_underflow()?;

    sac_client.mint(&user, &amount_out);
    token_client.burn(&user, &amount_in);

    // >>>> DRAFT >>>>
    // // Swap all initial and borrowed tokens
    // let swap_router_address =
    //     Address::from_string(&String::from_str(&e, SOROSWAP_ROUTER_TESTNET_ADDRESS));
    // let swap_router_contract = swap_router::Client::new(&e, &swap_router_address);

    // let (mut reserve_a, mut reserve_b) = (0, 0);

    // // #[cfg(feature = "deploy")]
    // // {
    // //     let factory = swap_router_contract.get_factory();

    // //     let (a, b) = soroswap_library::get_reserves_with_factory(
    // //         e.clone(),
    // //         factory.clone(),
    // //         collateral_pool.token_address.clone(),
    // //         deposit_pool.token_address.clone(),
    // //     )
    // //     .unwrap();

    // //     reserve_a = a;
    // //     reserve_b = b;
    // // };

    // #[cfg(not(feature = "deploy"))]
    // {
    //     reserve_a = 1_000_000;
    //     reserve_b = 1_000_000;
    // };

    // let amount_in = collateral_amount
    //     .checked_add(flash_borrow_amount)
    //     .map_over_or_underflow()?;

    // let amount_out = swap_router_contract.router_get_amount_out(&amount_in, &reserve_a, &reserve_b);

    // let amount_out_min = amount_out
    //     .checked_mul(BPS_FACTOR - TEST_SLIPPAGE_BPS)
    //     .map_over_or_underflow()?
    //     .checked_div(BPS_FACTOR)
    //     .map_over_or_underflow()?;

    // let path = soroban_sdk::vec![
    //     &e,
    //     collateral_pool.token_address,
    //     deposit_pool.token_address.clone()
    // ];

    // let amounts = swap_router_contract.swap_exact_tokens_for_tokens(
    //     &amount_in,
    //     &amount_out_min,
    //     &path,
    //     &user,
    //     &0,
    // );
    // <<<< DRAFT <<<<

    Ok(amount_out)
}
