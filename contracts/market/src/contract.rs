// use aggregated_oracle::PriceFeedClient; // TODO: Check why this breaks WASM build
use soroban_sdk::{Address, BytesN, Env, Map, String, Symbol, Vec, contract, contractimpl};

use crate::{
    error::MCError,
    events,
    helpers::require_admin,
    interest_rate::{AnnualPercentageRates, AnnualPercentageYields},
    multiply_pair::MultiplyPair,
    obligation::{Obligation, ObligationKey},
    oracle::{get_asset_price, get_oracle_price_decimals},
    pool::{Pool, PoolConfig},
    processors::*,
    storage::{self, GlobalState, get_global_state},
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
        deployer: Address,
    ) -> Result<(), MCError> {
        let global_state = GlobalState {
            // TODO: Introduce different market statuses
            status: true,
            admin: admin.clone(),
            name: name.clone(),
            deployer: deployer.clone(),
        };

        storage::set_global_state(&e, &global_state);
        storage::set_oracle_address(&e, &oracle);

        events::constructor(&e, &admin, &name, &oracle);

        Ok(())
    }

    /// Upgrades the lending contract
    ///
    /// ### Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
    ///   version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        // TODO: Implement decentralized governance of the contract
        // or remove this at some point after mainnet deployment
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

    /// Redeems accumulated market fees
    ///
    /// ### Arguments
    /// * `user` - user that tries to redeem market fees
    /// * `pool_address` - address of a pool whose fees are redeemed
    /// * `amount` - desired amount of fees to redeem as tokens
    pub fn redeem_accumulated_market_fees(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        let admin = get_global_state(&e).admin;
        admin.require_auth();

        process_redeem_accumulated_market_fees(&e, &user, &pool_address, amount)
    }

    /// Redeems accumulated host fees
    ///
    /// ### Arguments
    /// * `user` - user that tries to redeem host fees
    /// * `pool_address` - address of a pool whose fees are redeemed
    /// * `amount` - desired amount of fees to redeem as tokens
    pub fn redeem_accumulated_host_fees(
        e: Env,
        user: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        let host = get_global_state(&e).deployer;
        host.require_auth();

        process_redeem_accumulated_host_fees(&e, &user, &pool_address, amount)
    }

    /// Covers fully or partially bad debt if it exists under a user obligation. Socializes all
    /// remaining bad debt in case the market reserves doesn't contain enough funds to cover it
    /// completely
    ///
    /// ### Arguments
    /// * `bad_debt_obligation_user` - user that has a bad debt
    pub fn cover_obligation_bad_debt(
        e: Env,
        bad_debt_obligation_user: Address,
    ) -> Result<(), MCError> {
        let obligation_key = ObligationKey::new(bad_debt_obligation_user);

        process_cover_obligation_bad_debt_and_socialize_any_remaining_loss(&e, obligation_key)?;

        Ok(())
    }

    /// Covers fully or partially bad debt if it exists under a multiply pair user obligation.
    /// Socializes all remaining bad debt in case the reserve doesn't contain enough funds to
    /// cover it completely
    ///
    /// ### Arguments
    /// * `bad_debt_obligation_user` - user that has a bad debt
    /// * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
    /// * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
    pub fn cover_multiply_pair_bad_debt(
        e: Env,
        bad_debt_obligation_user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError> {
        let mp_seed = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?.seed;
        let obligation_key = ObligationKey::new_with_seed(bad_debt_obligation_user, mp_seed);

        process_cover_obligation_bad_debt_and_socialize_any_remaining_loss(&e, obligation_key)?;

        Ok(())
    }

    /// Returns asset's decimals
    pub fn get_asset_decimals() -> u32 {
        // See - <https://github.com/stellar/rs-soroban-env/blob/main/soroban-env-host/src/builtin_contracts/stellar_asset_contract/contract.rs#L374>
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

        get_asset_price(&e, &pool.token_address)
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
    /// * `deposit_pool_address` - address of a deposit pool from the pair
    /// * `borrow_pool_address` - address of a borrow pool from the pair
    pub fn get_multiply_pair_obligation(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<Obligation, MCError> {
        let mp_seed = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?.seed;
        let obligation_key = ObligationKey::new_with_seed(user, mp_seed);
        let obligation = Obligation::try_get(&e, &obligation_key)?;

        obligation.accrue_interest(&e)?;
        obligation.set(&e, &obligation_key);

        Ok(obligation)
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
    pub fn get_all_obligations(e: Env) -> Vec<Address> {
        let obligations_map = Obligation::get_all(&e);

        let mut obligations_vec = Vec::new(&e);
        for (obligation_addr, _) in obligations_map {
            obligations_vec.push_back(obligation_addr.user)
        }

        obligations_vec
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
        // TODO: This method is
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
