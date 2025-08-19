use soroban_sdk::{Address, Env};

pub const ROUTER_ADDRESS: &str = "CCMAPXWVZD4USEKDWRYS7DA4Y3D7E2SDMGBFJUCEXTC7VN6CUBGWPFUS";
pub const USDC_SAC_ADDRESS: &str = "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA";
pub const XLM_SAC_ADDRESS: &str = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC";

mod router {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/soroswap-router.wasm");
}

mod pair {
    use soroban_sdk::contractimport;
    contractimport!(file = "../../wasms/downloads/soroswap-pair.wasm");
}

pub fn get_price(e: &Env, token: &Address) -> i128 {
    // let

    // let (reserve_0, reserve_1) =
    todo!()
}

/// # Returns
/// `(i128, i128)` reserves in the liquidity pool, ordered correspondingly to the order of their addresses.
/// See: <https://docs.rs/soroban-sdk/latest/src/soroban_sdk/address.rs.html#81>
fn get_reserves(e: &Env, token_a: &Address, token_b: &Address) -> (i128, i128) {
    let router_client = router::Client::new(e, &Address::from_str(e, ROUTER_ADDRESS));
    let pair_contract_address = router_client.router_pair_for(token_a, token_b);

    let pair_client = pair::Client::new(e, &pair_contract_address);

    pair_client.get_reserves()
}
