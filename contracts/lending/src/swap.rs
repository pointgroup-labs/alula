//! Encapsulates operations related to the swapping of two tokens.

use {
    crate::{
        constants::{
            LCError, BPS_FACTOR, DEFAULT_MAX_SLIPPAGE_BPS, SOROSWAP_ROUTER_TESTNET_ADDRESS,
        },
        math_utils::MathUtils,
        soroswap_router,
    },
    core::u64,
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{Address, Env},
};

/// Gets the amount that user would receive if performed a swap at the current moment
///
/// ### Arguments
/// * `token_in` - address of a token that would be taken from the user
/// * `token_out` - address of a token that would be given to the user
/// * `amount_in` - an exact amount of `token_in` that would be taken from the user
#[allow(unused)]
pub fn get_amount_out(
    e: &Env,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
) -> Result<i128, LCError> {
    let amount_out;

    #[cfg(feature = "deploy")]
    {
        use {crate::constants::SOROSWAP_FACTORY_TESTNET_ADDRESS, soroswap_library};

        let soroswap_router_client = soroswap_router::Client::new(
            &e,
            &Address::from_str(e, SOROSWAP_ROUTER_TESTNET_ADDRESS),
        );

        let soroswap_factory_address = Address::from_str(e, SOROSWAP_FACTORY_TESTNET_ADDRESS);
        let (reserve_in, reserve_out) = soroswap_library::get_reserves_with_factory(
            e.clone(),
            soroswap_factory_address,
            token_in.clone(),
            token_out.clone(),
        )
        .unwrap();
        amount_out =
            soroswap_router_client.router_get_amount_out(&amount_in, &reserve_in, &reserve_out);
    }

    #[cfg(not(feature = "deploy"))]
    {
        amount_out = amount_in;
    }

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
        if slippage <= 0 || slippage > BPS_FACTOR {
            return Err(LCError::InvalidSwapSlippage);
        }

        slippage
    } else {
        DEFAULT_MAX_SLIPPAGE_BPS
    };

    let soroswap_router_client =
        soroswap_router::Client::new(&e, &Address::from_str(&e, SOROSWAP_ROUTER_TESTNET_ADDRESS));

    let amount_in_max = amount_in
        .checked_add(
            amount_in
                .fixed_div_floor(max_slippage_bps, BPS_FACTOR)
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
        &u64::MAX, // WARN: What should be this deadline here?
    );

    // TODO: What warning\error\event exactly must happen here?
    let received_amount = swap_amounts.last().ok_or(LCError::InternalError)?;

    Ok(received_amount)

    // todo!()
    // let max_slippage_bps = if let Some(slippage) = max_slippage_bps {
    //     if slippage <= 0 || slippage > BPS_FACTOR {
    //         return Err(LCError::InvalidSwapSlippage);
    //     }

    //     slippage
    // } else {
    //     DEFAULT_MAX_SLIPPAGE_BPS
    // };

    // let sac_client = StellarAssetClient::new(e, token_out);
    // let token_client = TokenClient::new(e, token_in);

    // let max_slippage_amount = amount_in
    //     .checked_mul(max_slippage_bps)
    //     .map_over_or_underflow()?
    //     .checked_div(BPS_FACTOR)
    //     .map_over_or_underflow()?;

    // let amount_in = amount_in
    //     .checked_sub(max_slippage_amount)
    //     .map_over_or_underflow()?;

    // sac_client.mint(user, &amount_out);
    // token_client.burn(user, &amount_in);

    // Ok(amount_in)
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
        if slippage <= 0 || slippage > BPS_FACTOR {
            return Err(LCError::InvalidSwapSlippage);
        }

        slippage
    } else {
        DEFAULT_MAX_SLIPPAGE_BPS
    };

    let soroswap_router_client =
        soroswap_router::Client::new(&e, &Address::from_str(&e, SOROSWAP_ROUTER_TESTNET_ADDRESS));

    let amount_out_min = amount_out
        .checked_sub(
            amount_out
                .fixed_div_floor(max_slippage_bps, BPS_FACTOR)
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
        &u64::MAX, // WARN: What should be this deadline here?
    );

    // TODO: What warning\error\event exactly must happen here?
    let received_amount = swap_amounts.last().ok_or(LCError::InternalError)?;

    Ok(received_amount)

    // let sac_client = StellarAssetClient::new(e, token_out);
    // let token_client = TokenClient::new(e, token_in);

    // let max_slippage_amount = amount_in
    //     .checked_mul(max_slippage_bps)
    //     .map_over_or_underflow()?
    //     .checked_div(BPS_FACTOR)
    //     .map_over_or_underflow()?;

    // let amount_out = amount_out
    //     .checked_sub(max_slippage_amount)
    //     .map_over_or_underflow()?;

    // sac_client.mint(user, &amount_out);
    // token_client.burn(user, &amount_in);

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

    // Ok(amount_out)
}
