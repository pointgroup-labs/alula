#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contracterror, contractimpl,
    token::{StellarAssetClient, TokenClient},
    Address, Env, Vec,
};

const SOROSWAP_FACTORY_TESTNET_ADDRESS: &str =
    "CB7X4DSYW4UTKJSJMO7A3ZX2YQQG4NQUD3TQOTAZ7UHOK2BGGLRW2ZIC";

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
    pub fn initialize(_e: Env, _factory: Address) -> Result<(), CombinedRouterError> {
        unimplemented!()
    }

    pub fn add_liquidity(
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

    pub fn remove_liquidity(
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

    // For now we assume 1:1 swap rate
    pub fn swap_exact_tokens_for_tokens(
        e: Env,
        amount_in: i128,
        amount_out_min: i128,
        path: Vec<Address>,
        to: Address,
        _deadline: u64,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        to.require_auth();

        if amount_out_min > amount_in {
            return Err(CombinedRouterError::LibraryInsufficientAmount);
        }

        if path.len() < 2 {
            return Err(CombinedRouterError::LibraryInvalidPath);
        }

        let burnt_token_address = path.first().unwrap(); // safe
        let minted_token_address = path.last().unwrap(); // safe

        let minted_sac_client = StellarAssetClient::new(&e, &minted_token_address);
        let burnt_token_client = TokenClient::new(&e, &burnt_token_address);

        minted_sac_client.mint(&to, &amount_out_min);
        burnt_token_client.burn(&to, &amount_in);

        Ok(soroban_sdk::vec![&e, amount_in, amount_out_min])
    }

    pub fn swap_tokens_for_exact_tokens(
        _e: Env,
        _amount_out: i128,
        _amount_in_max: i128,
        _path: Vec<Address>,
        _to: Address,
        _deadline: u64,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        unimplemented!()
    }

    pub fn get_factory(e: Env) -> Result<Address, CombinedRouterError> {
        Ok(Address::from_str(&e, SOROSWAP_FACTORY_TESTNET_ADDRESS))
    }

    pub fn router_pair_for(
        _e: Env,
        _token_a: Address,
        _token_b: Address,
    ) -> Result<Address, CombinedRouterError> {
        unimplemented!()
    }

    pub fn router_quote(
        _amount_a: i128,
        _reserve_a: i128,
        _reserve_b: i128,
    ) -> Result<i128, CombinedRouterError> {
        unimplemented!()
    }

    pub fn router_get_amount_out(
        amount_in: i128,
        _reserve_in: i128,
        _reserve_out: i128,
    ) -> Result<i128, CombinedRouterError> {
        Ok(amount_in)
    }

    pub fn router_get_amount_in(
        _amount_out: i128,
        _reserve_in: i128,
        _reserve_out: i128,
    ) -> Result<i128, CombinedRouterError> {
        unimplemented!()
    }

    pub fn router_get_amounts_out(
        _e: Env,
        _amount_in: i128,
        _path: Vec<Address>,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        unimplemented!()
    }

    pub fn router_get_amounts_in(
        _e: Env,
        _amount_out: i128,
        _path: Vec<Address>,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        unimplemented!()
    }
}
