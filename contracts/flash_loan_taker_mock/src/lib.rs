#![no_std]

use moderc3156::ModErc3156;
use soroban_sdk::{
    Address, Env, Symbol, contract, contractimpl, symbol_short,
    token::{StellarAssetClient, TokenClient},
};

const FAILING_CALL_AMOUNT: i128 = 777;
const MARKET_KEY: Symbol = symbol_short!("market");
const TRUSTED_INIT_KEY: Symbol = symbol_short!("trusted");

#[contract]
pub struct FlashLoanTakerContract;

#[contractimpl]
impl FlashLoanTakerContract {
    pub fn __constructor(e: Env, market: Address) {
        e.storage().instance().set(&MARKET_KEY, &market);
    }

    fn market(e: &Env) -> Address {
        e.storage().instance().get(&MARKET_KEY).expect("not initialized via __constructor")
    }
}

#[contractimpl]
impl ModErc3156 for FlashLoanTakerContract {
    fn exec_op(e: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        // Real-world receivers MUST gate this against an allowlist of trusted
        // initiators, e.g. `assert!(initiator == self.trusted_initiator(&e))`.
        // The mock just authenticates the initiator to demonstrate the pattern.
        initiator.require_auth();

        let flash_loan_token_client = TokenClient::new(&e, &token);
        let flash_loan_received = flash_loan_token_client.balance(&e.current_contract_address());

        assert_eq!(flash_loan_received, amount, "Flash borrow should've taken place");

        if amount == FAILING_CALL_AMOUNT {
            simulate_failed_strategy(&e, &token, amount);
        } else {
            simulate_successful_strategy(&e, &Self::market(&e), &token, amount, fee);
        }
    }
}

/// Simulates a successful strategy that earns 10% on top of the flash loan
fn simulate_successful_strategy(
    e: &Env,
    market: &Address,
    token_address: &Address,
    amount: i128,
    fee: i128,
) {
    let sac_client = StellarAssetClient::new(e, token_address);
    sac_client.mint(&e.current_contract_address(), &(amount / 10));

    sac_client.approve(
        &e.current_contract_address(),
        market,
        &(amount + fee),
        &(e.ledger().sequence()),
    );
}

/// Simulates a failed strategy that loses 10% of the flash loan
fn simulate_failed_strategy(e: &Env, token_address: &Address, amount: i128) {
    let token_client = TokenClient::new(e, token_address);
    token_client.burn(&e.current_contract_address(), &(amount / 10));
}

/// A receiver that enforces an initiator allowlist — the EIP-3156-style defense.
/// Used to verify the security fix: attacker-initiated flash loans against this
/// receiver must revert before any fee can be charged.
#[contract]
pub struct StrictFlashLoanTakerContract;

#[contractimpl]
impl StrictFlashLoanTakerContract {
    pub fn __constructor(e: Env, market: Address, trusted_initiator: Address) {
        e.storage().instance().set(&MARKET_KEY, &market);
        e.storage().instance().set(&TRUSTED_INIT_KEY, &trusted_initiator);
    }
}

#[contractimpl]
impl ModErc3156 for StrictFlashLoanTakerContract {
    fn exec_op(e: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
        let trusted: Address =
            e.storage().instance().get(&TRUSTED_INIT_KEY).expect("not initialized");

        // The defense the audit finding requires: reject any initiator that isn't
        // explicitly trusted. Without this check, a third party could force this
        // contract to pay flash-loan fees.
        assert_eq!(initiator, trusted, "untrusted initiator");

        let market: Address = e.storage().instance().get(&MARKET_KEY).expect("not initialized");
        simulate_successful_strategy(&e, &market, &token, amount, fee);
    }
}

#[cfg(test)]
mod test {
    use market::{error::MCError, obligation::ObligationKey};
    use soroban_sdk::Address;
    use tests::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

    use super::{FAILING_CALL_AMOUNT, FlashLoanTakerContract, StrictFlashLoanTakerContract};

    struct FlashLoanTest<'a> {
        test_fixture: TestMarketFixture<'a>,
        flash_loan_taker_contract_id: Address,
    }

    impl FlashLoanTest<'_> {
        fn new() -> Self {
            let test_fixture = TestMarketFixture::new();
            let lender = &test_fixture.users[0];
            let flash_loan_taker_contract_id = test_fixture
                .e
                .register(FlashLoanTakerContract, (test_fixture.contract_id.clone(),));

            // Deposit usdc as some lender to have a non-empty loan pool
            test_fixture.contract_client.deposit(
                &ObligationKey::new(lender.clone()),
                &test_fixture.usdc_pool_address,
                &DEFAULT_DEPOSIT_AMOUNT,
                &None,
            );

            Self { test_fixture, flash_loan_taker_contract_id }
        }
    }

    #[test]
    fn test_flash_loan_zero() {
        let FlashLoanTest { test_fixture, flash_loan_taker_contract_id, .. } = FlashLoanTest::new();
        let caller = &test_fixture.users[1];

        assert_eq!(
            test_fixture.contract_client.try_flash_loan(
                &flash_loan_taker_contract_id,
                caller,
                &test_fixture.usdc_pool_address,
                &0,
            ),
            Err(Ok(MCError::InvalidInputAmount))
        );
    }

    #[test]
    fn test_flash_loan_success() {
        let FlashLoanTest { test_fixture, flash_loan_taker_contract_id, .. } = FlashLoanTest::new();
        let caller = &test_fixture.users[1];

        test_fixture.contract_client.flash_loan(
            &flash_loan_taker_contract_id,
            caller,
            &test_fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );
    }

    #[test]
    fn test_flash_loan_failure() {
        let FlashLoanTest { test_fixture, flash_loan_taker_contract_id, .. } = FlashLoanTest::new();
        let caller = &test_fixture.users[1];

        assert!(
            test_fixture
                .contract_client
                .try_flash_loan(
                    &flash_loan_taker_contract_id,
                    caller,
                    &test_fixture.usdc_pool_address,
                    &FAILING_CALL_AMOUNT,
                )
                .is_err()
        );
    }

    #[test]
    fn test_flash_loan_overbalance() {
        let FlashLoanTest { test_fixture, flash_loan_taker_contract_id, .. } = FlashLoanTest::new();
        let caller = &test_fixture.users[1];

        assert_eq!(
            test_fixture.contract_client.try_flash_loan(
                &flash_loan_taker_contract_id,
                caller,
                &test_fixture.usdc_pool_address,
                &(DEFAULT_DEPOSIT_AMOUNT + 1)
            ),
            Err(Ok(MCError::NotEnoughPoolFunds))
        );
    }

    /// Regression test for the audit finding. A receiver that allowlists a single
    /// trusted initiator must accept flash loans  triggered by that initiator.
    /// This proves `initiator` is forwarded correctly
    /// (if the market still passed `e.current_contract_address()` as it did before
    /// the fix, the assert_eq inside the receiver would fail and this test would
    /// revert).
    #[test]
    fn test_strict_receiver_accepts_trusted_initiator() {
        let test_fixture = TestMarketFixture::new();
        let lender = &test_fixture.users[0];
        let trusted = &test_fixture.users[1];

        let strict_receiver_id = test_fixture.e.register(
            StrictFlashLoanTakerContract,
            (test_fixture.contract_id.clone(), trusted.clone()),
        );

        test_fixture.contract_client.deposit(
            &ObligationKey::new(lender.clone()),
            &test_fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );

        // Trusted initiator triggers the flash loan — must succeed.
        test_fixture.contract_client.flash_loan(
            &strict_receiver_id,
            trusted,
            &test_fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );
    }

    /// Regression test for the audit finding. An attacker should not be able to
    /// force a flash loan against a receiver they don't control. With initiator
    /// forwarding in place, the receiver's `assert_eq!(initiator, trusted)` rejects
    /// the call before any fee is charged.
    #[test]
    #[should_panic(expected = "untrusted initiator")]
    fn test_strict_receiver_rejects_untrusted_initiator() {
        let test_fixture = TestMarketFixture::new();
        let lender = &test_fixture.users[0];
        let trusted = &test_fixture.users[1];
        let attacker = &test_fixture.users[2];

        let strict_receiver_id = test_fixture.e.register(
            StrictFlashLoanTakerContract,
            (test_fixture.contract_id.clone(), trusted.clone()),
        );

        test_fixture.contract_client.deposit(
            &ObligationKey::new(lender.clone()),
            &test_fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
            &None,
        );

        // Attacker (not the trusted initiator) tries to force a flash loan
        // against the receiver. Must revert inside the receiver's allowlist check.
        test_fixture.contract_client.flash_loan(
            &strict_receiver_id,
            attacker,
            &test_fixture.usdc_pool_address,
            &DEFAULT_DEPOSIT_AMOUNT,
        );
    }
}
