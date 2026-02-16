use sep_40_oracle::{Asset, PriceData, PriceFeedClient};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, BytesN, Env, Symbol, Vec, contract, contractclient, contractimpl, panic_with_error,
};

use crate::{
    computations::compute_median,
    constants::BPS_FACTOR,
    error::AOCError,
    events,
    storage::{self, OracleConfig, OracleConfigInput},
};

/// Trait that contains a subset of [`sep_40_oracle::PriceFeedTrait`] behavior, reasonable for price
/// aggregation
#[contractclient(name = "AggregatedPriceFeedClient")]
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
    /// # Arguments
    /// * `admin` - contract's administrator
    /// * `base_asset` - asset that will be the result of the `base()` endpoint call
    /// * `decimals` - number of decimals in the aggregated price
    /// * `max_age` - max allowed age(in seconds) of oracle's price that's being aggregated
    /// * `oracles` - list of information about oracles that are being aggregated
    pub fn __constructor(
        e: Env,
        admin: Address,
        base_asset_symbol: Symbol, /* TODO: Use `Asset` before deployment. Passing enums as arguments doesn't work well with stellar-cli */
        decimals: u32,
        max_age: u64,
        oracles: Vec<OracleConfigInput>,
    ) {
        let base_asset = Asset::Other(base_asset_symbol);

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

        storage::extend_instance(&e);
    }

    /// Gives away the admin role
    pub fn give_away_admin(e: Env, new_admin: Address) {
        require_admin(&e);

        storage::set_admin(&e, new_admin);
    }

    // NB: The ability to update the contract must be removed before the mainnet deployment
    /// Upgrades the aggregated oracle contract
    ///
    /// # Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
    ///   new version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        require_admin(&e);

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Adds an asset to the aggregation list
    ///
    /// # Arguments
    /// * `ticker` - symbol of the asset that is added
    /// * `token_address` - token contract's address on the Stellar ledger of the asset that is being
    ///   added
    /// * `max_dev_bps` - max allowed deviation in basis points between 2 consecutive price retrievals
    /// * `max_dev_consecutive_diff_secs` - max time difference in seconds between 2 consecutive price retrievals that will account for deviation
    pub fn add_asset(
        e: Env,
        ticker: Symbol,
        token_address: Address,
        max_dev_bps: u32,
        max_dev_consecutive_diff_secs: u64,
    ) -> Result<(), AOCError> {
        storage::extend_instance(&e);
        require_admin(&e);

        storage::add_asset(&e, ticker, token_address, max_dev_bps, max_dev_consecutive_diff_secs)?;

        Ok(())
    }

    /// Returns the list of all aggregated oracles configurations
    pub fn get_oracles(e: Env) -> Vec<OracleConfig> {
        storage::extend_instance(&e);

        storage::get_oracles(&e)
    }

    // TODO: Remove before deployment..
    pub fn address_lastprice(e: Env, asset_address: Address) -> Option<PriceData> {
        let stellar_asset = Asset::Stellar(asset_address);

        process_lastprice(&e, &stellar_asset)
    }
}

#[contractimpl]
impl AggregatedPriceFeedTrait for AggregatedOracleContract {
    fn base(e: Env) -> Asset {
        storage::extend_instance(&e);

        storage::get_base_asset(&e)
    }

    /// # Important:
    /// Returns a list of registered assets as [`Asset::Stellar`] variants
    fn assets(e: Env) -> Vec<Asset> {
        storage::extend_instance(&e);

        storage::get_assets(&e)
    }

    fn decimals(e: Env) -> u32 {
        storage::extend_instance(&e);

        storage::get_decimals(&e)
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&e);

        process_lastprice(&e, &asset)
    }
}

// ---- Helpers ----

/// Retrieves oracles' info and registers it in the contract's instance storage
fn register_oracles(e: &Env, input_oracles_configs: Vec<OracleConfigInput>, max_age: u64) {
    let mut oracles_to_register = Vec::<OracleConfig>::new(e);

    for input_config in input_oracles_configs {
        let OracleConfigInput { address, is_stellar_data_based } = input_config;

        let oracle_client = PriceFeedClient::new(e, &address);

        let decimals = oracle_client.decimals();
        let resolution = oracle_client.resolution();

        if (resolution as u64) > max_age {
            panic_with_error!(e, AOCError::InvalidOracleConfig);
        }

        let oracle_config = OracleConfig { address, decimals, resolution, is_stellar_data_based };

        oracles_to_register.push_back(oracle_config);
    }

    storage::set_oracles(e, oracles_to_register);
}

fn require_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}

fn process_lastprice(e: &Env, asset: &Asset) -> Option<PriceData> {
    storage::extend_instance(e);

    let Asset::Stellar(token_address) = asset else {
        // Oracle supports only assets existing as tokens on the Stellar ledger
        return None;
    };

    if !storage::is_asset_registered(e, token_address) {
        events::AssetIsNotRegistered { token_address: token_address.clone() }.publish(e);

        return None;
    }

    let price: i128 = compute_median(e, token_address)?;

    let current_timestamp = e.ledger().timestamp();

    let res_lastprice = PriceData { price, timestamp: current_timestamp };

    if let Some(previous_median_lastprice) =
        storage::get_previous_median_lastprice(e, token_address)
    {
        let PriceData { price: cached_price, timestamp: cached_timestamp } =
            previous_median_lastprice;
        let asset_data =
            storage::get_asset(e, token_address).expect("Asset must exist at this point");

        let time_diff = current_timestamp
            .checked_sub(cached_timestamp)
            .expect("The current timestamp cannot be smaller than the cached one");

        if time_diff <= asset_data.max_dev_consecutive_diff_secs {
            // Check price deviation
            let abs_price_diff = price.checked_sub(cached_price)?.abs();

            let div_bps = abs_price_diff.fixed_div_ceil(cached_price, BPS_FACTOR)?;
            if div_bps > asset_data.max_dev_bps as i128 {
                events::PriceDeviationExceedsMax {
                    asset_data,
                    cached_price_data: previous_median_lastprice,
                    new_price_data: res_lastprice,
                }
                .publish(e);

                return None;
            }
        }
    }

    storage::set_previous_median_lastprice(e, token_address, &res_lastprice);

    Some(res_lastprice)
}
