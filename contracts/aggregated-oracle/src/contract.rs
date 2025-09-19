use sep_40_oracle::{Asset, PriceData, PriceFeedClient};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Map, Symbol, Vec};

use crate::{
    computations::compute_median,
    error::AOCError,
    storage::{self, OracleConfig, OracleConfigInput},
};

/// Trait that contains a subset of [`sep_40_oracle::PriceFeedTrait`] behavior, reasonable for price
/// aggregation
pub trait AggregatedPriceFeedTrait {
    /// Returns the base asset the price is reported in
    fn base(e: Env) -> Asset;

    /// Returns all assets quoted by the price feed
    fn assets(e: Env) -> Vec<Asset>;

    /// Returns the number of decimals for all assets quoted by the oracle
    fn decimals(e: Env) -> u32;

    /// Gets the most recent price for an asset
    fn lastprice(e: Env, asset: Asset) -> Option<PriceData>;
}

#[contract]
pub struct AggregatedOracleContract;

#[contractimpl]
impl AggregatedOracleContract {
    /// Constructs the oracle contract
    ///
    /// ### Arguments
    /// * `admin` - contract's administrator
    /// * `base_asset` - asset that will be the result of the `base()` endpoint call
    /// * `decimals` - number of decimals in the aggregated price
    /// * `max_age` - max allowed age(in seconds) of oracle's price that's being aggregated
    /// * `oracles` - list of information about oracles that are being aggregated
    pub fn __constructor(
        e: Env,
        admin: Address,
        base_asset: Asset,
        decimals: u32,
        max_age: u64,
        oracles: Vec<OracleConfigInput>,
    ) {
        const MIN_MAX_AGE: u64 = 360;
        const MAX_MAX_AGE: u64 = 3_600;

        if !(MIN_MAX_AGE..=MAX_MAX_AGE).contains(&max_age) {
            panic_with_error!(e, AOCError::InvalidMaxAge);
        }

        const MIN_ORACLES_LEN: u32 = 1;
        const MAX_ORACLES_LEN: u32 = 10;

        if !(MIN_ORACLES_LEN..=MAX_ORACLES_LEN).contains(&oracles.len()) {
            panic_with_error!(e, AOCError::InvalidOraclesAmount);
        }

        storage::set_admin(&e, admin);
        storage::set_decimals(&e, decimals);
        storage::set_max_age(&e, max_age);
        storage::set_base_asset(&e, base_asset);
        register_oracles(&e, oracles, max_age);

        storage::extend_instance_storage(&e);
    }

    /// Adds an asset to the aggregation list
    ///
    /// ### Arguments
    /// * `ticker` - symbol of the asset that is added
    /// * `token_address` - token contract's address on the Stellar ledger of the asset that is
    ///   added
    pub fn add_asset(e: Env, ticker: Symbol, token_address: Address) -> Result<(), AOCError> {
        storage::extend_instance_storage(&e);
        require_admin(&e);

        storage::add_asset(&e, ticker, token_address)?;

        Ok(())
    }

    /// Returns the list of all aggregated oracles configurations
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

/// Retrieves oracles' info and registers it in the contract's instance storage
fn register_oracles(e: &Env, input_oracles_configs: Vec<OracleConfigInput>, max_age: u64) {
    let mut oracles_to_register = Vec::<OracleConfig>::new(e);

    for input_config in input_oracles_configs {
        let OracleConfigInput {
            address,
            is_stellar_data_based,
        } = input_config;

        let oracle_client = PriceFeedClient::new(e, &address);

        let lastprices_cached = Map::new(e);
        let decimals = oracle_client.decimals();
        let resolution = oracle_client.resolution();

        if (resolution as u64) > max_age {
            panic_with_error!(e, AOCError::InvalidOracleConfig);
        }

        let oracle_config = OracleConfig {
            address,
            decimals,
            resolution,
            lastprices_cached,
            is_stellar_data_based,
        };

        oracles_to_register.push_back(oracle_config);
    }

    storage::set_oracles(e, oracles_to_register);
}

fn require_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}
