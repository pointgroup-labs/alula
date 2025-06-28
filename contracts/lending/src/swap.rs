//! Encapsulates operations related to the swapping of two tokens.

use {
    crate::{
        constants::{LCError, BPS_FACTOR, DEFAULT_MAX_SLIPPAGE_BPS},
        math_utils::MathUtils,
    },
    soroban_sdk::{
        token::{StellarAssetClient, TokenClient},
        Address, Env,
    },
};

/// Gets the amount that user would receive if performed a swap at the current moment
///
/// ### Arguments
/// * `token_in` - address of a token that would be taken from the user
/// * `token_out` - address of a token that would be given to the user
/// * `amount_in` - an exact amount of `token_in` that would be taken from the user
pub fn get_amount_out(
    _e: &Env,
    _token_in: &Address,
    _token_out: &Address,
    amount_in: i128,
) -> Result<i128, LCError> {
    Ok(amount_in)
}

/// Swaps user's tokens
///
/// ### Arguments
/// * `user` - user that performs a swap
/// * `token_in` - address of a token that is taken from the user
/// * `token_out` - address of a token that is given to the user
/// * `amount_in` - exact amount of the `token_in`
/// * `amount_out` - desired amount of the `token_out`
/// * `max_slippage_bps` - basis points percentage of the maximum allowed `amount_out` slippage
pub fn swap_exact_tokens_for_tokens(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    max_slippage_bps: Option<i128>,
) -> Result<i128, LCError> {
    // TODO: `Swap` logic must be encapsulated in a module
    // so that whenever we want to change the way we do our swap - it's not tedious
    // Mint and burn for now...

    let max_slippage_bps = if let Some(slippage) = max_slippage_bps {
        if slippage <= 0 || slippage > BPS_FACTOR {
            return Err(LCError::InvalidSwapSlippage);
        }

        slippage
    } else {
        DEFAULT_MAX_SLIPPAGE_BPS
    };

    let sac_client = StellarAssetClient::new(e, token_out);
    let token_client = TokenClient::new(e, token_in);

    let max_slippage_amount = amount_in
        .checked_mul(max_slippage_bps)
        .map_over_or_underflow()?
        .checked_div(BPS_FACTOR)
        .map_over_or_underflow()?;

    let amount_out = amount_out
        .checked_sub(max_slippage_amount)
        .map_over_or_underflow()?;

    sac_client.mint(user, &amount_out);
    token_client.burn(user, &amount_in);

    // >>>> DRAFT >>>>
    // // Swap all initial and borrowed tokens
    // let soroswap_router_address =
    //     Address::from_string(&String::from_str(&e, SOROSWAP_ROUTER_TESTNET_ADDRESS));
    // let soroswap_router_contract = soroswap_router::Client::new(&e, &swap_router_address);

    // let (mut reserve_a, mut reserve_b) = (0, 0);

    // // #[cfg(feature = "deploy")]
    // // {
    // //     let factory = soroswap_router_contract.get_factory();

    // //     let (a, b) = soroswap_library::get_reserves_with_factory(
    // //         e.clone(),
    // //         factory.clone(),
    // //         collateral_pool.token_address.clone(),
    // //         deposit_pool.token_address.clone(),
    // //     )
    // //     .unwrap();

    // //     reserve_a = a;
    // //     reserve_b = b;
    // // };

    // #[cfg(not(feature = "deploy"))]
    // {
    //     reserve_a = 1_000_000;
    //     reserve_b = 1_000_000;
    // };

    // let amount_in = collateral_amount
    //     .checked_add(flash_borrow_amount)
    //     .map_over_or_underflow()?;

    // let amount_out = soroswap_router_contract.router_get_amount_out(&amount_in, &reserve_a, &reserve_b);

    // let amount_out_min = amount_out
    //     .checked_mul(BPS_FACTOR - TEST_SLIPPAGE_BPS)
    //     .map_over_or_underflow()?
    //     .checked_div(BPS_FACTOR)
    //     .map_over_or_underflow()?;

    // let path = soroban_sdk::vec![
    //     &e,
    //     collateral_pool.token_address,
    //     deposit_pool.token_address.clone()
    // ];

    // let amounts = soroswap_router_contract.swap_exact_tokens_for_tokens(
    //     &amount_in,
    //     &amount_out_min,
    //     &path,
    //     &user,
    //     &0,
    // );
    // <<<< DRAFT <<<<

    Ok(amount_out)
}
