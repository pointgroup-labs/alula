#![no_std]

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, contract, contractimpl, vec};

use crate::market::{Request, StandardRequest, SwapExactTokensRequest};

mod market {
    use soroban_sdk::contractimport;

    contractimport!(file = "../../wasms/deploy_optimized/market.optimized.wasm");
}

mod router {
    #![allow(clippy::too_many_arguments)]
    use soroban_sdk::contractimport;

    contractimport!(file = "../../wasms/downloads/soroswap-router.wasm");
}

const XLM_ADDR: &str = "CDLZFC3SYJYDZT7K67VZ75HPJVIEUVNIXF47ZG2FB2RMQQVU2HHGCYSC";
const USDC_ADDR: &str = "CBIELTK6YBZJU5UP2WWQEUCYKLPU6AUNZ2BQ4WWFEIE3USCIHMXQDAMA";

const MARKET_ADDR: &str = "CDKKJYAG6TLTCBXK77ZUZLSJ2VNJ65B4WL7NFMH4KNKS2WRKXXT4Y7IB";
const ROUTER_ADDR: &str = "CCJUD55AG6W5HAI5LRVNKAE5WDP5XGZBUDS5WNTIVDU7O264UZZE7BRD";

const LEVERAGE_SCALE: i128 = 100;

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn multiply_borrow_as_margin_poc(
        e: &Env,
        user: &Address,
        amount: i128,
        leverage_multiplier: i128,
    ) {
        user.require_auth();

        let (xlm, usdc, market) = (
            Address::from_str(e, XLM_ADDR),
            Address::from_str(e, USDC_ADDR),
            Address::from_str(e, MARKET_ADDR),
        );

        let market_client = market::Client::new(e, &market);
        let leverage_multiplier_minus_1 = leverage_multiplier.checked_sub(LEVERAGE_SCALE).unwrap();

        let flash_borrow_amount =
            amount.fixed_mul_floor(leverage_multiplier_minus_1, LEVERAGE_SCALE).unwrap();

        // NB: Can check slippage here
        let amount_to_deposit = get_amount_out(e, &usdc, &xlm, amount + flash_borrow_amount) - 10;

        let requests = soroban_sdk::vec![
            e,
            Request::FlashBorrow(StandardRequest {
                amount: flash_borrow_amount,
                pool_address: usdc.clone(),
            }),
            Request::SwapExactTokens(SwapExactTokensRequest {
                token_in: usdc.clone(),
                token_out: xlm.clone(),
                amount_in: amount + flash_borrow_amount,
                min_amount_out: 1,
            }),
            Request::Deposit(StandardRequest {
                amount: amount_to_deposit,
                pool_address: xlm.clone(),
            }),
            Request::Borrow(StandardRequest { amount: flash_borrow_amount, pool_address: usdc }),
        ];

        market_client.submit_requests_batch(user, &None, &requests, &None);
    }
}

pub fn get_amount_out(e: &Env, token_in: &Address, token_out: &Address, amount_in: i128) -> i128 {
    let path = vec![&e, token_in.clone(), token_out.clone()];
    let router_client = router::Client::new(e, &Address::from_str(e, ROUTER_ADDR));

    let amounts_out = router_client.router_get_amounts_out(&amount_in, &path);

    amounts_out.last().unwrap()
}
