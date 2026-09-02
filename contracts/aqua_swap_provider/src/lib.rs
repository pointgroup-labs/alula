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

#[cfg(test)]
mod tests;

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
    NoPendingAdmin = 11,
}

#[contracttype]
pub enum DataKey {
    Admin,
    Pool(Address, Address),
    PendingAdmin,
}

#[contract]
pub struct AquaSwapProviderContract;

#[contractimpl]
impl AquaSwapProviderContract {
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

    pub fn add_asset_pool(e: Env, mut token_a: Address, mut token_b: Address, pool_addr: Address) {
        extend_instance(&e);

        let stored_admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        stored_admin.require_auth();

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

    /// Nominates `new_admin` as the next administrator. Admin-gated.
    /// The transfer is not effective until `new_admin` calls `accept_admin`.
    pub fn propose_admin(e: Env, new_admin: Address) {
        extend_instance(&e);

        let stored_admin: Address = e.storage().instance().get(&DataKey::Admin).unwrap();
        stored_admin.require_auth();

        e.storage().instance().set(&DataKey::PendingAdmin, &new_admin);
    }

    /// Completes the admin transfer initiated by `propose_admin`.
    /// Must be called by the address that was proposed.
    pub fn accept_admin(e: Env) {
        extend_instance(&e);

        let pending_admin: Address = e
            .storage()
            .instance()
            .get(&DataKey::PendingAdmin)
            .unwrap_or_else(|| panic_with_error!(&e, ASPError::NoPendingAdmin));
        pending_admin.require_auth();

        e.storage().instance().set(&DataKey::Admin, &pending_admin);
        e.storage().instance().remove(&DataKey::PendingAdmin);
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

        let hops = path.len() - 1;
        let min_out_u128 = convert_to_u128(&e, min_amount_out);

        let mut hop_amount_in = convert_to_u128(&e, amount_in);
        let mut received = 0i128;

        for i in 0..hops {
            let token_out = path.get(i + 1).unwrap();
            let is_last = i + 1 == hops;

            let (pool_client, in_idx, out_idx) = hop_pool(&e, &path.get(i).unwrap(), &token_out);

            // Intermediate hops swap at any price: their output is spent by the
            // next hop, so slippage only surfaces in the final `received` check,
            // which reverts the whole atomic call.
            let hop_min_out = if is_last { min_out_u128 } else { 0 };

            let balance_before = TokenClient::new(&e, &token_out).balance(&user);

            pool_client.swap(&user, &in_idx, &out_idx, &hop_amount_in, &hop_min_out);

            let balance_after = TokenClient::new(&e, &token_out).balance(&user);
            received = balance_after.checked_sub(balance_before).unwrap_or_else(|| {
                panic_with_error!(&e, ASPError::OverOrUnderflow);
            });

            if !is_last {
                hop_amount_in = convert_to_u128(&e, received);
            }
        }

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

        let hops = path.len() - 1;
        let token_in = path.first().unwrap();
        let hop_out_amounts = size_route_backwards(&e, &path, amount_out);

        let balance_before = TokenClient::new(&e, &token_in).balance(&user);

        // Only the first hop may draw on the caller's own funds. Every later hop
        // is capped at what the previous one actually delivered, so a pool that
        // charges more than it estimated reverts the route instead of reaching
        // into the caller's balance of an intermediate token — which `spent`,
        // measured on `path[0]`, would not account for.
        let mut hop_in_max = convert_to_u128(&e, max_amount_in);

        for i in 0..hops {
            let token_out = path.get(i + 1).unwrap();
            let is_last = i + 1 == hops;

            let (pool_client, in_idx, out_idx) = hop_pool(&e, &path.get(i).unwrap(), &token_out);

            let hop_before = (!is_last).then(|| TokenClient::new(&e, &token_out).balance(&user));

            pool_client.swap_strict_receive(
                &user,
                &in_idx,
                &out_idx,
                &hop_out_amounts.get(i).unwrap(),
                &hop_in_max,
            );

            if let Some(before) = hop_before {
                let after = TokenClient::new(&e, &token_out).balance(&user);
                let delta = after.checked_sub(before).unwrap_or_else(|| {
                    panic_with_error!(&e, ASPError::OverOrUnderflow);
                });

                hop_in_max = convert_to_u128(&e, delta);
            }
        }

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

        let mut amount_u128 = convert_to_u128(&e, amount_in);

        for i in 0..path.len() - 1 {
            let (pool_client, in_idx, out_idx) =
                hop_pool(&e, &path.get(i).unwrap(), &path.get(i + 1).unwrap());

            amount_u128 = pool_client.estimate_swap(&in_idx, &out_idx, &amount_u128);
        }

        convert_to_i128(&e, amount_u128)
    }

    fn get_amount_in(e: Env, path: Vec<Address>, amount_out: i128) -> i128 {
        extend_instance(&e);
        validate_path(&e, &path);

        let mut amount_u128 = convert_to_u128(&e, amount_out);

        for i in (0..path.len() - 1).rev() {
            let (pool_client, in_idx, out_idx) =
                hop_pool(&e, &path.get(i).unwrap(), &path.get(i + 1).unwrap());

            amount_u128 = pool_client.estimate_swap_strict_receive(&in_idx, &out_idx, &amount_u128);
        }

        convert_to_i128(&e, amount_u128)
    }
}

/// Walks the path in reverse to find, for each hop, the exact amount it must
/// deliver so the next hop can pay for itself. Aqua charges what
/// `estimate_swap_strict_receive` quotes, so the chain closes exactly as long as
/// no pool appears twice in the path.
fn size_route_backwards(e: &Env, path: &Vec<Address>, amount_out: i128) -> Vec<u128> {
    let mut hop_out_amounts = Vec::new(e);
    hop_out_amounts.push_front(convert_to_u128(e, amount_out));

    for i in (1..path.len() - 1).rev() {
        let (pool_client, in_idx, out_idx) =
            hop_pool(e, &path.get(i).unwrap(), &path.get(i + 1).unwrap());

        let required = pool_client.estimate_swap_strict_receive(
            &in_idx,
            &out_idx,
            &hop_out_amounts.first().unwrap(),
        );

        hop_out_amounts.push_front(required);
    }

    hop_out_amounts
}

fn hop_pool<'a>(
    e: &'a Env,
    token_in: &Address,
    token_out: &Address,
) -> (pool::Client<'a>, u32, u32) {
    let pool_addr = get_pool(e, token_in.clone(), token_out.clone());
    let pool_client = pool::Client::new(e, &pool_addr);
    let (in_idx, out_idx) = get_token_indices(e, &pool_client, token_in, token_out);

    (pool_client, in_idx, out_idx)
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
    if path.len() < 2 {
        panic_with_error!(e, ASPError::InvalidPath);
    }
}

fn get_token_indices(
    e: &Env,
    pool_client: &pool::Client,
    token_in: &Address,
    token_out: &Address,
) -> (u32, u32) {
    let pool_tokens = pool_client.get_tokens();

    let mut in_idx: Option<u32> = None;
    let mut out_idx: Option<u32> = None;

    for i in 0..pool_tokens.len() {
        let token = pool_tokens.get(i).unwrap();
        if token == *token_in {
            in_idx = Some(i);
        }
        if token == *token_out {
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
