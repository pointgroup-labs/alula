#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    Address, BytesN, Env, Map, String, Vec, contract, contractclient, contractimpl, token,
    vec as svec,
};

use crate::{
    constants::*,
    error::MCError,
    events, farms,
    multiply_pair::MultiplyPair,
    obligation::{Obligation, ObligationKey, WithdrawResult},
    oracle,
    pool::{Pool, PoolConfig},
    processors::*,
    request::Request,
    storage::{self, GlobalState, MarketStatus, PoolUpdate},
    utils::{
        MarketData, PoolData, require_admin, require_borrows_on_market_allowed,
        require_deposits_on_market_allowed, require_insurance_fund, require_market_not_frozen,
        require_nonnegative, require_owned_and_admin,
    },
};

#[contractclient(name = "MarketClient")]
pub trait Market {
    // Submits a request batch
    fn submit_requests_batch(
        e: Env,
        obligation_key: ObligationKey,
        requests: Vec<Request>,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Gets the contract's global state
    fn get_global_state(e: Env) -> GlobalState;

    // Updates the owned market's parameters
    //
    // # Arguments
    // * `new_max_positions` - updated maximum number of positions that a single obligation can have
    // * `new_min_collateral_value_cents` - updated minimum collateral allowed
    fn update_market(
        e: Env,
        new_max_positions: u32,
        new_min_collateral_value_cents: i128,
    ) -> Result<(), MCError>;

    // Updates the market status
    //
    // # Arguments
    // * `new_status` - numerical representation of the new market status
    fn update_market_status(e: Env, new_status: u32) -> Result<(), MCError>;

    // Updates the market status from the Insurance Fund contract
    //
    // # Arguments
    // * `new_status` - numerical representation of the new market status
    //
    // # Panics
    // If the Fund contract hasn't authorized the call
    fn fund_update_market_status(e: Env, new_status: u32) -> Result<(), MCError>;

    // Initializes a loan pool for a specific asset
    //
    // # Arguments
    // * `token_address` - address of a corresponding Soroban Asset Contract
    // * `token_ticker` - symbol which represents a pool's token ticker
    // * `salt` - optional salt data, which, when provided, is used along with `token_address` to
    //   derive a deterministic pool address
    // * `pool_config` - optional `PoolConfig` data. If not provided, a default pool config is used
    fn initialize_pool(
        e: Env,
        token_address: Address,
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>,
    ) -> Result<Address, MCError>;

    // Initializes a multiply pair
    //
    // # Arguments
    // * `deposit_pool_address` - address of a pool in a pair for a leveraged deposit
    // * `borrow_pool_address` - address of a pool in a pair for a leveraged borrow
    fn initialize_multiply_pair(
        e: Env,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError>;

    // Queues in pool's config update
    //
    // # Arguments
    // * `pool_address` - address of a pool to which the update is queued in
    // * `new_pool_config` - updated pool config
    fn queue_in_pool_config_update(
        e: Env,
        pool_address: Address,
        new_pool_config: PoolConfig,
    ) -> Result<(), MCError>;

    // Cancels pool's config update if it exists in the update queue
    //
    // # Arguments
    // * `pool_address` - address of a pool to which the update is being canceled
    fn cancel_pool_config_update(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Applies the pool's config update if it exists in a queue and has completed its queue period
    //
    // # Arguments
    // * `pool_address` - address of a pool to which the config update is being applied
    fn apply_pool_config_update(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Gets the pool's config update from the queue if it exists
    //
    // # Arguments
    // * `pool_address` - address of a pool, for which the config update is received
    fn get_pool_config_queued_in_update(
        e: Env,
        pool_address: Address,
    ) -> Result<PoolUpdate, MCError>;

    // Set the `take rate` fees beneficiaries list. Shares(in basis points) must add up to 100%
    //
    // # Arguments
    // * `pool_address` - address of a pool, for which the beneficiaries list is set
    // * `beneficiaries` - a list of beneficiaries' addresses and their shares(in basis points)
    fn set_take_rate_fees_beneficiaries(
        e: Env,
        pool_address: Address,
        beneficiaries: Map<Address, u32>,
    ) -> Result<(), MCError>;

    // Set the `operation` fees beneficiaries list. Shares(in basis points) must add up to 100%
    //
    // # Arguments
    // * `pool_address` - address of a pool, for which the beneficiaries list is set
    // * `beneficiaries` - a list of beneficiaries' addresses and their shares(in basis points)
    fn set_operation_fees_beneficiaries(
        e: Env,
        pool_address: Address,
        beneficiaries: Map<Address, u32>,
    ) -> Result<(), MCError>;

    // Incentivizes a pool's supply with a donated asset amount for a defined period of time. Useful for bootstrapping pools
    // after deployment
    fn bootstrap_pool(
        e: Env,
        pool_address: Address,
        sponsor: Address,
        amount: i128,
        start_period: u64,
        end_period: u64,
    ) -> Result<(), MCError>;

    // Deposits tokens into the loan pool
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    // * `pool_address` - address of a pool to which the deposit happens
    // * `amount` - amount of tokens which are going to be deposited
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn deposit(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Deposits tokens into the loan pool with leverage. Leverage is achieved by utilizing flash
    // loan and token swap
    //
    // # WARNING
    // This increases the perceived `supply APR` only for favorable supply and borrow APRs
    // on deposited and borrowed tokens respectively
    //
    // # Arguments
    // * `user` - user that deposits tokens with leverage
    // * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
    // * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
    // * `deposit_as_margin` - flag that determines which asset(deposited or borrowed) will be used
    //   as the provided by the user initial margin amount
    // * `amount` - original borrow amount before the leverage
    // * `leverage_multiplier` - leverage multiplier, where the last two digits represent decimal
    //   places (e.g., 700 for x7.00, 255 for x2.55, etc.)
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    #[allow(clippy::too_many_arguments)]
    fn deposit_with_leverage(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        deposit_as_margin: bool,
        amount: i128,
        leverage_multiplier: u32,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Withdraws deposited tokens from the loan pool to the user
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    // * `pool_address` - address of a pool from which the withdrawal happens
    // * `amount` - desired amount of tokens to withdraw.
    //   The actual amount withdrawn is capped to maintain the position's LTV at its Open LTV on the
    //   pool. Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to withdraw all tokens
    //   available for it
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn withdraw(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Simulates withdrawal of the deposited tokens from the loan pool to the user
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the 'withdraw' operation
    // * `pool_address` - address of a pool from which the withdrawal happens
    // * `amount` - desired amount of tokens to withdraw.
    //   The actual amount withdrawn is capped to maintain the position's LTV at its Open LTV on the
    //   pool. Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to withdraw all tokens
    //   available for it
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    //
    // # Returns
    // [`WithdrawResult`] with simulated withdrawal data
    fn simulate_withdraw(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<WithdrawResult, MCError>;

    // Withdraws tokens from the leveraged deposit position without affecting the leverage
    // multiplier
    //
    // # Arguments
    // * `user` - user that deleverages and withdraws from the position
    // * `deposit_pool_address` - address of a pool from the pair to which the deposit happened
    // * `borrow_pool_address` - address of a pool from the pair from which the borrow happened
    // * `amount` - desired amount of tokens to receive in the user wallet
    //   The actual amount withdrawn is capped by the value difference between deposited and borrowed
    //   tokens in the leveraged position (minus operational fees). Passing [`u64::MAX`] (or
    //   [`i128::MAX`]) can be used to withdraw all available tokens
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn withdraw_from_leveraged(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Borrows tokens from the loan pool
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    // * `pool_address` - address of a pool from which the borrow happens
    // * `amount` - amount of tokens which are going to be borrowed
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn borrow(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Adds tokens into the loan pool as collateral only.
    // This implies that they are always available for a healthy withdrawal for the
    // cost of not accruing an interest rate
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    // * `pool_address` - address of a pool to which the collateral is being added
    // * `amount` - amount of tokens which are being added as a collateral
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn add_collateral(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Removes collateral tokens from the loan pool to the user
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    // * `pool_address` - address of a pool from which the withdrawal happens
    // * `amount` - desired amount of collateral tokens to remove.
    //   The actual amount removed is capped to maintain the position's LTV at its Open LTV on the
    //   pool. Passing [`u64::MAX`] (or [`i128::MAX`]) effectively removes all available
    //   collateral
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn remove_collateral(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Repays borrowed tokens
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    // * `pool_address` - address of a pool from which the borrow happened
    // * `amount` - provided amount of tokens to repay. If this amount exceeds the total debt, only
    //   the outstanding debt will be repaid.
    //   Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to repay the entire debt
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn repay(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Liquidates the borrower's position if the position's health factor criterion isn't met
    //
    // # Arguments
    // * `liquidator` - agent that liquidates the borrower's position
    // * `borrower_obligation_key` - key of an obligation that's liquidated
    // * `borrow_pool_address` - address of a pool whose borrowed tokens are repaid by the
    //       liquidator
    // * `collateral_pool_address` - address of a pool whose tokens are sold to the liquidator with
    //       a discount
    // * `repay_amount` - amount of repaid tokens
    // * `demanded_collateral_amount` - min amount of collateral that liquidator finds sufficient for the amount of debt repaid
    #[allow(clippy::too_many_arguments)]
    fn liquidate(
        e: Env,
        liquidator: Address,
        borrower_obligation_key: ObligationKey,
        borrow_pool_address: Address,
        collateral_pool_address: Address,
        repay_amount: i128,
        demanded_collateral_amount: i128,
    ) -> Result<(), MCError>;

    // Creates a flash loan
    //
    // # Arguments
    // * `contract` - contract's address which leverages the flash loaned amount and adheres to
    //   `erc3156` standard
    // * `caller` - flash loan caller
    // * `pool_address` - address of a pool from which the flash loan happens
    // * `amount` - amount of lent tokens
    fn erc3156_flash_loan(
        e: Env,
        contract: Address,
        caller: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError>;

    // -- TO BE REMOVED --

    // Donates tokens to `total_available` on a pool
    //
    // # Arguments
    // * `user` - user that donates tokens
    // * `pool_address` - address of a pool to whose reserve the donation takes place
    // * `amount` - donation amount
    fn donate(e: Env, user: Address, pool_address: Address, amount: i128) -> Result<(), MCError>;

    // Issues `cover bad debt` requests on every bad debt borrow position on the user's obligation to the Insurance Fund contract
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    fn issue_cover_bad_debt(e: Env, obligation_key: ObligationKey) -> Result<(), MCError>;

    // Issues `cover bad debt` requests on a bad debt borrow position on the user's multiply pair obligation to the Insurance Fund contract
    //
    // # Arguments
    // * `user` - user that has a bad debt
    // * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
    // * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
    fn issue_cover_bad_debt_pair(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError>;

    // Claims `cover bad debt` requests results for the user's obligation from the Insurance Fund if they exist
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    fn claim_cover_bad_debt_results(e: Env, obligation_key: ObligationKey) -> Result<(), MCError>;

    // Claims `cover bad debt` request's result for the user's multiply pair obligation from the Insurance Fund if it exists
    //
    // # Arguments
    // * `user` - user that has an open `cover bad debt` requests
    // * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
    // * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
    fn claim_cover_bad_debt_result_pair(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError>;

    // Distributes a pool's fees to the beneficiaries
    fn distribute_pool_fees(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Distributes all pools' fees to the beneficiaries
    fn distribute_all_pools_fees(e: Env) -> Result<(), MCError>;

    // Returns asset's decimals
    fn get_asset_decimals() -> u32;

    // Returns oracle price's decimals
    fn get_oracle_price_decimals(e: Env) -> u32;

    // Returns pool asset's oracle price
    //
    // # Arguments
    // * `pool_address` - address of asset which price is returned
    fn get_pool_asset_oracle_price(e: Env, pool_address: Address) -> Result<i128, MCError>;

    // Returns the user's obligation which includes data about all of their deposits and borrows
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    fn get_user_obligation(e: Env, obligation_key: ObligationKey) -> Result<Obligation, MCError>;

    // Accrues interest on all pools to whose obligation has open positions
    fn refresh_obligation(e: Env, obligation_key: ObligationKey) -> Result<(), MCError>;

    // Accrues interest on all pools to whose multiply pair obligation has open positions
    fn refresh_multiply_pair_obligation(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError>;

    // Accrues interest on a pool
    fn refresh_pool(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Returns the user's obligation for a specific multiply pair
    //
    // # Arguments
    // * `user` - user whose obligation is returned
    // * `deposit_pool_address` - address of a deposit pool from the pair
    // * `borrow_pool_address` - address of a borrow pool from the pair
    fn get_multiply_pair_obligation(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<Obligation, MCError>;

    // Returns the specific loan pool
    //
    // # Arguments
    // * `pool_address` - pool which data is returned
    fn get_pool(e: Env, pool_address: Address) -> Result<Pool, MCError>;

    // Returns pool's data together with borrow/supply APYs and other additionally computed info.
    // Intended to be used in simulations only
    //
    // # Arguments
    // * `pool_address` - address of a pool for which data is returned
    fn get_pool_data(e: Env, pool_address: Address) -> Result<PoolData, MCError>;

    // Returns a list of all pool addresses in the protocol
    fn get_all_pools(e: Env) -> Vec<Address>;

    // Returns accumulated market data. Intended to be used in simulations only
    fn get_market_data(e: Env) -> Result<MarketData, MCError>;

    // Returns a list of all user obligations in the protocol
    //
    // WARNING: It is originally intended to be used in `read-only` simulations,
    // yet, simulations as well as on-ledger invocations are constrained by the resource limits.
    // A proper way of accessing a list of all obligations would be to read
    // the corresponding storage entry
    fn get_all_obligations(e: Env) -> Vec<ObligationKey>;

    // Returns the specific multiply pair
    //
    // # Arguments
    // * `deposit_pool_address` - deposit pool of a pair that is returned
    // * `borrow_pool_address` - borrow pool of a pair that is returned
    fn get_multiply_pair(
        e: Env,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<MultiplyPair, MCError>;

    // Returns a list of all multiply pairs registered for the market
    fn get_all_multiply_pairs(e: Env) -> Vec<MultiplyPair>;

    // Sets the farms contract address for this market
    //
    // # Arguments
    // * `farms_contract` - address of the Farms contract
    fn set_farms_contract(e: Env, farms_contract: Address) -> Result<(), MCError>;

    // Clears the farms contract address (disables farm integration)
    fn clear_farms_contract(e: Env) -> Result<(), MCError>;

    // Gets the farms contract address if configured
    fn get_farms_contract(e: Env) -> Option<Address>;

    // Sets the supply farm ID for a pool
    //
    // # Arguments
    // * `pool_address` - address of the pool
    // * `farm_id` - farm ID for supply incentives (j-token staking)
    fn set_pool_supply_farm(
        e: Env,
        pool_address: Address,
        farm_id: BytesN<32>,
    ) -> Result<(), MCError>;

    // Sets the debt farm ID for a pool
    //
    // # Arguments
    // * `pool_address` - address of the pool
    // * `farm_id` - farm ID for debt incentives (d-token staking)
    fn set_pool_debt_farm(
        e: Env,
        pool_address: Address,
        farm_id: BytesN<32>,
    ) -> Result<(), MCError>;

    // Clears all farm configuration for a pool
    //
    // # Arguments
    // * `pool_address` - Address of the pool
    fn clear_pool_farms(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Refreshes farm stakes for a user's obligation (permissionless)
    // Syncs all farm stakes for the user's deposit and borrow positions.
    //
    // # Arguments
    // * `obligation_key` - key of an obligation that's affected by the operation
    fn refresh_obligation_farms(e: Env, obligation_key: ObligationKey) -> Result<(), MCError>;

    // Refreshes farm stakes for a user's multiply pair obligation (permissionless).
    //
    // # Arguments
    // * `user` - user whose multiply pair farms to refresh
    // * `deposit_pool_address` - deposit pool of the multiply pair
    // * `borrow_pool_address` - borrow pool of the multiply pair
    fn refresh_multiply_pair_farms(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError>;

    // Upgrades the lending contract
    //
    // # Arguments
    // * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
    //   version of the contract
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);

    // Resets the contract's storage. Useful when the contract's invariants are broken and require
    // resetting on the testnet without re-deploying the contract
    #[cfg(not(feature = "mainnet"))]
    fn reset_storage(e: Env);
}

#[contract]
// Isolated Lending Market Smart Contract. Allows users to lend and borrow other users' assets
pub struct MarketContract;

#[contractimpl]
impl Market for MarketContract {
    // TODO: All upgrade possibilities will be removed prior to mainnet deployment

    // Upgrades the lending contract
    //
    // # Arguments
    // * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
    //   version of the contract
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        require_admin(&e);

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn get_global_state(e: Env) -> GlobalState {
        process_get_global_state(&e)
    }

    fn update_market(
        e: Env,
        new_max_positions: u32,
        new_min_collateral_value_cents: i128,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_owned_and_admin(&e)?;

        if !(2..=2 * MAX_RESERVES).contains(&new_max_positions)
            || new_min_collateral_value_cents.is_negative()
        {
            return Err(MCError::InvalidMarketUpdate);
        }
        storage::set_max_positions(&e, new_max_positions);
        storage::set_min_collateral_value_cents(&e, new_min_collateral_value_cents);

        Ok(())
    }

    fn update_market_status(e: Env, new_status: u32) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_owned_and_admin(&e)?;

        let new_status = MarketStatus::try_from(new_status)?;
        storage::set_market_status(&e, &new_status);

        Ok(())
    }

    fn fund_update_market_status(e: Env, new_status: u32) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_insurance_fund(&e)?;

        let old_status = storage::get_market_status(&e);
        let new_status = MarketStatus::try_from(new_status)?;
        if old_status.is_admin_protected() || new_status.is_admin_protected() {
            return Err(MCError::InvalidMarketStatusUpdate);
        }

        storage::set_market_status(&e, &new_status);

        Ok(())
    }

    fn initialize_pool(
        e: Env,
        token_address: Address,
        salt: Option<BytesN<32>>,
        pool_config: Option<PoolConfig>,
    ) -> Result<Address, MCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        process_initialize_pool(&e, &token_address, &salt, &pool_config)
    }

    fn initialize_multiply_pair(
        e: Env,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        process_initialize_multiply_pair(&e, &deposit_pool_address, &borrow_pool_address)
    }

    fn queue_in_pool_config_update(
        e: Env,
        pool_address: Address,
        new_pool_config: PoolConfig,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_owned_and_admin(&e)?;

        new_pool_config.validate()?;

        let pool = Pool::try_get(&e, &pool_address)?;
        pool.queue_in_config_update(&e, &new_pool_config)?;

        events::queue_in_pool_config_update(&e, pool_address, new_pool_config);

        Ok(())
    }

    fn cancel_pool_config_update(e: Env, pool_address: Address) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_owned_and_admin(&e)?;

        let pool = Pool::try_get(&e, &pool_address)?;
        pool.remove_pool_config_update(&e)?;

        events::cancel_pool_config_update(&e, pool_address);

        Ok(())
    }

    fn apply_pool_config_update(e: Env, pool_address: Address) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_owned_and_admin(&e)?;

        let mut pool = Pool::try_get(&e, &pool_address)?;
        pool.apply_pool_config_update(&e)?;

        events::apply_pool_config_update(&e, pool_address);

        Ok(())
    }

    fn get_pool_config_queued_in_update(
        e: Env,
        pool_address: Address,
    ) -> Result<PoolUpdate, MCError> {
        require_admin(&e);

        let pool = Pool::try_get(&e, &pool_address)?;

        pool.get_pool_config_update(&e)
    }

    fn set_take_rate_fees_beneficiaries(
        e: Env,
        pool_address: Address,
        beneficiaries: Map<Address, u32>,
    ) -> Result<(), MCError> {
        require_admin(&e);

        // NB: Distribute pool fees for valid fees tracking afterwards
        process_distribute_pool_fees(&e, &pool_address)?;

        let mut pool = storage::get_pool(&e, &pool_address).ok_or(MCError::PoolDoesNotExist)?;

        let mut new_config = pool.config;
        new_config.fee_config.take_rate_beneficiaries = Some(beneficiaries);
        new_config.validate()?;

        pool.config = new_config;
        pool.set(&e);

        Ok(())
    }

    fn set_operation_fees_beneficiaries(
        e: Env,
        pool_address: Address,
        beneficiaries: Map<Address, u32>,
    ) -> Result<(), MCError> {
        require_admin(&e);

        // NB: Distribute pool fees for valid fees tracking afterwards
        process_distribute_pool_fees(&e, &pool_address)?;

        let mut pool = storage::get_pool(&e, &pool_address).ok_or(MCError::PoolDoesNotExist)?;
        let mut new_config = pool.config;

        new_config.fee_config.operation_fee_beneficiaries = Some(beneficiaries);
        new_config.validate()?; // TODO: Validate only fee_config?

        pool.config = new_config;
        pool.set(&e);

        Ok(())
    }

    fn bootstrap_pool(
        e: Env,
        pool_address: Address,
        sponsor: Address,
        amount: i128,
        start_period: u64,
        end_period: u64,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        process_bootstrap_pool(&e, &pool_address, &sponsor, amount, start_period, end_period)
    }

    fn deposit(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        obligation_key.require_auth();

        process_deposit(&e, &obligation_key, &pool_address, amount, &referrer)?
            .execute_transfers(&e)
    }

    fn borrow(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        obligation_key.require_auth();

        process_borrow(&e, &obligation_key, &pool_address, amount, &referrer)?.execute_transfers(&e)
    }

    fn donate(e: Env, user: Address, pool_address: Address, amount: i128) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        user.require_auth();
        require_nonnegative(amount)?;

        let mut pool = Pool::try_get(&e, &pool_address)?;
        pool.accrue_interest(&e)?;
        pool.adjust_total_available(&e, amount)?;
        pool.set(&e);

        let token_client = token::Client::new(&e, &pool.token_address);
        token_client.transfer(&user, e.current_contract_address(), &amount);

        Ok(())
    }

    fn add_collateral(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        obligation_key.require_auth();
        require_market_not_frozen(&e)?;

        process_add_collateral(&e, &obligation_key, &pool_address, amount, &referrer)?
            .execute_transfers(&e)
    }

    fn remove_collateral(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        obligation_key.require_auth();
        require_market_not_frozen(&e)?;

        process_remove_collateral(&e, &obligation_key, &pool_address, amount, &referrer)?
            .execute_transfers(&e)
    }

    fn repay(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        obligation_key.require_auth();
        require_market_not_frozen(&e)?;

        process_repay(&e, &obligation_key, &pool_address, amount, &referrer)?.execute_transfers(&e)
    }

    #[allow(clippy::too_many_arguments)]
    fn liquidate(
        e: Env,
        liquidator: Address,
        borrower_obligation_key: ObligationKey,
        borrow_pool_address: Address,
        collateral_pool_address: Address,
        repay_amount: i128,
        min_demanded_collateral_amount: i128,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        liquidator.require_auth();
        require_market_not_frozen(&e)?;

        process_liquidate(
            &e,
            &liquidator,
            &borrower_obligation_key,
            &borrow_pool_address,
            &collateral_pool_address,
            repay_amount,
            min_demanded_collateral_amount,
        )?
        .execute_transfers(&e)
    }

    fn withdraw(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        obligation_key.require_auth();
        require_market_not_frozen(&e)?;

        process_withdraw(&e, &obligation_key, &pool_address, amount, &referrer)?
            .execute_transfers(&e)
    }

    fn simulate_withdraw(
        e: Env,
        obligation_key: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<WithdrawResult, MCError> {
        storage::extend_instance_storage(&e);
        obligation_key.require_auth();

        require_market_not_frozen(&e)?;
        process_simulate_withdraw(&e, &obligation_key, &pool_address, amount, &referrer)
    }

    fn erc3156_flash_loan(
        e: Env,
        contract: Address,
        caller: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        caller.require_auth();
        require_market_not_frozen(&e)?;

        // TODO: Add what's missing from the standard?
        // https://eips.ethereum.org/EIPS/eip-3156
        process_erc3156_flash_loan(&e, &contract, &pool_address, amount)
    }

    #[allow(clippy::too_many_arguments)]
    fn deposit_with_leverage(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        deposit_as_margin: bool,
        amount: i128,
        leverage_multiplier: u32,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        user.require_auth();
        require_deposits_on_market_allowed(&e)?;
        require_borrows_on_market_allowed(&e)?;

        let multiply_pair = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?;
        let obligation_key = ObligationKey::new_with_seed(user.clone(), multiply_pair.seed.clone());
        // TODO: We can allow to multiply more but only with the preserved current multiplier
        if storage::obligation_exists(&e, &obligation_key) {
            return Err(MCError::InvalidLeverageInputs);
        }

        process_deposit_with_leverage(
            &e,
            &obligation_key,
            &multiply_pair,
            deposit_as_margin,
            amount,
            leverage_multiplier,
            &referrer,
        )?;

        Ok(())
    }

    fn withdraw_from_leveraged(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        user.require_auth();
        require_deposits_on_market_allowed(&e)?;
        require_borrows_on_market_allowed(&e)?;

        let multiply_pair = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?;
        let obligation_key = ObligationKey::new_with_seed(user.clone(), multiply_pair.seed.clone());

        process_withdraw_from_leveraged(&e, &obligation_key, &multiply_pair, amount, &referrer)
    }

    fn issue_cover_bad_debt(e: Env, obligation_key: ObligationKey) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        process_issue_cover_bad_debt(&e, &obligation_key)
    }

    fn issue_cover_bad_debt_pair(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let mp_seed = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?.seed;
        let obligation_key = ObligationKey::new_with_seed(user, mp_seed);

        process_issue_cover_bad_debt(&e, &obligation_key)
    }

    fn claim_cover_bad_debt_results(e: Env, obligation_key: ObligationKey) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        process_claim_cover_bad_debt_results(&e, &obligation_key)
    }

    fn claim_cover_bad_debt_result_pair(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let mp_seed = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?.seed;
        let obligation_key = ObligationKey::new_with_seed(user, mp_seed);

        process_claim_cover_bad_debt_results(&e, &obligation_key)
    }

    fn distribute_pool_fees(e: Env, pool_address: Address) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        process_distribute_pool_fees(&e, &pool_address)
    }

    fn distribute_all_pools_fees(e: Env) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        process_distribute_all_pools_fees(&e)
    }

    fn get_asset_decimals() -> u32 {
        // See - <https://github.com/stellar/rs-soroban-env/blob/main/soroban-env-host/src/builtin_contracts/stellar_asset_contract/contract.rs#L374>
        7
    }

    fn get_oracle_price_decimals(e: Env) -> u32 {
        oracle::get_oracle_price_decimals(&e)
    }

    fn get_pool_asset_oracle_price(e: Env, pool_address: Address) -> Result<i128, MCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        oracle::get_asset_price(&e, &pool.token_address)
    }

    fn get_user_obligation(e: Env, obligation_key: ObligationKey) -> Result<Obligation, MCError> {
        let obligation = Obligation::try_get(&e, &obligation_key)?;

        Ok(obligation)
    }

    fn refresh_obligation(e: Env, obligation_key: ObligationKey) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let obligation = Obligation::try_get(&e, &obligation_key)?;
        obligation.accrue_interest(&e)?;

        Ok(())
    }

    fn refresh_multiply_pair_obligation(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let obligation_key = ObligationKey::new_with_seed(
            user.clone(),
            MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?.seed,
        );
        let obligation = Obligation::try_get(&e, &obligation_key)?;
        obligation.accrue_interest(&e)?;

        Ok(())
    }

    fn refresh_pool(e: Env, pool_address: Address) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let mut pool = Pool::try_get(&e, &pool_address)?;
        pool.accrue_interest(&e)?;
        pool.set(&e);

        Ok(())
    }

    fn get_multiply_pair_obligation(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<Obligation, MCError> {
        let mp_seed = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?.seed;
        let obligation_key = ObligationKey::new_with_seed(user, mp_seed);
        let obligation = Obligation::try_get(&e, &obligation_key)?;

        Ok(obligation)
    }

    fn get_pool(e: Env, pool_address: Address) -> Result<Pool, MCError> {
        Pool::try_get(&e, &pool_address)
    }

    fn get_all_pools(e: Env) -> Vec<Address> {
        Pool::get_all(&e)
    }

    fn get_market_data(e: Env) -> Result<MarketData, MCError> {
        let pool_addresses = storage::get_all_pools(&e);
        let mut pools_data = svec![&e];

        for pool_address in pool_addresses {
            let pool = Pool::try_get(&e, &pool_address).map_err(|_| {
                events::pool_is_unexpectedly_missing_in_storage(&e, &pool_address);

                MCError::InternalError
            })?;
            pools_data.push_back(pool.get_pool_data(&e)?);
        }
        let global_state = process_get_global_state(&e);
        let multiply_pairs = MultiplyPair::get_all(&e);
        let market_data = MarketData {
            global_state,
            pools_data,
            multiply_pairs,
            asset_decimals: 7,
            oracle_price_decimals: oracle::get_oracle_price_decimals(&e),
        };

        Ok(market_data)
    }

    fn get_all_obligations(e: Env) -> Vec<ObligationKey> {
        let obligations_map = Obligation::get_all(&e);

        let mut obligations_vec = Vec::new(&e);
        for obligation_key in obligations_map.keys() {
            obligations_vec.push_back(obligation_key)
        }

        obligations_vec
    }

    fn get_multiply_pair(
        e: Env,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<MultiplyPair, MCError> {
        MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)
    }

    fn get_all_multiply_pairs(e: Env) -> Vec<MultiplyPair> {
        MultiplyPair::get_all(&e)
    }

    fn get_pool_data(e: Env, pool_address: Address) -> Result<PoolData, MCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        pool.get_pool_data(&e)
    }

    fn set_farms_contract(e: Env, farms_contract: Address) -> Result<(), MCError> {
        require_admin(&e);

        storage::set_farms_contract(&e, &farms_contract);
        events::set_farms_contract(&e, &farms_contract);

        Ok(())
    }

    fn clear_farms_contract(e: Env) -> Result<(), MCError> {
        require_admin(&e);

        storage::clear_farms_contract(&e);
        events::clear_farms_contract(&e);

        Ok(())
    }

    fn get_farms_contract(e: Env) -> Option<Address> {
        storage::get_farms_contract(&e)
    }

    fn set_pool_supply_farm(
        e: Env,
        pool_address: Address,
        farm_id: BytesN<32>,
    ) -> Result<(), MCError> {
        require_admin(&e);

        farms::set_pool_supply_farm(&e, &pool_address, farm_id)
    }

    fn set_pool_debt_farm(
        e: Env,
        pool_address: Address,
        farm_id: BytesN<32>,
    ) -> Result<(), MCError> {
        require_admin(&e);

        farms::set_pool_debt_farm(&e, &pool_address, farm_id)
    }

    fn clear_pool_farms(e: Env, pool_address: Address) -> Result<(), MCError> {
        require_admin(&e);

        farms::clear_pool_farms(&e, &pool_address)
    }

    fn refresh_obligation_farms(e: Env, obligation_key: ObligationKey) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let Some(farms_contract) = storage::get_farms_contract(&e) else {
            return Ok(()); // No farms configured
        };

        farms::refresh_all_obligation_farms(&e, &farms_contract, &obligation_key)?;

        Ok(())
    }

    fn refresh_multiply_pair_farms(
        e: Env,
        user: Address,
        deposit_pool_address: Address,
        borrow_pool_address: Address,
    ) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let Some(farms_contract) = storage::get_farms_contract(&e) else {
            return Ok(()); // No farms configured
        };

        let pair = MultiplyPair::try_get(&e, &deposit_pool_address, &borrow_pool_address)?;
        let obligation_key = ObligationKey::new_with_seed(user, pair.seed.clone());
        farms::refresh_all_obligation_farms(&e, &farms_contract, &obligation_key)?;
        Ok(())
    }

    #[cfg(not(feature = "mainnet"))]
    fn reset_storage(e: Env) {
        require_admin(&e);

        storage::remove_all_obligations(&e);
        storage::remove_all_pools(&e);
        storage::remove_all_multiply_pairs(&e);
    }

    fn submit_requests_batch(
        e: Env,
        obligation_key: ObligationKey,
        requests: Vec<Request>,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        obligation_key.require_auth();
        require_market_not_frozen(&e)?; // TODO: Do we miss any other such things?

        process_submit_requests_batch(&e, &requests, &obligation_key, &referrer)
    }
}

// TODO: Move all admin/management methods here from the trait
#[contractimpl]
impl MarketContract {
    // Constructs the market contract
    //
    // # Arguments
    // * `admin` - market's administrator
    // * `name` - market's name(not necessarily unique)
    // * `oracle` - SEP-40 compliant oracle's contract address
    // * `insurance_fund` - `Insurance Fund` trait compliant contract's address
    // * `deployer` - address of a deployer contract
    // * `max_positions` - max allowed number of positions in an obligation
    // * `min_collateral_value_cents` - minimum collateral value of a user's obligation in US dollar cents required
    //   to start receiving `Borrowing Capacity` increase
    // * `update_in_queue_period` - the time it takes for a market update to be in the update queue.
    //   `None` for permissionless markets since they cannot be updated
    pub fn __constructor(
        e: Env,
        name: String,
        admin: Address,
        oracle: Address,
        insurance_fund: Address,
        deployer: Address,
        max_positions: u32,
        min_collateral_value_cents: i128,
        insolvency_ltv_bps: i128,
        update_in_queue_period: Option<u64>,
    ) -> Result<(), MCError> {
        let market_status = if update_in_queue_period.is_some() {
            // Owned markets begin in a frozen state
            MarketStatus::Frozen
        } else {
            MarketStatus::Active
        };

        storage::set_name(&e, &name);
        storage::set_admin(&e, &admin);
        storage::set_oracle(&e, &oracle);
        storage::set_deployer(&e, &deployer);
        storage::set_market_status(&e, &market_status);
        storage::set_insurance_fund(&e, &insurance_fund);
        storage::set_max_positions(&e, max_positions);
        storage::set_update_in_queue_period(&e, update_in_queue_period);
        storage::set_min_collateral_value_cents(&e, min_collateral_value_cents);
        storage::set_insolvency_ltv_bps(&e, insolvency_ltv_bps);

        Ok(())
    }

    // Updates pool's status
    //
    // # Arguments
    // * `pool_address` - address of a pool whose status is updated
    // * `new_status_flags` - new pool's status flags bits
    pub fn update_pool_status(
        e: Env,
        pool_address: Address,
        new_status_flags: u32,
    ) -> Result<(), MCError> {
        require_owned_and_admin(&e)?;

        let mut pool = Pool::try_get(&e, &pool_address)?;
        pool.config.status.flags = new_status_flags;

        pool.set(&e);

        Ok(())
    }

    // Proposes a new market's admin
    //
    // # Arguments
    // * `new_admin` - proposed admin
    pub fn propose_new_admin(e: Env, new_admin: Address) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        storage::set_proposed_admin(&e, &new_admin);

        events::propose_new_admin(&e, new_admin);

        Ok(())
    }

    // Accepts the proposal to become a new admin
    pub fn accept_proposed_admin(e: Env) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);

        let proposed_admin = storage::get_proposed_admin(&e).ok_or(MCError::InvalidMarketUpdate)?;
        proposed_admin.require_auth();

        storage::set_admin(&e, &proposed_admin);
        storage::remove_proposed_admin(&e);

        events::accept_proposed_admin(&e);

        Ok(())
    }

    pub fn collect_pools_excessive_tokens(e: Env) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        process_collect_excessive_tokens_from_pools(&e, &admin)
    }

    pub fn collect_non_pool_excessive_token(e: Env, token: Address) -> Result<(), MCError> {
        storage::extend_instance_storage(&e);
        let admin = storage::get_admin(&e);
        admin.require_auth();

        process_collect_excessive_token(&e, &admin, &token)?;

        Ok(())
    }

    pub fn proxy_get_amount_in(
        e: Env,
        swap_provider: Address,
        token_in: Address,
        token_out: Address,
        amount_out: i128,
    ) -> Result<i128, MCError> {
        storage::extend_instance_storage(&e);

        process_proxy_get_amount_in(&e, &swap_provider, &token_in, &token_out, amount_out)
    }

    pub fn proxy_get_amount_out(
        e: Env,
        swap_provider: Address,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
    ) -> Result<i128, MCError> {
        storage::extend_instance_storage(&e);

        process_proxy_get_amount_out(&e, &swap_provider, &token_in, &token_out, amount_in)
    }

    pub fn proxy_swap_exact(
        e: Env,
        swap_provider: Address,
        user: Address,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> Result<i128, MCError> {
        storage::extend_instance_storage(&e);
        user.require_auth();

        process_proxy_swap_exact_tokens(
            &e,
            &swap_provider,
            &user,
            &token_in,
            &token_out,
            amount_in,
            min_amount_out,
        )
    }

    pub fn proxy_swap_for_exact(
        e: Env,
        swap_provider: Address,
        user: Address,
        token_in: Address,
        token_out: Address,
        max_amount_in: i128,
        amount_out: i128,
    ) -> Result<i128, MCError> {
        storage::extend_instance_storage(&e);
        user.require_auth();

        process_proxy_swap_for_exact_tokens(
            &e,
            &swap_provider,
            &user,
            &token_in,
            &token_out,
            max_amount_in,
            amount_out,
        )
    }
}
