#![allow(clippy::too_many_arguments)]
use soroban_sdk::{
    Address, BytesN, Env, String, contract, contractclient, contractimpl, contracttype,
};

use crate::{
    constants::MANAGER_UPGRADE_IN_QUEUE_SECONDS,
    error::MMCError,
    storage::{self, QueuedInUpgrade, extend_instance},
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
    /// * `market_wasm_hash` - hash of a previously-uploaded market WASM
    /// * `admin` - admin of the deployed market
    /// * `name` - name of the deployed market
    /// * `oracle` - address of SEP-40—compliant oracle contract
    /// * `insurance_fund` - `Insurance Fund` trait compliant contract's address
    /// * `params` - market initialization parameters
    /// * `upgrade_in_queue_period` - upgrade timelock in
    ///   seconds
    #[allow(clippy::too_many_arguments)]
    fn deploy(
        e: Env,
        salt: BytesN<32>,
        market_wasm_hash: BytesN<32>,
        admin: Address,
        name: String,
        oracle: Address,
        insurance_fund: Address,
        params: MarketInitParams,
        upgrade_in_queue_period: u64,
    ) -> Result<Address, MMCError>;

    /// Returns contract's admin account address
    fn get_admin(e: Env) -> Address;

    /// Returns `true` iff `market_address` was deployed via this manager's
    /// `deploy` entrypoint
    fn is_deployed_by_manager(e: Env, market_address: Address) -> bool;

    /// Returns a queued in market contract upgrade info for a
    /// deployed market, if such exists
    fn get_queued_in_market_upgrade(e: Env, market_address: Address) -> Option<QueuedInUpgrade>;

    /// Returns a queued in manager contract upgrade info if such exists
    fn get_queued_in_manager_upgrade(e: Env) -> Option<QueuedInUpgrade>;

    /// Queues in an upgrade for a deployed market.
    ///
    /// # Arguments
    /// * `market_address` - address of a market previously deployed by
    ///   this manager. Must satisfy `is_deployed_by_manager`.
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the
    ///   network, that will be used as the new version of the target
    ///   market contract instance.
    fn queue_in_market_upgrade(
        e: Env,
        market_address: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), MMCError>;

    /// Queues in manager upgrade
    ///
    /// # Arguments
    /// * `new_wasm_hash` - hash of the WASM binary uploaded to the network, used as a
    ///   version of the depl contract instances
    fn queue_in_manager_upgrade(e: Env, new_wasm_hash: BytesN<32>) -> Result<(), MMCError>;

    /// Cancels the upgrade queued for a single market, if one exists.
    fn cancel_market_upgrade(e: Env, market_address: Address) -> Result<(), MMCError>;

    /// Cancels a manager upgrade if such exists in a queue
    fn cancel_manager_upgrade(e: Env) -> Result<(), MMCError>;

    /// Applies a queued in market upgrade
    fn apply_market_upgrade(e: Env, market_address: Address) -> Result<(), MMCError>;

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
        market_wasm_hash: BytesN<32>,
        market_admin: Address,
        name: String,
        oracle: Address,
        insurance_fund: Address,
        params: MarketInitParams,
        upgrade_in_queue_period: u64,
    ) -> Result<Address, MMCError> {
        extend_instance(&e);
        require_admin(&e);

        let market_address = e.deployer().with_current_contract(salt).deploy_v2(
            market_wasm_hash,
            (name, market_admin, oracle, insurance_fund, e.current_contract_address(), params),
        );

        storage::register_market(&e, &market_address, upgrade_in_queue_period)?;

        Ok(market_address)
    }

    fn get_admin(e: Env) -> Address {
        extend_instance(&e);

        storage::get_admin(&e)
    }

    fn is_deployed_by_manager(e: Env, market_address: Address) -> bool {
        extend_instance(&e);

        storage::is_market_deployed(&e, &market_address)
    }

    fn get_queued_in_market_upgrade(e: Env, market_address: Address) -> Option<QueuedInUpgrade> {
        extend_instance(&e);

        storage::get_queued_in_market_upgrade(&e, &market_address)
    }

    fn get_queued_in_manager_upgrade(e: Env) -> Option<QueuedInUpgrade> {
        extend_instance(&e);

        storage::get_queued_in_manager_upgrade(&e)
    }

    fn queue_in_market_upgrade(
        e: Env,
        market_address: Address,
        new_wasm_hash: BytesN<32>,
    ) -> Result<(), MMCError> {
        extend_instance(&e);
        require_admin(&e);
        require_deployed_market(&e, &market_address)?;

        if storage::get_queued_in_market_upgrade(&e, &market_address).is_some() {
            return Err(MMCError::UpgradeAlreadyExists);
        }

        let upgrade = QueuedInUpgrade {
            wasm_hash: new_wasm_hash,
            queued_in_timestamp: e.ledger().timestamp(),
        };
        storage::set_queued_in_market_upgrade(&e, &market_address, &upgrade);

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

    fn cancel_market_upgrade(e: Env, market_address: Address) -> Result<(), MMCError> {
        extend_instance(&e);
        require_admin(&e);

        if storage::get_queued_in_market_upgrade(&e, &market_address).is_none() {
            return Err(MMCError::UpgradeDoesNotExist);
        }

        storage::remove_queued_in_market_upgrade(&e, &market_address);

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

    fn apply_market_upgrade(e: Env, market_address: Address) -> Result<(), MMCError> {
        extend_instance(&e);

        let Some(QueuedInUpgrade { wasm_hash, queued_in_timestamp }) =
            storage::get_queued_in_market_upgrade(&e, &market_address)
        else {
            return Err(MMCError::UpgradeDoesNotExist);
        };
        let upgrade_in_queue_period =
            storage::get_market_upgrade_in_queue_period(&e, &market_address);

        if queued_in_timestamp
            .checked_add(upgrade_in_queue_period)
            .ok_or(MMCError::OverOrUnderflow)?
            > e.ledger().timestamp()
        {
            return Err(MMCError::UpgradeIsNotYetApplicable);
        }

        let market_client = market::Client::new(&e, &market_address);
        market_client.upgrade(&wasm_hash);

        storage::remove_queued_in_market_upgrade(&e, &market_address);

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
            .checked_add(MANAGER_UPGRADE_IN_QUEUE_SECONDS)
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
    pub fn __constructor(e: Env, admin: Address) {
        storage::set_admin(&e, &admin);
    }
}

#[inline(always)]
fn require_admin(e: &Env) {
    storage::get_admin(e).require_auth();
}

#[inline(always)]
fn require_deployed_market(e: &Env, market_address: &Address) -> Result<(), MMCError> {
    if !storage::is_market_deployed(e, market_address) {
        return Err(MMCError::MarketNotDeployedByManager);
    }

    Ok(())
}
