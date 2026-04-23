#![no_std]
#![allow(clippy::too_many_arguments)]
use proxy_swap_interface::ProxySwap;
use soroban_sdk::{
    Address, Env, Vec, contract, contracterror, contractimpl, contracttype, panic_with_error,
    token::TokenClient,
};

mod pool {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/aqua-pool.wasm");
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
    TokenNotFoundInPool = 5,
    AmountTooLarge = 6,
}

#[contracttype]
enum DataKey {
    Pool,
}

#[contract]
pub struct AquaSwapProviderContract;

#[contractimpl]
impl AquaSwapProviderContract {
    pub fn __constructor(e: Env, pool: Address) {
        e.storage().instance().set(&DataKey::Pool, &pool);
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

        let pool_client = pool::Client::new(&e, &get_pool(&e));
        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);

        let in_amount_u128 = convert_to_u128(&e, amount_in);
        let min_out_u128 = convert_to_u128(&e, min_amount_out);

        let token_out = path.last().unwrap();
        let balance_before = TokenClient::new(&e, &token_out).balance(&user);

        pool_client.swap(&user, &in_idx, &out_idx, &in_amount_u128, &min_out_u128);

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

        let pool_client = pool::Client::new(&e, &get_pool(&e));

        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);
        let max_in_u128 = convert_to_u128(&e, max_amount_in);
        let out_amount_u128 = convert_to_u128(&e, amount_out);

        let token_in = path.first().unwrap();
        let balance_before = TokenClient::new(&e, &token_in).balance(&user);

        pool_client.swap_strict_receive(&user, &in_idx, &out_idx, &out_amount_u128, &max_in_u128);

        let balance_after = TokenClient::new(&e, &token_in).balance(&user);
        let spent = balance_before.checked_sub(balance_after).unwrap_or_else(|| {
            panic_with_error!(&e, ASPError::OverOrUnderflow);
        });

        if spent <= 0 {
            panic_with_error!(&e, ASPError::ZeroSwapResult);
        }

        spent
    }

    fn get_amount_out(e: Env, path: Vec<Address>, amount_in: i128) -> i128 {
        extend_instance(&e);
        validate_path(&e, &path);

        let pool_client = pool::Client::new(&e, &get_pool(&e));

        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);
        let in_amount_u128 = convert_to_u128(&e, amount_in);

        let amount_out_u128 = pool_client.estimate_swap(&in_idx, &out_idx, &in_amount_u128);

        convert_to_i128(&e, amount_out_u128)
    }

    fn get_amount_in(e: Env, path: Vec<Address>, amount_out: i128) -> i128 {
        extend_instance(&e);
        validate_path(&e, &path);

        let pool_client = pool::Client::new(&e, &get_pool(&e));

        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);

        let out_amount_u128 = convert_to_u128(&e, amount_out);
        let amount_in_u128 =
            pool_client.estimate_swap_strict_receive(&in_idx, &out_idx, &out_amount_u128);

        convert_to_i128(&e, amount_in_u128)
    }
}

fn get_pool(e: &Env) -> Address {
    e.storage().instance().get(&DataKey::Pool).expect("Pool must be set")
}

fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

fn validate_path(e: &Env, path: &Vec<Address>) {
    if path.len() != 2 {
        panic_with_error!(e, ASPError::InvalidPath);
    }
}

/// Get the token indices for a pair in the pool
/// The pool stores tokens in a specific order, we need to find the indices
fn get_token_indices(e: &Env, pool_client: &pool::Client, path: &Vec<Address>) -> (u32, u32) {
    let token_in = path.first().unwrap();
    let token_out = path.last().unwrap();

    let pool_tokens = pool_client.get_tokens();

    let mut in_idx: Option<u32> = None;
    let mut out_idx: Option<u32> = None;

    for i in 0..pool_tokens.len() {
        let token = pool_tokens.get(i).unwrap();
        if token == token_in {
            in_idx = Some(i);
        }
        if token == token_out {
            out_idx = Some(i);
        }
    }

    let in_idx = in_idx.unwrap_or_else(|| {
        panic_with_error!(e, ASPError::TokenNotFoundInPool);
    });
    let out_idx = out_idx.unwrap_or_else(|| {
        panic_with_error!(e, ASPError::TokenNotFoundInPool);
    });

    (in_idx, out_idx)
}

fn convert_to_u128(e: &Env, amount: i128) -> u128 {
    if amount < 0 {
        panic_with_error!(e, ASPError::NegativeAmount);
    }

    amount as u128
}

fn convert_to_i128(e: &Env, amount: u128) -> i128 {
    if amount > i128::MAX as u128 {
        panic_with_error!(e, ASPError::AmountTooLarge);
    }

    amount as i128
}
