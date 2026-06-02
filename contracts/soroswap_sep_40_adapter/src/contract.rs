use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
use soroban_sdk::{Address, BytesN, Env, Vec, contract, contractimpl, panic_with_error};

use crate::{constants::*, error::SS40ACError, storage, swap};

#[contract]
pub struct SoroswapSep40AdapterContract;

#[contractimpl]
impl SoroswapSep40AdapterContract {
    /// Constructs the adapter contract
    ///
    /// # Arguments
    /// * `admin` - contract's administrator
    pub fn __constructor(e: Env, admin: Address) {
        storage::set_admin(&e, &admin);
    }

    /// Upgrades the contract
    ///
    /// # Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
    ///   new version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        require_admin(&e);

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }
}

#[contractimpl]
impl PriceFeedTrait for SoroswapSep40AdapterContract {
    fn base(_e: Env) -> Asset {
        Asset::Other(USD_SYMBOL)
    }

    fn assets(e: Env) -> Vec<Asset> {
        panic_with_error!(&e, SS40ACError::Unimplemented)
    }

    fn decimals(_e: Env) -> u32 {
        DECIMALS
    }

    fn resolution(_e: Env) -> u32 {
        RESOLUTION
    }

    fn price(e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        panic_with_error!(&e, SS40ACError::Unimplemented)
    }

    fn prices(e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        panic_with_error!(&e, SS40ACError::Unimplemented)
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        let Asset::Stellar(address) = asset else {
            // Adapter abstracts only `Asset::Stellar` assets
            return None;
        };

        let price = swap::get_price(&e, address)?;
        let price_data = PriceData { price, timestamp: e.ledger().timestamp() };

        Some(price_data)
    }
}

// ---- Helpers ----

fn require_admin(e: &Env) {
    let admin =
        storage::get_admin(e).unwrap_or_else(|| panic_with_error!(e, SS40ACError::InternalError));
    admin.require_auth();
}
