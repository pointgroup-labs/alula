//! Encapsulates operations related to the swapping of two
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env};

use crate::{
    constants::{BPS_FACTOR, DEFAULT_MAX_SLIPPAGE_BPS, SOROSWAP_ROUTER_TESTNET_ADDRESS},
    math_utils::MathUtils,
    soroswap_router, LCError,
};
// TODO: Maybe, create some internal trait for common swap operations and
// implement it for different swap providers?

/// Gets the amount that the user must provide to receive a specific amount if a swap is performed
/// at the current moment
///
/// ### Arguments
/// * `token_in` - address of a token that would be taken from the user
/// * `token_out` - address of a token that would be given to the user
/// * `amount_out` - an exact amount of `token_in` that would be given to the user
pub fn get_amount_in(
    e: &Env,
    token_in: &Address,
    token_out: &Address,
    amount_out: i128,
) -> Result<i128, LCError> {
    let path = soroban_sdk::vec![&e, token_in.clone(), token_out.clone()];
    let soroswap_router_client =
        soroswap_router::Client::new(e, &Address::from_str(e, SOROSWAP_ROUTER_TESTNET_ADDRESS));

    let amounts_in = soroswap_router_client.router_get_amounts_in(&amount_out, &path);
    let Some(amount_in) = amounts_in.first() else {
        return Err(LCError::DependencyContractError);
    };

    Ok(amount_in)
}

/// Gets the amount that user would receive if performed a swap at the current moment
///
/// ### Arguments
/// * `token_in` - address of a token that would be taken from the user
/// * `token_out` - address of a token that would be given to the user
/// * `amount_in` - an exact amount of `token_in` that would be taken from the user
pub fn get_amount_out(
    e: &Env,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
) -> Result<i128, LCError> {
    let path = soroban_sdk::vec![&e, token_in.clone(), token_out.clone()];
    let soroswap_router_client =
        soroswap_router::Client::new(e, &Address::from_str(e, SOROSWAP_ROUTER_TESTNET_ADDRESS));

    let amounts_out = soroswap_router_client.router_get_amounts_out(&amount_in, &path);
    let Some(amount_out) = amounts_out.last() else {
        return Err(LCError::DependencyContractError);
    };

    Ok(amount_out)
}

/// Swaps user's tokens
///
/// ### Arguments
/// * `user` - user that performs a swap
/// * `token_in` - address of a token that is taken from the user
/// * `token_out` - address of a token that is given to the user
/// * `amount_in` - amount of the desired `token_in`
/// * `amount_out` - exact amount of the `token_out`
/// * `max_slippage_bps` - basis points percentage of the maximum allowed `amount_in` slippage
///
/// # Returns
/// Taken from user `token_in` amount
pub fn swap_tokens_for_exact_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    max_slippage_bps: Option<i128>,
) -> Result<i128, LCError> {
    let max_slippage_bps = if let Some(slippage) = max_slippage_bps {
        if !(0..=BPS_FACTOR).contains(&slippage) {
            return Err(LCError::InvalidSwapSlippage);
        }

        slippage
    } else {
        DEFAULT_MAX_SLIPPAGE_BPS
    };

    let soroswap_router_client =
        soroswap_router::Client::new(e, &Address::from_str(e, SOROSWAP_ROUTER_TESTNET_ADDRESS));

    let amount_in_max = amount_in
        .checked_add(
            amount_in
                .fixed_mul_floor(max_slippage_bps, BPS_FACTOR)
                .map_over_or_underflow()?,
        )
        .map_over_or_underflow()?;

    let path = soroban_sdk::vec![e, token_in.clone(), token_out.clone()];

    // TODO: For now we can only swap tokens with a direct path
    let swap_amounts = soroswap_router_client.swap_tokens_for_exact_tokens(
        &amount_out,
        &amount_in_max,
        &path,
        user,
        &u64::MAX,
    );

    // TODO: What warning\error\event exactly must happen here?
    let received_amount = swap_amounts
        .last()
        .ok_or(LCError::DependencyContractError)?;

    Ok(received_amount)
}

/// Swaps user's tokens
///
/// ### Arguments
/// * `user` - user that performs a swap
/// * `token_in` - address of a token that is taken from the user
/// * `token_out` - address of a token that is given to the user
/// * `amount_in` - exact amount of the `token_in`
/// * `amount_out` - desired amount of the `token_out`
/// * `max_slippage_bps` - basis points percentage of the maximum allowed `amount_out` slippage.
///   [`DEFAULT_MAX_SLIPPAGE_BPS`] if [`None`]
/// # Returns
/// Given to user `token_out` amount
pub fn swap_exact_tokens_for_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    max_slippage_bps: Option<i128>,
) -> Result<i128, LCError> {
    let max_slippage_bps = if let Some(slippage) = max_slippage_bps {
        if !(0..=BPS_FACTOR).contains(&slippage) {
            return Err(LCError::InvalidSwapSlippage);
        }

        slippage
    } else {
        DEFAULT_MAX_SLIPPAGE_BPS
    };

    let soroswap_router_client =
        soroswap_router::Client::new(e, &Address::from_str(e, SOROSWAP_ROUTER_TESTNET_ADDRESS));

    let amount_out_min = amount_out
        .checked_sub(
            amount_out
                .fixed_mul_floor(max_slippage_bps, BPS_FACTOR)
                .map_over_or_underflow()?,
        )
        .map_over_or_underflow()?;

    let path = soroban_sdk::vec![e, token_in.clone(), token_out.clone()];

    // TODO: For now we can only swap tokens with a direct path
    let swap_amounts = soroswap_router_client.swap_exact_tokens_for_tokens(
        &amount_in,
        &amount_out_min,
        &path,
        user,
        &u64::MAX,
    );

    // TODO: What warning\error\event exactly must happen here?
    let received_amount = swap_amounts
        .last()
        .ok_or(LCError::DependencyContractError)?;

    Ok(received_amount)
}
