#![no_std]

use {
    moderc3156::ModErc3156,
    soroban_sdk::{
        contract, contractimpl, contracttype,
        token::{StellarAssetClient, TokenClient},
        Address, Env,
    },
};

const FAILING_CALL_AMOUNT: i128 = 777;

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
pub struct FlashLoanLiquidatorContract;
#[contractimpl]

impl moderc3156::ModErc3156 for FlashLoanLiquidatorContract {
    fn exec_op(e: Env, caller: Address, token: Address, amount: i128, _fee: i128) {
        caller.require_auth();

        let flash_loan_token_client = TokenClient::new(&e, &token);
        let flash_loan_received = flash_loan_token_client.balance(&e.current_contract_address());
        assert_eq!(flash_loan_received, amount);

        if amount == FAILING_CALL_AMOUNT {
            simulate_failed_strategy(&e, &token, amount);
        } else {
            simulate_successful_strategy(&e, &token, amount);
        }
    }
}

/// Simulates a successful strategy that earns 10% on top of the flash loan
fn simulate_successful_strategy(e: &Env, token_address: &Address, amount: i128) {
    let sac_client = StellarAssetClient::new(e, token_address);
    sac_client.mint(&e.current_contract_address(), &(amount / 10));
}

/// Simulates a failed strategy that burns 10% of the flash loan
fn simulate_failed_strategy(e: &Env, token_address: &Address, amount: i128) {
    let token_client = TokenClient::new(e, token_address);
    token_client.burn(&e.current_contract_address(), &(amount / 10));
}

#[cfg(test)]
mod test {
    use {
        super::FlashLoanLiquidatorContract,
        crate::FAILING_CALL_AMOUNT,
        lending::constants::LCError,
        soroban_sdk::Address,
        tests::{TestFixture, DEFAULT_DEPOSIT_AMOUNT},
    };

    #[test]
    fn test_flash_loan_success() {
        let TestFixture {
            e,
            contract_client: lending_contract_client,
            gold_pool_address,
            usdc_pool_address,
            users,
            ..
        } = TestFixture::new();

        let flash_loan_taker_contract_address = e.register(FlashLoanLiquidatorContract, ());

        let user: Address = users.get(0).unwrap();
        let user2 = users.get(1).unwrap();

        // Deposit gold to satisfy the health factor threshold
        lending_contract_client.deposit(&user, &gold_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));
        // Deposit usdc as another user to have a non-empty loan pool
        lending_contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

        lending_contract_client.flash_loan(
            &flash_loan_taker_contract_address,
            &usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );
    }

    #[test]
    fn test_flash_loan_failure() {
        let TestFixture {
            e,
            contract_client: lending_contract_client,
            gold_pool_address,
            usdc_pool_address,
            users,
            ..
        } = TestFixture::new();

        let flash_loan_taker_contract_address = e.register(FlashLoanLiquidatorContract, ());

        let user: Address = users.get(0).unwrap();
        let user2 = users.get(1).unwrap();

        // Deposit gold to satisfy the health factor threshold
        lending_contract_client.deposit(&user, &gold_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));
        // Deposit usdc as another user to have a non-empty loan pool
        lending_contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

        assert!(lending_contract_client
            .try_flash_loan(
                &flash_loan_taker_contract_address,
                &usdc_pool_address,
                &FAILING_CALL_AMOUNT,
            )
            .is_err());
    }

    #[test]
    fn test_flash_loan_overbalance() {
        let TestFixture {
            e,
            contract_client: lending_contract_client,
            gold_pool_address,
            usdc_pool_address,
            users,
            ..
        } = TestFixture::new();

        let flash_loan_taker_contract_address = e.register(FlashLoanLiquidatorContract, ());

        let user: Address = users.get(0).unwrap();
        let user2 = users.get(1).unwrap();

        // Deposit gold to satisfy the health factor threshold
        lending_contract_client.deposit(&user, &gold_pool_address, &(3 * DEFAULT_DEPOSIT_AMOUNT));
        // Deposit usdc as another user to have a non-empty loan pool
        lending_contract_client.deposit(&user2, &usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT));

        assert_eq!(
            lending_contract_client.try_flash_loan(
                &flash_loan_taker_contract_address,
                &usdc_pool_address,
                &(DEFAULT_DEPOSIT_AMOUNT + 1)
            ),
            Err(Ok(LCError::NotEnoughPoolFunds))
        );
    }
}
