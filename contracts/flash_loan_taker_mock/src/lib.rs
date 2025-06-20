#![no_std]

use {
    lending::contract::LendingContractClient,
    moderc3156::ModErc3156,
    soroban_sdk::{
        contract, contractimpl, contracttype,
        token::{StellarAssetClient, TokenClient},
        Address, Env,
    },
};

#[contracttype]
enum DataKey {
    Liquidatable,
}

#[contracttype]
struct Liquidatable {
    borrower: Address,
    collateral_pool_address: Address,
}

#[contract]
pub struct Contract;

#[contractimpl]
impl Contract {
    pub fn register_liquidatable(e: Env, borrower: Address, collateral_pool_address: Address) {
        e.storage().instance().set(
            &DataKey::Liquidatable,
            &Liquidatable {
                borrower,
                collateral_pool_address,
            },
        );
    }
}

#[contractimpl]
impl moderc3156::ModErc3156 for Contract {
    fn exec_op(e: Env, caller: Address, token: Address, amount: i128, fee: i128) {
        caller.require_auth();

        let Liquidatable {
            borrower,
            collateral_pool_address,
            ..
        } = e.storage().instance().get(&DataKey::Liquidatable).unwrap();

        let flash_loan_token_client = TokenClient::new(&e, &token);
        let flash_loan_received = flash_loan_token_client.balance(&e.current_contract_address());
        assert_eq!(flash_loan_received, amount);

        let collateral_token_client = TokenClient::new(&e, &collateral_pool_address);
        let collateral_received = collateral_token_client.balance(&e.current_contract_address());
        assert_eq!(collateral_received, 0);

        let lending_contract_client = LendingContractClient::new(&e, &caller);
        lending_contract_client.liquidate(
            &e.current_contract_address(),
            &borrower,
            &token,
            &collateral_pool_address,
            &amount,
        );

        let flash_loan_balance = flash_loan_token_client.balance(&e.current_contract_address());
        assert_eq!(
            flash_loan_balance, 0,
            "Liquidation must use all of the loaned token balance"
        );

        let collateral_received = collateral_token_client.balance(&e.current_contract_address());
        assert!(collateral_received > amount, "
            With 1:1 simulated price rate, the liquidator must receive more than the repaid amount due to the liquidation spread
        ");

        simulate_swap(&e, &token, &collateral_pool_address, collateral_received);

        let collateral_balance = collateral_token_client.balance(&e.current_contract_address());
        assert_eq!(collateral_balance, 0);

        let flash_loan_balance = flash_loan_token_client.balance(&e.current_contract_address());
        assert!(
            flash_loan_balance >= (amount + fee),
            "Liquidation profit must exceed flash loan fees"
        );
    }
}

/// Simulates 1:1 token swap
fn simulate_swap(e: &Env, token_bought: &Address, token_sold: &Address, amount_sold: i128) {
    let sac_client = StellarAssetClient::new(e, token_bought);
    sac_client.mint(&e.current_contract_address(), &amount_sold);

    let token_client = TokenClient::new(e, token_sold);
    token_client.burn(&e.current_contract_address(), &amount_sold);
}
