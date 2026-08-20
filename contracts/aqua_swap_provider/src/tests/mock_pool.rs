use soroban_sdk::{
    Address, Env, Vec, contract, contracterror, contractimpl, contracttype, panic_with_error,
    token::TokenClient,
};

const BPS: u128 = 10_000;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum MockPoolError {
    SlippageExceeded = 1,
    InsufficientReserve = 2,
}

#[contracttype]
pub enum DataKey {
    Tokens,
    FeeBps,
    StrictReceiveSurcharge,
}

/// Constant-product stand-in for an Aqua liquidity pool. Reserves are the
/// pool's own token balances, so the contract never needs to track them.
#[contract]
pub struct MockAquaPool;

#[contractimpl]
impl MockAquaPool {
    pub fn __constructor(e: Env, tokens: Vec<Address>, fee_bps: u128) {
        e.storage().instance().set(&DataKey::Tokens, &tokens);
        e.storage().instance().set(&DataKey::FeeBps, &fee_bps);
    }

    pub fn get_tokens(e: Env) -> Vec<Address> {
        e.storage().instance().get(&DataKey::Tokens).unwrap()
    }

    /// Makes `swap_strict_receive` charge more than `estimate_swap_strict_receive`
    /// quoted, so tests can exercise a pool whose quote does not bind.
    pub fn set_strict_receive_surcharge(e: Env, amount: u128) {
        e.storage().instance().set(&DataKey::StrictReceiveSurcharge, &amount);
    }

    pub fn estimate_swap(e: Env, in_idx: u32, out_idx: u32, in_amount: u128) -> u128 {
        let (reserve_in, reserve_out) = reserves(&e, in_idx, out_idx);
        let in_after_fee = in_amount * (BPS - fee_bps(&e)) / BPS;

        reserve_out * in_after_fee / (reserve_in + in_after_fee)
    }

    pub fn estimate_swap_strict_receive(
        e: Env,
        in_idx: u32,
        out_idx: u32,
        out_amount: u128,
    ) -> u128 {
        let (reserve_in, reserve_out) = reserves(&e, in_idx, out_idx);

        if out_amount >= reserve_out {
            panic_with_error!(&e, MockPoolError::InsufficientReserve);
        }

        let in_after_fee = (reserve_in * out_amount).div_ceil(reserve_out - out_amount);

        (in_after_fee * BPS).div_ceil(BPS - fee_bps(&e))
    }

    pub fn swap(
        e: Env,
        user: Address,
        in_idx: u32,
        out_idx: u32,
        in_amount: u128,
        out_min: u128,
    ) -> u128 {
        user.require_auth();

        let out_amount = Self::estimate_swap(e.clone(), in_idx, out_idx, in_amount);
        if out_amount < out_min {
            panic_with_error!(&e, MockPoolError::SlippageExceeded);
        }

        settle(&e, &user, in_idx, out_idx, in_amount, out_amount);

        out_amount
    }

    pub fn swap_strict_receive(
        e: Env,
        user: Address,
        in_idx: u32,
        out_idx: u32,
        out_amount: u128,
        in_max: u128,
    ) -> u128 {
        user.require_auth();

        let surcharge: u128 =
            e.storage().instance().get(&DataKey::StrictReceiveSurcharge).unwrap_or(0);
        let in_amount =
            Self::estimate_swap_strict_receive(e.clone(), in_idx, out_idx, out_amount) + surcharge;

        if in_amount > in_max {
            panic_with_error!(&e, MockPoolError::SlippageExceeded);
        }

        settle(&e, &user, in_idx, out_idx, in_amount, out_amount);

        in_amount
    }
}

fn fee_bps(e: &Env) -> u128 {
    e.storage().instance().get(&DataKey::FeeBps).unwrap()
}

fn token(e: &Env, idx: u32) -> Address {
    let tokens: Vec<Address> = e.storage().instance().get(&DataKey::Tokens).unwrap();

    tokens.get(idx).unwrap()
}

fn reserves(e: &Env, in_idx: u32, out_idx: u32) -> (u128, u128) {
    let pool = e.current_contract_address();
    let reserve_in = TokenClient::new(e, &token(e, in_idx)).balance(&pool) as u128;
    let reserve_out = TokenClient::new(e, &token(e, out_idx)).balance(&pool) as u128;

    (reserve_in, reserve_out)
}

fn settle(e: &Env, user: &Address, in_idx: u32, out_idx: u32, in_amount: u128, out_amount: u128) {
    let pool = e.current_contract_address();

    TokenClient::new(e, &token(e, in_idx)).transfer(user, &pool, &(in_amount as i128));
    TokenClient::new(e, &token(e, out_idx)).transfer(&pool, user, &(out_amount as i128));
}
