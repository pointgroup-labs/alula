#![no_std]
#![allow(clippy::too_many_arguments)]

use soroban_sdk::{
    contract, contracterror, contractimpl,
    token::{StellarAssetClient, TokenClient},
    Address, Env, Vec,
};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
// WARN: This is a plain copied enum and it is not synchronized with a network contract's errors
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
        e: Env,
        amount_out: i128,
        amount_in_max: i128,
        path: Vec<Address>,
        to: Address,
        _deadline: u64,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        to.require_auth();

        if path.len() < 2 {
            return Err(CombinedRouterError::LibraryInvalidPath);
        }

        let burnt_token_address = path.first().unwrap(); // safe
        let minted_token_address = path.last().unwrap(); // safe

        let minted_sac_client = StellarAssetClient::new(&e, &minted_token_address);
        let burnt_token_client = TokenClient::new(&e, &burnt_token_address);

        minted_sac_client.mint(&to, &amount_out);
        burnt_token_client.burn(&to, &amount_in_max);

        Ok(soroban_sdk::vec![&e, amount_in_max, amount_out])
    }
}
