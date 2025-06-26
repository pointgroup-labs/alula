#![no_std]
use core::ops::Add;

use soroban_sdk::{contract, contracterror, contractimpl, vec, Address, Env, Vec};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
// Define a new set of integer literals for the CombinedError enum
pub enum CombinedRouterError {
    RouterNotInitialized = 501,
    RouterNegativeNotAllowed = 502,
    RouterDeadlineExpired = 503,
    RouterInitializeAlreadyInitialized = 504,
    RouterInsufficientAAmount = 505,
    RouterInsufficientBAmount = 506,
    RouterInsufficientOutputAmount = 507,
    RouterExcessiveInputAmount = 508,
    RouterPairDoesNotExist = 509,

    LibraryInsufficientAmount = 510,
    LibraryInsufficientLiquidity = 511,
    LibraryInsufficientInputAmount = 512,
    LibraryInsufficientOutputAmount = 513,
    LibraryInvalidPath = 514,
    LibrarySortIdenticalTokens = 515,
}

#[contract]
pub struct MockSoroswapRouterContract;

#[contractimpl]
impl MockSoroswapRouterContract {
    fn initialize(_e: Env, _factory: Address) -> Result<(), CombinedRouterError> {
        Ok(())
    }

    fn add_liquidity(
        _e: Env,
        _token_a: Address,
        _token_b: Address,
        _amount_a_desired: i128,
        _amount_b_desired: i128,
        _amount_a_min: i128,
        _amount_b_min: i128,
        _to: Address,
        _deadline: u64,
    ) -> Result<(i128, i128, i128), CombinedRouterError> {
        unimplemented!()
    }

    fn remove_liquidity(
        _e: Env,
        _token_a: Address,
        _token_b: Address,
        _liquidity: i128,
        _amount_a_min: i128,
        _amount_b_min: i128,
        _to: Address,
        _deadline: u64,
    ) -> Result<(i128, i128), CombinedRouterError> {
        unimplemented!()
    }

    fn swap_exact_tokens_for_tokens(
        e: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        todo!()
    }

    fn swap_tokens_for_exact_tokens(
        e: Env,
        amount_out: i128,
        amount_in_max: i128,
        path: Vec<Address>,
        to: Address,
        deadline: u64,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        todo!()
    }

    fn get_factory(e: Env) -> Result<Address, CombinedRouterError> {
        unimplemented!()
    }

    fn router_pair_for(
        e: Env,
        token_a: Address,
        token_b: Address,
    ) -> Result<Address, CombinedRouterError> {
        todo!()
    }

    fn router_quote(
        amount_a: i128,
        reserve_a: i128,
        reserve_b: i128,
    ) -> Result<i128, CombinedRouterError> {
        unimplemented!()
    }

    fn router_get_amount_out(
        amount_in: i128,
        reserve_in: i128,
        reserve_out: i128,
    ) -> Result<i128, CombinedRouterError> {
        unimplemented!()
    }

    fn router_get_amount_in(
        amount_out: i128,
        reserve_in: i128,
        reserve_out: i128,
    ) -> Result<i128, CombinedRouterError> {
        unimplemented!()
    }

    fn router_get_amounts_out(
        e: Env,
        amount_in: i128,
        path: Vec<Address>,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        unimplemented!()
    }

    fn router_get_amounts_in(
        e: Env,
        amount_out: i128,
        path: Vec<Address>,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        unimplemented!()
    }
}
