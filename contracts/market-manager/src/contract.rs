use market::contract::MarketContractClient;
use soroban_sdk::{Address, BytesN, Env, String, Vec, contract, contractclient, contractimpl};

use crate::{
    MMError,
    storage::{self, Config},
};

#[contractclient(name = "MarketManagerClient")]
pub trait MarketManager {
    /// Deploys a lending market
    ///
    /// ### Arguments
    /// * `salt` - salt bytes that are used to derive a deterministic market address
    /// * `admin` - admin of the deployed market
    /// * `name` - name of the deployed market
    /// * `oracle_address` - address of SEP-40—compliant oracle contract
    /// * `max_positions` - maximum amount of open positions per market's user
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
        oracle_address: Address,
        max_positions: u32,
        // TODO: min_collateral, what would be the reasons for this parameter?
    ) -> Result<Address, MMError>;

    /// Returns a list of all lending markets deployed by the manager
    fn get_market_list(e: Env) -> Option<Vec<Address>>;
}

/// Market Manager Contract. Responsible for deploying and updating existing market contracts
#[contract]
pub struct MarketManagerContract;

#[contractimpl]
impl MarketManager for MarketManagerContract {
    #[allow(unused)]
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
        oracle: Address,
        max_positions: u32,
    ) -> Result<Address, MMError> {
        let Config {
            admin,
            market_contract_wasm_hash,
        } = storage::get_config(&e);
        admin.require_auth();

        let market_address = e
            .deployer()
            .with_current_contract(salt) // NB: what's going to happen if we do this twice?
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
    /// Constructs the manager contract
    pub fn __constructor(e: Env, manager_config: Config) {
        storage::set_config(&e, manager_config);
    }

    /// Upgrades the market manager contract
    ///
    /// ### Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
    ///   new version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let config = storage::get_config(&e);
        config.admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Upgrades all deployed market contracts
    ///
    /// ### Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
    ///   new version of the contract for every deployed market
    pub fn upgrade_deployed_markets(e: Env, new_wasm_hash: BytesN<32>) {
        let mut config = storage::get_config(&e);
        config.admin.require_auth();

        if let Some(deployed_markets) = storage::get_markets(&e) {
            for market_address in deployed_markets {
                let market_client = MarketContractClient::new(&e, &market_address);
                market_client.upgrade(&new_wasm_hash);
            }
        }

        config.market_contract_wasm_hash = new_wasm_hash;
        storage::set_config(&e, config);
    }
}
