use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
use soroban_sdk::{Address, Env, Map, Vec, contract, contractimpl, panic_with_error, vec as svec};

use crate::{constants::*, error::RS40ACError, redstone::RedStonePriceFeedClient, storage};

#[contract]
pub struct RedStoneSep40AdapterContract;

#[contractimpl]
impl RedStoneSep40AdapterContract {
    /// Constructs the adapter contract.
    ///
    /// Assets are fixed at construction — no post-deploy mutations. To change the
    /// set of supported assets, deploy a new contract instance.
    ///
    /// # Arguments
    /// * `admin` - contract's administrator
    /// * `base_asset` - asset that will be the result of the `base()` endpoint call
    /// * `decimals` - number of decimals in the output price
    /// * `feeds` - token address → RedStone price feed contract address mapping
    pub fn __constructor(e: Env, base_asset: Asset, decimals: u32, feeds: Map<Address, Address>) {
        storage::set_base_asset(&e, &base_asset);
        storage::set_decimals(&e, decimals);

        let mut assets = svec![&e];
        for (token_address, price_feed) in feeds.iter() {
            storage::set_feed(&e, &token_address, &price_feed);
            assets.push_back(Asset::Stellar(token_address));
        }
        storage::set_assets(&e, &assets);
    }
}

#[contractimpl]
impl PriceFeedTrait for RedStoneSep40AdapterContract {
    fn base(e: Env) -> Asset {
        storage::extend_instance(&e);

        storage::get_base_asset(&e)
    }

    fn assets(e: Env) -> Vec<Asset> {
        storage::extend_instance(&e);

        storage::get_assets(&e)
    }

    fn decimals(e: Env) -> u32 {
        storage::extend_instance(&e);

        storage::get_decimals(&e)
    }

    fn resolution(_e: Env) -> u32 {
        RESOLUTION
    }

    fn price(e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        panic_with_error!(&e, RS40ACError::Unimplemented)
    }

    fn prices(e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        panic_with_error!(&e, RS40ACError::Unimplemented)
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&e);

        let Asset::Stellar(address) = asset else {
            return None;
        };

        let price_feed_address = storage::get_feed(&e, &address)?;
        let client = RedStonePriceFeedClient::new(&e, &price_feed_address);

        let entry = client.try_read_price_data().ok()?.ok()?;

        let price_raw = i128::try_from(entry.price.to_u128()?).ok()?;
        let price = rescale(price_raw, REDSTONE_DECIMALS, storage::get_decimals(&e))?;
        if price <= 0 {
            return None;
        }

        let timestamp = entry.write_timestamp.checked_div(MILLIS_PER_SECOND)?;

        Some(PriceData { price, timestamp })
    }
}

fn rescale(price: i128, from_decimals: u32, to_decimals: u32) -> Option<i128> {
    if from_decimals >= to_decimals {
        let diff = from_decimals - to_decimals;
        let magnitude = i128::checked_pow(10, diff)?;
        price.checked_div(magnitude)
    } else {
        let diff = to_decimals - from_decimals;
        let magnitude = i128::checked_pow(10, diff)?;
        price.checked_mul(magnitude)
    }
}
