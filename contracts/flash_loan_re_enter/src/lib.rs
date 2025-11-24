#![no_std]
use market::contract::MarketClient;
use moderc3156::ModErc3156;
use soroban_sdk::{Address, Env, contract, contractimpl, contracttype, token};

#[contracttype]
enum DataKey {
    Caller,
    MarketContractAddress,
    BorrowPoolAddress,
}

#[contract]
pub struct Contract;

impl Contract {
    fn __constructor(e: Env, market_contract_address: Address, caller: Address, bpa: Address) {
        e.storage().instance().set(&DataKey::Caller, &caller);
        e.storage().instance().set(&DataKey::BorrowPoolAddress, &bpa);
        e.storage().instance().set(&DataKey::MarketContractAddress, &market_contract_address);
    }
}

#[contractimpl]
impl ModErc3156 for Contract {
    fn exec_op(e: Env, caller: Address, token: Address, amount: i128, fee_bps: i128) {
        let caller: Address = e.storage().instance().get(&DataKey::Caller).unwrap();
        let bpa: Address = e.storage().instance().get(&DataKey::BorrowPoolAddress).unwrap();
        let market_contract_id: Address =
            e.storage().instance().get(&DataKey::MarketContractAddress).unwrap();

        caller.require_auth();

        let contract_client = MarketClient::new(&e, &market_contract_id);
        contract_client.borrow(&caller, &bpa, &1);

        let token_client = token::Client::new(&e, &token);
        token_client.approve(
            &e.current_contract_address(),
            &caller,
            &(amount + fee_bps),
            &e.ledger().sequence(),
        );
    }
}
