use soroban_sdk::{
    Address, Bytes, BytesN, Env, String, Vec, contract, contractclient, contractimpl, xdr::ToXdr,
};

use crate::{
    error::MMCError,
    storage::{self, Config, extend_instance_storage},
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
    /// # Arguments
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
        // Maybe, it's reasonable to have them to avoid liquidation fragmentation
    ) -> Result<Address, MMCError>;

    /// Returns a list of all lending markets deployed by the manager
    fn get_market_list(e: Env) -> Vec<Address>;

    /// Returns contract's [`Config`]
    fn get_config(e: Env) -> Config;
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
        extend_instance_storage(&e);

        let Config { admin, market_contract_wasm_hash } = storage::get_config(&e);
        admin.require_auth();

        // NB: `soroban_sdk 22` doesn't have an obvious and easy-to-implement way
        // calculating new salt based on `String` or `Symbol`. Newer `soroban_sdk 23` has a way
        // of doing this, see - <https://github.com/stellar/stellar-protocol/blob/master/core/cap-0069.md>,
        // yet, most of our dependencies can rely only on `soroban sdk 22`, so, instead, we
        // calculate new salt based on admin address and provided salt, as is done on other
        // Soroban platforms, deployed with `soroban sdk 22`

        // TODO: Should we do it like this or like Blend does it?
        // let mut seed = Bytes::new(&e);
        // seed.extend_from_slice(admin.to_xdr(&e).to_buffer::<40>().as_slice());
        // seed.extend_from_array(&salt.to_array());
        // let new_salt = e.crypto().keccak256(&seed);

        let market_address = e.deployer().with_current_contract(salt).deploy_v2(
            market_contract_wasm_hash,
            (name, market_admin, oracle, e.current_contract_address()),
        );

        storage::register_market(&e, &market_address)?;

        Ok(market_address)
    }

    fn get_market_list(e: Env) -> Vec<Address> {
        extend_instance_storage(&e);

        storage::get_markets(&e).unwrap_or(Vec::new(&e))
    }

    fn get_config(e: Env) -> Config {
        extend_instance_storage(&e);

        storage::get_config(&e)
    }
}

#[contractimpl]
impl MarketManagerContract {
    /// Constructs the manager contract
    ///
    /// # Arguments
    /// * `admin` - manager's admin
    /// * `market_contract_wasm_hash` - hash of the WASM binary uploaded to the network, used as a
    ///  version of the deployed market contract instances
    pub fn __constructor(e: Env, admin: Address, market_contract_wasm_hash: BytesN<32>) {
        storage::set_admin(&e, &admin);
        storage::set_market_contract_wasm_hash(&e, &market_contract_wasm_hash);
    }

    /// Upgrades the market manager contract
    ///
    /// # Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
    ///   new version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        let config = storage::get_config(&e);
        config.admin.require_auth();

        e.deployer().update_current_contract_wasm(new_wasm_hash);
    }

    /// Upgrades all deployed market contracts
    ///
    /// # Arguments
    /// * `new_market_contract_wasm_hash` - hash of the WASM binary uploaded to the network that
    ///   will be used as a new version of the contract for every deployed market
    pub fn upgrade_deployed_markets(e: Env, new_market_contract_wasm_hash: BytesN<32>) {
        require_admin(&e);

        if let Some(deployed_markets) = storage::get_markets(&e) {
            for market_address in deployed_markets {
                let market_client = market::Client::new(&e, &market_address);
                market_client.upgrade(&new_market_contract_wasm_hash);
            }
        }

        storage::set_market_contract_wasm_hash(&e, &new_market_contract_wasm_hash);
    }
}

// -- Helpers --

fn require_admin(e: &Env) {
    storage::get_admin(e).require_auth();
}
