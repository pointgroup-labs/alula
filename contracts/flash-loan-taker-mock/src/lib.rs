#![no_std]

use {
    moderc3156::ModErc3156,
    soroban_sdk::{
        contract, contractimpl,
        token::{StellarAssetClient, TokenClient},
        Address, Env,
    },
};

const FAILING_CALL_AMOUNT: i128 = 777;

#[contract]
pub struct FlashLoanLiquidatorContract;

#[contractimpl]
impl moderc3156::ModErc3156 for FlashLoanLiquidatorContract {
    fn exec_op(e: Env, caller: Address, token: Address, amount: i128, _fee: i128) {
        caller.require_auth();

        let flash_loan_token_client = TokenClient::new(&e, &token);
        let flash_loan_received = flash_loan_token_client.balance(&e.current_contract_address());
        assert_eq!(
            flash_loan_received, amount,
            "Flash borrow should've taken place"
        );

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

/// Simulates a failed strategy that loses 10% of the flash loan
fn simulate_failed_strategy(e: &Env, token_address: &Address, amount: i128) {
    let token_client = TokenClient::new(e, token_address);
    token_client.burn(&e.current_contract_address(), &(amount / 10));
}

#[cfg(test)]
mod test {
    use {
        super::{FlashLoanLiquidatorContract, FAILING_CALL_AMOUNT},
        lending::constants::LCError,
        soroban_sdk::Address,
        tests::{TestFixture, DEFAULT_DEPOSIT_AMOUNT},
    };

    struct FlashLoanTest<'a> {
        test_fixture: TestFixture<'a>,
        flash_loan_taker_contract_id: Address,
    }

    impl FlashLoanTest<'_> {
        fn new() -> Self {
            let test_fixture = TestFixture::new();
            let lender = test_fixture.users.get(0).unwrap();
            let flash_loan_taker_contract_id =
                test_fixture.e.register(FlashLoanLiquidatorContract, ());

            // Deposit usdc as some lender to have a non-empty loan pool
            test_fixture.contract_client.deposit(
                &lender,
                &test_fixture.usdc_pool_address,
                &(DEFAULT_DEPOSIT_AMOUNT),
            );

            Self {
                test_fixture,
                flash_loan_taker_contract_id,
            }
        }
    }

    #[test]
    fn test_flash_loan_success() {
        let FlashLoanTest {
            test_fixture,
            flash_loan_taker_contract_id,
            ..
        } = FlashLoanTest::new();

        test_fixture.contract_client.flash_loan(
            &flash_loan_taker_contract_id,
            &test_fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );
    }

    #[test]
    fn test_flash_loan_failure() {
        let FlashLoanTest {
            test_fixture,
            flash_loan_taker_contract_id,
            ..
        } = FlashLoanTest::new();

        assert!(test_fixture
            .contract_client
            .try_flash_loan(
                &flash_loan_taker_contract_id,
                &test_fixture.usdc_pool_address,
                &FAILING_CALL_AMOUNT,
            )
            .is_err());
    }

    #[test]
    fn test_flash_loan_overbalance() {
        let FlashLoanTest {
            test_fixture,
            flash_loan_taker_contract_id,
            ..
        } = FlashLoanTest::new();

        assert_eq!(
            test_fixture.contract_client.try_flash_loan(
                &flash_loan_taker_contract_id,
                &test_fixture.usdc_pool_address,
                &(DEFAULT_DEPOSIT_AMOUNT + 1)
            ),
            Err(Ok(LCError::NotEnoughPoolFunds))
        );
    }
}
