#![no_std]
use insurance_fund_interface::{CoverageStatus, InsuranceFund, IssueRequestResult};
use soroban_sdk::{Address, Env, contract, contractevent, contractimpl, panic_with_error, token};
use storage::Request;

use crate::error::ContractError;

#[contract]
pub struct ControlledInsuranceFundContract;

#[contractimpl]
impl ControlledInsuranceFundContract {
    /// Constructs an account-controlled insurance fund contract
    pub fn __constructor(e: Env, admin: Address) {
        storage::set_admin(&e, &admin);
        storage::init_requests_counter(&e);
    }

    /// Sets an insured market contract address in the storage
    pub fn set_market(e: Env, market: Address) -> Result<(), ContractError> {
        require_admin(&e);
        storage::extend_instance(&e);

        if storage::is_market_set(&e) {
            return Err(ContractError::MarketIsAlreadySet);
        }
        storage::set_market(&e, market);

        Ok(())
    }

    /// # Returns
    /// Request with given `request_id` if exists
    pub fn get_request(e: &Env, request_id: u64) -> Option<Request> {
        storage::extend_instance(e);

        storage::get_request(e, request_id)
    }

    /// Withdraws tokens, respecting the locked liquidity needed for pending claims
    ///
    /// # Panics
    /// If the fund does not have enough *free* liquidity for available for withdraw
    pub fn withdraw(e: Env, token: Address, to: Address, amount: i128) {
        require_admin(&e);

        let token_client = token::Client::new(&e, &token);
        let current_balance = token_client.balance(&e.current_contract_address());
        let locked_amount = storage::get_locked_amount(&e, &token);

        let free_balance = current_balance.checked_sub(locked_amount).unwrap_or(0);
        if amount > free_balance {
            panic_with_error!(&e, ContractError::InsufficientContractBalance);
        }

        token_client.transfer(&e.current_contract_address(), &to, &amount);
    }

    /// Marks existing request as ready and locks the liquidity
    ///
    /// # Panics
    /// If the fund does not have enough *free* liquidity to cover this request
    pub fn mark_ready(e: Env, request_id: u64, covered_amount: i128) {
        require_admin(&e);

        let Some(mut request) = storage::get_request(&e, request_id) else {
            panic_with_error!(&e, ContractError::RequestDoesNotExist);
        };
        if !matches!(request.status, CoverageStatus::Pending) {
            panic_with_error!(&e, ContractError::RequestIsReady);
        }

        let real_covered_amount = i128::min(covered_amount, request.amount);

        let token_client = token::Client::new(&e, &request.token);
        let current_balance = token_client.balance(&e.current_contract_address());
        let total_locked = storage::get_locked_amount(&e, &request.token);

        if current_balance < total_locked {
            TotalLockedExceedsBalance { request_id, total_locked, current_balance }.publish(&e);

            panic_with_error!(&e, ContractError::InternalError);
        }

        let free_liquidity = current_balance - total_locked; // safe
        if free_liquidity < real_covered_amount {
            panic_with_error!(&e, ContractError::InsufficientContractBalance);
        }

        let new_locked_total = total_locked.checked_add(real_covered_amount).unwrap_or_else(|| {
            panic_with_error!(&e, ContractError::OverOrUnderflow);
        });
        storage::set_locked_amount(&e, &request.token, new_locked_total);

        request.status = CoverageStatus::Ready(real_covered_amount);
        storage::update_request(&e, request_id, request);
    }

    /// Updates market status (Descriptive/Helper). Funds can decide that a market is not insured
    /// enough to proceed safely and update its status accordingly. Owned market admins can always overwrite
    /// this behavior
    pub fn update_market_status(e: Env, new_status: u32) {
        require_admin(&e);

        let market = storage::get_market(&e);
        let market_client = market::MarketPartialClient::new(&e, &market);
        market_client.fund_update_market_status(&new_status);
    }

    /// Proposes a new fund's admin
    pub fn propose_new_admin(e: Env, proposed_admin: Address) -> Result<(), ContractError> {
        require_admin(&e);
        storage::extend_instance(&e);

        storage::set_proposed_admin(&e, proposed_admin);

        Ok(())
    }

    // Accepts the proposal to become a new admin
    pub fn accept_proposed_admin(e: Env) -> Result<(), ContractError> {
        storage::extend_instance(&e);

        let proposed_admin =
            storage::get_proposed_admin(&e).ok_or(ContractError::ProposedAdminIsNotSet)?;
        proposed_admin.require_auth();

        storage::set_admin(&e, &proposed_admin);
        storage::remove_proposed_admin(&e);

        Ok(())
    }
}

#[contractimpl]
impl InsuranceFund for ControlledInsuranceFundContract {
    fn add_reserves(e: Env, _token: Address, _amount: i128) {
        require_market(&e); // NB: Only validation for this token-balance driven implementation
        storage::extend_instance(&e);
    }

    fn request_coverage(e: Env, token: Address, amount: i128) -> IssueRequestResult {
        require_market(&e);
        storage::extend_instance(&e);

        let request = Request::new(token, amount);
        let request_id = storage::set_request(&e, request);

        IssueRequestResult::Recorded(request_id) // NB: `Controlled` implementation always returns `Recorded`
    }

    fn get_status(e: Env, request_id: u64) -> Option<CoverageStatus> {
        storage::extend_instance(&e);
        let request = storage::get_request(&e, request_id)?;

        Some(request.status)
    }

    fn claim_coverage(e: Env, request_id: u64) -> i128 {
        let market = storage::get_market(&e);
        market.require_auth();
        storage::extend_instance(&e);

        let Some(request) = storage::get_request(&e, request_id) else {
            panic_with_error!(&e, ContractError::RequestDoesNotExist);
        };
        let CoverageStatus::Ready(coverage_amount) = request.status else {
            panic_with_error!(&e, ContractError::RequestIsNotReady);
        };

        let token_client = token::Client::new(&e, &request.token);
        token_client.transfer(&e.current_contract_address(), &market, &coverage_amount);

        let current_locked = storage::get_locked_amount(&e, &request.token);
        if current_locked < coverage_amount {
            CoverageExceedsTotalLocked {
                request_id,
                total_locked: current_locked,
                coverage_amount,
            }
            .publish(&e);

            panic_with_error!(&e, ContractError::InternalError);
        }
        let new_locked = current_locked - coverage_amount; // safe

        storage::set_locked_amount(&e, &request.token, new_locked);
        storage::remove_request(&e, request_id);

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

// -- Events --

#[contractevent]
struct CoverageExceedsTotalLocked {
    request_id: u64,
    total_locked: i128,
    coverage_amount: i128,
}

#[contractevent]
struct TotalLockedExceedsBalance {
    request_id: u64,
    total_locked: i128,
    current_balance: i128,
}

pub mod error;
pub mod market;
pub mod storage;
