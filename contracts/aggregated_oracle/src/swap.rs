use sep_40_oracle::Asset;
use soroban_sdk::{panic_with_error, Address, Env};

use crate::{
    constants::{DECIMALS, ROUTER_ADDRESS, USDC_SAC_ADDRESS},
    error::AOCError,
    storage,
};

mod router {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/soroswap-router.wasm");
}

mod pair {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/soroswap-pair.wasm");
}

pub fn get_price(e: &Env, asset: &Asset) -> Option<i128> {
    let token_address = get_token_address(e, asset)?;
    let usdc_sac_address = Address::from_str(e, USDC_SAC_ADDRESS);

    let (reserve_0, reserve_1) =
        get_reserves(e, &Address::from_str(e, USDC_SAC_ADDRESS), &token_address);

    let price = if token_address == usdc_sac_address {
        i128::pow(10, DECIMALS)
    } else {
        // See: https://github.com/soroswap/core/blob/main/contracts/library/src/tokens.rs#L37
        let (token_reserve, usdc_reserve) = if token_address < usdc_sac_address {
            (reserve_0, reserve_1)
        } else {
            (reserve_1, reserve_0)
        };

        let token_reserve_scaled = token_reserve
            .checked_mul(i128::pow(10, DECIMALS))
            .unwrap_or_else(|| panic_with_error!(e, AOCError::OverOrUnderflow));

        // 'price' = reserve_x / reserve_y
        let price = token_reserve_scaled
            .checked_div(usdc_reserve)
            .unwrap_or_else(|| panic_with_error!(e, AOCError::OverOrUnderflow));

        price
    };

    Some(price)
}

/// # Returns
/// `(i128, i128)` reserves in the liquidity pool, ordered correspondingly to the order of their
/// addresses. See: https://docs.rs/soroban-sdk/latest/src/soroban_sdk/address.rs.html#81
fn get_reserves(e: &Env, token_a: &Address, token_b: &Address) -> (i128, i128) {
    let router_client = router::Client::new(e, &Address::from_str(e, ROUTER_ADDRESS));
    let pair_contract_address = router_client.router_pair_for(token_a, token_b);

    let pair_client = pair::Client::new(e, &pair_contract_address);

    pair_client.get_reserves()
}

fn get_token_address(e: &Env, asset: &Asset) -> Option<Address> {
    storage::get_token_address(e, asset)
}
