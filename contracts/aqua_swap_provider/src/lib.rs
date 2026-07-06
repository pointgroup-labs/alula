#![no_std]
#![allow(clippy::too_many_arguments)]
use proxy_swap_interface::ProxySwap;
use soroban_sdk::{
    Address, Env, Map, Vec, contract, contracterror, contractimpl, contracttype, panic_with_error,
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
    InvalidSwapResult = 3,
    NegativeAmount = 4,
    TokenNotFoundInPool = 5,
    AmountTooLarge = 6,
    PoolNotFound = 7,
    DuplicatePool = 8,
    IdenticalTokens = 9,
    Unauthorized = 10,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Pool(Address, Address), 
}

#[contract]
pub struct AquaSwapProviderContract;

#[contractimpl]
impl AquaSwapProviderContract {
    /// Constructor now accepts an Admin address to secure future pool additions
    pub fn __constructor(e: Env, admin: Address, pools: Map<(Address, Address), Address>) {
        e.storage().instance().set(&DataKey::Admin, &admin);

        for (pair, pool_addr) in pools.iter() {
            let (mut token_a, mut token_b) = pair;

            if token_a == token_b {
                panic_with_error!(&e, ASPError::IdenticalTokens);
            }

            if token_a > token_b {
                let temp = token_a.clone();
                token_a = token_b;
                token_b = temp;
            }

            let key = DataKey::Pool(token_a, token_b);
            
            if e.storage().instance().has(&key) {
                panic_with_error!(&e, ASPError::DuplicatePool);
            }

            e.storage().instance().set(&key, &pool_addr);
        }
    }

    /// Admin-only method to extend the contract with new liquidity pools
    pub fn add_asset_pool(
        e: Env,
        mut token_a: Address,
        mut token_b: Address,
        pool_addr: Address,
    ) {
        extend_instance(&e);
        
        // 1. Verify authorization
        let stored_admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        stored_admin.require_auth();

        // 2. Validate tokens
        if token_a == token_b {
            panic_with_error!(&e, ASPError::IdenticalTokens);
        }

        // 3. Canonicalize token pair
        if token_a > token_b {
            let temp = token_a.clone();
            token_a = token_b;
            token_b = temp;
        }

        let key = DataKey::Pool(token_a, token_b);
        
        // 4. Ensure pool doesn't already exist to prevent overwriting
        if e.storage().instance().has(&key) {
            panic_with_error!(&e, ASPError::DuplicatePool);
        }

        // 5. Store the new pool
        e.storage().instance().set(&key, &pool_addr);
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

        let token_in = path.first().unwrap();
        let token_out = path.last().unwrap();

        let pool_addr = get_pool(&e, token_in.clone(), token_out.clone());
        let pool_client = pool::Client::new(&e, &pool_addr);
        
        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);

        let in_amount_u128 = convert_to_u128(&e, amount_in);
        let min_out_u128 = convert_to_u128(&e, min_amount_out);

        let balance_before = TokenClient::new(&e, &token_out).balance(&user);

        pool_client.swap(&user, &in_idx, &out_idx, &in_amount_u128, &min_out_u128);

        let balance_after = TokenClient::new(&e, &token_out).balance(&user);
        let received = balance_after.checked_sub(balance_before).unwrap_or_else(|| {
            panic_with_error!(&e, ASPError::OverOrUnderflow);
        });

        if received < min_amount_out {
            panic_with_error!(&e, ASPError::InvalidSwapResult);
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

        let token_in = path.first().unwrap();
        let token_out = path.last().unwrap();

        let pool_addr = get_pool(&e, token_in.clone(), token_out.clone());
        let pool_client = pool::Client::new(&e, &pool_addr);

        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);
        let max_in_u128 = convert_to_u128(&e, max_amount_in);
        let out_amount_u128 = convert_to_u128(&e, amount_out);

        let balance_before = TokenClient::new(&e, &token_in).balance(&user);

        pool_client.swap_strict_receive(&user, &in_idx, &out_idx, &out_amount_u128, &max_in_u128);

        let balance_after = TokenClient::new(&e, &token_in).balance(&user);
        let spent = balance_before.checked_sub(balance_after).unwrap_or_else(|| {
            panic_with_error!(&e, ASPError::OverOrUnderflow);
        });

        if spent > max_amount_in {
            panic_with_error!(&e, ASPError::InvalidSwapResult);
        }

        spent
    }

    fn get_amount_out(e: Env, path: Vec<Address>, amount_in: i128) -> i128 {
        extend_instance(&e);
        validate_path(&e, &path);

        let token_in = path.first().unwrap();
        let token_out = path.last().unwrap();

        let pool_addr = get_pool(&e, token_in, token_out);
        let pool_client = pool::Client::new(&e, &pool_addr);

        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);
        let in_amount_u128 = convert_to_u128(&e, amount_in);

        let amount_out_u128 = pool_client.estimate_swap(&in_idx, &out_idx, &in_amount_u128);

        convert_to_i128(&e, amount_out_u128)
    }

    fn get_amount_in(e: Env, path: Vec<Address>, amount_out: i128) -> i128 {
        extend_instance(&e);
        validate_path(&e, &path);

        let token_in = path.first().unwrap();
        let token_out = path.last().unwrap();

        let pool_addr = get_pool(&e, token_in, token_out);
        let pool_client = pool::Client::new(&e, &pool_addr);

        let (in_idx, out_idx) = get_token_indices(&e, &pool_client, &path);

        let out_amount_u128 = convert_to_u128(&e, amount_out);
        let amount_in_u128 =
            pool_client.estimate_swap_strict_receive(&in_idx, &out_idx, &out_amount_u128);

        convert_to_i128(&e, amount_in_u128)
    }
}

fn get_pool(e: &Env, mut token_a: Address, mut token_b: Address) -> Address {
    if token_a > token_b {
        let temp = token_a.clone();
        token_a = token_b;
        token_b = temp;
    }

    let key = DataKey::Pool(token_a, token_b);
    e.storage().instance().get(&key).unwrap_or_else(|| {
        panic_with_error!(e, ASPError::PoolNotFound);
    })
}

fn extend_instance(e: &Env) {
    e.storage().instance().extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

fn validate_path(e: &Env, path: &Vec<Address>) {
    if path.len() != 2 {
        panic_with_error!(e, ASPError::InvalidPath);
    }
}

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