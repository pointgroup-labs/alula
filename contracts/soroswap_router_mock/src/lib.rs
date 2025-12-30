#![no_std]
#![allow(clippy::too_many_arguments)]

use sep_40_oracle::{Asset, PriceFeedClient};
use soroban_fixed_point_math::*;
use soroban_sdk::{
    Address, Env, Vec, contract, contracterror, contractimpl, contracttype,
    token::{StellarAssetClient, TokenClient},
};

#[contracttype]
enum DataKey {
    BaseAssetTokenAddress,
}

const FEE_NUMERATOR: i128 = 997;
const FEE_DENOMINATOR: i128 = 1000;

// Mock Router relies on the locally deployed Oracle Contract for prices.
// In this way, swapping stays consistent with Oracle prices in integration tests
const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
// NB: This is a plain copied enum, and its errors are not in sync with those in the deployed contract.
// Likely, the Soroswap team will not break the backward compatibility, so this is fine
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
    pub fn __constructor(e: Env, base_asset_token_address: Address) {
        e.storage().instance().set(&DataKey::BaseAssetTokenAddress, &base_asset_token_address);
    }

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

    pub fn router_pair_for(
        _e: Env,
        token_a: Address,
        _token_b: Address,
    ) -> Result<Address, CombinedRouterError> {
        Ok(token_a)
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
    let sac_client = StellarAssetClient::new(e, minted_token);
    let token_client = TokenClient::new(e, burnt_token);

    sac_client.mint(to, &minted_amount);
    token_client.burn(to, &burnt_amount);
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

    let (first_address, last_address) = get_end_addresses_from_path(path);

    let oracle_address = Address::from_str(e, ORACLE_ADDRESS);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);

    let decimals = oracle_contract.decimals();
    let price_scaling_factor = i128::pow(10, decimals);

    let usdc_sac_address = e.storage().instance().get(&DataKey::BaseAssetTokenAddress).unwrap();

    let usdc_as_token_in = first_address == usdc_sac_address;
    let usdc_as_token_out = last_address == usdc_sac_address;

    let amount_in = if usdc_as_token_in {
        // Case 1: USDC -> Token (Calculating A_in in USDC)
        // A_in = (A_out * Price) / Price_Scaling_Factor
        let price = oracle_contract.lastprice(&Asset::Stellar(last_address.clone())).unwrap().price;

        let numerator = amount_out.checked_mul(price).unwrap();
        let denominator = price_scaling_factor;

        // Apply Ceiling: (numerator + denominator - 1) / denominator
        let numerator_plus_denominator = numerator.checked_add(denominator).unwrap();
        let numerator_minus_one = numerator_plus_denominator.checked_sub(1).unwrap();

        numerator_minus_one.checked_div(denominator).unwrap()
    } else if usdc_as_token_out {
        // Case 2: Token -> USDC (Calculating A_in in Token)
        // A_in = (A_out * Price_Scaling_Factor) / Price
        let price =
            oracle_contract.lastprice(&Asset::Stellar(first_address.clone())).unwrap().price;

        let numerator = amount_out.checked_mul(price_scaling_factor).unwrap();
        let denominator = price;

        // Apply Ceiling: (numerator + denominator - 1) / denominator
        let numerator_plus_denominator = numerator.checked_add(denominator).unwrap();
        let numerator_minus_one = numerator_plus_denominator.checked_sub(1).unwrap();

        numerator_minus_one.checked_div(denominator).unwrap()
    } else {
        // Mocked router supports only pairs with USDC
        return Err(CombinedRouterError::RouterPairDoesNotExist);
    };

    // 'amount_in_plus_fees' = amount_in * (1000/997)
    let amount_in_plus_fees = amount_in.fixed_mul_ceil(FEE_DENOMINATOR, FEE_NUMERATOR).unwrap();

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

    let (first_address, last_address) = get_end_addresses_from_path(path);

    let oracle_address = Address::from_str(e, ORACLE_ADDRESS);
    let oracle_contract = PriceFeedClient::new(e, &oracle_address);
    let decimals = oracle_contract.decimals();

    // 'amount_in_minus_fees' = amount_in * (997/1000)
    let amount_in_minus_fees = amount_in.fixed_mul_floor(FEE_NUMERATOR, FEE_DENOMINATOR).unwrap();

    let price_scaling_factor = i128::pow(10, decimals);
    let usdc_sac_address = e.storage().instance().get(&DataKey::BaseAssetTokenAddress).unwrap();
    let usdc_as_token_in = first_address == usdc_sac_address;
    let usdc_as_token_out = last_address == usdc_sac_address;

    let amount_out = if usdc_as_token_in {
        let price = oracle_contract.lastprice(&Asset::Stellar(last_address.clone())).unwrap().price;
        let amount_in_scaled = amount_in_minus_fees.checked_mul(price_scaling_factor).unwrap();

        amount_in_scaled.checked_div(price).unwrap()
    } else if usdc_as_token_out {
        let price =
            oracle_contract.lastprice(&Asset::Stellar(first_address.clone())).unwrap().price;
        let value = amount_in_minus_fees.checked_mul(price).unwrap();

        value.checked_div(price_scaling_factor).unwrap()
    } else {
        // Mocked router supports only pairs with USDC
        return Err(CombinedRouterError::RouterPairDoesNotExist);
    };

    Ok(soroban_sdk::vec![&e, amount_in, amount_out])
}

fn get_end_addresses_from_path(path: &Vec<Address>) -> (Address, Address) {
    let last_address = path.last().unwrap();
    let first_address = path.first().unwrap();

    (first_address, last_address)
}
