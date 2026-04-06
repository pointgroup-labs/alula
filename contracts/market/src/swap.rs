//! Encapsulates operations related to the swapping of two tokens via a chosen swap provider

use soroban_sdk::{Address, Env, Vec};

use crate::error::MCError;

pub fn swap_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    path: &Vec<Address>,
    amount_in: i128,
    min_amount_out: i128,
) -> Result<i128, MCError> {
    let proxy_swap_contract_client = proxy_swap_interface::ProxySwapClient::new(e, swap_provider);

    Ok(proxy_swap_contract_client.swap_exact(user, path, &amount_in, &min_amount_out))
}

pub fn swap_for_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    path: &Vec<Address>,
    max_amount_in: i128,
    amount_out: i128,
) -> Result<i128, MCError> {
    let proxy_swap_contract_client = proxy_swap_interface::ProxySwapClient::new(e, swap_provider);

    Ok(proxy_swap_contract_client.swap_for_exact(user, path, &max_amount_in, &amount_out))
}
