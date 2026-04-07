use insurance_fund_interface::{CoverageStatus, InsuranceFundClient, IssueRequestResult};
use moderc3156::FlashLoanClient;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, Vec, map as smap,
    token::{self, TokenClient},
};

use crate::{
    constants::*,
    error::MCError,
    events, farms,
    math_utils::MathUtils,
    misc::{
        require_borrows_on_market_allowed, require_deposits_on_market_allowed, require_nonnegative,
        require_positive,
    },
    obligation::{Obligation, ObligationKey, WithdrawResult},
    pool::{Pool, PoolConfig},
    request::{
        LiquidateRequest, Request, RequestTransfers, StandardRequest, SwapExactTokensRequest,
        SwapForExactTokensRequest,
    },
    storage::{self, GlobalState},
    swap,
};

pub fn process_submit_requests_batch(
    e: &Env,
    requests: &Vec<Request>,
    obligation_key: &ObligationKey,
    referrer: &Option<Address>,
) -> Result<(), MCError> {
    let mut transfers = RequestTransfers::new(
        e,
        obligation_key.user.clone(),
        referrer.clone(),
        smap![&e],
        smap![&e],
        smap![&e],
        None,
    );

    for request in requests {
        match request {
            Request::Deposit(standard_request) => {
                let new_transfers = process_deposit(
                    e,
                    obligation_key,
                    &standard_request.pool_address,
                    standard_request.amount,
                    referrer,
                )?;

                transfers.merge(new_transfers)?;
            }
            Request::Borrow(standard_request) => {
                let new_transfers = process_borrow(
                    e,
                    obligation_key,
                    &standard_request.pool_address,
                    standard_request.amount,
                    referrer,
                )?;

                transfers.merge(new_transfers)?;
            }
            Request::Withdraw(standard_request) => {
                let new_transfers = process_withdraw(
                    e,
                    obligation_key,
                    &standard_request.pool_address,
                    standard_request.amount,
                    referrer,
                )?;

                transfers.merge(new_transfers)?;
            }
            Request::Repay(standard_request) => {
                let new_transfers = process_repay(
                    e,
                    obligation_key,
                    &standard_request.pool_address,
                    standard_request.amount,
                    referrer,
                )?;

                transfers.merge(new_transfers)?;
            }
            Request::AddCollateral(standard_request) => {
                let new_transfers = process_add_collateral(
                    e,
                    obligation_key,
                    &standard_request.pool_address,
                    standard_request.amount,
                    referrer,
                )?;

                transfers.merge(new_transfers)?;
            }
            Request::RemoveCollateral(standard_request) => {
                let new_transfers = process_remove_collateral(
                    e,
                    obligation_key,
                    &standard_request.pool_address,
                    standard_request.amount,
                    referrer,
                )?;

                transfers.merge(new_transfers)?;
            }
            Request::FlashBorrow(standard_request) => {
                transfers.flush_transfers(e)?;

                let new_transfers =
                    process_flash_borrow(e, &obligation_key.user, standard_request)?;
                transfers.merge(new_transfers)?;
            }
            Request::SwapExactTokens(SwapExactTokensRequest {
                swap_provider,
                path,
                amount_in,
                min_amount_out,
            }) => {
                transfers.flush_transfers(e)?;

                process_swap_exact(
                    e,
                    &swap_provider,
                    &obligation_key.user,
                    &path,
                    amount_in,
                    min_amount_out,
                )?;
            }
            Request::SwapForExactTokens(SwapForExactTokensRequest {
                swap_provider,
                path,
                max_amount_in,
                amount_out,
            }) => {
                transfers.flush_transfers(e)?;

                process_swap_for_exact(
                    e,
                    &swap_provider,
                    &obligation_key.user,
                    &path,
                    max_amount_in,
                    amount_out,
                )?;
            }
            Request::Liquidate(LiquidateRequest {
                borrower_obligation_key,
                borrow_pool_address,
                collateral_pool_address,
                repay_amount,
                min_demanded_collateral_amount,
            }) => {
                transfers.flush_transfers(e)?;

                let liquidation_transfers = process_liquidate(
                    e,
                    &obligation_key.user,
                    &borrower_obligation_key,
                    &borrow_pool_address,
                    &collateral_pool_address,
                    repay_amount,
                    min_demanded_collateral_amount,
                )?;
                liquidation_transfers.execute_transfers(e)?;
            }
        };
    }

    transfers.execute_transfers(e)?;

    Ok(())
}

pub fn process_get_global_state(e: &Env) -> GlobalState {
    let update_in_queue_period = storage::get_update_in_queue_period(e);
    let name = storage::get_name(e);
    let admin = storage::get_admin(e);
    let oracle = storage::get_oracle(e);
    let insurance_fund = storage::get_insurance_fund(e);
    let deployer = storage::get_deployer(e);
    let status = storage::get_market_status(e) as u32;
    let is_owned = storage::get_is_owned(e);
    let max_positions = storage::get_max_positions(e);
    let min_collateral_value_cents = storage::get_min_collateral_value_cents(e);
    let insolvency_ltv_bps = storage::get_insolvency_ltv_bps(e);
    let bad_debt_lock_d = storage::get_bad_debt_lock_d(e);

    GlobalState {
        name,
        admin,
        status,
        oracle,
        deployer,
        is_owned,
        max_positions,
        insurance_fund,
        insolvency_ltv_bps,
        min_collateral_value_cents,
        update_in_queue_period,
        bad_debt_lock_d,
    }
}

pub fn process_initialize_pool(
    e: &Env,
    token_address: &Address,
    pool_config: &PoolConfig,
) -> Result<Address, MCError> {
    let pool_address: Address = token_address.clone();

    Pool::require_does_not_exist(e, &pool_address)?;

    let token_client = TokenClient::new(e, token_address);
    let name = token_client.name();
    let token_symbol = token_client.symbol();
    let token_decimals = token_client.decimals();

    events::initialize_pool(e, token_address, &pool_address, &token_symbol);

    let pool = Pool {
        total_d_tokens: 0,
        total_j_tokens: 0,
        total_borrowed: 0,
        total_available: 0,
        total_collateral: 0,

        operation_fees_sum: 0,
        take_rate_fees_sum: 0,

        name,
        config: pool_config.clone(),
        pool_address: pool_address.clone(),
        token_symbol,
        token_decimals,
        token_address: token_address.clone(),
        last_accrual_timestamp: e.ledger().timestamp(),

        borrow_apr_bps: 0,
        supply_apr_bps: 0,

        target_utilization_ratio_bps: DEFAULT_TARGET_UTILIZATION_RATIO_BPS,
        interest_rate_modifier: BPS_FACTOR,

        farm_supply: None,
        farm_debt: None,
        bad_debt_lock_d: 0,
        bad_debt_request_count: 0,
    };

    pool.set(e);
    pool.register(e);

    Ok(pool_address)
}

pub fn process_deposit<'a>(
    e: &'a Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
    referrer: &Option<Address>,
) -> Result<RequestTransfers<'a>, MCError> {
    require_positive(amount)?;
    require_deposits_on_market_allowed(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_deposit_enabled()?;
    pool.require_bad_debt_unlocked(e)?;
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
    obligation.require_no_active_cover_bad_debt_requests_exists()?;

    let deposit_result = obligation.deposit(e, &pool, amount, referrer)?;
    pool.deposit(e, &deposit_result)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    // Auto-refresh supply farm stake
    farms::try_refresh_pool_farm(e, &obligation, obligation_key, &pool, farms::FarmKind::Supply)?;

    let user_transfers = smap![&e, (pool.token_address.clone(), amount)];

    let referrer_fee = deposit_result.operation_fees.referrer_fee;
    let referrer_fee_transfers =
        if referrer_fee > 0 { smap![&e, (pool.token_address, referrer_fee)] } else { smap![e] };

    let transfers = RequestTransfers::new_with_user_transfers(
        e,
        obligation_key.user.clone(),
        referrer.clone(),
        user_transfers,
        referrer_fee_transfers,
    );

    events::deposit(e, pool_address, obligation_key, obligation, deposit_result);

    Ok(transfers)
}

pub fn process_borrow<'a>(
    e: &'a Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
    referrer: &Option<Address>,
) -> Result<RequestTransfers<'a>, MCError> {
    require_positive(amount)?;
    require_borrows_on_market_allowed(e)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.require_no_active_cover_bad_debt_requests_exists()?;
    obligation.require_no_deposit_position_exists(pool_address)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_borrow_enabled()?;
    pool.accrue_interest(e)?;

    let borrow_result = obligation.borrow(e, &pool, amount, referrer)?;
    pool.borrow(e, &borrow_result)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    // Auto-refresh debt farm stake
    farms::try_refresh_pool_farm(e, &obligation, obligation_key, &pool, farms::FarmKind::Debt)?;

    let market_transfers =
        smap![&e, (pool.token_address.clone(), borrow_result.borrower_to_receive)];

    let referrer_fee = borrow_result.operation_fees.referrer_fee;
    let referrer_fee_transfers =
        if referrer_fee > 0 { smap![&e, (pool.token_address, referrer_fee)] } else { smap![e] };

    let transfers = RequestTransfers::new_with_market_transfers(
        e,
        obligation_key.user.clone(),
        referrer.clone(),
        market_transfers,
        referrer_fee_transfers,
    );

    events::borrow(e, pool_address, obligation_key, obligation, borrow_result);

    Ok(transfers)
}

pub fn process_add_collateral<'a>(
    e: &'a Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
    referrer: &Option<Address>,
) -> Result<RequestTransfers<'a>, MCError> {
    require_positive(amount)?;

    let mut obligation =
        Obligation::try_get(e, obligation_key).unwrap_or(Obligation::new(e, obligation_key));
    obligation.require_no_active_cover_bad_debt_requests_exists()?;
    obligation.require_no_borrow_position_exists(pool_address)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_add_collateral_enabled()?;

    let add_collateral_result = obligation.add_collateral(e, &pool, amount, referrer)?;
    pool.add_collateral(e, &add_collateral_result)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let user_transfers = smap![&e, (pool.token_address.clone(), amount)];

    let referrer_fee = add_collateral_result.operation_fees.referrer_fee;
    let referrer_fee_transfers =
        if referrer_fee > 0 { smap![&e, (pool.token_address, referrer_fee)] } else { smap![e] };

    let transfers = RequestTransfers::new_with_user_transfers(
        e,
        obligation_key.user.clone(),
        referrer.clone(),
        user_transfers,
        referrer_fee_transfers,
    );

    events::add_collateral(e, pool_address, obligation_key, obligation, add_collateral_result);

    Ok(transfers)
}

pub fn process_repay<'a>(
    e: &'a Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
    referrer: &Option<Address>,
) -> Result<RequestTransfers<'a>, MCError> {
    require_positive(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.require_no_active_cover_bad_debt_requests_exists()?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let repay_result = obligation.repay(e, &pool, amount, referrer)?;
    pool.repay(e, &repay_result)?;

    if obligation.is_empty() {
        // NB: Obligation shouldn't be empty at this point due to some amount of collateral or
        // deposit required to back up the debt
        events::obligation_is_unexpectedly_empty(e, obligation_key, pool_address);

        return Err(MCError::InternalError);
    }

    obligation.set(e, obligation_key);
    pool.set(e);

    // Auto-refresh debt farm stake
    farms::try_refresh_pool_farm(e, &obligation, obligation_key, &pool, farms::FarmKind::Debt)?;

    // Since interest accrual happens each second, to sign and to simulate a deterministic transfer
    // from the borrower's account - 2 transfers take place: borrower => contract(original
    // amount), contract => borrower(excess amount). See - <https://discord.com/channels/897514728459468821/1424779244189520145>
    let user_transfers = smap![e, (pool.token_address.clone(), amount)];
    let market_transfers = smap![e, (pool.token_address.clone(), repay_result.amount_to_send_back)];

    let referrer_fee = repay_result.operation_fees.referrer_fee;
    let referrer_fee_transfers =
        if referrer_fee > 0 { smap![&e, (pool.token_address, referrer_fee)] } else { smap![e] };

    let transfers = RequestTransfers::new(
        e,
        obligation_key.user.clone(),
        referrer.clone(),
        market_transfers,
        user_transfers,
        referrer_fee_transfers,
        None,
    );

    events::repay(e, pool_address, obligation_key, obligation, repay_result);

    Ok(transfers)
}

pub fn process_remove_collateral<'a>(
    e: &'a Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
    referrer: &Option<Address>,
) -> Result<RequestTransfers<'a>, MCError> {
    require_positive(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.require_no_active_cover_bad_debt_requests_exists()?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let remove_collateral_result = obligation.remove_collateral(e, &pool, amount, referrer)?;
    pool.remove_collateral(e, &remove_collateral_result)?;

    pool.set(e);

    let event_obligation = if obligation.is_empty() {
        obligation.remove(e, obligation_key);

        None
    } else {
        obligation.set(e, obligation_key);

        Some(obligation)
    };

    let market_transfers = smap![
        e,
        (pool.token_address.clone(), remove_collateral_result.collateral_remover_to_receive)
    ];

    let referrer_fee = remove_collateral_result.operation_fees.referrer_fee;
    let referrer_fee_transfers =
        if referrer_fee > 0 { smap![&e, (pool.token_address, referrer_fee)] } else { smap![e] };

    let transfers = RequestTransfers::new_with_market_transfers(
        e,
        obligation_key.user.clone(),
        referrer.clone(),
        market_transfers,
        referrer_fee_transfers,
    );

    events::remove_collateral(
        e,
        pool_address,
        obligation_key,
        event_obligation,
        remove_collateral_result,
    );

    Ok(transfers)
}

pub fn process_flash_borrow<'a>(
    e: &'a Env,
    user: &Address,
    request: StandardRequest,
) -> Result<RequestTransfers<'a>, MCError> {
    let StandardRequest { amount, pool_address } = &request;

    require_positive(*amount)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_flash_loan_enabled()?;
    pool.require_total_available(*amount)?;

    pool.adjust_total_available(e, amount.checked_neg().map_over_or_underflow()?)?;
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&e.current_contract_address(), user, amount);

    events::flash_borrow(e, user, pool_address, *amount);

    Ok(RequestTransfers::new(e, user.clone(), None, smap![e], smap![e], smap![e], Some(request)))
}

pub fn process_withdraw<'a>(
    e: &'a Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
    referrer: &Option<Address>,
) -> Result<RequestTransfers<'a>, MCError> {
    require_positive(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.require_no_active_cover_bad_debt_requests_exists()?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_bad_debt_unlocked(e)?;

    let withdraw_result = obligation.withdraw(e, &pool, amount, referrer)?;
    pool.withdraw(e, &withdraw_result)?;

    // Auto-refresh supply farm stake before potentially removing obligation
    farms::try_refresh_pool_farm(e, &obligation, obligation_key, &pool, farms::FarmKind::Supply)?;

    let event_obligation = if obligation.is_empty() {
        obligation.remove(e, obligation_key);

        None
    } else {
        obligation.set(e, obligation_key);

        Some(obligation)
    };

    pool.set(e);

    let market_transfers =
        smap![e, (pool.token_address.clone(), withdraw_result.withdrawer_to_receive)];

    let referrer_fee = withdraw_result.operation_fees.referrer_fee;
    let referrer_fee_transfers =
        if referrer_fee > 0 { smap![&e, (pool.token_address, referrer_fee)] } else { smap![e] };

    let transfers = RequestTransfers::new_with_market_transfers(
        e,
        obligation_key.user.clone(),
        referrer.clone(),
        market_transfers,
        referrer_fee_transfers,
    );

    events::withdraw(e, pool_address, obligation_key, event_obligation, withdraw_result);

    Ok(transfers)
}

pub fn process_simulate_withdraw(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
    referrer: &Option<Address>,
) -> Result<WithdrawResult, MCError> {
    require_positive(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.require_no_active_cover_bad_debt_requests_exists()?;
    obligation.accrue_interest(e)?;

    let pool = Pool::try_get(e, pool_address)?;
    pool.require_bad_debt_unlocked(e)?;

    let withdraw_result = obligation.withdraw(e, &pool, amount, referrer)?;

    Ok(withdraw_result)
}

pub fn process_flash_loan(
    e: &Env,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_positive(amount)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.require_flash_loan_enabled()?;
    pool.require_total_available(amount)?;

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

    token_client.transfer_from(
        &e.current_contract_address(),
        contract,
        &e.current_contract_address(),
        &amount_to_repay,
    );

    pool.adjust_operation_fees_sum(e, fees)?;
    pool.set(e);

    events::flash_loan(e, contract, pool_address, amount, fees);

    Ok(())
}

pub fn process_liquidate<'a>(
    e: &'a Env,
    liquidator: &'a Address,
    borrower_obligation_key: &ObligationKey,
    borrow_pool_address: &Address,
    collateral_pool_address: &Address,
    repay_amount: i128,
    min_demanded_collateral_amount: i128,
) -> Result<RequestTransfers<'a>, MCError> {
    require_positive(repay_amount)?;
    require_nonnegative(min_demanded_collateral_amount)?;

    if borrow_pool_address == collateral_pool_address || liquidator == &borrower_obligation_key.user
    {
        return Err(MCError::InvalidLiquidationInputs);
    }

    let mut obligation = Obligation::try_get(e, borrower_obligation_key)?;
    obligation.require_no_active_cover_bad_debt_requests_exists()?;

    obligation.accrue_interest(e)?;

    let (mut borrow_pool, mut collateral_pool) = (
        Pool::try_get(e, borrow_pool_address).map_err(|_| MCError::BorrowPoolDoesNotExist)?,
        Pool::try_get(e, collateral_pool_address)
            .map_err(|_| MCError::CollateralPoolDoesNotExist)?,
    );
    collateral_pool.require_collateral_is_seizable()?;

    let liquidation_result = obligation.liquidate(
        e,
        &borrow_pool,
        &collateral_pool,
        repay_amount,
        min_demanded_collateral_amount,
    )?;

    let liquidator_obligation_key = ObligationKey::new(liquidator.clone());
    let mut liquidator_event_obligation: Option<Obligation> =
        Obligation::try_get(e, &liquidator_obligation_key).ok();

    if liquidation_result.j_tokens_seized.is_positive() {
        let mut liquidator_obligation =
            liquidator_event_obligation.unwrap_or(Obligation::new(e, &liquidator_obligation_key));

        liquidator_obligation.liquidation_increase_j_tokens(
            e,
            &collateral_pool,
            liquidation_result.j_tokens_seized,
        )?;
        liquidator_obligation.set(e, &liquidator_obligation_key);

        // Auto-refresh liquidator's supply farm stake (they received j-tokens)
        farms::try_refresh_pool_farm(
            e,
            &liquidator_obligation,
            &liquidator_obligation_key,
            &collateral_pool,
            farms::FarmKind::Supply,
        )?;

        liquidator_event_obligation = Some(liquidator_obligation);
    }

    borrow_pool.liquidation_repay_debt(e, &liquidation_result)?;
    collateral_pool.liquidation_redeem_collateral(e, &liquidation_result)?;

    // Auto-refresh borrower's farms before potentially removing obligation
    farms::try_refresh_pool_farm(
        e,
        &obligation,
        borrower_obligation_key,
        &borrow_pool,
        farms::FarmKind::Debt,
    )?;
    farms::try_refresh_pool_farm(
        e,
        &obligation,
        borrower_obligation_key,
        &collateral_pool,
        farms::FarmKind::Supply,
    )?;

    let borrower_event_obligation = if obligation.is_empty() {
        obligation.remove(e, borrower_obligation_key);

        None
    } else {
        obligation.set(e, borrower_obligation_key);

        Some(obligation)
    };

    borrow_pool.set(e);
    collateral_pool.set(e);

    let user_transfers = smap![e, (borrow_pool.token_address.clone(), repay_amount)];
    let market_transfers = smap![
        e,
        (collateral_pool.token_address.clone(), liquidation_result.plain_collateral_seized),
        (borrow_pool.token_address, liquidation_result.amount_to_send_back)
    ];

    let transfers = RequestTransfers::new(
        e,
        liquidator.clone(),
        None,
        market_transfers,
        user_transfers,
        smap![e],
        None,
    );

    events::liquidate(
        e,
        liquidator,
        borrower_obligation_key,
        borrow_pool_address,
        collateral_pool_address,
        borrower_event_obligation,
        liquidator_event_obligation,
        liquidation_result,
    );

    Ok(transfers)
}

pub fn process_issue_cover_bad_debt(e: &Env, obligation_key: ObligationKey) -> Result<(), MCError> {
    let mut obligation = Obligation::try_get(e, &obligation_key)?;
    obligation.accrue_interest(e)?;
    obligation.require_borrow_exists()?;
    obligation.require_no_liquidatable_collateral_exists(e)?;
    obligation.require_no_active_cover_bad_debt_requests_exists()?;

    let insurance_fund = storage::get_insurance_fund(e);
    let insurance_fund_client = InsuranceFundClient::new(e, &insurance_fund);

    let mut borrow_positions_to_remove: Vec<Address> = Vec::new(e);

    for (pool_address, borrow_position) in obligation.borrows.iter() {
        let mut pool = Pool::try_get(e, &pool_address).map_err(|_| {
            events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

            MCError::InternalError
        })?;
        let obligation_debt_on_pool =
            pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;

        let token_client = token::Client::new(e, &pool.token_address);
        let market_balance_before = token_client.balance(&e.current_contract_address());

        match insurance_fund_client.request_coverage(&pool.token_address, &obligation_debt_on_pool)
        {
            IssueRequestResult::Recorded(request_id) => {
                if obligation
                    .insurance_fund_requests_ids
                    .contains_key((pool_address.clone(), request_id))
                {
                    events::insurance_fund_duplicate_request_id(
                        e,
                        obligation_key,
                        pool_address,
                        request_id,
                    );

                    return Err(MCError::DependencyContractError);
                }

                let duration = storage::get_bad_debt_lock_d(e);
                pool.bad_debt_lock_d =
                    e.ledger().timestamp().checked_add(duration).map_over_or_underflow()?;
                pool.bad_debt_request_count =
                    pool.bad_debt_request_count.checked_add(1).map_over_or_underflow()?;
                pool.set(e);

                events::pool_bad_debt_locked(e, &pool_address, pool.bad_debt_lock_d);

                obligation.insurance_fund_requests_ids.set((pool_address, request_id), ());
            }
            IssueRequestResult::Immediate(covered_amount) => {
                let market_balance_after = token_client.balance(&e.current_contract_address());
                let actual_received = market_balance_after - market_balance_before; // safe

                // Verify the insurance fund transferred what it claimed
                if actual_received != covered_amount {
                    events::inconsistent_immediate_insurance_fund_coverage(
                        e,
                        obligation_key,
                        pool_address,
                        actual_received,
                        covered_amount,
                    );

                    return Err(MCError::DependencyContractError);
                }

                // Cap coverage to actual debt (defensive, shouldn't happen with well-behaved fund)
                let effective_coverage = i128::min(obligation_debt_on_pool, actual_received);

                // Socialize any uncovered portion: add what was received to available,
                // and remove full debt - the gap reduces j-token value for suppliers
                pool.adjust_total_available(e, effective_coverage)?;
                pool.adjust_total_borrowed(
                    e,
                    obligation_debt_on_pool.checked_neg().map_over_or_underflow()?,
                )?;
                pool.adjust_total_d_tokens(
                    e,
                    borrow_position.d_tokens.checked_neg().map_over_or_underflow()?,
                )?;

                pool.set(e);
                borrow_positions_to_remove.push_back(pool_address);
            }
        }
    }

    for pool_address in borrow_positions_to_remove {
        obligation.try_remove_borrow_position(e, &pool_address)?;
    }

    // -- Use all non-liquidatable collateral to benefit suppliers --

    let mut deposit_positions_to_remove: Vec<Address> = Vec::new(e);

    for (pool_address, deposit_position) in obligation.deposits.iter() {
        let mut pool = Pool::try_get(e, &pool_address).map_err(|_| {
            events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

            MCError::InternalError
        })?;

        pool.adjust_total_available(e, deposit_position.collateral)?;
        pool.adjust_total_collateral(
            e,
            deposit_position.collateral.checked_neg().map_over_or_underflow()?,
        )?;
        pool.adjust_total_j_tokens(
            e,
            deposit_position.j_tokens.checked_neg().map_over_or_underflow()?,
        )?;

        pool.set(e);
        deposit_positions_to_remove.push_back(pool_address);
    }

    for pool_address in deposit_positions_to_remove {
        obligation.try_remove_deposit_position(e, &pool_address)?;
    }

    if obligation.is_empty() {
        obligation.remove(e, &obligation_key);
    } else {
        obligation.set(e, &obligation_key);
    }

    events::issue_cover_bad_debt(e, obligation_key);

    Ok(())
}

pub fn process_claim_cover_bad_debt_results(
    e: &Env,
    obligation_key: ObligationKey,
) -> Result<(), MCError> {
    let mut obligation = Obligation::try_get(e, &obligation_key)?;
    obligation.accrue_interest(e)?;

    let insurance_fund = storage::get_insurance_fund(e);
    let insurance_fund_client = InsuranceFundClient::new(e, &insurance_fund);

    let mut completed_requests: Vec<(Address, u64)> = Vec::new(e);

    for ((pool_address, request_id), _) in obligation.insurance_fund_requests_ids.iter() {
        let mut pool = Pool::try_get(e, &pool_address).map_err(|_| {
            events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

            MCError::InternalError
        })?;

        let request_status = insurance_fund_client.get_status(&request_id).ok_or_else(|| {
            events::insurance_fund_missing_request(
                e,
                obligation_key.clone(),
                pool_address.clone(),
                request_id,
            );
            MCError::DependencyContractError
        })?;

        if let CoverageStatus::Ready(approved_amount) = request_status {
            let token_client = token::Client::new(e, &pool.token_address);
            let balance_before = token_client.balance(&e.current_contract_address());

            insurance_fund_client.claim_coverage(&request_id);

            let balance_after = token_client.balance(&e.current_contract_address());
            let actual_received = balance_after - balance_before;

            if actual_received != approved_amount {
                events::insurance_fund_claim_mismatch(
                    e,
                    obligation_key,
                    pool_address,
                    request_id,
                    approved_amount,
                    actual_received,
                );

                return Err(MCError::DependencyContractError);
            }

            let borrow_position =
                obligation.borrows.get(pool_address.clone()).ok_or(MCError::InternalError)?;

            let total_debt = pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;
            let covered_amount = i128::min(total_debt, actual_received);

            // -- Socialize losses --

            pool.adjust_total_available(e, covered_amount)?;
            pool.adjust_total_borrowed(e, total_debt.checked_neg().map_over_or_underflow()?)?;
            pool.adjust_total_d_tokens(
                e,
                borrow_position.d_tokens.checked_neg().map_over_or_underflow()?,
            )?;

            pool.bad_debt_request_count =
                pool.bad_debt_request_count.checked_sub(1).map_over_or_underflow()?;
            if pool.bad_debt_request_count == 0 && pool.bad_debt_lock_d != 0 {
                pool.bad_debt_lock_d = 0;

                events::pool_bad_debt_unlocked(e, &pool_address);
            }

            pool.set(e);

            obligation.try_remove_borrow_position(e, &pool_address)?;
            completed_requests.push_back((pool_address, request_id));
        }
    }

    for (pool_addr, req_id) in completed_requests {
        obligation.insurance_fund_requests_ids.remove((pool_addr, req_id));
    }

    if obligation.is_empty() {
        obligation.remove(e, &obligation_key);
    } else {
        obligation.set(e, &obligation_key);
    }

    events::claim_cover_bad_debt_results(e, obligation_key);

    Ok(())
}

pub fn process_swap_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    path: &Vec<Address>,
    amount_in: i128,
    min_amount_out: i128,
) -> Result<i128, MCError> {
    require_nonnegative(amount_in)?;
    require_nonnegative(min_amount_out)?;

    if path.len() < 2 || path.first() == path.last() {
        return Err(MCError::InvalidSwap);
    }

    let received_amount =
        swap::swap_exact(e, swap_provider, user, path, amount_in, min_amount_out)?;

    if received_amount < min_amount_out {
        events::inconsistent_swap_received_amount(
            e,
            swap_provider,
            path,
            received_amount,
            min_amount_out,
        );

        return Err(MCError::SwapSlippageExceeded);
    }

    events::swap_exact(e, swap_provider, user, path, amount_in, min_amount_out, received_amount);

    Ok(received_amount)
}

pub fn process_swap_for_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    path: &Vec<Address>,
    max_amount_in: i128,
    amount_out: i128,
) -> Result<i128, MCError> {
    require_nonnegative(max_amount_in)?;
    require_nonnegative(amount_out)?;

    if path.len() < 2 || path.first() == path.last() {
        return Err(MCError::InvalidSwap);
    }

    let sent_amount =
        swap::swap_for_exact(e, swap_provider, user, path, max_amount_in, amount_out)?;

    if sent_amount > max_amount_in {
        events::inconsistent_swap_sent_amount(e, swap_provider, path, sent_amount, max_amount_in);

        return Err(MCError::SwapSlippageExceeded);
    }

    events::swap_for_exact(e, swap_provider, user, path, max_amount_in, amount_out, sent_amount);

    Ok(sent_amount)
}

pub fn process_distribute_pool_fees(e: &Env, pool_address: Address) -> Result<(), MCError> {
    let mut pool = Pool::try_get(e, &pool_address)?;
    let insurance_fund_addr = storage::get_insurance_fund(e);

    let token_client = token::Client::new(e, &pool.token_address);
    let market_addr = e.current_contract_address();

    if pool.total_available > pool.take_rate_fees_sum {
        // -- Distribute Take Rate Fees --

        if let Some(take_rate_beneficiaries) = &pool.config.fee_config.take_rate_beneficiaries {
            for (beneficiary_address, share_bps) in take_rate_beneficiaries.iter() {
                if share_bps == 0 {
                    continue;
                }
                let amount = pool
                    .take_rate_fees_sum
                    .fixed_mul_floor(share_bps as i128, BPS_FACTOR)
                    .map_over_or_underflow()?;

                token_client.transfer(&market_addr, &beneficiary_address, &amount);

                if beneficiary_address == insurance_fund_addr {
                    let fund_client = InsuranceFundClient::new(e, &insurance_fund_addr);
                    fund_client.add_reserves(&pool.token_address, &amount);
                }
            }
        }
        pool.adjust_total_available(
            e,
            pool.take_rate_fees_sum.checked_neg().map_over_or_underflow()?,
        )?;
        pool.take_rate_fees_sum = 0;
    }

    // -- Distribute Operation Fees --

    if let Some(operation_beneficiaries) = &pool.config.fee_config.operation_fee_beneficiaries {
        for (beneficiary_address, share_bps) in operation_beneficiaries.iter() {
            if share_bps == 0 {
                continue;
            }
            let amount = pool
                .operation_fees_sum
                .fixed_mul_floor(share_bps as i128, BPS_FACTOR)
                .map_over_or_underflow()?;

            token_client.transfer(&market_addr, &beneficiary_address, &amount);

            if beneficiary_address == insurance_fund_addr {
                let fund_client = InsuranceFundClient::new(e, &insurance_fund_addr);
                fund_client.add_reserves(&pool.token_address, &amount);
            }
        }
    }
    pool.operation_fees_sum = 0;

    pool.set(e);

    events::distribute_pool_fees(e, pool_address);

    Ok(())
}

pub fn process_distribute_all_pools_fees(e: &Env) -> Result<(), MCError> {
    for pool_address in Pool::get_all(e) {
        process_distribute_pool_fees(e, pool_address)?;
    }

    Ok(())
}

pub fn process_refresh_obligation(e: &Env, obligation_key: ObligationKey) -> Result<(), MCError> {
    let obligation = Obligation::try_get(e, &obligation_key)?;
    obligation.accrue_interest(e)?;

    events::refresh_obligation(e, obligation_key);

    Ok(())
}

// -- Helpers --
pub fn obligation_map_to_event(obligation: Obligation) -> Option<Obligation> {
    if !obligation.is_empty() { Some(obligation) } else { None }
}
