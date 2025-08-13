#![no_std]
#![allow(clippy::too_many_arguments)]

use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_fixed_point_math::*;
use soroban_sdk::{
    Address, Env, Symbol, Vec, contract, contracterror, contractimpl, contracttype, symbol_short,
    token::{StellarAssetClient, TokenClient},
};

#[contracttype]
enum DataKey {
    TickerByAddress(Address),
}

const FEE_NUMERATOR: i128 = 997;
const FEE_DENOMINATOR: i128 = 1000;

const USDC_SYMBOL: Symbol = symbol_short!("USDC");

// TODO: In order to avoid circular dependency(between `lending` and `soroswap_router_mock`),
// for now, we define the oracle address in 2 places. Maybe, it's possible to define it in only one place
const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
// WARN: This is a plain copied enum and it is not synchronized with the deployed contract's errors.
// Likely, the Soroswap team will not break the backward compatibility, so this is relatively fine
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
    pub fn router_get_amounts_in(
        e: Env,
        amount_out: i128,
        path: Vec<Address>,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        process_get_amounts_in(&e, amount_out, &path)
    }

    pub fn router_get_amounts_out(
        e: Env,
        amount_in: i128,
        path: Vec<Address>,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        process_get_amounts_out(&e, amount_in, &path)
    }

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

        let amount_out = process_get_amounts_out(&e, amount_in, &path)
            .unwrap()
            .first()
            .unwrap();

        if amount_out < amount_out_min {
            return Err(CombinedRouterError::RouterInsufficientBAmount);
        }

        let burnt_token_address = path.first().unwrap(); // safe
        let minted_token_address = path.last().unwrap(); // safe

        let minted_sac_client = StellarAssetClient::new(&e, &minted_token_address);
        let burnt_token_client = TokenClient::new(&e, &burnt_token_address);

        minted_sac_client.mint(&to, &amount_out);
        burnt_token_client.burn(&to, &amount_in);

        Ok(soroban_sdk::vec![&e, amount_in, amount_out])
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

        let amount_in = process_get_amounts_in(&e, amount_out, &path)
            .unwrap()
            .first()
            .unwrap();

        if amount_in > amount_in_max {
            return Err(CombinedRouterError::RouterInsufficientAAmount);
        }

        let burnt_token_address = path.first().unwrap(); // safe
        let minted_token_address = path.last().unwrap(); // safe

        let minted_sac_client = StellarAssetClient::new(&e, &minted_token_address);
        let burnt_token_client = TokenClient::new(&e, &burnt_token_address);

        minted_sac_client.mint(&to, &amount_out);
        burnt_token_client.burn(&to, &amount_in);

        Ok(soroban_sdk::vec![&e, amount_in, amount_out])
    }

    pub fn map_address_to_ticker(e: Env, ticker: Symbol, address: Address) {
        e.storage()
            .instance()
            .set(&DataKey::TickerByAddress(address), &ticker);
    }
}

fn process_get_amounts_in(
    e: &Env,
    amount_out: i128,
    path: &Vec<Address>,
) -> Result<Vec<i128>, CombinedRouterError> {
    if path.len() < 2 {
        return Err(CombinedRouterError::LibraryInvalidPath);
    }

    let first_ticker = get_ticker_by_address(&e, &path.first().unwrap()).unwrap();
    let last_ticker = get_ticker_by_address(&e, &path.last().unwrap()).unwrap();

    let oracle_address = Address::from_str(&e, ORACLE_ADDRESS);
    let oracle_contract = PriceFeedClient::new(&e, &oracle_address);
    let decimals = oracle_contract.decimals();

    let amount_in = if first_ticker == USDC_SYMBOL {
        // USDC is the asset that is given
        let price = oracle_contract
            .lastprice(&Asset::Other(last_ticker.clone()))
            .unwrap()
            .price;

        let value = amount_out.checked_mul(price).unwrap();
        let amount_in = value.checked_div(i128::pow(10, decimals)).unwrap();
        let amount_in_plus_fees = amount_in
            .fixed_mul_floor(FEE_DENOMINATOR, FEE_NUMERATOR)
            .unwrap();

        amount_in_plus_fees
    } else if last_ticker == USDC_SYMBOL {
        // USDC is the asset that is received
        let price = oracle_contract
            .lastprice(&Asset::Other(first_ticker.clone()))
            .unwrap()
            .price;

        let amount_out_scaled = amount_out.checked_mul(i128::pow(10, decimals)).unwrap();
        let amount_out = amount_out_scaled.checked_div(price).unwrap();
        let amount_out_plus_fees = amount_out
            .fixed_mul_floor(FEE_DENOMINATOR, FEE_NUMERATOR)
            .unwrap();

        amount_out_plus_fees
    } else {
        // Mocked router supports only pairs with USDC
        return Err(CombinedRouterError::RouterPairDoesNotExist);
    };

    Ok(soroban_sdk::vec![&e, amount_in, amount_out])
}

fn process_get_amounts_out(
    e: &Env,
    amount_in: i128,
    path: &Vec<Address>,
) -> Result<Vec<i128>, CombinedRouterError> {
    if path.len() < 2 {
        return Err(CombinedRouterError::LibraryInvalidPath);
    }

    let first_ticker = get_ticker_by_address(&e, &path.first().unwrap()).unwrap();
    let last_ticker = get_ticker_by_address(&e, &path.last().unwrap()).unwrap();

    let oracle_address = Address::from_str(&e, ORACLE_ADDRESS);
    let oracle_contract = PriceFeedClient::new(&e, &oracle_address);
    let decimals = oracle_contract.decimals();

    let amount_out = if first_ticker == USDC_SYMBOL {
        // USDC is the asset that is given
        let price = oracle_contract
            .lastprice(&Asset::Other(last_ticker.clone()))
            .unwrap()
            .price;

        let amount_in_minus_fees = amount_in
            .fixed_mul_floor(FEE_NUMERATOR, FEE_DENOMINATOR)
            .unwrap();
        let amount_in_minus_fees_scaled = amount_in_minus_fees
            .checked_mul(i128::pow(10, decimals))
            .unwrap();

        let amount_out = amount_in_minus_fees_scaled.checked_div(price).unwrap();

        amount_out
    } else if last_ticker == USDC_SYMBOL {
        // USDC is the asset that is received
        let amount_in_minus_fees = amount_in
            .fixed_mul_floor(FEE_NUMERATOR, FEE_DENOMINATOR)
            .unwrap();

        let price = oracle_contract
            .lastprice(&Asset::Other(first_ticker.clone()))
            .unwrap()
            .price;
        let value = amount_in_minus_fees.checked_mul(price).unwrap();
        let amount_out = value.checked_div(i128::pow(10, decimals)).unwrap();

        amount_out
    } else {
        // Mocked router supports only pairs with USDC
        return Err(CombinedRouterError::RouterPairDoesNotExist);
    };

    Ok(soroban_sdk::vec![&e, amount_in, amount_out])
}

fn get_ticker_by_address(e: &Env, address: &Address) -> Option<Symbol> {
    e.storage()
        .instance()
        .get(&DataKey::TickerByAddress(address.clone()))
}
