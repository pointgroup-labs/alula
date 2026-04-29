#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    Address, BytesN, Env, Map, String, Vec, contract, contractclient, contractimpl, vec as svec,
};

use crate::{
    constants::*,
    error::MCError,
    events, farms,
    misc::{
        MarketData, PoolData, require_admin, require_deployer, require_deposits_on_market_allowed,
        require_insurance_fund, require_nonnegative, require_owned, require_owned_and_admin,
    },
    obligation::{Obligation, ObligationKey, WithdrawResult},
    oracle,
    pool::{Pool, PoolConfig},
    processors::*,
    request::Request,
    storage::{self, GlobalState, MarketInitParams, MarketStatus, MarketUpdate, QueuedPoolSet},
};

#[contractclient(name = "MarketClient")]
pub trait Market {
    // Submits a request batch
    fn submit_requests_batch(
        e: Env,
        user: ObligationKey,
        requests: Vec<Request>,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Gets the contract's global state
    fn get_global_state(e: Env) -> GlobalState;

    // Queues in a market config update
    //
    // # Arguments
    // * `new_max_positions` - updated maximum number of positions that a single obligation can have
    // * `new_min_collateral_value_cents` - updated minimum collateral allowed
    // * `new_bad_debt_lock_d` - updated bad debt lock duration
    fn queue_in_market_update(
        e: Env,
        new_max_positions: u32,
        new_min_collateral_value_cents: i128,
        new_bad_debt_lock_d: u64,
    ) -> Result<(), MCError>;

    // Cancels market config update if it exists in the update queue
    fn cancel_market_update(e: Env) -> Result<(), MCError>;

    // Applies the market config update if it exists in a queue and has completed its queue period
    fn apply_market_update(e: Env) -> Result<(), MCError>;

    // Gets the market config update from the queue if it exists
    fn get_market_queued_in_update(e: Env) -> Result<MarketUpdate, MCError>;

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

    // Queues in a pool set (new pool creation or existing pool config update)
    //
    // # Arguments
    // * `pool_address` - address of a corresponding pool/token that's being updated or initialized
    // * `pool_config` - pool configuration to apply
    fn queue_in_pool_set(
        e: Env,
        pool_address: Address,
        pool_config: PoolConfig,
    ) -> Result<(), MCError>;

    // Cancels a queued pool set
    //
    // # Arguments
    // * `pool_address` - address of a corresponding pool/token which update is being canceled
    fn cancel_pool_set(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Applies a queued pool set (creates new pool or updates existing config)
    //
    // # Arguments
    // * `pool_address` - address of a pool whose pool set is being applied
    fn apply_pool_set(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Gets the queued pool set from the queue if it exists
    //
    // # Arguments
    // * `pool_address` - address of the pool/token for which the queued pool set is received
    fn get_queued_pool_set(e: Env, pool_address: Address) -> Result<QueuedPoolSet, MCError>;

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

    // Deposits tokens into the loan pool
    //
    // # Arguments
    // * `user` - user that deposits a token
    // * `pool_address` - address of a pool to which the deposit happens
    // * `amount` - amount of tokens which are going to be deposited
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn deposit(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Withdraws deposited tokens from the loan pool to the user
    //
    // # Arguments
    // * `user` - user which withdraws deposited tokens
    // * `pool_address` - address of a pool from which the withdrawal happens
    // * `amount` - desired amount of tokens to withdraw.
    //   The actual amount withdrawn is capped to maintain the position's LTV at its Open LTV on the
    //   pool. Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to withdraw all tokens
    //   available for it
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn withdraw(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Simulates withdrawal of the deposited tokens from the loan pool to the user
    //
    // # Arguments
    // * `user` - user which withdraws deposited tokens
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
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<WithdrawResult, MCError>;

    // Borrows tokens from the loan pool
    //
    // # Arguments
    // * `user` - user which borrows a token
    // * `pool_address` - address of a pool from which the borrow happens
    // * `amount` - amount of tokens which are going to be borrowed
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn borrow(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Adds tokens into the loan pool as collateral only.
    // This implies that they are always available for a healthy withdrawal for the
    // cost of not accruing an interest rate
    //
    // # Arguments
    // * `user` - user that adds collateral
    // * `pool_address` - address of a pool to which the collateral is being added
    // * `amount` - amount of tokens which are being added as a collateral
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn add_collateral(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Removes collateral tokens from the loan pool to the user
    //
    // # Arguments
    // * `user` - user which withdraws collateral tokens
    // * `pool_address` - address of a pool from which the withdrawal happens
    // * `amount` - desired amount of collateral tokens to remove.
    //   The actual amount removed is capped to maintain the position's LTV at its Open LTV on the
    //   pool. Passing [`u64::MAX`] (or [`i128::MAX`]) effectively removes all available
    //   collateral
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn remove_collateral(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Repays borrowed tokens
    //
    // # Arguments
    // * `user` - user which repays borrowed tokens
    // * `pool_address` - address of a pool from which the borrow happened
    // * `amount` - provided amount of tokens to repay. If this amount exceeds the total debt, only
    //   the outstanding debt will be repaid.
    //   Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to repay the entire debt
    // * `referrer` - optional referrer's address. Depending on the pool's configuration, referrers are eligible for immediate fees
    fn repay(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError>;

    // Liquidates the borrower's position if the position's health factor criterion isn't met
    //
    // # Arguments
    // * `liquidator` - agent that liquidates the borrower's position
    // * `borrower` - the borrower whose position is being liquidated
    // * `borrow_pool_address` - address of a pool whose borrowed tokens are repaid by the
    //       liquidator
    // * `collateral_pool_address` - address of a pool whose tokens are sold to the liquidator with
    //       a discount
    // * `repay_amount` - amount of repaid tokens
    // * `demanded_collateral_amount` - min amount of collateral that liquidator finds sufficient for the amount of debt repaid
    fn liquidate(
        e: Env,
        liquidator: Address,
        borrower: ObligationKey,
        borrow_pool_address: Address,
        collateral_pool_address: Address,
        repay_amount: i128,
        demanded_collateral_amount: i128,
    ) -> Result<(), MCError>;

    // Creates a flash loan
    //
    // # Arguments
    // * `contract` - receiver contract that implements the `ModErc3156` callback. The
    //   receiver is responsible for validating `initiator` against its trusted set
    //   and granting the market a just-in-time `amount + fee` allowance during the
    //   `exec_op` callback. See `moderc3156` for the receiver pattern.
    // * `caller` - flash loan initiator. Forwarded to the receiver as `initiator` so
    //   the receiver can validate against its trusted-initiator set.
    // * `pool_address` - address of a pool from which the flash loan happens
    // * `amount` - amount of lent tokens
    fn flash_loan(
        e: Env,
        contract: Address,
        caller: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError>;

    // Issues `cover bad debt` requests on every bad debt borrow position on the user's obligation to the Insurance Fund contract
    //
    // # Arguments
    // * `user` - user that has a bad debt
    fn issue_cover_bad_debt(e: Env, user: ObligationKey) -> Result<(), MCError>;

    // Claims `cover bad debt` requests results for the user's obligation from the Insurance Fund if they exist
    //
    // # Arguments
    // * `user` - user that has open `cover bad debt` requests
    fn claim_cover_bad_debt_results(e: Env, user: ObligationKey) -> Result<(), MCError>;

    // Distributes a pool's fees to the beneficiaries
    fn distribute_pool_fees(e: Env, pool_address: Address) -> Result<(), MCError>;

    // Distributes all pools' fees to the beneficiaries
    fn distribute_all_pools_fees(e: Env) -> Result<(), MCError>;

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
    // * `user` - user which obligation is returned
    fn get_user_obligation(e: Env, user: ObligationKey) -> Result<Obligation, MCError>;

    // Accrues interest on all pools to whose obligation has open positions
    fn refresh_obligation(e: Env, user: ObligationKey) -> Result<(), MCError>;

    // Accrues interest on a pool
    fn refresh_pool(e: Env, pool_address: Address) -> Result<(), MCError>;

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
    // * `user` - User whose farms to refresh
    fn refresh_obligation_farms(e: Env, user: Address) -> Result<(), MCError>;

    // Upgrades the lending contract
    //
    // # Arguments
    // * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
    //   version of the contract
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>);
}

#[contract]
// Isolated Lending Market Smart Contract. Allows users to lend and borrow other users' assets
pub struct MarketContract;

#[contractimpl]
impl Market for MarketContract {
    // Upgrades the lending contract
    //
    // # Arguments
    // * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
    //   version of the contract
    fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        require_deployer(&e);

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    fn submit_requests_batch(
        e: Env,
        user: ObligationKey,
        requests: Vec<Request>,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        user.require_auth();
        storage::extend_instance(&e);

        process_submit_requests_batch(&e, &requests, &user, &referrer)
    }

    fn get_global_state(e: Env) -> GlobalState {
        process_get_global_state(&e)
    }

    fn queue_in_market_update(
        e: Env,
        new_max_positions: u32,
        new_min_collateral_value_cents: i128,
        new_bad_debt_lock_d: u64,
    ) -> Result<(), MCError> {
        require_owned_and_admin(&e)?;
        require_nonnegative(new_min_collateral_value_cents)?;
        storage::extend_instance(&e);

        if !(2..=MAX_RESERVES).contains(&new_max_positions)
            || !(MIN_COLLATERAL_VALUE_CENTS..=MAX_COLLATERAL_VALUE_CENTS)
                .contains(&new_min_collateral_value_cents)
            || !(MIN_BAD_DEBT_LOCK_D..=MAX_BAD_DEBT_LOCK_D).contains(&new_bad_debt_lock_d)
        {
            return Err(MCError::InvalidMarketConfigOrUpdate);
        }

        storage::queue_in_market_config_update(
            &e,
            new_max_positions,
            new_min_collateral_value_cents,
            new_bad_debt_lock_d,
        )?;

        events::queue_in_market_config_update(
            &e,
            new_max_positions,
            new_min_collateral_value_cents,
            new_bad_debt_lock_d,
        );

        Ok(())
    }

    fn cancel_market_update(e: Env) -> Result<(), MCError> {
        require_owned_and_admin(&e)?;
        storage::extend_instance(&e);

        storage::remove_market_config_update(&e)?;

        events::cancel_market_config_update(&e);

        Ok(())
    }

    fn apply_market_update(e: Env) -> Result<(), MCError> {
        require_owned_and_admin(&e)?;
        storage::extend_instance(&e);

        let MarketUpdate {
            new_max_positions,
            new_min_collateral_value_cents,
            new_bad_debt_lock_d,
            queued_in_timestamp,
        } = storage::get_market_config_update(&e)
            .ok_or(MCError::MarketDoesNotHaveQueuedInConfigUpdate)?;

        let update_period = storage::get_update_in_queue_period(&e);
        let current_time = e.ledger().timestamp();

        if current_time
            < queued_in_timestamp.checked_add(update_period).ok_or(MCError::OverOrUnderflow)?
        {
            return Err(MCError::MarketConfigUpdateIsNotYetApplicable);
        }

        storage::set_max_positions(&e, new_max_positions);
        storage::set_min_collateral_value_cents(&e, new_min_collateral_value_cents);
        storage::set_bad_debt_lock_d(&e, new_bad_debt_lock_d);
        storage::remove_market_config_update(&e)?;

        events::apply_market_config_update(
            &e,
            new_max_positions,
            new_min_collateral_value_cents,
            new_bad_debt_lock_d,
        );

        Ok(())
    }

    fn get_market_queued_in_update(e: Env) -> Result<MarketUpdate, MCError> {
        storage::get_market_config_update(&e).ok_or(MCError::MarketDoesNotHaveQueuedInConfigUpdate)
    }

    fn update_market_status(e: Env, new_status: u32) -> Result<(), MCError> {
        require_owned_and_admin(&e)?;
        storage::extend_instance(&e);

        let new_status = MarketStatus::try_from(new_status)?;

        storage::set_market_status(&e, &new_status);
        events::update_market_status(&e, new_status);

        Ok(())
    }

    fn fund_update_market_status(e: Env, new_status: u32) -> Result<(), MCError> {
        require_insurance_fund(&e)?;
        storage::extend_instance(&e);

        let old_status = storage::get_market_status(&e);
        let new_status = MarketStatus::try_from(new_status)?;
        if old_status.is_admin_protected() || new_status.is_admin_protected() {
            return Err(MCError::InvalidMarketConfigOrUpdate);
        }

        storage::set_market_status(&e, &new_status);
        events::fund_update_market_status(&e, new_status);

        Ok(())
    }

    fn queue_in_pool_set(
        e: Env,
        pool_address: Address,
        pool_config: PoolConfig,
    ) -> Result<(), MCError> {
        require_admin(&e);
        storage::extend_instance(&e);

        // For existing pools, validate against the current config so the
        // fee-bump constraints kick in. For new pools, pass `None`
        let current_pool_config = if Pool::exists(&e, &pool_address) {
            require_owned(&e)?;

            Some(Pool::try_get(&e, &pool_address)?.config)
        } else {
            None
        };
        pool_config.validate(current_pool_config)?;

        storage::queue_in_pool_set(&e, &pool_address, &pool_config)?;

        events::queue_in_pool_set(&e, pool_address, pool_config);

        Ok(())
    }

    fn cancel_pool_set(e: Env, pool_address: Address) -> Result<(), MCError> {
        require_admin(&e);
        storage::extend_instance(&e);

        if Pool::exists(&e, &pool_address) {
            require_owned(&e)?;
        }

        storage::remove_queued_pool_set(&e, &pool_address)?;

        events::cancel_pool_set(&e, pool_address);

        Ok(())
    }

    fn apply_pool_set(e: Env, pool_address: Address) -> Result<(), MCError> {
        require_admin(&e);
        storage::extend_instance(&e);

        let queued_pool_set = storage::get_queued_pool_set(&e, &pool_address)
            .ok_or(MCError::PoolDoesNotHaveQueuedPoolSet)?;

        let update_period = storage::get_update_in_queue_period(&e);
        let current_time = e.ledger().timestamp();

        if current_time
            < queued_pool_set
                .queued_in_timestamp
                .checked_add(update_period)
                .ok_or(MCError::OverOrUnderflow)?
        {
            return Err(MCError::PoolSetIsNotYetApplicable);
        }

        if let Ok(mut pool) = Pool::try_get(&e, &pool_address) {
            pool.config = queued_pool_set.new_config;
            pool.set(&e);
        } else {
            process_initialize_pool(&e, &pool_address, &queued_pool_set.new_config)?;
        }

        storage::remove_queued_pool_set(&e, &pool_address)?;

        events::apply_pool_set(&e, pool_address);

        Ok(())
    }

    fn get_queued_pool_set(e: Env, pool_address: Address) -> Result<QueuedPoolSet, MCError> {
        storage::get_queued_pool_set(&e, &pool_address).ok_or(MCError::PoolDoesNotHaveQueuedPoolSet)
    }

    fn set_take_rate_fees_beneficiaries(
        e: Env,
        pool_address: Address,
        beneficiaries: Map<Address, u32>,
    ) -> Result<(), MCError> {
        require_admin(&e);

        // NB: Distribute pool fees for valid fees tracking afterwards
        let mut pool = storage::get_pool(&e, &pool_address).ok_or(MCError::PoolDoesNotExist)?;
        pool.accrue_interest(&e)?;

        process_distribute_pool_fees(&e, pool_address.clone())?;
        pool.refresh(&e)?;

        let mut new_config = pool.config;
        new_config.fee_config.take_rate_beneficiaries = Some(beneficiaries.clone());
        new_config.validate(None)?;

        pool.config = new_config;
        pool.set(&e);

        events::set_take_rate_fees_beneficiaries(&e, pool_address, beneficiaries);

        Ok(())
    }

    fn set_operation_fees_beneficiaries(
        e: Env,
        pool_address: Address,
        beneficiaries: Map<Address, u32>,
    ) -> Result<(), MCError> {
        require_admin(&e);

        // NB: Distribute pool fees for valid fees tracking afterwards
        process_distribute_pool_fees(&e, pool_address.clone())?;

        let mut pool = storage::get_pool(&e, &pool_address).ok_or(MCError::PoolDoesNotExist)?;
        let mut new_config = pool.config;

        new_config.fee_config.operation_fee_beneficiaries = Some(beneficiaries.clone());
        new_config.validate(None)?;

        pool.config = new_config;
        pool.set(&e);

        events::set_operation_fees_beneficiaries(&e, pool_address, beneficiaries);

        Ok(())
    }

    fn deposit(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        user.require_auth();
        require_deposits_on_market_allowed(&e)?;
        storage::extend_instance(&e);

        process_deposit(&e, &user, &pool_address, amount, &referrer)?.execute_transfers(&e)
    }

    fn borrow(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        user.require_auth();
        storage::extend_instance(&e);

        process_borrow(&e, &user, &pool_address, amount, &referrer)?.execute_transfers(&e)
    }

    fn add_collateral(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        user.require_auth();
        storage::extend_instance(&e);

        process_add_collateral(&e, &user, &pool_address, amount, &referrer)?.execute_transfers(&e)
    }

    fn remove_collateral(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        user.require_auth();
        storage::extend_instance(&e);

        process_remove_collateral(&e, &user, &pool_address, amount, &referrer)?
            .execute_transfers(&e)
    }

    fn repay(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        user.require_auth();
        storage::extend_instance(&e);

        process_repay(&e, &user, &pool_address, amount, &referrer)?.execute_transfers(&e)
    }

    fn liquidate(
        e: Env,
        liquidator: Address,
        borrower: ObligationKey,
        borrow_pool_address: Address,
        collateral_pool_address: Address,
        repay_amount: i128,
        min_demanded_collateral_amount: i128,
    ) -> Result<(), MCError> {
        storage::extend_instance(&e);
        liquidator.require_auth();
        process_liquidate(
            &e,
            &liquidator,
            &borrower,
            &borrow_pool_address,
            &collateral_pool_address,
            repay_amount,
            min_demanded_collateral_amount,
        )?
        .execute_transfers(&e)
    }

    fn withdraw(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<(), MCError> {
        user.require_auth();
        storage::extend_instance(&e);

        process_withdraw(&e, &user, &pool_address, amount, &referrer)?.execute_transfers(&e)
    }

    fn simulate_withdraw(
        e: Env,
        user: ObligationKey,
        pool_address: Address,
        amount: i128,
        referrer: Option<Address>,
    ) -> Result<WithdrawResult, MCError> {
        process_simulate_withdraw(&e, &user, &pool_address, amount, &referrer)
    }

    fn flash_loan(
        e: Env,
        contract: Address,
        caller: Address,
        pool_address: Address,
        amount: i128,
    ) -> Result<(), MCError> {
        caller.require_auth();
        storage::extend_instance(&e);

        process_flash_loan(&e, &caller, &contract, &pool_address, amount)
    }

    fn issue_cover_bad_debt(e: Env, user: ObligationKey) -> Result<(), MCError> {
        storage::extend_instance(&e);

        process_issue_cover_bad_debt(&e, user)
    }

    fn claim_cover_bad_debt_results(e: Env, user: ObligationKey) -> Result<(), MCError> {
        storage::extend_instance(&e);

        process_claim_cover_bad_debt_results(&e, user)
    }

    fn distribute_pool_fees(e: Env, pool_address: Address) -> Result<(), MCError> {
        storage::extend_instance(&e);

        process_distribute_pool_fees(&e, pool_address)
    }

    fn distribute_all_pools_fees(e: Env) -> Result<(), MCError> {
        storage::extend_instance(&e);

        process_distribute_all_pools_fees(&e)
    }

    fn get_oracle_price_decimals(e: Env) -> u32 {
        oracle::get_oracle_price_decimals(&e)
    }

    fn get_pool_asset_oracle_price(e: Env, pool_address: Address) -> Result<i128, MCError> {
        let pool = Pool::try_get(&e, &pool_address)?;

        oracle::get_asset_price(&e, &pool.token_address)
    }

    fn get_user_obligation(e: Env, user: ObligationKey) -> Result<Obligation, MCError> {
        let obligation = Obligation::try_get(&e, &user)?;

        Ok(obligation)
    }

    fn refresh_obligation(e: Env, user: ObligationKey) -> Result<(), MCError> {
        storage::extend_instance(&e);

        process_refresh_obligation(&e, user)?;

        Ok(())
    }

    fn refresh_pool(e: Env, pool_address: Address) -> Result<(), MCError> {
        storage::extend_instance(&e);

        let mut pool = Pool::try_get(&e, &pool_address)?;
        pool.accrue_interest(&e)?;
        pool.set(&e);

        events::refresh_pool(&e, pool_address);

        Ok(())
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
        let market_data = MarketData {
            global_state,
            pools_data,
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

    fn refresh_obligation_farms(e: Env, user: Address) -> Result<(), MCError> {
        storage::extend_instance(&e);

        let Some(farms_contract) = storage::get_farms_contract(&e) else {
            return Ok(()); // No farms configured
        };

        let obligation_key = ObligationKey::new(user);
        farms::refresh_all_obligation_farms(&e, &farms_contract, &obligation_key)?;

        Ok(())
    }
}

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
    // * `params` - market initialization parameters
    pub fn __constructor(
        e: Env,
        name: String,
        admin: Address,
        oracle: Address,
        insurance_fund: Address,
        deployer: Address,
        params: MarketInitParams,
    ) -> Result<(), MCError> {
        verify_market_params(&params)?;

        let MarketInitParams {
            max_positions,
            min_collateral_value_cents,
            insolvency_ltv_bps,
            update_in_queue_period,
            is_owned,
            bad_debt_lock_d,
        } = params;

        let market_status = if is_owned { MarketStatus::Frozen } else { MarketStatus::Active };

        storage::set_name(&e, &name);
        storage::set_admin(&e, &admin);
        storage::set_oracle(&e, &oracle);
        storage::set_deployer(&e, &deployer);
        storage::set_market_status(&e, &market_status);
        storage::set_insurance_fund(&e, &insurance_fund);
        storage::set_max_positions(&e, max_positions);
        storage::set_update_in_queue_period(&e, update_in_queue_period);
        storage::set_is_owned(&e, is_owned);
        storage::set_min_collateral_value_cents(&e, min_collateral_value_cents);
        storage::set_insolvency_ltv_bps(&e, insolvency_ltv_bps);
        storage::set_bad_debt_lock_d(&e, bad_debt_lock_d);

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
        require_admin(&e);
        storage::extend_instance(&e);

        storage::set_proposed_admin(&e, &new_admin);

        events::propose_new_admin(&e, new_admin);

        Ok(())
    }

    // Accepts the proposal to become a new admin
    pub fn accept_proposed_admin(e: Env) -> Result<(), MCError> {
        storage::extend_instance(&e);

        let proposed_admin =
            storage::get_proposed_admin(&e).ok_or(MCError::InvalidMarketConfigOrUpdate)?;
        proposed_admin.require_auth();

        storage::set_admin(&e, &proposed_admin);
        storage::remove_proposed_admin(&e);

        events::accept_proposed_admin(&e);

        Ok(())
    }
}

// -- Helpers --

fn verify_market_params(params: &MarketInitParams) -> Result<(), MCError> {
    let &MarketInitParams {
        min_collateral_value_cents,
        bad_debt_lock_d,
        update_in_queue_period: _,
        insolvency_ltv_bps,
        max_positions,
        is_owned: _,
    } = params;

    require_nonnegative(min_collateral_value_cents)?;

    if !(2..=MAX_RESERVES).contains(&max_positions)
        || !(MIN_INSOLVENCY_LTV_BPS..=MAX_INSOLVENCY_LTV_BPS).contains(&insolvency_ltv_bps)
        || !(MIN_COLLATERAL_VALUE_CENTS..=MAX_COLLATERAL_VALUE_CENTS)
            .contains(&min_collateral_value_cents)
        || !(MIN_BAD_DEBT_LOCK_D..=MAX_BAD_DEBT_LOCK_D).contains(&bad_debt_lock_d)
    {
        return Err(MCError::InvalidMarketConfigOrUpdate);
    }

    Ok(())
}
