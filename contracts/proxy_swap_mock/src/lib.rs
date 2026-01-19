#![no_std]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, contract, contractimpl, contracttype,
    token::{StellarAssetClient, TokenClient},
};

const BPS_FACTOR: i128 = 10_000;

#[contracttype]
enum DataKey {
    DiffBps,
}

#[contract]
struct ProxySwapMockContract;

#[contractimpl]
impl ProxySwapMockContract {
    pub fn set_diff(e: Env, diff_bps: i128) {
        e.storage().instance().set(&DataKey::DiffBps, &diff_bps);
    }

    pub fn swap_exact(
        e: Env,
        _swap_provider: &Address,
        to: &Address,
        token_in: &Address,
        token_out: &Address,
        amount_in: i128,
        min_amount_out: i128,
    ) -> i128 {
        // let diff_bps: i128 = e.storage().instance().get(&DataKey::DiffBps).unwrap_or(0);

        // let diff = min_amount_out.fixed_mul_ceil(diff_bps, BPS_FACTOR).unwrap();
        

        // burn_and_mint_tokens(&e, &token_in, &token_out, amount_in, amount_out, &to);

        min_amount_out
    }

    pub fn swap_for_exact(
        e: Env,
        _swap_provider: &Address,
        to: &Address,
        token_in: &Address,
        token_out: &Address,
        max_amount_in: i128,
        amount_out: i128,
    ) -> i128 {
        let diff_bps: i128 = e.storage().instance().get(&DataKey::DiffBps).unwrap_or(0);

        let diff = max_amount_in.fixed_mul_ceil(diff_bps, BPS_FACTOR).unwrap();
        let amount_in = max_amount_in - diff; // safe

        burn_and_mint_tokens(&e, token_in, token_out, amount_in, amount_out, to);

        amount_in
    }
}

fn burn_and_mint_tokens(
    e: &Env,
    burnt_token: &Address,
    minted_token: &Address,
    burnt_amount: i128,
    minted_amount: i128,
    to: &Address,
) {
    let sac_client = StellarAssetClient::new(e, minted_token);
    let token_client = TokenClient::new(e, burnt_token);

    sac_client.mint(to, &minted_amount);
    token_client.burn(to, &burnt_amount);
}
