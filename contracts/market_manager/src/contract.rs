#![allow(clippy::too_many_arguments)]
use soroban_sdk::{Address, BytesN, Env, Map, String, contract, contractclient, contractimpl};

use crate::{
    constants::MAX_RESERVES,
    error::MMCError,
    storage::{self, Config, extend_instance_storage},
};

// --- TODO: Remove this before deployment ---

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
    /// * `oracle` - address of SEP-40—compliant oracle contract
    /// * `insurance_fund` - `Insurance Fund` trait compliant contract's address
    /// * `max_positions` - maximum number of positions for a single obligation to have at a single moment
    /// * `min_collateral` - minimum allowed value of a collateral position at a single moment
    /// * `insolvency_ltv_bps` - unparameterized LTV(i.e., not scaled with closeLTV\openLTV\liability factors) that marks obligation in market as insolvent
    /// * `update_in_queue_period` - amount of seconds required to pass before applying an issued pool's config update in an owned pool. Passing here `None` means that the market is permissionless
    ///   and its pools and parameters cannot be modified(except for new pools initialization)
    #[allow(clippy::too_many_arguments)]
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
        oracle: Address,
        insurance_fund: Address,
        max_positions: u32,
        min_collateral: i128,
        insolvency_ltv_bps: i128,
        update_in_queue_period: Option<u64>,
    ) -> Result<Address, MMCError>;

    /// Returns a set of all lending markets deployed by the manager
    fn get_markets(e: Env) -> Map<Address, ()>;

    /// Returns contract's [`Config`]
    fn get_config(e: Env) -> Config;
}

/// Market Manager Contract. Responsible for deploying and updating existing market contracts
#[contract]
pub struct MarketManagerContract;

#[contractimpl]
impl MarketManager for MarketManagerContract {
    #[allow(clippy::too_many_arguments)]
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        market_admin: Address,
        name: String,
        oracle: Address,
        insurance_fund: Address,
        max_positions: u32,
        min_collateral: i128,
        insolvency_ltv_bps: i128,
        update_in_queue_period: Option<u64>,
    ) -> Result<Address, MMCError> {
        extend_instance_storage(&e);

        if !(2..=(2 * MAX_RESERVES)).contains(&max_positions) || min_collateral < 0 {
            return Err(MMCError::InvalidMarketState);
        }

        let Config { admin, market_contract_wasm_hash } = storage::get_config(&e);
        admin.require_auth();

        let market_address = e.deployer().with_current_contract(salt).deploy_v2(
            market_contract_wasm_hash,
            (
                name,
                market_admin,
                oracle,
                insurance_fund,
                e.current_contract_address(),
                max_positions,
                min_collateral,
                insolvency_ltv_bps,
                update_in_queue_period,
            ),
        );

        storage::register_market(&e, &market_address)?;

        Ok(market_address)
    }

    fn get_markets(e: Env) -> Map<Address, ()> {
        extend_instance_storage(&e);

        storage::get_markets(&e).unwrap_or(Map::new(&e))
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

    // --- TODO: TO BE REMOVED ---

    /// Upgrades the market manager contract
    ///
    /// # Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
    ///   new version of the contract
    pub fn upgrade(e: Env, new_wasm_hash: BytesN<32>) {
        require_admin(&e);

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
            for market_address in deployed_markets.keys() {
                let market_client = market::Client::new(&e, &market_address);
                market_client.upgrade(&new_market_contract_wasm_hash);
            }
        }

        storage::set_market_contract_wasm_hash(&e, &new_market_contract_wasm_hash);
    }
}

// -- Helpers --

#[inline(always)] // TODO: to be removed
fn require_admin(e: &Env) {
    storage::get_admin(e).require_auth();
}

#[inline(always)]
pub fn require_nonnegative(amount: i128) -> Result<(), MMCError> {
    if amount < 0 {
        return Err(MMCError::NegativeInputAmount);
    }

    Ok(())
}
