use soroban_sdk::{Address, BytesN, Env, String, Vec, contract, contractclient, contractimpl};

use crate::{
    error::MMCError,
    storage::{self, Config},
};

mod market {
    use soroban_sdk::contractimport;

    #[cfg(not(feature = "deploy"))]
    contractimport!(file = "../../wasms/market.wasm");

    #[cfg(feature = "deploy")]
    contractimport!(file = "../../wasms/deploy/market.wasm");
}

#[contractclient(name = "MarketManagerClient")]
pub trait MarketManager {
    /// Deploys a lending market
    ///
    /// ### Arguments
    /// * `salt` - salt bytes that are used to derive a deterministic market address
    /// * `admin` - admin of the deployed market
    /// * `name` - name of the deployed market
    /// * `oracle_address` - address of SEP-40—compliant oracle contract
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
        oracle_address: Address,
        // TODO: max_positions,
        // TODO: min_collateral,
        // what would be the reasons for these parameters?
    ) -> Result<Address, MMCError>;

    /// Returns a list of all lending markets deployed by the manager
    fn get_market_list(e: Env) -> Option<Vec<Address>>;
}

/// Market Manager Contract. Responsible for deploying and updating existing market contracts
#[contract]
pub struct MarketManagerContract;

#[contractimpl]
impl MarketManager for MarketManagerContract {
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        market_admin: Address,
        name: String,
        oracle: Address,
    ) -> Result<Address, MMCError> {
        let Config {
            admin,
            market_contract_wasm_hash,
        } = storage::get_config(&e);
        admin.require_auth();

        let market_address = e
            .deployer()
            .with_current_contract(salt)
            .deploy_v2(market_contract_wasm_hash, (name, market_admin, oracle));

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
    ///
    /// ### Arguments
    /// * `admin` - manager's admin
    /// * `market_contract_wasm_hash` - hash of the WASM binary uploaded to the network, used as a
    ///  version of the deployed market contract instances
    pub fn __constructor(e: Env, admin: Address, market_contract_wasm_hash: BytesN<32>) {
        let config = Config {
            admin,
            market_contract_wasm_hash,
        };

        storage::set_config(&e, config);
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
    /// * `new_market_contract_wasm_hash` - hash of the WASM binary uploaded to the network that
    ///   will be used as a new version of the contract for every deployed market
    pub fn upgrade_deployed_markets(e: Env, new_market_contract_wasm_hash: BytesN<32>) {
        let mut config = storage::get_config(&e);
        config.admin.require_auth();

        if let Some(deployed_markets) = storage::get_markets(&e) {
            for market_address in deployed_markets {
                let market_client = market::Client::new(&e, &market_address);
                market_client.upgrade(&new_market_contract_wasm_hash);
            }
        }

        config.market_contract_wasm_hash = new_market_contract_wasm_hash;
        storage::set_config(&e, config);
    }
}
