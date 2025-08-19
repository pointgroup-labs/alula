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

        storage::add_asset(&e, &asset);
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
        // NB: Same as in https://stellar.expert/explorer/testnet/contract/CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63
        14
    }

    fn resolution(_e: Env) -> u32 {
        // NB: Same as in https://stellar.expert/explorer/testnet/contract/CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63
        300
    }

    fn price(_e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
        unimplemented!()
    }

    fn prices(_e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
        unimplemented!()
    }

    fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        todo!()
    }
}

// ---- Helpers ----

fn require_admin(e: &Env) {
    let admin = storage::get_admin(e)
        .unwrap_or_else(|| panic_with_error!(e, AggregatedOracleContractError::NotAnAdmin));
    admin.require_auth();
}
