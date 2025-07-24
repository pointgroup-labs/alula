use {
    crate::{
        constants::{
            ACCRUAL_INIT, BPS_FACTOR, BPS_IN_PERCENT, DEFAULT_FLASH_LOAN_FEE_BPS,
            DEFAULT_LIQUIDATION_THRESHOLD, LEVERAGE_SCALE, MAX_LEVERAGE_MULTIPLIER,
            MIN_LEVERAGE_MULTIPLIER, REFLECTOR_TESTNET_ADDRESS,
        },
        events,
        interest_rate::CompoundRates,
        math_utils::MathUtils,
        obligation::{LiquidationValues, Obligation},
        oracle,
        pool::{MultiplyPair, Pool, PoolAddress, PoolConfig},
        storage::{self, GlobalState},
        swap, LCError,
    },
    moderc3156::FlashLoanClient,
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{
        contract, contractimpl, log,
        token::{self, TokenClient},
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

        events::constructor(&e, &admin, liquidation_threshold_percent);

        let liquidation_threshold_bps = liquidation_threshold_percent * BPS_IN_PERCENT;
        let global_state = GlobalState {
            admin,
            status: true,
            liquidation_threshold_bps,
        };

        storage::set_global_state(&e, &global_state);

        Ok(())
    }

    /// Upgrades the lending contract
    ///
    /// ### Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a new version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        // TODO: Implement decentralized governance of the contract
        let admin = storage::get_global_state(&e).admin;
        admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
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
    ) -> Result<Address, LCError> {
        process_initialize_pool(&e, &token_address, &token_ticker, &salt, &pool_config)
    }

    /// Registers a multiply pair
    ///
    /// ### Arguments
    /// * `deposit_pool` - address of a pool in a pair for a leveraged deposit
    /// * `borrow_pool` - address of a pool in a pair for a leveraged borrow
    pub fn register_multiply_pair(
        e: Env,
        deposit_pool: Address,
        borrow_pool: Address,
    ) -> Result<(), LCError> {
        let admin = storage::get_global_state(&e).admin;
        admin.require_auth();

        if !(Pool::exists(&e, &deposit_pool) && Pool::exists(&e, &borrow_pool)) {
            return Err(LCError::PoolDoesNotExist);
        }

        let pair = MultiplyPair {
            deposit_pool,
            borrow_pool,
        };

        Pool::register_multiply_pair(&e, pair);

        Ok(())
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

    /// Swap tokens via a swap provider contract
    ///
    /// ### Arguments
    /// * `user` - user which deposits a token
    /// * `token_in` - address of a token that would be taken from the user
    /// * `token_out` - address of a token that would be given to the user
    /// * `amount` - exact amount of the `token_in`
    pub fn swap(
        e: Env,
        user: Address,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
    ) -> Result<i128, LCError> {
        user.require_auth();

        process_swap_exact_tokens(&e, &user, &token_in, &token_out, amount_in)
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
    /// * `user` - user that adds collateral
    /// * `pool_address` - address of a pool to which the collateral is being added
    /// * `amount` - amount of tokens which are being added as a collateral
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
    /// * `amount` - provided amount of tokens to repay. If this amount exceeds the total debt, only the outstanding debt will be repaid.
    /// Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to repay the entire debt
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
    /// * `amount` - desired amount of collateral tokens to remove.
    /// The actual amount removed is capped to maintain the position's LTV at its Open LTV on the pool.
    /// Passing [`u64::MAX`] (or [`i128::MAX`]) effectively removes all available collateral
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
    /// * `amount` - desired amount of tokens to withdraw.
    /// The actual amount withdrawn is capped to maintain the position's LTV at its Open LTV on the pool.
    /// Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to withdraw all tokens available for it
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

    /// Deposits tokens into the loan pool with leverage. Leverage is achieved by utilizing flash loan and token swap
    ///
    /// # WARNING
    /// This increases the perceived `supply APR` only
    /// when `(borrowed token borrow APR < supply token supply APR)` holds true
    ///
    /// ### Arguments
    /// * `user` - user that deposits tokens with leverage
    /// * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
    /// * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
    /// * `amount` - original borrow amount before the leverage
    /// * `leverage_multiplier` - leverage multiplier as a decimal (e.g., 700 for x7, 255 for x2.55, etc)
    pub fn deposit_with_leverage(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        amount: i128,
        leverage_multiplier: u32,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_deposit_with_leverage(
            &e,
            &user,
            &deposit_pool_address,
            &borrow_pool_address,
            amount,
            leverage_multiplier,
        )
    }

    /// Withdraws tokens from the leveraged deposit position without affecting the leverage multiplier
    ///
    /// ### Arguments
    /// * `user` - user that deleverages and withdraws from the position
    /// * `deposit_pool_address` - address of a pool from the pair to which the deposit happened
    /// * `borrow_pool_address` - address of a pool from the pair from which the borrow happened
    /// * `amount` - desired amount of deposited tokens to withdraw.
    /// The actual amount withdrawn is capped by the value difference between deposited and borrowed tokens in
    /// the leveraged position (minus operational fees). Passing [`u64::MAX`] (or [`i128::MAX`])
    /// can be used to withdraw all available tokens
    pub fn withdraw_from_leveraged(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        amount: i128,
    ) -> Result<(), LCError> {
        user.require_auth();

        process_withdraw_from_leveraged(
            &e,
            &user,
            &deposit_pool_address,
            &borrow_pool_address,
            amount,
        )
    }

    /// Returns asset's decimals
    pub fn get_asset_decimals() -> u32 {
        // See - https://github.com/stellar/rs-soroban-env/blob/main/soroban-env-host/src/builtin_contracts/stellar_asset_contract/contract.rs#L374
        7
    }

    /// Returns oracle price's decimals
    pub fn get_oracle_price_decimals(e: Env) -> u32 {
        get_oracle_price_decimals(&e)
    }

    /// Returns pool asset's oracle price
    ///
    /// ### Arguments
    /// * `pool_address` - address of asset which price is returned
    pub fn get_pool_asset_oracle_price(e: Env, pool_address: Address) -> Result<i128, LCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        get_asset_price(&e, &pool.token_ticker)
    }

    /// Returns the user's obligation which includes data about all of their deposits and borrows
    ///
    /// ### Arguments
    /// * `user` - user which obligation is returned
    pub fn get_user_obligation(e: Env, user: Address) -> Result<Obligation, LCError> {
        let mut obligation = Obligation::try_get(&e, &user)?;

        obligation.accrue_interest(&e)?;
        obligation.set(&e);

        Ok(obligation)
    }

    /// Accrues interest on a specific user's obligation and on its pools
    ///
    /// ### Arguments
    /// * `user` - user whose obligation interest is accrued
    pub fn accrue_interest(e: Env, user: Address) -> Result<(), LCError> {
        let mut obligation = Obligation::try_get(&e, &user)?;

        obligation.accrue_interest(&e)?;
        obligation.set(&e);

        Ok(())
    }

    /// Returns the specific loan pool
    ///
    /// ### Arguments
    /// * `pool_address` - pool which data is returned
    pub fn get_pool(e: Env, pool_address: Address) -> Result<Pool, LCError> {
        Pool::try_get(&e, &pool_address)
    }

    /// Returns a list of all pool addresses in the protocol
    pub fn get_all_pools(e: Env) -> Vec<Address> {
        Pool::get_all(&e)
    }

    /// Returns a list of all user obligations in the protocol
    pub fn get_all_obligations(e: Env) -> Vec<Address> {
        Obligation::get_all(&e)
    }

    /// Returns a list of all multiply pairs in the protocol
    pub fn get_all_multiply_pairs(e: Env) -> Vec<MultiplyPair> {
        Pool::get_all_multiply_pairs(&e)
    }

    /// Returns APY calculated for the current utilization ratio of a pool in basis points (e.g., 2912 = 29.12%, etc)
    ///
    /// ### Arguments
    /// * `pool_address` - address of a pool for which APY is returned
    pub fn get_apy(e: Env, pool_address: Address) -> Result<CompoundRates, LCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        pool.get_apy()
    }

    /// Returns APY calculated for the optimal utilization ratio of a pool in basis points (e.g., 4000 = 40.00%, etc)
    ///
    /// ### Arguments
    /// * `pool_address` - address of a pool for which optimal APY is returned
    pub fn get_optimal_apy(_e: Env, _pool_address: Address) -> Result<CompoundRates, LCError> {
        // TODO: Start calculating this dynamically
        Ok(CompoundRates {
            borrow_bps: 4_000,
            supply_bps: 1_500,
        })
    }

    /// Resets the contract's storage. Useful when the contract's invariants are broken and require resetting on the testnet
    /// without re-deploying the contract
    pub fn reset_storage(e: Env) {
        let admin = storage::get_global_state(&e).admin;
        admin.require_auth();

        storage::remove_all_obligations(&e);
        storage::remove_all_pools(&e);
        storage::remove_all_multiply_pairs(&e);
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

    if Pool::exists(e, &pool_address) {
        return Err(LCError::PoolAlreadyExists);
    }

    let config: PoolConfig = if let Some(config) = pool_config {
        if let Err(err) = config.validate() {
            log!(&e, "pool config error", err);
            return Err(LCError::InvalidLoanPoolConfig);
        }

        *config
    } else {
        Default::default()
    };

    let token_client = TokenClient::new(e, token_address);
    let name = token_client.name();

    let pool = Pool {
        name,
        config,
        pool_address: pool_address.clone(),
        token_ticker: token_ticker.clone(),
        token_address: token_address.clone(),
        available: 0,
        total_shares: 0,
        total_borrowed: 0,
        total_collateral: 0,
        last_accrual: ACCRUAL_INIT,
        last_accrual_timestamp: e.ledger().timestamp(),
    };

    pool.set(e);
    pool.register(e);

    events::initialize_pool(e, token_address, &pool_address, token_ticker);

    Ok(pool_address)
}

pub fn process_deposit(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    // NB: Here and in all other `process_` functions we allow 0 amounts, since
    // in this way we can always simulate method execution even when the contract's method
    // demands transferring tokens from the user's account(whose might not have this token at all)
    if amount < 0 {
        return Err(LCError::NegativeDeposit);
    }

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.accrue_interest(e)?;

    let supply_limit = pool.config.supply_limit;
    if supply_limit != 0
        && pool
            .total_supply()?
            .checked_add(amount)
            .map_over_or_underflow()?
            > supply_limit
    {
        return Err(LCError::SupplyLimitExceeded);
    }

    let issued_shares = pool.compute_shares_from_tokens(amount)?;

    let mut obligation = Obligation::try_get(e, user).unwrap_or(Obligation::new(e, user.clone()));
    obligation.deposit(e, pool_address, issued_shares)?;

    // NB: Should the depositor accrue interest on his obligation in this place?
    // obligation.accrue_interest(&e);

    pool.adjust_total_shares(e, issued_shares)?;
    pool.adjust_available(e, amount)?;

    obligation.set(e);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(user, &e.current_contract_address(), &amount);

    events::deposit(e, pool_address, user, amount, issued_shares);

    Ok(())
}

fn process_swap_for_exact_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_out: i128,
) -> Result<i128, LCError> {
    let amount_in = swap::get_amount_in(e, token_in, token_out, amount_out)?;

    let received_amount = swap::swap_tokens_for_exact_tokens(
        e, user, token_in, token_out, amount_in, amount_out, None,
    )?;

    events::swap(
        e,
        user,
        token_in,
        token_out,
        amount_in,
        amount_out,
        received_amount,
    );

    Ok(received_amount)
}

fn process_swap_exact_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
) -> Result<i128, LCError> {
    let amount_out = swap::get_amount_out(e, token_in, token_out, amount_in)?;

    let received_amount = swap::swap_exact_tokens_for_tokens(
        e, user, token_in, token_out, amount_in, amount_out, None,
    )?;

    events::swap(
        e,
        user,
        token_in,
        token_out,
        amount_in,
        amount_out,
        received_amount,
    );

    Ok(received_amount)
}

fn process_borrow(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeBorrow);
    }

    let mut obligation = Obligation::try_get(e, user)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let max_healthy_borrow_amount =
        obligation.compute_max_healthy_borrow_added_amount(e, pool_address)?;
    let available_borrow = pool.compute_available_borrow(e)?;

    let borrow_amount = i128::min(
        max_healthy_borrow_amount,
        i128::min(available_borrow, amount),
    );

    // TODO: Should this check actually happen here?
    // TODO: Is this a good idea to let somebody borrow less then they want?
    // I am not so sure, to be honest...
    if borrow_amount > pool.available {
        return Err(LCError::NotEnoughPoolFunds);
    }

    obligation.borrow(e, pool_address, borrow_amount)?;

    pool.adjust_total_borrowed(e, borrow_amount)?;
    pool.adjust_available(e, -borrow_amount)?;

    if !obligation.is_healthy(e)? {
        return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
    }

    obligation.set(e);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), user, &borrow_amount);

    events::borrow(e, pool_address, user, borrow_amount);

    Ok(())
}

fn process_add_collateral(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeCollateralAddition);
    }

    let mut pool = Pool::try_get(e, pool_address)?;
    let mut obligation = Obligation::try_get(e, user).unwrap_or(Obligation::new(e, user.clone()));
    obligation.accrue_interest(e)?;

    obligation.add_collateral(e, pool_address, amount)?;
    pool.adjust_total_collateral(e, amount)?;

    obligation.set(e);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(user, &e.current_contract_address(), &amount);

    events::add_collateral(e, pool_address, user, amount);

    Ok(())
}

fn process_repay(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeRepay);
    }

    let mut obligation = Obligation::try_get(e, user)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let repaid_amount = obligation.repay(e, pool_address, amount)?;

    pool.adjust_total_borrowed(e, -repaid_amount)?;
    pool.adjust_available(e, repaid_amount)?;

    if obligation.is_empty() {
        // NB: This will never be hit because of the collateral required?
        obligation.remove(e);
    } else {
        obligation.set(e);
    }
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(user, &e.current_contract_address(), &repaid_amount);

    events::repay(e, pool_address, user, repaid_amount);

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
    if amount < 0 {
        return Err(LCError::NegativeLiquidation);
    }

    if liquidator == borrower {
        return Err(LCError::SelfLiquidation);
    }

    if borrow_pool_address == collateral_pool_address {
        return Err(LCError::LiquidationWithEqualCollateralAndDepositPools);
    }

    let mut obligation = Obligation::try_get(e, borrower)?;

    obligation.accrue_interest(e)?;
    if obligation.is_healthy(e)? {
        return Err(LCError::LiquidatedPositionIsHealthy);
    }

    let Ok(mut borrow_pool) = Pool::try_get(e, borrow_pool_address) else {
        return Err(LCError::BorrowPoolDoesNotExist);
    };
    let Ok(mut collateral_pool) = Pool::try_get(e, collateral_pool_address) else {
        return Err(LCError::CollateralPoolDoesNotExist);
    };

    let LiquidationValues {
        liquidated_amount,
        collateral_amount_sold,
        shares_amount_sold,
        tokens_from_sold_shares,
    } = obligation.liquidate(
        e,
        borrow_pool_address,
        collateral_pool_address,
        &borrow_pool,
        &collateral_pool,
        amount,
    )?;

    let collateral_seized_amount = tokens_from_sold_shares
        .checked_add(collateral_amount_sold)
        .map_over_or_underflow()?;

    borrow_pool.adjust_total_borrowed(e, -liquidated_amount)?;
    borrow_pool.adjust_available(e, liquidated_amount)?;

    collateral_pool.adjust_total_shares(e, -shares_amount_sold)?;
    collateral_pool.adjust_available(e, -tokens_from_sold_shares)?;
    collateral_pool.adjust_total_collateral(e, -collateral_amount_sold)?;

    obligation.set(e);

    borrow_pool.set(e);
    collateral_pool.set(e);

    let borrowed_token_client = token::Client::new(e, &borrow_pool.token_address);
    borrowed_token_client.transfer(
        liquidator,
        &e.current_contract_address(),
        &liquidated_amount,
    );

    let collateral_token_client = token::Client::new(e, &collateral_pool.token_address);
    collateral_token_client.transfer(
        &e.current_contract_address(),
        liquidator,
        &collateral_seized_amount,
    );

    events::liquidate(
        e,
        liquidator,
        borrower,
        borrow_pool_address,
        collateral_pool_address,
        liquidated_amount,
        collateral_seized_amount,
    );

    Ok(())
}

fn process_remove_collateral(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeCollateralRemoval);
    }

    let mut obligation = Obligation::try_get(e, user)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let max_possible_collateral_removed_amount =
        obligation.compute_max_healthy_collateral_removed_amount(e, pool_address)?;
    let removed_tokens_amount = i128::min(
        i128::min(amount, max_possible_collateral_removed_amount),
        obligation.get_collateral(pool_address)?,
    );

    obligation.remove_collateral(e, pool_address, removed_tokens_amount)?;
    pool.adjust_total_collateral(e, -removed_tokens_amount)?;

    if !obligation.is_healthy(e)? {
        return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
    }

    if obligation.is_empty() {
        obligation.remove(e);
    } else {
        obligation.set(e);
    }
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), user, &removed_tokens_amount);

    events::remove_collateral(e, pool_address, user, removed_tokens_amount);

    Ok(())
}

fn process_withdraw(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeWithdraw);
    }

    let mut obligation = Obligation::try_get(e, user)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let max_healthy_collateral_removed_amount =
        obligation.compute_max_healthy_collateral_removed_amount(e, pool_address)?;

    let cap_withdrawn_tokens_amount = i128::min(amount, max_healthy_collateral_removed_amount);

    let obligation_shares = obligation.get_shares(pool_address)?;
    let cap_shares_amount = pool.compute_shares_from_tokens(cap_withdrawn_tokens_amount)?;

    let burnt_shares_amount = i128::min(cap_shares_amount, obligation_shares);

    let withdrawn_tokens_amount = pool.compute_tokens_from_shares(e, burnt_shares_amount)?;

    if withdrawn_tokens_amount > pool.available {
        return Err(LCError::NotEnoughPoolFunds);
    }

    obligation.withdraw(e, pool_address, burnt_shares_amount)?;

    pool.adjust_total_shares(e, -burnt_shares_amount)?;
    pool.adjust_available(e, -withdrawn_tokens_amount)?;

    // TODO: For now this check seems to be redundant
    if !obligation.is_healthy(e)? {
        return Err(LCError::HealthFactorIsLowerThanRequiredThreshold);
    }

    if obligation.is_empty() {
        obligation.remove(e);
    } else {
        obligation.set(e);
    }
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(
        &e.current_contract_address(),
        user,
        &withdrawn_tokens_amount,
    );

    events::withdraw(e, pool_address, user, withdrawn_tokens_amount);

    Ok(())
}

fn process_flash_loan(
    e: &Env,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeFlashLoan);
    }

    let pool = Pool::try_get(e, pool_address)?;
    if pool.available < amount {
        return Err(LCError::NotEnoughPoolFunds);
    }

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), contract, &amount);

    let flash_loan_taker_client = FlashLoanClient::new(e, contract);
    flash_loan_taker_client.exec_op(
        &e.current_contract_address(),
        &pool.token_address,
        &amount,
        &DEFAULT_FLASH_LOAN_FEE_BPS,
    );

    // WARN: Does this have enough precision?
    let fees = amount
        .fixed_div_floor(BPS_FACTOR, DEFAULT_FLASH_LOAN_FEE_BPS)
        .map_over_or_underflow()?;
    let amount_to_repay = amount.checked_add(fees).map_over_or_underflow()?;

    token_client.transfer(contract, &e.current_contract_address(), &amount_to_repay);

    events::flash_loan(e, contract, pool_address, amount, fees);

    Ok(())
}

fn process_deposit_with_leverage(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    amount: i128,
    leverage_multiplier: u32,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeDeposit);
    }

    if !(MIN_LEVERAGE_MULTIPLIER..=MAX_LEVERAGE_MULTIPLIER).contains(&leverage_multiplier) {
        return Err(LCError::InvalidLeverageMultiplier);
    }

    let flash_borrow_amount = {
        let leverage_multiplier = leverage_multiplier as i128;

        let scaled = leverage_multiplier
            .checked_mul(amount)
            .map_over_or_underflow()?
            - amount
                .checked_mul(LEVERAGE_SCALE as i128)
                .map_over_or_underflow()?; // safe
        scaled / (LEVERAGE_SCALE as i128) // safe
    };

    let Ok(mut borrow_pool) = Pool::try_get(e, borrow_pool_address) else {
        return Err(LCError::CollateralPoolDoesNotExist);
    };

    let Ok(deposit_pool) = Pool::try_get(e, deposit_pool_address) else {
        return Err(LCError::DepositDoesNotExist);
    };

    let flash_loaned_token_client = token::Client::new(e, borrow_pool_address);
    if leverage_multiplier > MIN_LEVERAGE_MULTIPLIER {
        // Flash Borrow
        // TODO: Think of why it can be beneficial to account for flash borrow limits as in other lending protocols
        if borrow_pool.available < flash_borrow_amount {
            return Err(LCError::NotEnoughPoolFunds);
        }

        flash_loaned_token_client.transfer(
            &e.current_contract_address(),
            user,
            &flash_borrow_amount,
        );

        borrow_pool.adjust_available(e, -flash_borrow_amount)?;
        // TODO: This `set` is required, since 'available' amount is later accounted when calling `process_borrow`
        borrow_pool.set(e);
    }

    // Swap
    let amount_in = amount
        .checked_add(flash_borrow_amount)
        .map_over_or_underflow()?;
    let amount_out = swap::get_amount_out(
        e,
        &borrow_pool.token_address,
        &deposit_pool.token_address,
        amount_in,
    )?;
    let deposit_amount = swap::swap_exact_tokens_for_tokens(
        e,
        user,
        &borrow_pool.token_address,
        &deposit_pool.token_address,
        amount_in,
        amount_out,
        None,
    )?;

    // Deposit swapped tokens
    process_deposit(e, user, deposit_pool_address, deposit_amount)?;

    if leverage_multiplier > MIN_LEVERAGE_MULTIPLIER {
        // Borrow to repay the flash loan
        let flash_loan_fee = flash_borrow_amount
            .fixed_div_floor(BPS_FACTOR, DEFAULT_FLASH_LOAN_FEE_BPS)
            .map_over_or_underflow()?;
        let flash_repay_amount = flash_loan_fee
            .checked_add(flash_borrow_amount)
            .map_over_or_underflow()?;

        let Ok(obligation) = Obligation::try_get(e, user) else {
            // TODO: Internal Error?
            return Err(LCError::ObligationDoesNotExist);
        };

        // TODO: Likely, this check is redundant, since when depositing with leverage
        // you must always be able to borrow
        let max_healthy_borrow_amount =
            obligation.compute_max_healthy_borrow_added_amount(e, &borrow_pool_address)?;

        if flash_borrow_amount > max_healthy_borrow_amount {
            // TODO: Some other error?
            return Err(LCError::BorrowLimitExceeded);
        }

        process_borrow(e, user, borrow_pool_address, flash_repay_amount)?;
        borrow_pool.refresh(e)?;

        // Repay flash loan
        flash_loaned_token_client.transfer(
            user,
            &e.current_contract_address(),
            &flash_repay_amount,
        );

        borrow_pool.adjust_available(e, flash_repay_amount)?;
        borrow_pool.set(e);

        events::deposit_with_leverage(
            e,
            user,
            deposit_pool_address,
            borrow_pool_address,
            amount,
            leverage_multiplier,
            deposit_amount,
            flash_borrow_amount,
        );
    }

    Ok(())
}

pub fn process_withdraw_from_leveraged(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    if amount < 0 {
        return Err(LCError::NegativeWithdraw);
    }

    let Ok(mut borrow_pool) = Pool::try_get(e, borrow_pool_address) else {
        return Err(LCError::BorrowPoolDoesNotExist);
    };

    let Ok(mut deposit_pool) = Pool::try_get(e, deposit_pool_address) else {
        return Err(LCError::DepositDoesNotExist);
    };

    let obligation = Obligation::try_get(e, user)?;
    let borrowed = obligation.get_borrowed(borrow_pool_address)?;

    if borrowed == 0 {
        // No leverage case is equivalent to a simple deposit
        return process_withdraw(e, user, deposit_pool_address, amount);
    }

    let obligation_shares = obligation.get_shares(deposit_pool_address)?;
    let tokens_per_obligation_shares =
        deposit_pool.compute_tokens_from_shares(e, obligation_shares)?;

    // Compute the max withdrawable amount
    let max_withdrawable_amount = compute_leveraged_position_max_withdrawable_amount(
        e,
        user,
        &deposit_pool.token_address,
        &borrow_pool.token_address,
        tokens_per_obligation_shares,
        borrowed,
    )?;

    let expected_withdrawn_amount = i128::min(amount, max_withdrawable_amount);

    // Compute the flash borrow amount for deleverage
    let withdrawn_ratio_bps = expected_withdrawn_amount
        .fixed_div_ceil(max_withdrawable_amount, BPS_FACTOR)
        .map_over_or_underflow()?;

    let plain_leverage_amount = tokens_per_obligation_shares - max_withdrawable_amount; // safe
    let plain_leverage_to_be_withdrawn = plain_leverage_amount
        .fixed_div_floor(BPS_FACTOR, withdrawn_ratio_bps)
        .map_over_or_underflow()?;

    // To maintain LTV for the leveraged position, the amount of borrowed tokens to be repaid
    // must be proportional to the withdrawn amount of the deposited tokens
    let flash_borrow_amount = borrowed
        .fixed_div_ceil(BPS_FACTOR, withdrawn_ratio_bps)
        .map_over_or_underflow()?;

    if borrow_pool.available < flash_borrow_amount {
        return Err(LCError::NotEnoughPoolFunds);
    }

    // Flash Borrow
    let flash_borrowed_token_client = token::Client::new(e, borrow_pool_address);
    flash_borrowed_token_client.transfer(&e.current_contract_address(), user, &flash_borrow_amount);

    // Repay Debt
    process_repay(e, user, borrow_pool_address, flash_borrow_amount)?;
    borrow_pool.refresh(e)?;

    // Withdraw

    let withdrawn_amount = expected_withdrawn_amount
        .checked_add(plain_leverage_to_be_withdrawn)
        .map_over_or_underflow()?;

    process_withdraw(e, user, deposit_pool_address, withdrawn_amount)?;
    deposit_pool.refresh(e)?;

    // Swap to get the flash repay amount

    let flash_loan_fee = flash_borrow_amount
        .checked_mul(DEFAULT_FLASH_LOAN_FEE_BPS)
        .map_over_or_underflow()?
        .checked_div(BPS_FACTOR)
        .map_over_or_underflow()?;
    let flash_repay_amount = flash_loan_fee
        .checked_add(flash_borrow_amount)
        .map_over_or_underflow()?;

    // TODO: Here we must call an outer swap, since they are swapped from the user's wallet, right?
    process_swap_for_exact_tokens(
        e,
        user,
        &deposit_pool_address,
        &borrow_pool_address,
        flash_repay_amount,
    )?;

    // Flash Repay
    flash_borrowed_token_client.transfer(user, &e.current_contract_address(), &flash_repay_amount);

    borrow_pool.adjust_available(e, flash_repay_amount)?;
    borrow_pool.set(e);

    events::withdraw_from_leveraged(
        e,
        user,
        deposit_pool_address,
        borrow_pool_address,
        amount,
        withdrawn_amount,
    );

    Ok(())
}

// WARN: will everything be ok here with precision?
fn compute_leveraged_position_max_withdrawable_amount(
    e: &Env,
    user: &Address,
    deposited_token: &Address,
    borrowed_token: &Address,
    deposited_amount: i128,
    borrowed_amount: i128,
) -> Result<i128, LCError> {
    let borrowed_token_swapped_amount =
        swap::get_amount_out(e, borrowed_token, deposited_token, borrowed_amount)?;

    let flash_loan_fee = borrowed_token_swapped_amount
        .fixed_div_ceil(BPS_FACTOR, DEFAULT_FLASH_LOAN_FEE_BPS)
        .map_over_or_underflow()?;

    let swapped_amount_with_fees = borrowed_token_swapped_amount
        .checked_add(flash_loan_fee)
        .map_over_or_underflow()?;

    if swapped_amount_with_fees > deposited_amount {
        // WARN: This can happen when multiply position contains a bad debt
        events::leveraged_position_bad_debt(
            e,
            user,
            deposited_token,
            borrowed_token,
            deposited_amount,
            borrowed_amount,
            borrowed_token_swapped_amount,
        );

        // TODO: This has to be thought of when implementing security mechanisms
        return Err(LCError::InternalError);
    }

    Ok(deposited_amount - swapped_amount_with_fees) // safe
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

pub fn get_oracle_price_decimals(e: &Env) -> u32 {
    let reflector_address = Address::from_str(e, REFLECTOR_TESTNET_ADDRESS);
    let reflector_contract = oracle::Client::new(e, &reflector_address);

    reflector_contract.decimals()
}
