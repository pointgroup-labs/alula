#![no_std]
use soroban_sdk::{Address, Env, contractclient};

#[contractclient(name = "ProxySwapClient")]
pub trait ProxySwap {
    fn swap_exact(
        e: &Env,
        swap_provider: &Address,
        user: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> i128;

    fn swap_for_exact(
        e: &Env,
        swap_provider: &Address,
        user: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in_max: i128,
        amount_out: i128,
    ) -> i128;
}
