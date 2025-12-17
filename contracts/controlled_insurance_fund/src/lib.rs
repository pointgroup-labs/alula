#![no_std]
use insurance_fund_trait::{CoverageStatus, InsuranceFund};
use soroban_sdk::{Address, Env, contract, contractimpl, panic_with_error, token};
use storage::Request;

use crate::error::ContractError;

#[contract]
pub struct ControlledInsuranceFundContract;

#[contractimpl]
impl ControlledInsuranceFundContract {
    /// Constructs an account-controlled insurance fund contract
    pub fn __constructor(e: Env, admin: Address) {
        storage::set_admin(&e, admin);
        storage::init_requests_counter(&e);
        storage::set_must_claim(&e, false); // MustClaim mutex starts as unlocked
    }

    /// Sets an insured market contract address in the storage
    pub fn set_market(e: Env, market: Address) {
        require_admin(&e);
        storage::extend_instance_storage(&e);

        storage::set_market(&e, market);
    }

    /// # Returns
    /// Request with given `request_id` if exists
    pub fn get_request(e: &Env, request_id: u64) -> Option<Request> {
        storage::extend_instance_storage(e);

        storage::get_request(e, request_id)
    }

    /// Withdraws tokens if there are no coverage results that must be claimed first
    pub fn withdraw(e: Env, token: Address, to: Address, amount: i128) {
        require_admin(&e);
        if storage::get_must_claim(&e) {
            panic_with_error!(&e, ContractError::MustClaimCoverage);
        }

        let token_client = token::Client::new(&e, &token);
        token_client.transfer(&e.current_contract_address(), &to, &amount);
    }

    /// Marks existing request as ready
    ///
    /// # Panics
    /// If a request that's ready and must be claimed already exists, or if a request
    /// with given `request_id` does not exist
    pub fn mark_ready(e: Env, request_id: u64, covered_amount: i128) {
        require_admin(&e);
        if storage::get_must_claim(&e) {
            panic_with_error!(&e, ContractError::MustClaimCoverage);
        }

        let Some(mut request) = storage::get_request(&e, request_id) else {
            panic_with_error!(&e, ContractError::RequestDoesNotExist);
        };
        if !matches!(request.status, CoverageStatus::Pending) {
            panic_with_error!(&e, ContractError::InternalError);
        }

        let fund_balance =
            token::Client::new(&e, &request.token).balance(&e.current_contract_address());
        let real_covered_amount = i128::min(covered_amount, request.amount);
        if fund_balance < real_covered_amount {
            panic_with_error!(&e, ContractError::InsufficientContractBalance);
        }

        request.status = CoverageStatus::Ready(real_covered_amount);
        storage::update_request(&e, request_id, request);

        storage::set_must_claim(&e, true);
    }

    pub fn update_market_status(_e: Env) {
        todo!()
    }
}

#[contractimpl]
impl InsuranceFund for ControlledInsuranceFundContract {
    fn request_coverage(e: Env, token: Address, amount: i128) -> Option<u64> {
        require_market(&e);
        storage::extend_instance_storage(&e);

        let request = Request::new(token, amount);
        let request_id = storage::set_request(&e, request);

        Some(request_id)
    }

    fn get_status(e: Env, request_id: u64) -> Option<CoverageStatus> {
        storage::extend_instance_storage(&e);

        let request = storage::get_request(&e, request_id)?;

        Some(request.status)
    }

    fn claim_coverage(e: Env, request_id: u64) -> i128 {
        let market = storage::get_market(&e);
        market.require_auth();
        if !storage::get_must_claim(&e) {
            panic_with_error!(&e, ContractError::InternalError);
        }
        storage::extend_instance_storage(&e);

        let Some(request) = storage::get_request(&e, request_id) else {
            panic_with_error!(&e, ContractError::RequestDoesNotExist);
        };
        let CoverageStatus::Ready(coverage_amount) = request.status else {
            panic_with_error!(&e, ContractError::RequestIsNotReady);
        };

        let token_client = token::Client::new(&e, &request.token);
        token_client.transfer(&e.current_contract_address(), &market, &coverage_amount);

        storage::remove_request(&e, request_id);
        storage::set_must_claim(&e, false);

        coverage_amount
    }
}

// -- Helpers --

fn require_admin(e: &Env) {
    let admin = storage::get_admin(e);
    admin.require_auth();
}

fn require_market(e: &Env) {
    let market = storage::get_market(e);
    market.require_auth();
}

pub mod error;
pub mod market;
pub mod storage;
