#![no_std]
#![allow(clippy::too_many_arguments)]
use proxy_swap_interface::ProxySwap;
use soroban_sdk::{
    Address, Env, Vec, contract, contracterror, contractimpl, contracttype, panic_with_error,
    token::TokenClient,
};

mod router {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/soroswap-router.wasm");
}

const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
const SECONDS_PER_LEDGER: u32 = 6;
const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;
const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SSPError {
    OverOrUnderflow = 1,
    InvalidPath = 2,
    ZeroSwapResult = 3,
}

#[contracttype]
enum DataKey {
    Router,
}

#[contract]
pub struct SoroSwapProviderContract;

#[contractimpl]
impl SoroSwapProviderContract {
    pub fn __constructor(e: Env, router: Address) {
        e.storage().instance().set(&DataKey::Router, &router);
    }
}

#[contractimpl]
impl ProxySwap for SoroSwapProviderContract {
    fn swap_exact(
        e: Env,
        user: Address,
        path: Vec<Address>,
        amount_in: i128,
        min_amount_out: i128,
    ) -> i128 {
        extend_instance(&e);
        user.require_auth();
        validate_path(&e, &path);

        let router_client = router::Client::new(&e, &get_router(&e));

        let token_out = path.last().unwrap();
        let balance_before = TokenClient::new(&e, &token_out).balance(&user);

        router_client.swap_exact_tokens_for_tokens(
            &amount_in,
            &min_amount_out,
            &path,
            &user,
            &u64::MAX,
        );

        let balance_after = TokenClient::new(&e, &token_out).balance(&user);
        let received = balance_after.checked_sub(balance_before).unwrap_or_else(|| {
            panic_with_error!(&e, SSPError::OverOrUnderflow);
        });

        if received <= 0 {
            panic_with_error!(&e, SSPError::ZeroSwapResult);
        }

        received
    }

    fn swap_for_exact(
        e: Env,
        user: Address,
        path: Vec<Address>,
        max_amount_in: i128,
        amount_out: i128,
    ) -> i128 {
        extend_instance(&e);
        user.require_auth();
        validate_path(&e, &path);

        let router_client = router::Client::new(&e, &get_router(&e));

        let token_in = path.first().unwrap();
        let balance_before = TokenClient::new(&e, &token_in).balance(&user);

        router_client.swap_tokens_for_exact_tokens(
            &amount_out,
            &max_amount_in,
            &path,
            &user,
            &u64::MAX,
        );

        let balance_after = TokenClient::new(&e, &token_in).balance(&user);
        let spent = balance_before.checked_sub(balance_after).unwrap_or_else(|| {
            panic_with_error!(&e, SSPError::OverOrUnderflow);
        });

        if spent <= 0 {
            panic_with_error!(&e, SSPError::ZeroSwapResult);
        }

        spent
    }

    fn get_amount_out(e: Env, path: Vec<Address>, amount_in: i128) -> i128 {
        extend_instance(&e);
        validate_path(&e, &path);

        let router_client = router::Client::new(&e, &get_router(&e));

        router_client.router_get_amounts_out(&amount_in, &path).last().unwrap()
    }

    fn get_amount_in(e: Env, path: Vec<Address>, amount_out: i128) -> i128 {
        extend_instance(&e);
        validate_path(&e, &path);

        let router_client = router::Client::new(&e, &get_router(&e));

        router_client.router_get_amounts_in(&amount_out, &path).first().unwrap()
    }
}

fn get_router(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Router).expect("Router must be set")
}

fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

fn validate_path(e: &Env, path: &Vec<Address>) {
    if path.len() < 2 {
        panic_with_error!(e, SSPError::InvalidPath);
    }
}
