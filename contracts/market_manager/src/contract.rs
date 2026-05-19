#![allow(clippy::too_many_arguments)]
use soroban_sdk::{
    Address, BytesN, Env, Map, String, contract, contractclient, contractimpl, contracttype,
};

use crate::{
    constants::UPGRADE_IN_QUEUE_SECONDS,
    error::MMCError,
    storage::{self, Config, QueuedInUpgrade, extend_instance},
};

mod market {
    use soroban_sdk::contractimport;

    #[cfg(not(feature = "deploy"))]
    contractimport!(file = "../../wasms/market.wasm");

    #[cfg(feature = "deploy")]
    contractimport!(file = "../../wasms/deploy/market.wasm");
}

#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MarketInitParams {
    pub max_positions: u32,
    pub min_collateral_value_cents: i128,
    pub insolvency_ltv_bps: i128,
    pub update_in_queue_period: u64,
    pub is_owned: bool,
    pub bad_debt_lock_d: u64,
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
    /// * `params` - market initialization parameters
    #[allow(clippy::too_many_arguments)]
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        admin: Address,
        name: String,
        oracle: Address,
        insurance_fund: Address,
        params: MarketInitParams,
    ) -> Result<Address, MMCError>;

    /// Returns a set of all lending markets deployed by the manager
    fn get_markets(e: Env) -> Map<Address, ()>;

    /// Returns contract's [`Config`]
    fn get_config(e: Env) -> Config;

    /// Returns the current active market WASM hash used for deployments and upgrades
    fn get_market_wasm_hash(e: Env) -> BytesN<32>;

    /// Returns a queued in market contract upgrade info if such exists
    fn get_queued_in_market_upgrade(e: Env) -> Option<QueuedInUpgrade>;

    /// Returns a queued in manager contract upgrade info if such exists
    fn get_queued_in_manager_upgrade(e: Env) -> Option<QueuedInUpgrade>;

    /// Queues in market upgrade
    ///
    /// # Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network, that will be used as a
    ///   version of the deployed market contract instances
    fn queue_in_market_upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), MMCError>;

    /// Queues in manager upgrade
    ///
    /// # Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network, used as a
    ///   version of the depl contract instances
    fn queue_in_manager_upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), MMCError>;

    /// Cancels a market upgrade if such exists in a queue
    fn cancel_market_upgrade(e: Env) -> Result<(), MMCError>;

    /// Cancels a manager upgrade if such exists in a queue
    fn cancel_manager_upgrade(e: Env) -> Result<(), MMCError>;

    /// Applies a queued in market upgrade
    fn apply_market_upgrade(e: Env) -> Result<(), MMCError>;

    /// Applies a queued in manager upgrade
    fn apply_manager_upgrade(e: Env) -> Result<(), MMCError>;

    /// Proposes a new admin for the manager contract.
    /// Must be called by the current admin. The proposal is stored until
    /// the proposed address calls `accept_admin`.
    ///
    /// # Arguments
    /// * `new_admin` - the address being proposed as the new admin
    fn propose_admin(e: Env, new_admin: Address) -> Result<(), MMCError>;

    /// Accepts a pending admin proposal.
    /// Must be called by the address that was previously proposed via `propose_admin`.
    /// On success the caller becomes the new admin and the proposal is cleared.
    fn accept_admin(e: Env) -> Result<(), MMCError>;
}

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
        params: MarketInitParams,
    ) -> Result<Address, MMCError> {
        extend_instance(&e);

        let Config { admin, market_wasm_hash } = storage::get_config(&e);
        admin.require_auth();

        let market_address = e.deployer().with_current_contract(salt).deploy_v2(
            market_wasm_hash,
            (name, market_admin, oracle, insurance_fund, e.current_contract_address(), params),
        );

        storage::register_market(&e, &market_address)?;

        Ok(market_address)
    }

    fn get_markets(e: Env) -> Map<Address, ()> {
        extend_instance(&e);

        storage::get_markets(&e).unwrap_or(Map::new(&e))
    }

    fn get_config(e: Env) -> Config {
        extend_instance(&e);

        storage::get_config(&e)
    }

    fn get_market_wasm_hash(e: Env) -> BytesN<32> {
        extend_instance(&e);

        storage::get_market_wasm_hash(&e)
    }

    fn get_queued_in_market_upgrade(e: Env) -> Option<QueuedInUpgrade> {
        extend_instance(&e);

        storage::get_queued_in_market_upgrade(&e)
    }

    fn get_queued_in_manager_upgrade(e: Env) -> Option<QueuedInUpgrade> {
        extend_instance(&e);

        storage::get_queued_in_manager_upgrade(&e)
    }

    fn queue_in_market_upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), MMCError> {
        extend_instance(&e);
        require_admin(&e);

        if storage::get_queued_in_market_upgrade(&e).is_some() {
            return Err(MMCError::UpgradeAlreadyExists);
        }

        let upgrade = QueuedInUpgrade {
            wasm_hash: new_wasm_hash,
            queued_in_timestamp: e.ledger().timestamp(),
        };
        storage::set_queued_in_market_upgrade(&e, &upgrade);

        Ok(())
    }

    fn queue_in_manager_upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), MMCError> {
        extend_instance(&e);
        require_admin(&e);

        if storage::get_queued_in_manager_upgrade(&e).is_some() {
            return Err(MMCError::UpgradeAlreadyExists);
        }

        let upgrade = QueuedInUpgrade {
            wasm_hash: new_wasm_hash,
            queued_in_timestamp: e.ledger().timestamp(),
        };
        storage::set_queued_in_manager_upgrade(&e, &upgrade);

        Ok(())
    }

    fn cancel_market_upgrade(e: Env) -> Result<(), MMCError> {
        extend_instance(&e);
        require_admin(&e);

        if storage::get_queued_in_market_upgrade(&e).is_none() {
            return Err(MMCError::UpgradeDoesNotExist);
        }

        storage::remove_queued_in_market_upgrade(&e);

        Ok(())
    }

    fn cancel_manager_upgrade(e: Env) -> Result<(), MMCError> {
        extend_instance(&e);
        require_admin(&e);

        if storage::get_queued_in_manager_upgrade(&e).is_none() {
            return Err(MMCError::UpgradeDoesNotExist);
        }

        storage::remove_queued_in_manager_upgrade(&e);

        Ok(())
    }

    fn apply_market_upgrade(e: Env) -> Result<(), MMCError> {
        extend_instance(&e);

        let Some(QueuedInUpgrade { wasm_hash, queued_in_timestamp }) =
            storage::get_queued_in_market_upgrade(&e)
        else {
            return Err(MMCError::UpgradeDoesNotExist);
        };

        if queued_in_timestamp
            .checked_add(UPGRADE_IN_QUEUE_SECONDS)
            .ok_or(MMCError::OverOrUnderflow)?
            > e.ledger().timestamp()
        {
            return Err(MMCError::UpgradeIsNotYetApplicable);
        }

        if let Some(deployed_markets) = storage::get_markets(&e) {
            for market_address in deployed_markets.keys() {
                let market_client = market::Client::new(&e, &market_address);
                market_client.upgrade(&wasm_hash);
            }
        }

        storage::set_market_wasm_hash(&e, &wasm_hash);
        storage::remove_queued_in_market_upgrade(&e);

        Ok(())
    }

    fn apply_manager_upgrade(e: Env) -> Result<(), MMCError> {
        extend_instance(&e);

        let Some(QueuedInUpgrade { wasm_hash, queued_in_timestamp }) =
            storage::get_queued_in_manager_upgrade(&e)
        else {
            return Err(MMCError::UpgradeDoesNotExist);
        };

        if queued_in_timestamp
            .checked_add(UPGRADE_IN_QUEUE_SECONDS)
            .ok_or(MMCError::OverOrUnderflow)?
            > e.ledger().timestamp()
        {
            return Err(MMCError::UpgradeIsNotYetApplicable);
        }

        storage::remove_queued_in_manager_upgrade(&e);
        e.deployer().update_current_contract_wasm(wasm_hash);

        Ok(())
    }

    fn propose_admin(e: Env, new_admin: Address) -> Result<(), MMCError> {
        extend_instance(&e);
        require_admin(&e);

        storage::set_pending_admin(&e, &new_admin);

        Ok(())
    }

    fn accept_admin(e: Env) -> Result<(), MMCError> {
        extend_instance(&e);

        let pending_admin = storage::get_pending_admin(&e).ok_or(MMCError::NoPendingAdmin)?;
        pending_admin.require_auth();

        storage::set_admin(&e, &pending_admin);
        storage::remove_pending_admin(&e);

        Ok(())
    }
}

#[contractimpl]
impl MarketManagerContract {
    pub fn __constructor(e: Env, admin: Address, market_contract_wasm_hash: BytesN<32>) {
        storage::set_admin(&e, &admin);
        storage::set_market_wasm_hash(&e, &market_contract_wasm_hash);
    }
}

#[inline(always)]
fn require_admin(e: &Env) {
    storage::get_admin(e).require_auth();
}
