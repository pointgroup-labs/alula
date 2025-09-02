use crate::{
    storage::{self, Config},
    MMError,
};
use soroban_sdk::{contract, contractclient, contractimpl, Address, BytesN, Env, String, Vec};

#[contractclient(name = "MarketManagerClient")]
pub trait MarketManager {
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
        oracle: Address,
        /* max_positions, */
        /* min_collateral, */
    ) -> Result<Address, MMError>;

    fn get_market_list(e: Env) -> Option<Vec<Address>>;
}

#[contract]
pub struct MarketManagerContract;

impl MarketManager for MarketManagerContract {
    fn deploy(
        e: Env,
        salt: BytesN<32>, // optional or not?
        _admin: Address,
        _name: String, // String or Symbol?
        _oracle: Address,
    ) -> Result<Address, MMError> {
        let Config {
            admin,
            market_contract_wasm_hash,
        } = storage::get_config(&e);
        admin.require_auth();

        let market_address = e
            .deployer()
            .with_current_contract(salt) // by the way, what's going to happen if we do this twice?
            .deploy_v2(market_contract_wasm_hash, ());

        storage::register_market(&e, &market_address)?;

        Ok(market_address)
    }

    fn get_market_list(e: Env) -> Option<Vec<Address>> {
        storage::get_markets(&e)
    }
}

#[contractimpl]
impl MarketManagerContract {
    pub fn __constructor(e: Env, manager_config: Config) {
        storage::set_config(&e, manager_config);
    }

    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let mut config = storage::get_config(&e);
        config.admin.require_auth();

        config.market_contract_wasm_hash = new_wasm_hash;

        // TODO: Update all market contract instances?

        storage::set_config(&e, config);
    }

    // TODO: pub fn upgrade_admin?? | likely no
}
