//! Encapsulates operations related to the swapping of two

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, IntoVal, Symbol,
    auth::{ContractContext, InvokerContractAuthEntry, SubContractInvocation},
    vec,
};

use crate::{constants::*, error::MCError, math_utils::MathUtils, soroswap_router as router};

// TODO: Maybe, create some internal trait for common swap operations and
//  implement it for different swap providers?

// Gets the amount that the user must provide to receive a specific amount if a swap is performed
// at the current moment
//
// # Arguments
// * `token_in` - address of a token that would be taken from the user
// * `token_out` - address of a token that would be given to the user
// * `amount_out` - an exact amount of `token_in` that would be given to the user
//
// # Returns
// Amount of `token_in` that must be provided by the user
pub fn get_amount_in(
    e: &Env,
    token_in: &Address,
    token_out: &Address,
    amount_out: i128,
) -> Result<i128, MCError> {
    if token_in == token_out {
        return Err(MCError::SwappingIdenticalTokens);
    }

    let path = vec![&e, token_in.clone(), token_out.clone()];
    let router_client = router::Client::new(e, &Address::from_str(e, ROUTER_ADDRESS));

    let amounts_in = router_client.router_get_amounts_in(&amount_out, &path);
    let Some(amount_in) = amounts_in.first() else {
        return Err(MCError::DependencyContractError);
    };

    Ok(amount_in)
}

// Gets the amount that user would receive if performed a swap at the current moment
//
// # Arguments
// * `token_in` - address of a token that would be taken from the user
// * `token_out` - address of a token that would be given to the user
// * `amount_in` - an exact amount of `token_in` that would be taken from the user
//
// # Returns
// Amount of `token_out` that would be given to the user
pub fn get_amount_out(
    e: &Env,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
) -> Result<i128, MCError> {
    if token_in == token_out {
        return Err(MCError::SwappingIdenticalTokens);
    }

    let path = vec![&e, token_in.clone(), token_out.clone()];
    let router_client = router::Client::new(e, &Address::from_str(e, ROUTER_ADDRESS));

    let amounts_out = router_client.router_get_amounts_out(&amount_in, &path);
    let Some(amount_out) = amounts_out.last() else {
        return Err(MCError::DependencyContractError);
    };

    Ok(amount_out)
}

// Swaps user's tokens
//
// # Arguments
// * `user` - user that performs a swap
// * `token_in` - address of a token that is taken from the user
// * `token_out` - address of a token that is given to the user
// * `amount_in` - amount of the desired `token_in`
// * `amount_out` - exact amount of the `token_out`
// * `max_slippage_bps` - basis points percentage of the maximum allowed `amount_in` slippage
//
// # Returns
// Taken from user `token_in` amount
pub fn swap_tokens_for_exact_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    max_slippage_bps: Option<i128>,
) -> Result<i128, MCError> {
    if token_in == token_out {
        return Err(MCError::SwappingIdenticalTokens);
    }

    let max_slippage_bps = resolve_max_slippage(max_slippage_bps)?;
    let router_address = Address::from_str(e, ROUTER_ADDRESS);
    let router_client = router::Client::new(e, &router_address);
    let pair = router_client.router_pair_for(token_in, token_out);

    let amount_in_max = amount_in
        .checked_add(
            amount_in.fixed_mul_floor(max_slippage_bps, BPS_FACTOR).map_over_or_underflow()?,
        )
        .map_over_or_underflow()?;

    // TODO: For now we swap tokens with a direct path only
    let path = vec![e, token_in.clone(), token_out.clone()];

    if user == &e.current_contract_address() {
        let auth_entry = InvokerContractAuthEntry::Contract(SubContractInvocation {
            context: ContractContext {
                contract: token_in.clone(),
                fn_name: Symbol::new(e, "transfer"),
                args: (e.current_contract_address(), pair, amount_in_max as i128).into_val(e),
            },
            sub_invocations: vec![&e],
        });
        e.authorize_as_current_contract(soroban_sdk::vec![e, auth_entry]);
    }

    let swap_amounts = router_client.swap_tokens_for_exact_tokens(
        &amount_out,
        &amount_in_max,
        &path,
        user,
        &u64::MAX,
    );

    let received_amount = swap_amounts.last().ok_or(MCError::DependencyContractError)?;

    Ok(received_amount)
}

// Swaps user's tokens
//
// # Arguments
// * `user` - user that performs a swap
// * `token_in` - address of a token that is taken from the user
// * `token_out` - address of a token that is given to the user
// * `amount_in` - exact amount of the `token_in`
// * `amount_out` - desired amount of the `token_out`
// * `max_slippage_bps` - basis points percentage of the maximum allowed `amount_out` slippage.
//   [`DEFAULT_MAX_SLIPPAGE_BPS`] if [`None`]
//
// # Returns
// Given to user `token_out` amount
pub fn swap_exact_tokens_for_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    max_slippage_bps: Option<i128>,
) -> Result<i128, MCError> {
    if token_in == token_out {
        return Err(MCError::SwappingIdenticalTokens);
    }

    let max_slippage_bps = resolve_max_slippage(max_slippage_bps)?;
    let router_client = router::Client::new(e, &Address::from_str(e, ROUTER_ADDRESS));

    let amount_out_min = amount_out
        .checked_sub(
            amount_out.fixed_mul_floor(max_slippage_bps, BPS_FACTOR).map_over_or_underflow()?,
        )
        .map_over_or_underflow()?;

    let path = vec![e, token_in.clone(), token_out.clone()];

    // TODO: For now we can only swap tokens with a direct path

    let swap_amounts = router_client.swap_exact_tokens_for_tokens(
        &amount_in,
        &amount_out_min,
        &path,
        user,
        &u64::MAX,
    );

    let received_amount = swap_amounts.last().ok_or(MCError::DependencyContractError)?;

    Ok(received_amount)
}

// Resolves the max slippage basis points percentage
//
// # Arguments
// * `max_slippage_bps` - optional basis points percentage of the maximum allowed slippage
//
// # Returns
// Resolved basis points percentage of the maximum allowed slippage
fn resolve_max_slippage(max_slippage_bps: Option<i128>) -> Result<i128, MCError> {
    if let Some(slippage) = max_slippage_bps {
        if !(0..=BPS_FACTOR).contains(&slippage) {
            return Err(MCError::InvalidSwapSlippage);
        }
        Ok(slippage)
    } else {
        Ok(DEFAULT_MAX_SLIPPAGE_BPS)
    }
}
