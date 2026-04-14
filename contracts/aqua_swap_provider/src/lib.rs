#![no_std]
#![allow(clippy::too_many_arguments)]
use proxy_swap_interface::ProxySwap;
use soroban_sdk::{
    Address, BytesN, Env, Vec, contract, contracterror, contractimpl, contracttype,
    panic_with_error, token::TokenClient,
};

mod router {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/aqua-router.wasm");
}

const SECONDS_PER_DAY: u32 = 24 * 60 * 60;
const SECONDS_PER_LEDGER: u32 = 6;
const LEDGERS_PER_DAY: u32 = SECONDS_PER_DAY / SECONDS_PER_LEDGER;
const INSTANCE_THRESHOLD: u32 = 40 * LEDGERS_PER_DAY;
const INSTANCE_BUMP: u32 = INSTANCE_THRESHOLD + LEDGERS_PER_DAY;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ASPError {
    OverOrUnderflow = 1,
    InvalidPath = 2,
    ZeroSwapResult = 3,
    NegativeAmount = 4,
}

#[contracttype]
enum DataKey {
    Admin,
    Router,
    PoolRoute(Address, Address),
}

#[derive(Clone)]
#[contracttype]
struct PoolRoute {
    pool_tokens: Vec<Address>,
    pool_index: BytesN<32>,
}

#[contract]
pub struct AquaSwapProviderContract;

#[contractimpl]
impl AquaSwapProviderContract {
    pub fn __constructor(e: Env, router: Address, admin: Address) {
        e.storage().instance().set(&DataKey::Router, &router);
        e.storage().instance().set(&DataKey::Admin, &admin);
    }

    pub fn get_router(e: Env) -> Address {
        extend_instance(&e);
        get_router(&e)
    }

    pub fn upgrade(e: Env, new_contract_wasm_hash: BytesN<32>) {
        extend_instance(&e);
        get_admin(&e).require_auth();

        e.deployer().update_current_contract_wasm(new_contract_wasm_hash);
    }

    pub fn configure_pool_route(
        e: Env,
        token_a: Address,
        token_b: Address,
        pool_tokens: Vec<Address>,
        pool_index: BytesN<32>,
    ) {
        extend_instance(&e);
        get_admin(&e).require_auth();

        let route = PoolRoute { pool_tokens, pool_index };
        e.storage().persistent().set(&DataKey::PoolRoute(token_a.clone(), token_b.clone()), &route);
        e.storage().persistent().set(&DataKey::PoolRoute(token_b, token_a), &route);
    }
}

#[contractimpl]
impl ProxySwap for AquaSwapProviderContract {
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
        validate_positive(&e, amount_in);
        validate_positive(&e, min_amount_out);

        let router_client = router::Client::new(&e, &get_router(&e));
        let swaps_chain = build_swaps_chain(&e, &path);
        let token_out = path.last().unwrap();

        let balance_before = TokenClient::new(&e, &token_out).balance(&user);

        router_client.swap_chained(
            &user,
            &swaps_chain,
            &path.first().unwrap(),
            &(amount_in as u128),
            &(min_amount_out as u128),
        );

        let balance_after = TokenClient::new(&e, &token_out).balance(&user);
        let received = balance_after.checked_sub(balance_before).unwrap_or_else(|| {
            panic_with_error!(&e, ASPError::OverOrUnderflow);
        });

        if received <= 0 {
            panic_with_error!(&e, ASPError::ZeroSwapResult);
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
        validate_positive(&e, max_amount_in);
        validate_positive(&e, amount_out);

        let router_client = router::Client::new(&e, &get_router(&e));
        let swaps_chain = build_swaps_chain(&e, &path);
        let token_in = path.first().unwrap();

        let balance_before = TokenClient::new(&e, &token_in).balance(&user);

        router_client.swap_chained_strict_receive(
            &user,
            &swaps_chain,
            &token_in,
            &(amount_out as u128),
            &(max_amount_in as u128),
        );

        let balance_after = TokenClient::new(&e, &token_in).balance(&user);
        let spent = balance_before.checked_sub(balance_after).unwrap_or_else(|| {
            panic_with_error!(&e, ASPError::OverOrUnderflow);
        });

        if spent <= 0 {
            panic_with_error!(&e, ASPError::ZeroSwapResult);
        }

        spent
    }
}

fn get_router(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Router).expect("Router must be set")
}

fn get_admin(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Admin).expect("Admin must be set")
}

fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

fn validate_path(e: &Env, path: &Vec<Address>) {
    if path.len() < 2 {
        panic_with_error!(e, ASPError::InvalidPath);
    }
}

fn validate_positive(e: &Env, amount: i128) {
    if amount < 0 {
        panic_with_error!(e, ASPError::NegativeAmount);
    }
}

/// Converts a simple token path into the Aqua router's `swaps_chain` format
/// by looking up pre-configured pool routes for each consecutive pair
fn build_swaps_chain(e: &Env, path: &Vec<Address>) -> Vec<(Vec<Address>, BytesN<32>, Address)> {
    let len = path.len();

    let mut chain = Vec::new(e);
    for i in 0..len - 1 {
        let token_a = path.get(i).unwrap();
        let token_b = path.get(i + 1).unwrap();

        let route: PoolRoute = e
            .storage()
            .persistent()
            .get(&DataKey::PoolRoute(token_a, token_b.clone()))
            .expect("Pool route not configured for pair");

        chain.push_back((route.pool_tokens, route.pool_index, token_b));
    }
    chain
}
