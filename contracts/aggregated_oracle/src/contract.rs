use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
use soroban_sdk::{
    contract, contractimpl, panic_with_error, symbol_short, Address, Env, Symbol, Vec,
};

use crate::{error::AggregatedOracleContractError, storage};

const USD_SYMBOL: Symbol = symbol_short!("USD");

#[contract]
pub struct AggregatedOracleContract;

#[contractimpl]
impl AggregatedOracleContract {
    /// Constructs the aggregated oracle contract
    ///
    /// ### Arguments
    /// * `admin` - contract's administrator
    pub fn __constructor(e: Env, admin: Address) {
        storage::set_admin(&e, &admin);
    }

    pub fn add_asset(e: Env, asset: Asset) {
        require_admin(&e);
    }

    // Some constructor or some initialization method?? Likely a good idea
    pub fn remove_asset(env: Env, asset: Asset) {
        // vec![&env, String::from_str(&env, "Hello"), to]
    }
}

#[contractimpl]
impl PriceFeedTrait for AggregatedOracleContract {
    fn base(_env: Env) -> Asset {
        Asset::Other(USD_SYMBOL)
    }

    fn assets(env: Env) -> Vec<Asset> {
        todo!()
    }

    fn decimals(env: Env) -> u32 {
        8
    }

    fn resolution(env: Env) -> u32 {
        60
    }

    fn price(env: Env, asset: Asset, x: u64) -> Option<PriceData> {
        todo!()
    }

    fn prices(env: Env, asset: Asset, x: u32) -> Option<Vec<PriceData>> {
        todo!()
    }

    fn lastprice(env: Env, asset: Asset) -> Option<PriceData> {
        todo!()
    }
}

// ---- Helpers ----
/// Verifies that a caller is the registered admin
/// Panics:
///
/// If caller is not a registered admin
fn require_admin(e: &Env) {
    let admin = storage::get_admin(e)
        .unwrap_or_else(|| panic_with_error!(e, AggregatedOracleContractError::InternalError));
    admin.require_auth();
}
