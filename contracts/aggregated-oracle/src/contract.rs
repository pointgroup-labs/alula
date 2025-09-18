use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Symbol, Vec};

use crate::{
    computations::compute_median,
    error::AOCError,
    storage::{self, OracleConfig},
};

pub trait AggregatedPriceFeedTrait {
    /// Return the base asset the price is reported in
    fn base(e: Env) -> Asset;

    /// Return all assets quoted by the price feed
    fn assets(e: Env) -> Vec<Asset>;

    /// Return the number of decimals for all assets quoted by the oracle
    fn decimals(e: Env) -> u32;

    /// Get the most recent price for an asset
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData>;
}

#[contract]
pub struct AggregatedOracleContract;

#[contractimpl]
impl AggregatedOracleContract {
    pub fn __constructor(
        e: Env,
        base_asset: Asset,
        decimals: u32,
        admin: Address,
        max_age: u64,
        oracles: Vec<OracleConfig>,
    ) {
        const MIN_MAX_AGE: u64 = 360;
        const MAX_MAX_AGE: u64 = 3_600;

        if !(MIN_MAX_AGE..=MAX_MAX_AGE).contains(&max_age) {
            panic_with_error!(e, AOCError::InvalidMaxAge);
        }

        if oracles.is_empty() {
            panic_with_error!(e, AOCError::NoOraclesToRegister);
        }

        storage::set_admin(&e, admin);
        storage::set_decimals(&e, decimals);
        storage::set_max_age(&e, max_age);
        storage::set_base_asset(&e, base_asset);
        storage::set_oracles(&e, oracles);

        storage::extend_instance_storage(&e);
    }

    pub fn add_asset(e: Env, ticker: Symbol, token_address: Address) -> Result<(), AOCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        storage::add_asset(&e, ticker, token_address)
    }

    pub fn get_oracles(e: Env) -> Vec<OracleConfig> {
        storage::extend_instance_storage(&e);

        storage::get_oracles(&e)
    }
}

#[contractimpl]
impl AggregatedPriceFeedTrait for AggregatedOracleContract {
    fn base(e: Env) -> Asset {
        storage::extend_instance_storage(&e);

        storage::get_base_asset(&e)
    }

    fn assets(e: Env) -> Vec<Asset> {
        storage::extend_instance_storage(&e);

        storage::get_assets(&e)
    }

    fn decimals(e: Env) -> u32 {
        storage::extend_instance_storage(&e);

        storage::get_decimals(&e)
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance_storage(&e);

        let Asset::Stellar(token_address) = asset else {
            // Oracle supports only assets existing as tokens on the Stellar ledger
            return None;
        };

        if !storage::is_asset_registered(&e, &token_address) {
            return None;
        }

        let price = compute_median(&e, &token_address)?;
        let timestamp = e.ledger().timestamp();

        let price_data = PriceData { price, timestamp };

        Some(price_data)
    }
}

// ---- Helpers ----

fn require_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}
