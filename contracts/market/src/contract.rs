<<<<<<< Updated upstream:contracts/market/src/contract.rs
<<<<<<< Updated upstream:contracts/market/src/contract.rs
<<<<<<< Updated upstream:contracts/market/src/contract.rs
// use aggregated_oracle::AggregatedPriceFeedClient;
=======
// use aggregated_oracle::PriceFeedClient;
>>>>>>> Stashed changes:contracts/lending/src/contract.rs
=======
// use aggregated_oracle::PriceFeedClient;
>>>>>>> Stashed changes:contracts/lending/src/contract.rs
=======
// use aggregated_oracle::PriceFeedClient;
>>>>>>> Stashed changes:contracts/lending/src/contract.rs
use moderc3156::FlashLoanClient;
use sep_40_oracle::{Asset, PriceData, PriceFeedClient};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, BytesN, Env, String, Symbol, Vec, contract, contractimpl,
    token::{self, TokenClient},
};

use crate::{
    accrual::AccrualModel,
    constants::{
        BPS_FACTOR, DEFAULT_FLASH_LOAN_FEE_BPS, LEVERAGE_SCALE, MAX_ORACLE_PRICE_AGE_SECONDS,
    },
    error::MCError,
    events,
    interest_rate::{AnnualPercentageRates, AnnualPercentageYields},
    interest_rate_model::{InterestRateModel, kinked::KinkedIRConfig},
    math_utils::MathUtils,
    multiply_pair::MultiplyPair,
    obligation::{LiquidationValues, Obligation, ObligationKey},
    pool::{Pool, PoolConfig},
    storage::{self, GlobalState},
    swap,
};

// TODO: Consider adding a trait that defines contract's API

#[contract]
/// Isolated Lending Market Smart Contract. Allows users to lend and borrow other users' assets
pub struct MarketContract;

#[contractimpl]
impl MarketContract {
    /// Constructs the market contract
    ///
    /// ### Arguments
    /// * `admin` - market's administrator
    /// * `name` - market's name(not necessarily unique)
    /// * `oracle` - SEP-40 compliant oracle's contract address
    pub fn __constructor(
        e: Env,
        name: String,
        admin: Address,
        oracle: Address,
    ) -> Result<(), MCError> {
        let global_state = GlobalState {
            status: true,
            admin: admin.clone(),
            name: name.clone(),
        };

        storage::set_global_state(&e, &global_state);
        storage::set_oracle_address(&e, &oracle);

        events::constructor(&e, &admin, &name, &oracle);

        Ok(())
    }

<<<<<<< Updated upstream:contracts/market/src/contract.rs
<<<<<<< Updated upstream:contracts/market/src/contract.rs
<<<<<<< Updated upstream:contracts/market/src/contract.rs
    /// Upgrades the market contract
=======
=======
>>>>>>> Stashed changes:contracts/lending/src/contract.rs
=======
>>>>>>> Stashed changes:contracts/lending/src/contract.rs
    pub fn check_oracles(e: Env, asset_address: Address) -> Option<PriceData> {
        const AGGREGATED_ORACLE_ADDRESS: &str =
            "CCPVLUQYLVHMSYUJBRQGIGFND6BMFG37ZZLP7GV3EWHLF3KPQASKGZBD";

        let address = Address::from_str(&e, AGGREGATED_ORACLE_ADDRESS);
        let oracle_client = PriceFeedClient::new(&e, &address);

        let asset = Asset::Stellar(asset_address);

        oracle_client.lastprice(&asset)
    }

    /// Upgrades the lending contract
>>>>>>> Stashed changes:contracts/lending/src/contract.rs
    ///
    /// ### Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
    ///   version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        // TODO: Implement decentralized governance of the contract
        require_admin(&e);

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Gets the contract's global state
    pub fn get_global_state(e: Env) -> GlobalState {
        storage::get_global_state(&e)
    }

    /// Gets the contract's oracle address
    pub fn get_oracle_address(e: Env) -> Address {
        storage::get_oracle_address(&e)
    }

    /// Initializes a loan pool for a specific asset
    ///
    /// ### Arguments
    /// * `token_address` - address of a corresponding Soroban Asset Contract
    /// * `token_ticker` - symbol which represents a pool's token ticker
    /// * `salt` - optional salt data, which, when provided, is used along with `token_address` to
    ///   derive a deterministic pool address
    /// * `pool_config` - optional `PoolConfig` data. If not provided, a default pool config is used
    pub fn initialize_pool(
        e: Env,
        token_address: Address,
        token_ticker: Symbol, /* NB: Token Interface contains a `.symbol()` endpoint, which can
                               * be used for retrieving a token's ticker */
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>,
    ) -> Result<Address, MCError> {
        require_admin(&e);

        process_initialize_pool(&e, &token_address, &token_ticker, &salt, &pool_config)
    }

    /// Initializes a multiply pair
    ///
    /// ### Arguments
    /// * `deposit_pool_address` - address of a pool in a pair for a leveraged deposit
    /// * `borrow_pool_address` - address of a pool in a pair for a leveraged borrow
    pub fn initialize_multiply_pair(
        e: Env,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError> {
        require_admin(&e);

        process_initialize_multiply_pair(&e, &deposit_pool_address, &borrow_pool_address)
    }

    /// Deposits tokens into the loan pool
    ///
    /// ### Arguments
    /// * `user` - user that deposits a token
    /// * `pool_address` - address of a pool to which the deposit happens
    /// * `amount` - amount of tokens which are going to be deposited
    pub fn deposit(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        user.require_auth();

        let obligation_key = ObligationKey::new(user);

        process_deposit(&e, &obligation_key, &pool_address, amount)
    }

    /// Swap tokens via a swap provider contract. This guarantees a swap
    /// and is agnostic to the possible price slippage
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
    ) -> Result<i128, MCError> {
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
    ) -> Result<(), MCError> {
        user.require_auth();

        let obligation_key = ObligationKey::new(user);

        process_borrow(&e, &obligation_key, &pool_address, amount)
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
    ) -> Result<(), MCError> {
        user.require_auth();

        let obligation_key = ObligationKey::new(user);

        process_add_collateral(&e, &obligation_key, &pool_address, amount)
    }

    /// Removes collateral tokens from the loan pool to the user
    ///
    /// ### Arguments
    /// * `user` - user which withdraws collateral tokens
    /// * `pool_address` - address of a pool from which the withdrawal happens
    /// * `amount` - desired amount of collateral tokens to remove.
    /// The actual amount removed is capped to maintain the position's LTV at its Open LTV on the
    /// pool. Passing [`u64::MAX`] (or [`i128::MAX`]) effectively removes all available
    /// collateral
    pub fn remove_collateral(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        user.require_auth();

        let obligation_key = ObligationKey::new(user);

        process_remove_collateral(&e, &obligation_key, &pool_address, amount)
    }

    /// Repays borrowed tokens
    ///
    /// ### Arguments
    /// * `user` - user which repays borrowed tokens
    /// * `pool_address` - address of a pool from which the borrow happened
    /// * `amount` - provided amount of tokens to repay. If this amount exceeds the total debt, only
    ///   the outstanding debt will be repaid.
    /// Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to repay the entire debt
    pub fn repay(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        user.require_auth();

        let obligation_key = ObligationKey::new(user);

        process_repay(&e, &obligation_key, &pool_address, amount)
    }

    /// Liquidates borrower's position if position's health factor criterion isn't met
    ///
    /// ### Arguments
    /// * `liquidator` - agent which liquidates the borrower's position
    /// * `borrower` - the borrower whose position is being liquidated
    /// * `borrow_pool_address` - address of a pool whose borrowed tokens are repaid by the
    ///   liquidator
    /// * `collateral_pool_address` - address of a pool whose tokens are sold to the liquidator with
    ///   a discount
    /// * `amount` - amount of repaid tokens
    pub fn liquidate(
        e: Env,
        liquidator: Address,
        borrower: Address,
        borrow_pool_address: Address,
        collateral_pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        liquidator.require_auth();

        let borrower_obligation_key = ObligationKey::new(borrower);

        process_liquidate(
            &e,
            &liquidator,
            &borrower_obligation_key,
            &borrow_pool_address,
            &collateral_pool_address,
            amount,
        )
    }

    /// Withdraws deposited tokens from the loan pool to the user
    ///
    /// ### Arguments
    /// * `user` - user which withdraws deposited tokens
    /// * `pool_address` - address of a pool from which the withdrawal happens
    /// * `amount` - desired amount of tokens to withdraw.
    /// The actual amount withdrawn is capped to maintain the position's LTV at its Open LTV on the
    /// pool. Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to withdraw all tokens
    /// available for it
    pub fn withdraw(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        user.require_auth();

        let obligation_key = ObligationKey::new(user);

        process_withdraw(&e, &obligation_key, &pool_address, amount)
    }

    /// Creates a flash loan
    ///
    /// ### Arguments
    /// * `contract` - contract's address which leverages the flash loaned amount and adheres to
    ///   `erc3156` standard
    /// * `pool_address` - address of a pool from which the flash loan happens
    /// * `amount` - amount of lent tokens
    pub fn flash_loan(
        e: Env,
        contract: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        contract.require_auth();

        process_flash_loan(&e, &contract, &pool_address, amount)
    }

    pub fn clean_multiply_pairs(e: Env) {
        require_admin(&e);

        storage::remove_all_multiply_pairs(&e);
    }

    pub fn check_multiply_pair_exists(
        e: Env,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> bool {
        MultiplyPair::exists(&e, &deposit_pool_address, &borrow_pool_address)
    }

    /// Deposits tokens into the loan pool with leverage. Leverage is achieved by utilizing flash
    /// loan and token swap
    ///
    /// # WARNING
    /// This increases the perceived `supply APR` only
    /// when `(borrowed token borrow APR < supply token supply APR)` holds true
    ///
    /// ### Arguments
    /// * `user` - user that deposits tokens with leverage
    /// * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
    /// * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
    /// * `deposit_as_margin` - flag that determines which asset(deposited or borrowed) will be used
    ///   as the provided by the user initial margin amount
    /// * `amount` - original borrow amount before the leverage
    /// * `leverage_multiplier` - leverage multiplier, where the last two digits represent decimal
    ///   places (e.g., 700 for x7.00, 255 for x2.55, etc.)
    pub fn deposit_with_leverage(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        deposit_as_margin: bool,
        amount: i128,
        leverage_multiplier: u32,
    ) -> Result<(), MCError> {
        user.require_auth();

        process_deposit_with_leverage(
            &e,
            &user,
            &deposit_pool_address,
            &borrow_pool_address,
            deposit_as_margin,
            amount,
            leverage_multiplier,
        )
    }

    /// Withdraws tokens from the leveraged deposit position without affecting the leverage
    /// multiplier
    ///
    /// ### Arguments
    /// * `user` - user that deleverages and withdraws from the position
    /// * `deposit_pool_address` - address of a pool from the pair to which the deposit happened
    /// * `borrow_pool_address` - address of a pool from the pair from which the borrow happened
    /// * `amount` - desired amount of deposited tokens to withdraw.
    /// The actual amount withdrawn is capped by the value difference between deposited and borrowed
    /// tokens in the leveraged position (minus operational fees). Passing [`u64::MAX`] (or
    /// [`i128::MAX`]) can be used to withdraw all available tokens
    pub fn withdraw_from_leveraged(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
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
    pub fn get_pool_asset_oracle_price(e: Env, pool_address: Address) -> Result<i128, MCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        get_asset_price(&e, &pool.token_ticker)
    }

    /// Returns the user's obligation which includes data about all of their deposits and borrows
    ///
    /// ### Arguments
    /// * `user` - user which obligation is returned
    pub fn get_user_obligation(e: Env, user: Address) -> Result<Obligation, MCError> {
        let obligation_key = ObligationKey::new(user);
        let obligation = Obligation::try_get(&e, &obligation_key)?;

        obligation.accrue_interest(&e)?;
        obligation.set(&e, &obligation_key);

        Ok(obligation)
    }

    /// Returns the user's obligation for a specific multiply pair
    ///
    /// ### Arguments
    /// * `user` - user whose obligation is returned
    pub fn get_multiply_pair_obligation(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<Obligation, MCError> {
        let seed =
            Some(MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?.seed);
        let obligation_key = ObligationKey::new_with_seed(user, seed);
        let obligation = Obligation::try_get(&e, &obligation_key)?;

        Ok(obligation)
    }

    /// Accrues interest on a specific user's obligation and on its pools
    ///
    /// ### Arguments
    /// * `user` - user whose obligation interest is accrued
    pub fn accrue_interest(e: Env, user: Address) -> Result<(), MCError> {
        let obligation_key = ObligationKey::new(user);
        let obligation = Obligation::try_get(&e, &obligation_key)?;

        obligation.accrue_interest(&e)?;
        obligation.set(&e, &obligation_key);

        Ok(())
    }

    // TODO:
    // pub fn accrue_interest_on_multiply_pair_obligation() {}

    /// Returns the specific loan pool
    ///
    /// ### Arguments
    /// * `pool_address` - pool which data is returned
    pub fn get_pool(e: Env, pool_address: Address) -> Result<Pool, MCError> {
        let mut pool = Pool::try_get(&e, &pool_address)?;
        pool.accrue_interest(&e)?;

        Ok(pool)
    }

    /// Returns a list of all pool addresses in the protocol
    pub fn get_all_pools(e: Env) -> Vec<Address> {
        Pool::get_all(&e)
    }

    /// Returns a list of all user obligations in the protocol
    pub fn get_all_obligations(e: Env) -> Vec<ObligationKey> {
        Obligation::get_all(&e)
    }

    /// Returns the specific multiply pair
    ///
    /// ### Arguments
    /// * `deposit_pool_address` - deposit pool of a pair that is returned
    /// * `borrow_pool_address` - borrow pool of a pair that is returned
    pub fn get_multiply_pair(
        e: Env,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<MultiplyPair, MCError> {
        MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)
    }

    /// Returns a list of all multiply pairs registered for the market
    pub fn get_all_multiply_pairs(e: Env) -> Vec<MultiplyPair> {
        MultiplyPair::get_all(&e)
    }

    /// Returns APR calculated for the current utilization ratio of a pool in basis points (e.g.,
    /// 2912 = 29.12%, etc)
    ///
    /// ### Arguments
    /// * `pool_address` - address of a pool for which APR is returned
    pub fn get_apr(e: Env, pool_address: Address) -> Result<AnnualPercentageRates, MCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        pool.get_apr()
    }

    /// Returns APY calculated for the current utilization ratio of a pool in basis points (e.g.,
    /// 2912 = 29.12%, etc)
    ///
    /// ### Arguments
    /// * `pool_address` - address of a pool for which APY is returned
    pub fn get_apy(e: Env, pool_address: Address) -> Result<AnnualPercentageYields, MCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        pool.get_apy()
    }

    /// Resets the contract's storage. Useful when the contract's invariants are broken and require
    /// resetting on the testnet without re-deploying the contract
    pub fn reset_storage(e: Env) {
        require_admin(&e);

        storage::remove_all_obligations(&e);
        storage::remove_all_pools(&e);
        storage::remove_all_multiply_pairs(&e);
    }
}

// ---- Endpoints processors ----

fn process_initialize_pool(
    e: &Env,
    token_address: &Address,
    token_ticker: &Symbol,
    salt: &Option<BytesN<32>>,
    pool_config: &Option<PoolConfig>,
) -> Result<Address, MCError> {
    let pool_address: Address = if let Some(salt) = salt {
        e.deployer()
            .with_address(token_address.clone(), salt.clone())
            .deployed_address()
    } else {
        token_address.clone()
    };

    Pool::require_does_not_exist(e, &pool_address)?;

    let pool_config: PoolConfig = match pool_config {
        Some(cfg) => {
            if cfg.validate().is_err() {
                return Err(MCError::InvalidLoanPoolConfig);
            }

            *cfg
        }
        None => Default::default(),
    };

    let accrual_model = AccrualModel::Compounded;
    let interest_rate_model = InterestRateModel::Kinked(KinkedIRConfig::default());
    let name = TokenClient::new(e, token_address).name();

    let pool = Pool {
        total_d_tokens: 0,
        total_j_tokens: 0,
        total_borrowed: 0,
        total_available: 0,
        total_collateral: 0,

        name,
        accrual_model,
        interest_rate_model,
        config: pool_config,
        pool_address: pool_address.clone(),
        token_ticker: token_ticker.clone(),
        token_address: token_address.clone(),
        last_accrual_timestamp: e.ledger().timestamp(),
    };

    pool.set(e);
    pool.register(e);

    events::initialize_pool(e, token_address, &pool_address, token_ticker);

    Ok(pool_address)
}

pub fn process_initialize_multiply_pair(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<(), MCError> {
    MultiplyPair::require_does_not_exists(e, deposit_pool_address, borrow_pool_address)?;

    let (collateral_pool_liability_factor_bps, borrow_pool_open_ltv_bps) = (
        Pool::try_get(e, deposit_pool_address)
            .map_err(|_| MCError::DepositPoolDoesNotExist)?
            .config
            .liability_factor_bps,
        Pool::try_get(e, borrow_pool_address)
            .map_err(|_| MCError::BorrowPoolDoesNotExist)?
            .config
            .open_ltv_bps,
    );

    let pair = MultiplyPair::new(
        e,
        deposit_pool_address,
        borrow_pool_address,
        borrow_pool_open_ltv_bps,
        collateral_pool_liability_factor_bps,
    );

    pair.set(e);
    pair.register(e);

    Ok(())
}

fn process_deposit(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.accrue_interest(e)?;

    let supply_limit = pool.config.supply_limit;

    // NB: 0 indicates unlimited supply
    if supply_limit != 0 {
        let new_supply = pool
            .total_supply()?
            .checked_add(amount)
            .map_over_or_underflow()?;

        if new_supply > supply_limit {
            return Err(MCError::PoolSupplyLimitExceeded);
        }
    }

    let mut obligation =
        Obligation::try_get(e, obligation_key).unwrap_or(Obligation::new(e, obligation_key));

    let j_tokens_to_issue = pool.compute_j_tokens_from_tokens(e, amount)?;
    obligation.deposit(e, pool_address, j_tokens_to_issue, amount)?;

    pool.adjust_total_j_tokens(e, j_tokens_to_issue)?;
    pool.adjust_total_available(e, amount)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&obligation_key.user, &e.current_contract_address(), &amount);

    events::deposit(
        e,
        pool_address,
        &obligation_key.user,
        amount,
        j_tokens_to_issue,
    );

    Ok(())
}

fn process_borrow(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.accrue_interest(e)?;

    let max_healthy_borrow_added_amount =
        obligation.compute_max_healthy_debt_added_amount(e, &pool)?;
    let real_borrowed_amount = i128::min(max_healthy_borrow_added_amount, amount);

    pool.require_preserves_utilization_ratio_cap(e, real_borrowed_amount)?;

    let d_tokens_issued = pool.compute_d_tokens_from_tokens(e, real_borrowed_amount)?;
    obligation.borrow(e, pool_address, d_tokens_issued, real_borrowed_amount)?;

    pool.adjust_total_d_tokens(e, d_tokens_issued)?;
    pool.adjust_total_borrowed(e, real_borrowed_amount)?;
    pool.adjust_total_available(
        e,
        real_borrowed_amount.checked_neg().map_over_or_underflow()?,
    )?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(
        &e.current_contract_address(),
        &obligation_key.user,
        &real_borrowed_amount,
    );

    events::borrow(
        e,
        pool_address,
        &obligation_key.user,
        real_borrowed_amount,
        d_tokens_issued,
    );

    Ok(())
}

fn process_add_collateral(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut obligation =
        Obligation::try_get(e, obligation_key).unwrap_or(Obligation::new(e, obligation_key));

    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    obligation.add_collateral(e, pool_address, amount)?;
    pool.adjust_total_collateral(e, amount)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&obligation_key.user, &e.current_contract_address(), &amount);

    events::add_collateral(e, pool_address, &obligation_key.user, amount);

    Ok(())
}

fn process_repay(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;
    // NB: Accruing interest on an obligation must precede pool retrieval
    let mut pool = Pool::try_get(e, pool_address)?;

    let (real_repaid_amount, d_tokens_burnt) = obligation.repay(e, pool_address, &pool, amount)?;

    pool.adjust_total_d_tokens(e, d_tokens_burnt.checked_neg().map_over_or_underflow()?)?;
    pool.adjust_total_borrowed(e, real_repaid_amount.checked_neg().map_over_or_underflow()?)?;
    pool.adjust_total_available(e, real_repaid_amount)?;

    if obligation.is_empty() {
        // NB: Obligation shouldn't be empty at this point due to some amount of collateral or
        // deposit required to repay the debt
        events::obligation_is_unexpectedly_empty(e, obligation_key, pool_address);

        return Err(MCError::InternalError);
    }

    obligation.set(e, obligation_key);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(
        &obligation_key.user,
        &e.current_contract_address(),
        &real_repaid_amount,
    );

    events::repay(e, pool_address, obligation_key, real_repaid_amount);

    Ok(())
}

fn process_liquidate(
    e: &Env,
    liquidator: &Address,
    borrower_obligation_key: &ObligationKey,
    borrow_pool_address: &Address,
    collateral_pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    if *liquidator == borrower_obligation_key.user {
        // TODO: Can there any need for you to liquidate oneself?
        return Err(MCError::SelfLiquidation);
    }

    if borrow_pool_address == collateral_pool_address {
        // NB: Is this really a problem?
        return Err(MCError::LiquidationWithEqualCollateralAndDepositPools);
    }

    let mut obligation = Obligation::try_get(e, borrower_obligation_key)?;
    obligation.accrue_interest(e)?;

    obligation.require_non_healthy(e)?;

    let (mut borrow_pool, mut collateral_pool) = (
        Pool::try_get(e, borrow_pool_address).map_err(|_| MCError::BorrowPoolDoesNotExist)?,
        Pool::try_get(e, collateral_pool_address)
            .map_err(|_| MCError::CollateralPoolDoesNotExist)?,
    );

    // // TODO: Accrue interest on pools for consistency?
    // borrow_pool.accrue_interest(e)?;
    // collateral_pool.accrue_interest(e)?;

    let LiquidationValues {
        liquidated_amount,
        d_tokens_repaid,
        collateral_amount_sold,
        j_tokens_amount_sold,
        tokens_from_sold_j_tokens,
    } = obligation.liquidate(
        e,
        borrow_pool_address,
        collateral_pool_address,
        &borrow_pool,
        &collateral_pool,
        amount,
    )?;

    // Validate liquidation amounts to prevent zero or negative
    require_nonnegative(liquidated_amount)?;
    require_nonnegative(collateral_amount_sold)?;

    collateral_pool.adjust_total_available(
        e,
        tokens_from_sold_j_tokens
            .checked_neg()
            .map_over_or_underflow()?,
    )?;
    collateral_pool.adjust_total_j_tokens(
        e,
        j_tokens_amount_sold.checked_neg().map_over_or_underflow()?,
    )?;
    borrow_pool
        .adjust_total_borrowed(e, liquidated_amount.checked_neg().map_over_or_underflow()?)?;
    borrow_pool.adjust_total_d_tokens(e, d_tokens_repaid.checked_neg().map_over_or_underflow()?)?;

    collateral_pool.adjust_total_collateral(
        e,
        collateral_amount_sold
            .checked_neg()
            .map_over_or_underflow()?,
    )?;

    obligation.set(e, borrower_obligation_key);

    collateral_pool.set(e);
    borrow_pool.set(e);

    let borrowed_token_client = token::Client::new(e, &borrow_pool.token_address);
    borrowed_token_client.transfer(
        liquidator,
        &e.current_contract_address(),
        &liquidated_amount,
    );

    let collateral_seized_amount = tokens_from_sold_j_tokens
        .checked_add(collateral_amount_sold)
        .map_over_or_underflow()?;

    let collateral_token_client = token::Client::new(e, &collateral_pool.token_address);
    collateral_token_client.transfer(
        &e.current_contract_address(),
        liquidator,
        &collateral_seized_amount,
    );

    events::liquidate(
        e,
        liquidator,
        borrower_obligation_key,
        borrow_pool_address,
        collateral_pool_address,
        liquidated_amount,
        collateral_seized_amount,
    );

    Ok(())
}

fn process_remove_collateral(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let max_possible_collateral_removed_amount =
        obligation.compute_max_healthy_collateral_removed_amount(e, &pool)?;

    let removed_tokens_amount = i128::min(
        i128::min(amount, max_possible_collateral_removed_amount),
        obligation.get_collateral(pool_address)?,
    );

    obligation.remove_collateral(e, pool_address, removed_tokens_amount)?;
    pool.adjust_total_collateral(e, -removed_tokens_amount)?;

    if obligation.is_empty() {
        obligation.remove(e, obligation_key);
    } else {
        obligation.set(e, obligation_key);
    }
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(
        &e.current_contract_address(),
        &obligation_key.user,
        &removed_tokens_amount,
    );

    events::remove_collateral(e, pool_address, &obligation_key.user, removed_tokens_amount);

    Ok(())
}

fn process_withdraw(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    // NB: Accruing interest on an obligation must precede pool retrieval
    obligation.accrue_interest(e)?;
    let mut pool = Pool::try_get(e, pool_address)?;

    let max_healthy_withdrawn_amount =
        obligation.compute_max_healthy_collateral_removed_amount(e, &pool)?;
    let real_withdrawn_amount = i128::min(amount, max_healthy_withdrawn_amount);

    pool.require_preserves_utilization_ratio_cap(e, real_withdrawn_amount)?;
    // pool.require_available(real_withdrawn_amount)?; // TODO: Should we keep this check?

    let j_tokens_burnt = pool.compute_j_tokens_from_tokens(e, real_withdrawn_amount)?;

    obligation.withdraw(
        e,
        &pool,
        pool_address,
        j_tokens_burnt,
        real_withdrawn_amount,
    )?;

    pool.adjust_total_available(
        e,
        real_withdrawn_amount
            .checked_neg()
            .map_over_or_underflow()?,
    )?;
    pool.adjust_total_j_tokens(e, j_tokens_burnt.checked_neg().map_over_or_underflow()?)?;

    if obligation.is_empty() {
        obligation.remove(e, obligation_key);
    } else {
        obligation.set(e, obligation_key);
    }
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(
        &e.current_contract_address(),
        &obligation_key.user,
        &real_withdrawn_amount,
    );

    events::withdraw(
        e,
        pool_address,
        obligation_key,
        j_tokens_burnt,
        real_withdrawn_amount,
    );

    Ok(())
}

fn process_flash_loan(
    e: &Env,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let pool = Pool::try_get(e, pool_address)?;
    pool.require_available(amount)?;

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), contract, &amount); // plain transfer, not allowance transfer?

    let flash_loan_taker_client = FlashLoanClient::new(e, contract);
    flash_loan_taker_client.exec_op(
        &e.current_contract_address(),
        &pool.token_address,
        &amount,
        &DEFAULT_FLASH_LOAN_FEE_BPS,
    );

    // WARN: Does this have enough precision?
    let fees = amount
        .fixed_mul_floor(DEFAULT_FLASH_LOAN_FEE_BPS, BPS_FACTOR)
        .map_over_or_underflow()?;
    let amount_to_repay = amount.checked_add(fees).map_over_or_underflow()?;

    // NB: Here you must issue `transfer_allowance`, since it's safer for a flash loan taker to
    // to implement their flash loan logic flow
    token_client.transfer(contract, &e.current_contract_address(), &amount_to_repay);

    events::flash_loan(e, contract, pool_address, amount, fees);

    Ok(())
}

fn process_deposit_with_leverage(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    deposit_as_margin: bool,
    amount: i128,
    leverage_multiplier: u32,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let pair = MultiplyPair::try_get(e, deposit_pool_address, borrow_pool_address)?;
    pair.require_valid_leverage_multiplier(leverage_multiplier)?;

    let (deposit_pool, mut borrow_pool) = (
        Pool::try_get(e, deposit_pool_address).map_err(|_| {
            events::pool_is_missing_in_storage(e, deposit_pool_address);

            MCError::InternalError
        })?,
        Pool::try_get(e, borrow_pool_address).map_err(|_| {
            events::pool_is_missing_in_storage(e, borrow_pool_address);

            MCError::InternalError
        })?,
    );

    //  -- Calculate parameters --
    let leverage_multiplier_minus_1 = leverage_multiplier - LEVERAGE_SCALE; // safe
    let (flash_borrow_amount, amount_in, amount_out) = if deposit_as_margin {
        // ----
        // 'flash_borrow_amount' = 'amount_in' you need to get the base_leverage as 'amount_out'
        // after swap
        // 'amount_in' = flash_borrow_amount
        // 'amount_out' = base_leverage
        // ----
        let scaled_base_leverage_amount = amount
            .checked_mul(leverage_multiplier_minus_1 as i128)
            .map_over_or_underflow()?;
        let base_leverage_amount = scaled_base_leverage_amount / (LEVERAGE_SCALE as i128); // safe

        // Calculate the flash borrow amount
        let amount_out = base_leverage_amount;
        let amount_in = swap::get_amount_in(
            e,
            &borrow_pool.token_address,
            &deposit_pool.token_address,
            amount_out,
        )?;
        let flash_borrow_amount = amount_in;

        (flash_borrow_amount, amount_in, amount_out)
    } else {
        // ----
        // 'flash_borrow_amount' = amount * (leverage_multiplier - 1)
        // 'amount_in' = amount + flash_borrow_amount
        // 'amount_out' = 'amount_out' you get after swapping 'amount_in'
        // ----
        let scaled_flash_borrow_amount = amount
            .checked_mul(leverage_multiplier_minus_1 as i128)
            .map_over_or_underflow()?;

        let flash_borrow_amount = scaled_flash_borrow_amount / (LEVERAGE_SCALE as i128); // safe
        let amount_in = amount
            .checked_add(flash_borrow_amount)
            .map_over_or_underflow()?;
        let amount_out = swap::get_amount_out(
            e,
            &borrow_pool.token_address,
            &deposit_pool.token_address,
            amount_in,
        )?;

        (flash_borrow_amount, amount_in, amount_out)
    };

    // -- Flash Borrow --
    // TODO: Think of why it can be beneficial to account for flash borrow limits as in other
    //  lending protocols
    borrow_pool.require_available(flash_borrow_amount)?;

    // TODO: Check, why on blend_v2 they use 'token_client.transfer_allowance' instead
    //  of 'token_client.transfer' for flash loans
    let flash_loan_token_client = token::Client::new(e, &borrow_pool.token_address);
    flash_loan_token_client.transfer(&e.current_contract_address(), user, &flash_borrow_amount);

    borrow_pool.adjust_total_available(e, -flash_borrow_amount)?;
    // NB: This `set` is required, since 'available' amount is later accounted when calling
    // `process_borrow`
    borrow_pool.set(e);

    // -- Swap --
    // NB: Since both 'amount_in' and 'amount_out' are calculated in the current contract call,
    // no slippage will take place and 'swap_exact_tokens_for_tokens' and
    // 'swap_tokens_for_exact_tokens' aren't different. This can likely be adjusted when
    // implementing safety mechanisms that account for slippage when depositing with leverage
    // via UI
    let received_amount = swap::swap_exact_tokens_for_tokens(
        e,
        user,
        &borrow_pool.token_address,
        &deposit_pool.token_address,
        amount_in,
        amount_out,
        Some(0),
    )?;

    // NB: For some reason, Soroswap adds 1 to `amount_in` in `router_get_amounts_in`.
    // See: https://github.com/soroswap/core/blob/c157c676d189a39ab8a3869a1011d3a259c36e22/contracts/library/src/quotes.rs#L77.
    // This is likely the reason why 'received_amount' is bigger than 'amount_out', which is not an
    // issue, though
    if received_amount < amount_out {
        events::received_unexpected_swap_amount(
            e,
            user,
            &borrow_pool.token_address,
            &deposit_pool.token_address,
            amount_in,
            received_amount,
            amount_in,
            amount_out,
        );

        return Err(MCError::DependencyContractError);
    }

    // -- Deposit swapped tokens --
    let deposit_amount = if deposit_as_margin {
        received_amount
            .checked_add(amount)
            .map_over_or_underflow()?
    } else {
        received_amount
    };

    let seed = Some(pair.seed.clone());
    let obligation_key = ObligationKey::new_with_seed(user.clone(), seed);

    process_deposit(e, &obligation_key, deposit_pool_address, deposit_amount)?;

    // -- Borrow to repay the flash loan --
    let flash_loan_fee = flash_borrow_amount
        .fixed_mul_ceil(DEFAULT_FLASH_LOAN_FEE_BPS, BPS_FACTOR)
        .map_over_or_underflow()?;
    let flash_repay_amount = flash_borrow_amount
        .checked_add(flash_loan_fee)
        .map_over_or_underflow()?;

    let obligation = Obligation::try_get(e, &obligation_key).map_err(|_| {
        events::obligation_is_missing_in_storage(e, user);

        MCError::InternalError
    })?;

    let max_healthy_borrow_amount =
        obligation.compute_max_healthy_debt_added_amount(e, &borrow_pool)?;

    if flash_repay_amount > max_healthy_borrow_amount {
        // TODO: Add an event
        // events::leverage_borrow_exceeds_healthy_limit(e, user,
        // flash_repay_amount,max_healthy_borrow_amount);
        // return Err(MCError::BorrowLimitExceeded);
        return Err(MCError::InternalError);
    }

    // NB: Notice that we 'flash borrow' and 'borrow' to repay the flash loan from the
    // same pool here. Can this be somehow utilized?
    // We for sure must get the `flash borrow` amount in order to swap, right?
    // After that, we borrow it to repay in the same pool, which seems redundant.
    // This approach, though, has as the advantage that we utilize `process_borrow`,
    // so, maybe, it's better to leave it as it is now

    process_borrow(e, &obligation_key, borrow_pool_address, flash_repay_amount)?;
    borrow_pool.refresh(e)?;

    // Repay the flash loan
    flash_loan_token_client.transfer(user, &e.current_contract_address(), &flash_repay_amount);

    borrow_pool.adjust_total_available(e, flash_repay_amount)?;
    borrow_pool.set(e);

    events::deposit_with_leverage(
        e,
        user,
        // TODO: seed: ...
        deposit_pool_address,
        borrow_pool_address,
        amount,
        leverage_multiplier,
        deposit_amount,
        flash_borrow_amount,
    );

    Ok(())
}

// TODO: adjust_leverage() {}

pub fn process_withdraw_from_leveraged(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let (mut borrow_pool, mut deposit_pool) = (
        Pool::try_get(e, borrow_pool_address).map_err(|_| MCError::BorrowPoolDoesNotExist)?,
        Pool::try_get(e, deposit_pool_address).map_err(|_| MCError::DepositPoolDoesNotExist)?,
    );

    borrow_pool.accrue_interest(e)?;
    deposit_pool.accrue_interest(e)?;

    let pair = MultiplyPair::try_get(e, deposit_pool_address, borrow_pool_address)?;
    let seed = Some(pair.seed.clone());

    let obligation_key = ObligationKey::new_with_seed(user.clone(), seed);

    let obligation = Obligation::try_get(e, &obligation_key)?;
    let total_debt = obligation.get_total_debt(e, borrow_pool_address)?;

    if total_debt == 0 {
        // NB: No leverage case is equivalent to a simple deposit
        return process_withdraw(e, &obligation_key, deposit_pool_address, amount);
    }

    let obligation_j_tokens = obligation.get_j_tokens(deposit_pool_address)?;
    let deposited_tokens = deposit_pool.compute_tokens_from_j_tokens(e, obligation_j_tokens)?;

    // Compute the max withdrawable amount
    let max_withdrawable_amount = compute_leveraged_position_max_withdrawable_amount(
        e,
        user,
        &deposit_pool.token_address,
        &borrow_pool.token_address,
        deposited_tokens,
        total_debt,
    )?;

    let expected_withdrawn_amount = i128::min(amount, max_withdrawable_amount);

    // Compute the flash borrow amount for deleverage
    let withdrawn_ratio_bps = expected_withdrawn_amount
        .fixed_div_floor(max_withdrawable_amount, BPS_FACTOR)
        .map_over_or_underflow()?;

    let plain_leverage_amount = deposited_tokens
        .checked_sub(max_withdrawable_amount)
        .map_over_or_underflow()?;
    let plain_leverage_to_be_withdrawn = plain_leverage_amount
        .fixed_mul_floor(withdrawn_ratio_bps, BPS_FACTOR)
        .map_over_or_underflow()?;

    // To maintain LTV for the leveraged position, the amount of borrowed tokens to be repaid
    // must be proportional to the withdrawn amount of the deposited tokens
    let flash_borrow_amount = total_debt
        .fixed_mul_floor(withdrawn_ratio_bps, BPS_FACTOR)
        .map_over_or_underflow()?;

    borrow_pool.require_available(flash_borrow_amount)?;

    // Flash Borrow
    let flash_borrowed_token_client = token::Client::new(e, &borrow_pool.token_address);
    flash_borrowed_token_client.transfer(&e.current_contract_address(), user, &flash_borrow_amount);
    borrow_pool.adjust_total_available(e, -flash_borrow_amount)?;
    borrow_pool.set(e);

    // Repay Debt
    process_repay(e, &obligation_key, borrow_pool_address, flash_borrow_amount)?;
    borrow_pool.refresh(e)?;

    // Withdraw
    let withdrawn_amount = expected_withdrawn_amount
        .checked_add(plain_leverage_to_be_withdrawn)
        .map_over_or_underflow()?;

    process_withdraw(e, &obligation_key, deposit_pool_address, withdrawn_amount)?;
    deposit_pool.refresh(e)?;

    // Swap to get the flash repay amount
    let flash_loan_fee = flash_borrow_amount
        .fixed_mul_ceil(DEFAULT_FLASH_LOAN_FEE_BPS, BPS_FACTOR)
        .map_over_or_underflow()?;

    let flash_repay_amount = flash_loan_fee
        .checked_add(flash_borrow_amount)
        .map_over_or_underflow()?;

    let amount_in = swap::get_amount_in(
        e,
        &deposit_pool.token_address,
        &borrow_pool.token_address,
        flash_repay_amount,
    )?;
    swap::swap_tokens_for_exact_tokens(
        e,
        user,
        &deposit_pool.token_address,
        &borrow_pool.token_address,
        amount_in,
        flash_repay_amount,
        Some(0),
    )?;

    // Flash Repay
    flash_borrowed_token_client.transfer(user, &e.current_contract_address(), &flash_repay_amount);

    borrow_pool.adjust_total_available(e, flash_repay_amount)?;
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

#[allow(unused)]
fn process_swap_for_exact_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_out: i128,
) -> Result<i128, MCError> {
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
) -> Result<i128, MCError> {
    require_nonnegative(amount_in)?;

    // Since `amount_out` is calculated within the call, there's no price slippage
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

// ---- Helpers ----

// WARN: will everything be ok here with precision?
fn compute_leveraged_position_max_withdrawable_amount(
    e: &Env,
    user: &Address,
    deposited_token: &Address,
    borrowed_token: &Address,
    deposited_amount: i128,
    borrowed_amount: i128,
) -> Result<i128, MCError> {
    require_nonnegative(deposited_amount)?;
    require_nonnegative(borrowed_amount)?;

    let flash_loan_fee = borrowed_amount
        .fixed_mul_ceil(DEFAULT_FLASH_LOAN_FEE_BPS, BPS_FACTOR)
        .map_over_or_underflow()?;
    let flash_repay_amount = borrowed_amount
        .checked_add(flash_loan_fee)
        .map_over_or_underflow()?;

    let deposit_tokens_to_repay_flash_loan =
        swap::get_amount_in(e, deposited_token, borrowed_token, flash_repay_amount)?;

    if deposit_tokens_to_repay_flash_loan > deposited_amount {
        // WARN: This can happen when multiply position contains a bad debt
        events::leveraged_position_bad_debt(
            e,
            user,
            deposited_token,
            borrowed_token,
            deposited_amount,
            borrowed_amount,
            deposit_tokens_to_repay_flash_loan,
        );

        // TODO: This has to be thought of when implementing security mechanisms
        return Err(MCError::InternalError);
    }

    Ok(deposited_amount - deposit_tokens_to_repay_flash_loan) // safe
}

pub fn get_asset_price(e: &Env, ticker: &Symbol) -> Result<i128, MCError> {
    let oracle_address = storage::get_oracle_address(e);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);

    let asset = Asset::Other(ticker.clone());

    let price_data = oracle_contract
        .lastprice(&asset)
        .ok_or(MCError::OracleDoesNotKnowAssetPrice)?;

    // TODO: Add sanity checks? I.e., a price range for the asset to be considered adequate
    require_nonnegative(price_data.price)?;

    // Validate price is not too old and not from the future
    let now = e.ledger().timestamp();
    let age = now.saturating_sub(price_data.timestamp);
    if age > MAX_ORACLE_PRICE_AGE_SECONDS || price_data.timestamp > now {
        return Err(MCError::OracleStalePrice);
    }

    Ok(price_data.price)
}

pub fn get_oracle_price_decimals(e: &Env) -> u32 {
    let oracle_address = storage::get_oracle_address(e);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);

    oracle_contract.decimals()
}

#[inline(always)]
fn require_nonnegative(amount: i128) -> Result<(), MCError> {
    if amount < 0 {
        return Err(MCError::NegativeAmount);
    }

    Ok(())
}

#[inline(always)]
fn require_admin(e: &Env) {
    let admin = storage::get_global_state(e).admin;
    admin.require_auth();
}
