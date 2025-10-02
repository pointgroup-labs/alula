// use aggregated_oracle::PriceFeedClient;
use moderc3156::FlashLoanClient;
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, BytesN, Env, Symbol,
    token::{self, TokenClient},
};

use crate::{
    accrual::AccrualModel,
    constants::*,
    error::MCError,
    events,
    helpers::require_nonnegative,
    interest_rate_model::{InterestRateModel, kinked::KinkedIRConfig},
    math_utils::MathUtils,
    multiply_pair::MultiplyPair,
    obligation::{
        AddCollateralResult, BorrowResult, CoverBadDebtResult, DepositResult, LiquidationValues,
        Obligation, ObligationKey, RemoveCollateralResult, RepayResult, WithdrawResult,
    },
    pool::{Pool, PoolConfig},
    swap,
};

pub fn process_initialize_pool(
    e: &Env,
    token_address: &Address,
    token_ticker: &Symbol,
    salt: &Option<BytesN<32>>,
    pool_config: &Option<PoolConfig>,
    // TODO: Option<InterestRateModel>,
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

        accumulated_host_fees: 0,
        accumulated_market_fees: 0,
        accumulated_reserve_fees: 0,

        name,
        // accrual_model,
        // interest_rate_model,
        config: pool_config,
        pool_address: pool_address.clone(),
        token_ticker: token_ticker.clone(),
        token_address: token_address.clone(),
        last_accrual_timestamp: e.ledger().timestamp(),
        // fee_config: Default::default(),
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

    let (collateral_pool, borrow_pool) = (
        Pool::try_get(e, deposit_pool_address).map_err(|_| MCError::DepositPoolDoesNotExist)?,
        Pool::try_get(e, borrow_pool_address).map_err(|_| MCError::BorrowPoolDoesNotExist)?,
    );

    let pair = MultiplyPair::new(
        e,
        deposit_pool_address,
        borrow_pool_address,
        borrow_pool.config.health_config.open_ltv_bps,
        borrow_pool.config.fee_config.flash_loan_fee_bps as i128,
        collateral_pool.config.health_config.liability_factor_bps,
    );

    pair.set(e);
    pair.register(e);

    Ok(())
}

pub fn process_deposit(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut pool = Pool::try_get(e, pool_address)?;
    pool.accrue_interest(e)?;

    let supply_limit = pool.config.health_config.supply_limit;

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

    let DepositResult {
        j_tokens_to_issue,
        deposited,
        market_fee,
        host_fee,
    } = obligation.deposit(e, &pool, amount)?;

    pool.adjust_total_j_tokens(e, j_tokens_to_issue)?;
    pool.adjust_total_available(e, deposited)?;

    pool.adjust_accumulated_host_fees(e, host_fee)?;
    pool.adjust_accumulated_market_fees(e, market_fee)?;

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

pub fn process_borrow(
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

    let BorrowResult {
        d_tokens_to_issue,
        borrower_new_debt,
        borrower_to_receive,
        market_fee,
        host_fee,
    } = obligation.borrow(e, &pool, amount)?;

    pool.adjust_total_d_tokens(e, d_tokens_to_issue)?;
    pool.adjust_total_borrowed(e, borrower_new_debt)?;
    pool.adjust_total_available(e, borrower_new_debt.checked_neg().map_over_or_underflow()?)?;

    pool.adjust_accumulated_host_fees(e, host_fee)?;
    pool.adjust_accumulated_market_fees(e, market_fee)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(
        &e.current_contract_address(),
        &obligation_key.user,
        &borrower_to_receive,
    );

    // TODO: Fix event
    // events::borrow(
    //     e,
    //     pool_address,
    //     &obligation_key.user,
    //     d_tokens_to_issue,
    //     borrower_new_debt,
    //     borrower_to_receive,
    //     market_fee,
    //     host_fee,
    // );

    Ok(())
}

pub fn process_add_collateral(
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

    let AddCollateralResult {
        added_collateral,
        market_fee,
        host_fee,
    } = obligation.add_collateral(e, &pool, amount)?;

    pool.adjust_total_collateral(e, added_collateral)?;

    pool.adjust_accumulated_host_fees(e, host_fee)?;
    pool.adjust_accumulated_market_fees(e, market_fee)?;

    obligation.set(e, obligation_key);
    pool.set(e);

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(&obligation_key.user, &e.current_contract_address(), &amount);

    // TODO: fix event
    // events::add_collateral(e, pool_address, &obligation_key.user, amount);

    Ok(())
}

pub fn process_repay(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let RepayResult {
        host_fee,
        market_fee,
        debt_repaid,
        d_tokens_to_burn,
        amount_to_take_from_borrower,
    } = obligation.repay(e, &pool, amount)?;

    pool.adjust_total_d_tokens(e, d_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;
    pool.adjust_total_borrowed(e, debt_repaid.checked_neg().map_over_or_underflow()?)?;
    pool.adjust_total_available(e, debt_repaid)?;

    pool.adjust_accumulated_host_fees(e, host_fee)?;
    pool.adjust_accumulated_market_fees(e, market_fee)?;

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
        &amount_to_take_from_borrower,
    );

    // TODO: Fix event
    // events::repay(e, pool_address, obligation_key, real_repaid_amount);

    Ok(())
}

pub fn process_remove_collateral(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    require_nonnegative(amount)?;

    let mut obligation = Obligation::try_get(e, obligation_key)?;
    obligation.accrue_interest(e)?;

    let mut pool = Pool::try_get(e, pool_address)?;

    let RemoveCollateralResult {
        collateral_decrease,
        collateral_remover_to_receive,
        market_fee,
        host_fee,
    } = obligation.remove_collateral(e, &pool, amount)?;

    pool.adjust_total_collateral(
        e,
        collateral_decrease.checked_neg().map_over_or_underflow()?,
    )?;

    pool.adjust_accumulated_host_fees(e, host_fee)?;
    pool.adjust_accumulated_market_fees(e, market_fee)?;

    pool.set(e);

    if obligation.is_empty() {
        obligation.remove(e, obligation_key);
    } else {
        obligation.set(e, obligation_key);
    }

    let token_client = token::Client::new(e, &pool.token_address);
    token_client.transfer(
        &e.current_contract_address(),
        &obligation_key.user,
        &collateral_remover_to_receive,
    );

    // TODO: Fix event
    // events::remove_collateral(e, pool_address, &obligation_key.user, removed_tokens_amount);

    Ok(())
}

pub fn process_withdraw(
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

    let WithdrawResult {
        j_tokens_to_burn,
        deposit_decrease,
        withdrawer_to_receive,
        market_fee,
        host_fee,
    } = obligation.withdraw(e, &pool, amount)?;

    pool.adjust_total_available(e, deposit_decrease.checked_neg().map_over_or_underflow()?)?;
    pool.adjust_total_j_tokens(e, j_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;

    pool.adjust_accumulated_host_fees(e, host_fee)?;
    pool.adjust_accumulated_market_fees(e, market_fee)?;

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
        &withdrawer_to_receive,
    );

    // TODO: Fix events
    // events::withdraw(
    //     e,
    //     pool_address,
    //     obligation_key,
    //     j_tokens_burnt,
    //     real_withdrawn_amount,
    // );

    Ok(())
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
    token_client.transfer(&e.current_contract_address(), contract, &amount); // plain transfer, not allowance transfer?

    let flash_loan_fee_bps = pool.config.fee_config.flash_loan_fee_bps as i128;

    let flash_loan_taker_client = FlashLoanClient::new(e, contract);
    flash_loan_taker_client.exec_op(
        &e.current_contract_address(),
        &pool.token_address,
        &amount,
        &flash_loan_fee_bps,
    );

    // WARN: Does this have enough precision?
    let fees = amount
        .fixed_mul_floor(flash_loan_fee_bps, BPS_FACTOR)
        .map_over_or_underflow()?;
    let amount_to_repay = amount.checked_add(fees).map_over_or_underflow()?;

    // NB: Here you must issue `transfer_allowance`, since it's safer for a flash loan taker to
    // to implement their flash loan logic flow
    token_client.transfer(contract, &e.current_contract_address(), &amount_to_repay);

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

    let seed = pair.seed.clone();
    let obligation_key = ObligationKey::new_with_seed(user.clone(), seed);

    process_deposit(e, &obligation_key, deposit_pool_address, deposit_amount)?;

    // -- Borrow to repay the flash loan --
    let flash_loan_fee = flash_borrow_amount
        .fixed_mul_ceil(
            borrow_pool.config.fee_config.flash_loan_fee_bps as i128,
            BPS_FACTOR,
        )
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
    let seed = pair.seed.clone();

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
        borrow_pool.config.fee_config.flash_loan_fee_bps,
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
        .fixed_mul_ceil(
            borrow_pool.config.fee_config.flash_loan_fee_bps as i128,
            BPS_FACTOR,
        )
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

pub fn process_liquidate(
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
    collateral_pool.adjust_total_collateral(
        e,
        collateral_amount_sold
            .checked_neg()
            .map_over_or_underflow()?,
    )?;

    borrow_pool
        .adjust_total_borrowed(e, liquidated_amount.checked_neg().map_over_or_underflow()?)?;
    borrow_pool.adjust_total_d_tokens(e, d_tokens_repaid.checked_neg().map_over_or_underflow()?)?;

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

pub fn process_redeem_accumulated_host_fees(
    e: &Env,
    user: &Address,
    pool_address: &Address,
    amount: i128,
) -> Result<(), MCError> {
    let mut pool = Pool::try_get(e, pool_address)?;

    let fees_to_be_redeemed = i128::min(amount, pool.accumulated_host_fees);

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

    let CoverBadDebtResult {
        borrows_to_be_repaid_from_reserves,
        collaterals_to_remove,
    } = obligation.cover_bad_debt(e)?;

    for (pool_address, d_tokens) in borrows_to_be_repaid_from_reserves {
        let mut pool = Pool::try_get(e, &pool_address).map_err(|_| {
            events::pool_is_missing_in_storage(e, &pool_address);

            MCError::InternalError
        })?;

        let obligation_pool_debt = pool.compute_tokens_from_d_tokens(e, d_tokens)?;
        let available_reserve_fees = pool.available_accumulated_reserve_fees();

        let debt_can_be_covered = i128::min(obligation_pool_debt, available_reserve_fees);
        let d_tokens_can_be_covered = pool.compute_d_tokens_from_tokens(e, debt_can_be_covered)?;

        // - Cover what can be covered from the reserves -

        pool.adjust_total_available(e, debt_can_be_covered)?;
        pool.adjust_total_borrowed(
            e,
            debt_can_be_covered.checked_neg().map_over_or_underflow()?,
        )?;
        pool.adjust_accumulated_reserve_fees(
            e,
            debt_can_be_covered.checked_neg().map_over_or_underflow()?,
        )?;
        pool.adjust_total_d_tokens(
            e,
            d_tokens_can_be_covered
                .checked_neg()
                .map_over_or_underflow()?,
        )?;

        // - Socialize all remaining bad debt -

        if obligation_pool_debt > debt_can_be_covered {
            let left_to_socialize = obligation_pool_debt - debt_can_be_covered; // safe
            let d_tokens_left = d_tokens
                .checked_sub(d_tokens_can_be_covered)
                .map_over_or_underflow()?;

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
    flash_loan_fee_bps: u32,
) -> Result<i128, MCError> {
    require_nonnegative(deposited_amount)?;
    require_nonnegative(borrowed_amount)?;

    let flash_loan_fee = borrowed_amount
        .fixed_mul_ceil(flash_loan_fee_bps as i128, BPS_FACTOR)
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
