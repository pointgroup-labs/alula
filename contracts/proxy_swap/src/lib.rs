#![no_std]
use constants::*;
use soroban_sdk::{Address, Env, contract, contractimpl};

use crate::{
    aqua_router::AquaRouter,
    error::PSCError,
    soroswap_router::SoroswapRouter,
    swap_trait::{Swap, SwapProvider},
};

mod aqua_router;
mod constants;
mod error;
mod soroswap_router;
mod swap_trait;

#[contract]
pub struct ProxySwapContract;

#[contractimpl]
impl ProxySwapContract {
    pub fn get_amount_out(
        e: Env,
        swap_provider: Address,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
    ) -> Result<i128, PSCError> {
        let provider: SwapProvider = try_map_address_to_swap_provider(&e, swap_provider)?;

        provider.get_amount_out(&e, &token_in, &token_out, amount_in)
    }

    pub fn get_amount_in(
        e: Env,
        swap_provider: Address,
        token_in: Address,
        token_out: Address,
        amount_out: i128,
    ) -> Result<i128, PSCError> {
        let provider: SwapProvider = try_map_address_to_swap_provider(&e, swap_provider)?;

        provider.get_amount_in(&e, &token_in, &token_out, amount_out)
    }

    pub fn swap_exact(
        e: Env,
        to: Address,
        swap_provider: Address,
        token_in: Address,
        token_out: Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> Result<i128, PSCError> {
        to.require_auth();
        extend_instance_storage(&e);

        let provider: SwapProvider = try_map_address_to_swap_provider(&e, swap_provider)?;

        provider.swap_exact(&e, &to, &token_in, &token_out, amount_in, min_amount_out)
    }

    pub fn swap_for_exact(
        e: Env,
        to: Address,
        swap_provider: Address,
        token_in: Address,
        token_out: Address,
        max_amount_in: i128,
        amount_out: i128,
    ) -> Result<i128, PSCError> {
        to.require_auth();
        extend_instance_storage(&e);

        let provider: SwapProvider = try_map_address_to_swap_provider(&e, swap_provider)?;

        provider.swap_for_exact(&e, &to, &token_in, &token_out, max_amount_in, amount_out)
    }
}

// NB: Maybe, using 'enum_dispatch' is overkill and simple enum with 'match' is enough
fn try_map_address_to_swap_provider(e: &Env, address: Address) -> Result<SwapProvider, PSCError> {
    let provider = if address == Address::from_str(e, SOROSWAP_ROUTER) {
        SwapProvider::SoroswapRouter(SoroswapRouter(address))
    } else if address == Address::from_str(e, AQUA_ROUTER) {
        SwapProvider::AquaRouter(AquaRouter(address))
    } else {
        return Err(PSCError::UnregisteredProviderAddress);
    };

    Ok(provider)
}

// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}
