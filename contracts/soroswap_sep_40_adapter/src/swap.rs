#![allow(clippy::too_many_arguments)] // Omitting Soroswap's clippy warnings

use soroban_sdk::{Address, Env, panic_with_error};

use crate::{constants::*, error::SS40ACError};

pub fn get_price(e: &Env, token_address: Address) -> Option<i128> {
    let usdc_sac_address = Address::from_str(e, USDC_SAC_ADDRESS);

    let price = if token_address == usdc_sac_address {
        PRICE_SCALING_FACTOR // '1' scaled
    } else {
        let (reserve_0, reserve_1) =
            get_reserves(e, &Address::from_str(e, USDC_SAC_ADDRESS), &token_address);

        // See: https://github.com/soroswap/core/blob/main/contracts/library/src/tokens.rs#L37
        let (token_reserve, usdc_reserve) = if token_address < usdc_sac_address {
            (reserve_0, reserve_1)
        } else {
            (reserve_1, reserve_0)
        };

        let usdc_reserve_scaled = usdc_reserve
            .checked_mul(PRICE_SCALING_FACTOR)
            .unwrap_or_else(|| panic_with_error!(e, SS40ACError::OverOrUnderflow));

        // 'price_y' = (reserve_x / reserve_y)
        usdc_reserve_scaled
            .checked_div(token_reserve)
            .unwrap_or_else(|| panic_with_error!(e, SS40ACError::OverOrUnderflow))
    };

    Some(price)
}

// ---- Helpers ----

mod router {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/soroswap-router.wasm");
}

mod pair {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/soroswap-pair.wasm");
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
