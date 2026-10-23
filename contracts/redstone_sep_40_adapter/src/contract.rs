use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
use soroban_sdk::{Address, Env, Map, Vec, contract, contractimpl, panic_with_error, vec as svec};

use crate::{
    constants::*,
    error::RS40ACError,
    events,
    redstone::RedStonePriceFeedClient,
    storage::{self, Feed},
};

#[contract]
pub struct RedStoneSep40AdapterContract;

#[contractimpl]
impl RedStoneSep40AdapterContract {
    /// Constructs the adapter contract.
    ///
    /// # Arguments
    /// * `admin` - contract's administrator (can add feeds and update resolution)
    /// * `base_asset` - asset that will be the result of the `base()` endpoint call
    /// * `decimals` - number of decimals in the output price
    /// * `resolution` - staleness threshold in seconds; prices older than this are rejected
    /// * `feeds` - token address → RedStone price feed contract address mapping; each feed's
    ///   `decimals()` is read once here and must be at most 18
    pub fn __constructor(
        e: Env,
        admin: Address,
        base_asset: Asset,
        decimals: u32,
        resolution: u32,
        feeds: Map<Address, Address>,
    ) {
        if !(MIN_RESOLUTION..=MAX_RESOLUTION).contains(&resolution) {
            panic_with_error!(&e, RS40ACError::BadConfig);
        }

        storage::set_admin(&e, &admin);
        storage::set_decimals(&e, decimals);
        storage::set_resolution(&e, resolution);
        storage::set_base_asset(&e, &base_asset);

        let mut assets = svec![&e];
        for (token_address, price_feed) in feeds.iter() {
            if storage::get_feed_token(&e, &price_feed).is_some() {
                panic_with_error!(&e, RS40ACError::TokenForFeedAlreadyInUse);
            }

            storage::set_feed(&e, &token_address, &read_feed(&e, price_feed));
            assets.push_back(Asset::Stellar(token_address));
        }

        storage::set_assets(&e, &assets);
    }

    /// Adds a price feed for a token. Admin-gated. Panics if a feed for this token already exists
    /// or if the feed's `decimals()` does not answer at most 18.
    pub fn add_feed(e: Env, token_address: Address, price_feed: Address) {
        require_admin(&e);
        storage::extend_instance(&e);

        if storage::get_feed(&e, &token_address).is_some() {
            panic_with_error!(&e, RS40ACError::FeedForTokenAlreadyInUse);
        }
        if storage::get_feed_token(&e, &price_feed).is_some() {
            panic_with_error!(&e, RS40ACError::TokenForFeedAlreadyInUse);
        }

        storage::set_feed(&e, &token_address, &read_feed(&e, price_feed.clone()));

        let mut assets = storage::get_assets(&e);
        assets.push_back(Asset::Stellar(token_address.clone()));
        storage::set_assets(&e, &assets);

        events::FeedAdded { token_address, price_feed }.publish(&e);
    }

    /// Updates the staleness threshold. Admin-gated.
    pub fn set_resolution(e: Env, resolution: u32) {
        require_admin(&e);
        storage::extend_instance(&e);

        if !(MIN_RESOLUTION..=MAX_RESOLUTION).contains(&resolution) {
            panic_with_error!(&e, RS40ACError::BadConfig);
        }

        storage::set_resolution(&e, resolution);
        events::ResolutionUpdated { resolution }.publish(&e);
    }

    /// Nominates `new_admin` as the next administrator. Admin-gated.
    /// The transfer is not effective until `new_admin` calls `accept_admin`.
    pub fn propose_admin(e: Env, new_admin: Address) {
        require_admin(&e);

        storage::extend_instance(&e);
        storage::set_pending_admin(&e, &new_admin);

        events::AdminProposed { proposed_admin: new_admin }.publish(&e);
    }

    /// Completes the admin transfer initiated by `propose_admin`.
    /// Must be called by the address that was proposed.
    pub fn accept_admin(e: Env) {
        storage::extend_instance(&e);

        let pending_admin = storage::get_pending_admin(&e)
            .unwrap_or_else(|| panic_with_error!(&e, RS40ACError::NoPendingAdmin));
        pending_admin.require_auth();

        storage::set_admin(&e, &pending_admin);
        storage::remove_pending_admin(&e);

        events::AdminAccepted { new_admin: pending_admin }.publish(&e);
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

    fn resolution(e: Env) -> u32 {
        storage::extend_instance(&e);

        storage::get_resolution(&e)
    }

    fn price(e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        panic_with_error!(&e, RS40ACError::Unimplemented)
    }

    fn prices(e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        panic_with_error!(&e, RS40ACError::Unimplemented)
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        storage::extend_instance(&e);

        let Asset::Stellar(token_address) = asset else {
            return None;
        };

        let feed = storage::get_feed(&e, &token_address)?;
        let client = RedStonePriceFeedClient::new(&e, &feed.address);

        let entry = client.try_read_price_data().ok()?.ok()?;

        let timestamp = entry.package_timestamp / REDSTONE_TS_PER_SEC;
        let now = e.ledger().timestamp();
        let resolution = storage::get_resolution(&e);

        if timestamp > now || now - timestamp > resolution as u64 {
            return None;
        }

        let price_raw = i128::try_from(entry.price.to_u128()?).ok()?;
        let price = rescale(price_raw, feed.decimals, storage::get_decimals(&e))?;
        if price <= 0 {
            return None;
        }

        Some(PriceData { price, timestamp })
    }
}

/// `price_feed` with the decimals it reports, which must be at most `MAX_FEED_DECIMALS`.
fn read_feed(e: &Env, price_feed: Address) -> Feed {
    match RedStonePriceFeedClient::new(e, &price_feed).try_decimals() {
        Ok(Ok(decimals)) if decimals <= MAX_FEED_DECIMALS => {
            Feed { address: price_feed, decimals: decimals as u32 }
        }
        _ => panic_with_error!(e, RS40ACError::BadFeedDecimals),
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

fn require_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}
