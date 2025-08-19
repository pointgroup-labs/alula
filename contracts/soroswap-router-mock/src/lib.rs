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
// for now, we define the oracle address in 2 places. Maybe it's possible to define it in one place
// onlyMaps a randomly generated token address to a token ticker. This is required to use a `sep-40`
// compliant oracle to get the swap price
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
        get_amounts_in(&e, amount_out, &path)
    }

    pub fn router_get_amounts_out(
        e: Env,
        amount_in: i128,
        path: Vec<Address>,
    ) -> Result<Vec<i128>, CombinedRouterError> {
        get_amounts_out(&e, amount_in, &path)
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

        let amount_out = get_amount_out(&e, amount_in, &path)?;
        if amount_out < amount_out_min {
            return Err(CombinedRouterError::RouterInsufficientOutputAmount);
        }

        burn_and_mint_tokens(
            &e,
            &path.first().unwrap(),
            &path.last().unwrap(),
            amount_in,
            amount_out,
            &to,
        );

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

        let amount_in = get_amount_in(&e, amount_out, &path)?;
        if amount_in > amount_in_max {
            return Err(CombinedRouterError::RouterExcessiveInputAmount);
        }

        burn_and_mint_tokens(
            &e,
            &path.first().unwrap(),
            &path.last().unwrap(),
            amount_in,
            amount_out,
            &to,
        );

        Ok(soroban_sdk::vec![&e, amount_in, amount_out])
    }

    /// Maps a randomly generated token address to a token ticker. This is required to use a
    /// `sep-40` compliant oracle as a price-source in mocked router
    pub fn map_address_to_ticker(e: Env, address: Address, ticker: Symbol) {
        e.storage()
            .instance()
            .set(&DataKey::TickerByAddress(address), &ticker);
    }
}

fn burn_and_mint_tokens(
    e: &Env,
    burnt_token: &Address,
    minted_token: &Address,
    burnt_amount: i128,
    minted_amount: i128,
    to: &Address,
) {
    let minted_sac_client = StellarAssetClient::new(&e, minted_token);
    let burnt_token_client = TokenClient::new(&e, burnt_token);

    minted_sac_client.mint(to, &minted_amount);
    burnt_token_client.burn(to, &burnt_amount);
}

fn get_amount_in(
    e: &Env,
    amount_out: i128,
    path: &Vec<Address>,
) -> Result<i128, CombinedRouterError> {
    let amounts_in = get_amounts_in(e, amount_out, path)?;

    Ok(amounts_in.first().unwrap())
}

fn get_amount_out(
    e: &Env,
    amount_in: i128,
    path: &Vec<Address>,
) -> Result<i128, CombinedRouterError> {
    let amounts_out = get_amounts_out(e, amount_in, path)?;

    Ok(amounts_out.last().unwrap())
}

fn get_amounts_in(
    e: &Env,
    amount_out: i128,
    path: &Vec<Address>,
) -> Result<Vec<i128>, CombinedRouterError> {
    if path.len() < 2 {
        return Err(CombinedRouterError::LibraryInvalidPath);
    }

    let (first_ticker, last_ticker) = get_end_tickers_from_path(e, path);

    let oracle_address = Address::from_str(&e, ORACLE_ADDRESS);
    let oracle_contract = PriceFeedClient::new(&e, &oracle_address);
    let decimals = oracle_contract.decimals();

    let price_scaling_factor = i128::pow(10, decimals);
    let usdc_as_token_in = first_ticker == USDC_SYMBOL;
    let usdc_as_token_out = last_ticker == USDC_SYMBOL;

    let amount_in = if usdc_as_token_in {
        let price = oracle_contract
            .lastprice(&Asset::Other(last_ticker.clone()))
            .unwrap()
            .price;

        let value = amount_out.checked_mul(price).unwrap();
        let amount_in = value.checked_div(price_scaling_factor).unwrap();

        amount_in
    } else if usdc_as_token_out {
        let price = oracle_contract
            .lastprice(&Asset::Other(first_ticker.clone()))
            .unwrap()
            .price;

        let amount_out_scaled = amount_out.checked_mul(price_scaling_factor).unwrap();
        let amount_in = amount_out_scaled.checked_div(price).unwrap();

        amount_in
    } else {
        // Mocked router supports only pairs with USDC
        return Err(CombinedRouterError::RouterPairDoesNotExist);
    };

    // 'amount_in_plus_fees' = amount_in * (1000/997)
    let amount_in_plus_fees = amount_in
        .fixed_mul_ceil(FEE_DENOMINATOR, FEE_NUMERATOR)
        .unwrap();

    Ok(soroban_sdk::vec![&e, amount_in_plus_fees, amount_out])
}

fn get_amounts_out(
    e: &Env,
    amount_in: i128,
    path: &Vec<Address>,
) -> Result<Vec<i128>, CombinedRouterError> {
    if path.len() < 2 {
        return Err(CombinedRouterError::LibraryInvalidPath);
    }

    let (first_ticker, last_ticker) = get_end_tickers_from_path(e, path);

    let oracle_address = Address::from_str(&e, ORACLE_ADDRESS);
    let oracle_contract = PriceFeedClient::new(&e, &oracle_address);
    let decimals = oracle_contract.decimals();

    // 'amount_in_minus_fees' = amount_in * (997/1000)
    let amount_in_minus_fees = amount_in
        .fixed_mul_floor(FEE_NUMERATOR, FEE_DENOMINATOR)
        .unwrap();

    let price_scaling_factor = i128::pow(10, decimals);
    let usdc_as_token_in = first_ticker == USDC_SYMBOL;
    let usdc_as_token_out = last_ticker == USDC_SYMBOL;

    let amount_out = if usdc_as_token_in {
        let price = oracle_contract
            .lastprice(&Asset::Other(last_ticker.clone()))
            .unwrap()
            .price;

        let amount_in_scaled = amount_in_minus_fees
            .checked_mul(price_scaling_factor)
            .unwrap();
        let amount_out = amount_in_scaled.checked_div(price).unwrap();

        amount_out
    } else if usdc_as_token_out {
        let price = oracle_contract
            .lastprice(&Asset::Other(first_ticker.clone()))
            .unwrap()
            .price;

        let value = amount_in_minus_fees.checked_mul(price).unwrap();
        let amount_out = value.checked_div(price_scaling_factor).unwrap();

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

fn get_end_tickers_from_path(e: &Env, path: &Vec<Address>) -> (Symbol, Symbol) {
    let first_ticker = get_ticker_by_address(&e, &path.first().unwrap()).unwrap();
    let last_ticker = get_ticker_by_address(&e, &path.last().unwrap()).unwrap();

    (first_ticker, last_ticker)
}
