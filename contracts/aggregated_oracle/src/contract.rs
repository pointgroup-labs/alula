use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
use soroban_sdk::{contract, contractimpl, panic_with_error, Address, Env, Vec};

use crate::{
    constants::{DECIMALS, RESOLUTION, USD_SYMBOL},
    error::AOCError,
    storage, swap,
};

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

    pub fn add_asset(e: Env, asset: Asset, token_address: Address) {
        require_admin(&e);

        storage::add_asset(&e, &asset, &token_address);
    }

    pub fn remove_asset(e: Env, asset: Asset) {
        require_admin(&e);

        storage::remove_asset(&e, &asset);
    }
}

#[contractimpl]
impl PriceFeedTrait for AggregatedOracleContract {
    fn base(_e: Env) -> Asset {
        Asset::Other(USD_SYMBOL)
    }

    fn assets(e: Env) -> Vec<Asset> {
        storage::get_assets(&e)
    }

    fn decimals(_e: Env) -> u32 {
        DECIMALS
    }

    fn resolution(_e: Env) -> u32 {
        RESOLUTION
    }

    fn price(e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        panic_with_error!(&e, AOCError::Unimplemented)
    }

    fn prices(e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        panic_with_error!(&e, AOCError::Unimplemented)
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        let price = swap::get_price(&e, &asset)?;
        let price_data = PriceData {
            price,
            timestamp: e.ledger().timestamp(),
        };

        Some(price_data)
    }
}

// ---- Helpers ----

fn require_admin(e: &Env) {
    let admin = storage::get_admin(e).unwrap_or_else(|| panic_with_error!(e, AOCError::NotAnAdmin));
    admin.require_auth();
}
