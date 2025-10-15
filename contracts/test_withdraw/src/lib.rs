#![no_std]

use soroban_sdk::{Address, Env, contract, contractimpl, token};

#[contract]
pub struct Contract;

// This is a sample contract. Replace this placeholder with your own contract logic.
// A corresponding test example is available in `test.rs`.
//
// For comprehensive examples, visit <https://github.com/stellar/soroban-examples>.
// The repository includes use cases for the Stellar ecosystem, such as data storage on
// the blockchain, token swaps, liquidity pools, and more.
//
// Refer to the official documentation:
// <https://developers.stellar.org/docs/build/smart-contracts/overview>.
#[contractimpl]
impl Contract {
    pub fn transfers_more(e: Env, amount: i128, token: Address, user: Address) {
        user.require_auth();
        let timestamp = e.ledger().timestamp();

        let client = token::Client::new(&e, &token);
        let amount = if !timestamp.is_multiple_of(2) { amount + 1 } else { amount };

        client.transfer(&user, e.current_contract_address(), &amount);
    }
}
