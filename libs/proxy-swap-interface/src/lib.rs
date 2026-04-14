#![no_std]
use soroban_sdk::{Address, Env, Vec, contractclient};

#[contractclient(name = "ProxySwapClient")]
pub trait ProxySwap {
    fn swap_exact(
        e: Env,
        user: Address,
        path: Vec<Address>,
        amount_in: i128,
        min_amount_out: i128,
    ) -> i128;

    fn swap_for_exact(
        e: Env,
        user: Address,
        path: Vec<Address>,
        max_amount_in: i128,
        amount_out: i128,
    ) -> i128;
}
