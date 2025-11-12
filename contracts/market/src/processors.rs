use moderc3156::FlashLoanClient;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, BytesN, Env, Map, Vec, map as smap,
    token::{self, TokenClient},
};

use crate::{
    constants::*,
    error::MCError,
    events,
    math_utils::MathUtils,
    misc::require_nonnegative,
    multiply_pair::MultiplyPair,
    obligation::{
        CoverBadDebtResult, LiquidationResult, LiquidationResult2, Obligation, ObligationKey,
    },
    pool::{Pool, PoolConfig},
    request::{Request, RequestTransfers, RequestType},
    storage::{self, GlobalState},
    swap,
};

pub fn process_submit_requests_batch<'a>(
    e: &'a Env,
    user: &'a Address,
    requests: &'a Vec<Request>,
    plain_obligation_key: &'a ObligationKey,
    earn_obligation_key: &'a ObligationKey,
) -> Result<RequestTransfers<'a>, MCError> {
    let mut transfers = RequestTransfers::new(e, user, smap![&e], smap![&e]);

    for request in requests {
        let Request { request_type, pool_address, amount } = request;
        let request_type = RequestType::try_from(request_type)?;

        let new_transfers = match request_type {
            RequestType::Deposit => {
                process_deposit(e, plain_obligation_key, &pool_address, amount)?
            }
            RequestType::Borrow => process_borrow(e, plain_obligation_key, &pool_address, amount)?,
            RequestType::Withdraw => {
                process_withdraw(e, plain_obligation_key, &pool_address, amount)?
            }
            RequestType::Repay => process_repay(e, plain_obligation_key, &pool_address, amount)?,
            RequestType::AddCollateral => {
                process_add_collateral(e, plain_obligation_key, &pool_address, amount)?
            }
            RequestType::RemoveCollateral => {
                process_remove_collateral(e, plain_obligation_key, &pool_address, amount)?
            }
            RequestType::DepositIntoEarnObligation => {
                process_deposit(e, earn_obligation_key, &pool_address, amount)?
            }
            RequestType::WithdrawFromEarnObligation => {
                process_deposit(e, earn_obligation_key, &pool_address, amount)?
            }
        };

        transfers.merge(&new_transfers)?;
    }

    Ok(transfers)
}

pub fn process_get_global_state(e: &Env) -> GlobalState {
    let update_in_queue_period = storage::get_update_in_queue_period(e);
    let name = storage::get_name(e);
    let admin = storage::get_admin(e);
    let oracle = storage::get_oracle(e);
    let deployer = storage::get_deployer(e);
    let status = storage::get_market_status(e) as u32;
    let is_owned = update_in_queue_period.is_some();
    let max_positions = storage::get_max_positions(e);
    let min_collateral_value = storage::get_min_collateral_value(e);
    let insolvency_ltv_bps = storage::get_insolvency_ltv_bps(e);

    GlobalState {
        name,
        admin,
        oracle,
        status,
        deployer,
        is_owned,
        max_positions,
        insolvency_ltv_bps,
        min_collateral_value,
        update_in_queue_period,
    }
}

pub fn process_initialize_pool(
    e: &Env,
    token_address: &Address,
    salt: &Option<BytesN<32>>,
    pool_config: &Option<PoolConfig>,
) -> Result<Address, MCError> {
    let pool_address: Address = if let Some(salt) = salt {
        e.deployer().with_address(token_address.clone(), salt.clone()).deployed_address()
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

    let token_client = TokenClient::new(e, token_address);
    let name = token_client.name();
    let token_symbol = token_client.symbol();

    events::initialize_pool(e, token_address, &pool_address, &token_symbol);

    let pool = Pool {
        total_d_tokens: 0,
        total_j_tokens: 0,
        total_borrowed: 0,
        total_available: 0,
        total_collateral: 0,

        accumulated_host_fees: 0,
        accumulated_market_fees: 0,
        accumulated_reserve_fees: 0,

        name,
        config: pool_config,
        pool_address: pool_address.clone(),
        token_symbol,
        token_address: token_address.clone(),
        last_accrual_timestamp: e.ledger().timestamp(),

        supply_incentives: Map::new(e),

        borrow_apr_bps: 0,
        supply_apr_bps: 0,
    };

    pool.set(e);
    pool.register(e);

    Ok(pool_address)
}

pub fn process_initialize_multiply_pair(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<(), MCError> {
    MultiplyPair::require_does_not_exists(e, deposit_pool_address, borrow_pool_address)?;

    let (deposit_pool, borrow_pool) = (
        Pool::try_get(e, deposit_pool_address).map_err(|_| MCError::DepositPoolDoesNotExist)?,
        Pool::try_get(e, borrow_pool_address).map_err(|_| MCError::BorrowPoolDoesNotExist)?,
    );
    let pair = MultiplyPair::new(
        e,
        deposit_pool_address,
        borrow_pool_address,
        borrow_pool.config.health_config.open_ltv_bps,
        borrow_pool.config.fee_config.flash_loan_fee_bps as i128,
        deposit_pool.config.health_config.liability_factor_bps,
    );

    pair.set(e);
    pair.register(e);

    events::initialize_multiply_pair(e, deposit_pool_address, borrow_pool_address);

    Ok(())
}

pub fn process_incentivize_pool(
    e: &Env,
    pool_address: &Address,
    sponsor: &Address,
    amount: i128,
    start_period: u64,
    end_period: u64,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let current_timestamp = e.ledger().timestamp();
    if start_period < current_timestamp {
        return Err(MCError::IncentivePeriodDoesNotStartInTheFuture);
    } else if start_period >= end_period {
        return Err(MCError::IncentivePeriodStartDoesNotPrecedePeriodEnd);
    }
    let period = (start_period, end_period);

    let mut pool = Pool::try_get(e, pool_address)?;

    pool.incentivize(amount, period, current_timestamp)?;
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer_from(
        &e.current_contract_address(),
        sponsor,
        &e.current_contract_address(),
        &amount,
    );

    events::incentivize_pool(e, pool_address, sponsor, amount, period);

    Ok(())
}

pub fn process_deposit<'a>(
    e: &'a Env,
    obligation_key: &'a ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_nonnegative(amount)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_deposit_enabled()?;
    pool.accrue_interest(e)?;

    let supply_limit = pool.config.health_config.supply_limit;
    if supply_limit != 0 {
        // 0 indicates unlimited supply
        let new_supply = pool.total_supply()?.checked_add(amount).map_over_or_underflow()?;

        if new_supply > supply_limit {
            return Err(MCError::PoolSupplyLimitExceeded);
        }
    }

    let mut obligation =
        Obligation::try_get(e, obligation_key).unwrap_or(Obligation::new(e, obligation_key));
    obligation.require_no_borrow_position_exists(pool_address)?;

    let deposit_result = obligation.deposit(e, &pool, amount)?;
    pool.deposit(e, &deposit_result)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let transfers = RequestTransfers::new_with_user_transfers(
        e,
        &obligation_key.user,
        smap![&e, (pool.token_address, amount)],
    );

    events::deposit(e, pool_address, obligation_key, deposit_result);

    Ok(transfers)
}

pub fn process_borrow<'a>(
    e: &'a Env,
    obligation_key: &'a ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_borrow_enabled()?;
    pool.accrue_interest(e)?;

    let borrow_result = obligation.borrow(e, &pool, amount)?;
    pool.borrow(e, &borrow_result)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let transfers = RequestTransfers::new_with_market_transfers(
        e,
        &obligation_key.user,
        smap![&e, (pool.token_address, borrow_result.borrower_to_receive)],
    );

    events::borrow(e, pool_address, obligation_key, borrow_result);

    Ok(transfers)
}

pub fn process_add_collateral<'a>(
    e: &'a Env,
    obligation_key: &'a ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_nonnegative(amount)?;

    let mut obligation =
        Obligation::try_get(e, obligation_key).unwrap_or(Obligation::new(e, obligation_key));
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let add_collateral_result = obligation.add_collateral(e, &pool, amount)?;
    pool.add_collateral(e, &add_collateral_result)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let transfers = RequestTransfers::new_with_user_transfers(
        e,
        &obligation_key.user,
        smap![&e, (pool.token_address, amount)],
    );

    events::add_collateral(e, pool_address, obligation_key, add_collateral_result);

    Ok(transfers)
}

pub fn process_repay<'a>(
    e: &'a Env,
    obligation_key: &'a ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let repay_result = obligation.repay(e, &pool, amount)?;
    pool.repay(e, &repay_result)?;

    if obligation.is_empty() {
        // NB: Obligation shouldn't be empty at this point due to some amount of collateral or
        // deposit required to back up the debt
        events::obligation_is_unexpectedly_empty(e, obligation_key, pool_address);

        return Err(MCError::InternalError);
    }

    obligation.set(e, obligation_key);
    pool.set(e);

    // Since interest accrual happens each second, to sign and to simulate a deterministic transfer
    // from the borrower's account - 2 transfers take place: borrower => contract(original
    // amount), contract => borrower(excess amount). See - <https://discord.com/channels/897514728459468821/1424779244189520145>
    let user_transfers = smap![e, (pool.token_address.clone(), amount)];
    let market_transfers = smap![e, (pool.token_address, repay_result.amount_to_send_back)];
    let transfers =
        RequestTransfers::new(e, &obligation_key.user, market_transfers, user_transfers);

    events::repay(e, pool_address, obligation_key, repay_result);

    Ok(transfers)
}

pub fn process_remove_collateral<'a>(
    e: &'a Env,
    obligation_key: &'a ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let remove_collateral_result = obligation.remove_collateral(e, &pool, amount)?;
    pool.remove_collateral(e, &remove_collateral_result)?;

    pool.set(e);

    if obligation.is_empty() {
        obligation.remove(e, obligation_key);
    } else {
        obligation.set(e, obligation_key);
    }

    let transfers = RequestTransfers::new_with_market_transfers(
        e,
        &obligation_key.user,
        smap![e, (pool.token_address, remove_collateral_result.collateral_remover_to_receive)],
    );

    events::remove_collateral(e, pool_address, obligation_key, remove_collateral_result);

    Ok(transfers)
}

pub fn process_withdraw<'a>(
    e: &'a Env,
    obligation_key: &'a ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    let withdraw_result = obligation.withdraw(e, &pool, amount)?;
    pool.withdraw(e, &withdraw_result)?;

    if obligation.is_empty() {
        obligation.remove(e, obligation_key);
    } else {
        obligation.set(e, obligation_key);
    }

    pool.set(e);

    let transfers = RequestTransfers::new_with_market_transfers(
        e,
        &obligation_key.user,
        smap![e, (pool.token_address, withdraw_result.withdrawer_to_receive)],
    );

    events::withdraw(e, pool_address, obligation_key, withdraw_result);

    Ok(transfers)
}

pub fn process_flash_loan(
    e: &Env,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let pool = Pool::try_get(e, pool_address)?;
    pool.require_available(amount)?;

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), contract, &amount);

    let flash_loan_fee_bps = pool.config.fee_config.flash_loan_fee_bps as i128;

    let flash_loan_taker_client = FlashLoanClient::new(e, contract);
    flash_loan_taker_client.exec_op(
        &e.current_contract_address(),
        &pool.token_address,
        &amount,
        &flash_loan_fee_bps,
    );

    let fees = amount.fixed_mul_ceil(flash_loan_fee_bps, BPS_FACTOR).map_over_or_underflow()?;
    let amount_to_repay = amount.checked_add(fees).map_over_or_underflow()?;

    // TODO: Register flash loan fees as fees

    // Repay the flash loan principal + fee using the funds generated by the exec_op strategy
    token_client.transfer_from(
        &e.current_contract_address(),
        contract,
        &e.current_contract_address(),
        &amount_to_repay,
    );

    events::flash_loan(e, contract, pool_address, amount, fees);

    Ok(())
}

pub fn process_deposit_with_leverage(
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
    let leverage_multiplier_minus_1 =
        leverage_multiplier.checked_sub(LEVERAGE_SCALE).map_over_or_underflow()?;

    let (flash_borrow_amount, amount_in, amount_out) = if deposit_as_margin {
        let base_leverage_amount = amount
            .fixed_mul_floor(leverage_multiplier_minus_1 as i128, LEVERAGE_SCALE as i128)
            .map_over_or_underflow()?;

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
        let flash_borrow_amount = amount
            .fixed_mul_floor(leverage_multiplier_minus_1 as i128, LEVERAGE_SCALE as i128)
            .map_over_or_underflow()?;

        let amount_in = amount.checked_add(flash_borrow_amount).map_over_or_underflow()?;
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
        received_amount.checked_add(amount).map_over_or_underflow()?
    } else {
        received_amount
    };

    let seed = pair.seed.clone();
    let obligation_key = ObligationKey::new_with_seed(user.clone(), seed);

    process_deposit(e, &obligation_key, deposit_pool_address, deposit_amount)?.execute();

    // -- Borrow to repay the flash loan --
    let flash_loan_fee = flash_borrow_amount
        .fixed_mul_ceil(borrow_pool.config.fee_config.flash_loan_fee_bps as i128, BPS_FACTOR)
        .map_over_or_underflow()?;
    let flash_repay_amount =
        flash_borrow_amount.checked_add(flash_loan_fee).map_over_or_underflow()?;

    let obligation = Obligation::try_get(e, &obligation_key).map_err(|_| {
        events::obligation_is_missing_in_storage(e, &obligation_key);

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

    process_borrow(e, &obligation_key, borrow_pool_address, flash_repay_amount)?.execute();
    borrow_pool.refresh(e)?;

    // Repay the flash loan
    flash_loan_token_client.transfer(user, e.current_contract_address(), &flash_repay_amount);

    borrow_pool.adjust_total_available(e, flash_repay_amount)?;
    borrow_pool.set(e);

    events::deposit_with_leverage(
        e,
        &obligation_key,
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
    let seed = pair.seed.clone();

    let obligation_key = ObligationKey::new_with_seed(user.clone(), seed);

    let obligation = Obligation::try_get(e, &obligation_key)?;
    let total_debt = obligation.get_total_debt(e, borrow_pool_address)?;

    if total_debt == 0 {
        // NB: No leverage case is equivalent to a simple deposit
        process_withdraw(e, &obligation_key, deposit_pool_address, amount)?.execute();

        return Ok(());
    }

    let obligation_j_tokens = obligation.get_j_tokens(deposit_pool_address)?;
    let deposited_tokens =
        deposit_pool.compute_tokens_from_j_tokens_floor(e, obligation_j_tokens)?;

    // Compute the max withdrawable amount
    let max_withdrawable_amount = compute_leveraged_position_max_withdrawable_amount(
        e,
        user,
        &deposit_pool.token_address,
        &borrow_pool.token_address,
        deposited_tokens,
        total_debt,
        borrow_pool.config.fee_config.flash_loan_fee_bps,
    )?;

    let expected_withdrawn_amount = i128::min(amount, max_withdrawable_amount);

    // Compute the flash borrow amount for deleverage
    let withdrawn_ratio_bps = expected_withdrawn_amount
        .fixed_div_floor(max_withdrawable_amount, BPS_FACTOR)
        .map_over_or_underflow()?;

    let plain_leverage_amount =
        deposited_tokens.checked_sub(max_withdrawable_amount).map_over_or_underflow()?;
    let plain_leverage_to_be_withdrawn = plain_leverage_amount
        .fixed_mul_floor(withdrawn_ratio_bps, BPS_FACTOR)
        .map_over_or_underflow()?;

    // To maintain LTV for the leveraged position, the amount of borrowed tokens to be repaid
    // must be proportional to the withdrawn amount of the deposited tokens
    let flash_borrow_amount =
        total_debt.fixed_mul_floor(withdrawn_ratio_bps, BPS_FACTOR).map_over_or_underflow()?;

    borrow_pool.require_available(flash_borrow_amount)?;

    // Flash Borrow
    let flash_borrowed_token_client = token::Client::new(e, &borrow_pool.token_address);
    flash_borrowed_token_client.transfer(&e.current_contract_address(), user, &flash_borrow_amount);
    borrow_pool.adjust_total_available(e, -flash_borrow_amount)?;
    borrow_pool.set(e);

    // Repay Debt
    process_repay(e, &obligation_key, borrow_pool_address, flash_borrow_amount)?.execute();
    borrow_pool.refresh(e)?;

    // Withdraw
    let withdrawn_amount = expected_withdrawn_amount
        .checked_add(plain_leverage_to_be_withdrawn)
        .map_over_or_underflow()?;

    process_withdraw(e, &obligation_key, deposit_pool_address, withdrawn_amount)?.execute();
    deposit_pool.refresh(e)?;

    // Swap to get the flash repay amount
    let flash_loan_fee = flash_borrow_amount
        .fixed_mul_ceil(borrow_pool.config.fee_config.flash_loan_fee_bps as i128, BPS_FACTOR)
        .map_over_or_underflow()?;

    let flash_repay_amount =
        flash_loan_fee.checked_add(flash_borrow_amount).map_over_or_underflow()?;

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
    flash_borrowed_token_client.transfer(user, e.current_contract_address(), &flash_repay_amount);

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

pub fn process_liquidate<'a>(
    e: &'a Env,
    liquidator: &Address,
    borrower_obligation_key: &ObligationKey,
    borrow_pool_address: &Address,
    collateral_pool_address: &Address,
    amount: i128,
    min_collateral_received_amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_nonnegative(amount)?;
    require_nonnegative(min_collateral_received_amount)?;

    if borrow_pool_address == collateral_pool_address {
        return Err(MCError::LiquidationWithEqualCollateralAndDepositPools);
    }
    if liquidator == &borrower_obligation_key.user {
        return Err(MCError::SelfLiquidation);
    }

    let mut obligation = Obligation::try_get(e, borrower_obligation_key)?;
    obligation.accrue_interest(e)?;

    let (borrow_pool, collateral_pool) = (
        Pool::try_get(e, borrow_pool_address).map_err(|_| MCError::BorrowPoolDoesNotExist)?,
        Pool::try_get(e, collateral_pool_address)
            .map_err(|_| MCError::CollateralPoolDoesNotExist)?,
    );
    collateral_pool.require_collateral_is_seizable()?;

    let liquidation_result: LiquidationResult2 = obligation.liquidate(
        e,
        &borrow_pool,
        &collateral_pool,
        amount,
        min_collateral_received_amount,
    )?;

    todo!()
}

pub fn process_redeem_accumulated_host_fees(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    let mut pool = Pool::try_get(e, pool_address)?;

    let fees_to_be_redeemed = i128::min(amount, pool.accumulated_host_fees);
    pool.require_available(fees_to_be_redeemed)?;

    pool.adjust_accumulated_host_fees(
        e,
        fees_to_be_redeemed.checked_neg().map_over_or_underflow()?,
    )?;
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), user, &fees_to_be_redeemed);

    Ok(())
}

pub fn process_redeem_accumulated_market_fees(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    let mut pool = Pool::try_get(e, pool_address)?;

    let fees_to_be_redeemed = i128::min(amount, pool.accumulated_market_fees);
    pool.require_available(fees_to_be_redeemed)?;

    pool.adjust_accumulated_market_fees(
        e,
        fees_to_be_redeemed.checked_neg().map_over_or_underflow()?,
    )?;
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), user, &fees_to_be_redeemed);

    Ok(())
}

pub fn process_cover_obligation_bad_debt_and_socialize_any_remaining_loss(
    e: &Env,
    obligation_key: ObligationKey,
) -> Result<(), MCError> {
    let obligation = Obligation::try_get(e, &obligation_key)?;

    let CoverBadDebtResult { borrows_to_be_compensated, collaterals_to_remove } =
        obligation.cover_bad_debt(e)?;

    for (pool_address, d_tokens) in borrows_to_be_compensated {
        let mut pool = Pool::try_get(e, &pool_address).map_err(|_| {
            events::pool_is_missing_in_storage(e, &pool_address);

            MCError::InternalError
        })?;

        let obligation_pool_debt = pool.compute_tokens_from_d_tokens_ceil(e, d_tokens)?;
        let available_reserve_fees = pool.available_accumulated_reserve_fees();

        let debt_can_be_covered = i128::min(obligation_pool_debt, available_reserve_fees);
        let d_tokens_can_be_covered =
            pool.compute_d_tokens_from_tokens_floor(e, debt_can_be_covered)?;

        // - Cover what can be covered from the reserves -

        pool.adjust_total_available(e, debt_can_be_covered)?;
        pool.adjust_total_borrowed(e, debt_can_be_covered.checked_neg().map_over_or_underflow()?)?;
        pool.adjust_accumulated_reserve_fees(
            e,
            debt_can_be_covered.checked_neg().map_over_or_underflow()?,
        )?;
        pool.adjust_total_d_tokens(
            e,
            d_tokens_can_be_covered.checked_neg().map_over_or_underflow()?,
        )?;

        // - Socialize all remaining bad debt -

        if obligation_pool_debt > debt_can_be_covered {
            let left_to_socialize = obligation_pool_debt - debt_can_be_covered; // safe
            let d_tokens_left =
                d_tokens.checked_sub(d_tokens_can_be_covered).map_over_or_underflow()?;

            pool.adjust_total_borrowed(
                e,
                left_to_socialize.checked_neg().map_over_or_underflow()?,
            )?;
            pool.adjust_total_d_tokens(e, d_tokens_left.checked_neg().map_over_or_underflow()?)?;
        }

        pool.set(e);
    }

    for (pool_address, j_tokens, collateral) in collaterals_to_remove {
        let mut pool = Pool::try_get(e, &pool_address).map_err(|_| {
            events::pool_is_missing_in_storage(e, &pool_address);

            MCError::InternalError
        })?;

        // - Remove any collateral(both deposit and collateral-only cases) from the obligation to
        //   benefit the pool -

        pool.adjust_total_j_tokens(e, j_tokens.checked_neg().map_over_or_underflow()?)?;
        pool.adjust_total_collateral(e, collateral.checked_neg().map_over_or_underflow()?)?;
        pool.adjust_total_available(e, collateral)?;

        pool.set(e);
    }

    // TODO: Check if removing the obligation is the only way to move on after bad debt
    obligation.remove(e, &obligation_key);

    Ok(())
}

#[allow(unused)]
pub fn process_swap_for_exact_tokens(
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

    events::swap(e, user, token_in, token_out, amount_in, amount_out, received_amount);

    Ok(received_amount)
}

pub fn process_swap_exact_tokens(
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

    events::swap(e, user, token_in, token_out, amount_in, amount_out, received_amount);

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
    flash_loan_fee_bps: u32,
) -> Result<i128, MCError> {
    require_nonnegative(deposited_amount)?;
    require_nonnegative(borrowed_amount)?;

    let flash_loan_fee = borrowed_amount
        .fixed_mul_ceil(flash_loan_fee_bps as i128, BPS_FACTOR)
        .map_over_or_underflow()?;
    let flash_repay_amount = borrowed_amount.checked_add(flash_loan_fee).map_over_or_underflow()?;

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
