#![no_std]
use soroban_sdk::{contract, contractimpl, token, Address, Env, Symbol};

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn sends_to_contract(e: Env, amount: i128, user: Address, token_address: Address) {
        let timestamp = e.ledger().timestamp();

        let topics = (Symbol::new(&e, "timestamp"),);
        let data = (timestamp,);

        e.events().publish(topics, data);

        let requires_sent_back = timestamp % 2 != 0;

        user.require_auth();

        token::TokenClient::new(&e, &token_address).transfer(
            &user,
            &e.current_contract_address(),
            &(amount + 1),
        );

        let sent_back = if requires_sent_back { 1 } else { 0 };

        token::TokenClient::new(&e, &token_address).transfer(
            &e.current_contract_address(),
            &user,
            &sent_back,
        );
    }

    pub fn change(e: Env, amount: i128) {
        e.storage().instance().set(&"INNER_CAP", &amount);
    }
}

mod test;
