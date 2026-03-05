//! Encapsulates operations related to the swapping of two tokens via a chosen swap provider

use soroban_sdk::{
    Address, Env,
};

use crate::{error::MCError, storage};

pub fn swap_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    min_amount_out: i128,
) -> Result<i128, MCError> {
    let provider_addr = storage::get_swap_provider(e);
    let proxy_swap_contract_client = proxy_swap_interface::ProxySwapClient::new(e, &provider_addr);

    Ok(proxy_swap_contract_client.swap_exact(
        swap_provider,
        user,
        token_in,
        token_out,
        &amount_in,
        &min_amount_out,
    ))
}

pub fn swap_for_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in_max: i128,
    amount_out: i128,
) -> Result<i128, MCError> {
    let proxy_swap = storage::get_swap_provider(e);
    let proxy_swap_contract_client = proxy_swap_interface::ProxySwapClient::new(e, &proxy_swap);

    Ok(proxy_swap_contract_client.swap_for_exact(
        swap_provider,
        user,
        token_in,
        token_out,
        &amount_in_max,
        &amount_out,
    ))
}
