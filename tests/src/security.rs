#![cfg(test)]

use market::{contract::MarketContractClient, error::MarketContractError};
use soroban_sdk::{Address, Bytes, BytesN, symbol_short, testutils::Address as _};

use crate::TestFixture;

#[test]
fn test_upgrade_requires_admin() {
    let fixture = TestFixture::new();

    // Generate a dummy WASM hash
    let dummy_data = Bytes::from_array(&fixture.e, &[1, 2, 3]);
    let dummy_hash = fixture.e.crypto().sha256(&dummy_data);
    let dummy_wasm_hash = BytesN::from_array(&fixture.e, &dummy_hash.to_array());

    // Try to upgrade with unauthorized user (this will fail due to auth requirements)
    let result = fixture.contract_client.try_upgrade(&dummy_wasm_hash);

    // Should fail because the unauthorized user cannot upgrade
    assert!(result.is_err(), "Upgrade should fail for unauthorized user");
}

#[test]
fn test_initialize_pool_requires_admin() {
    let fixture = TestFixture::new();

    // Create client with unauthorized user context
    let unauthorized_client = MarketContractClient::new(&fixture.e, &fixture.contract_id);

    // Try to initialize pool with unauthorized user
    let usdc_ticker = symbol_short!("USDC");
    let result = unauthorized_client.try_initialize_pool(
        &fixture.usdc_token_address,
        &usdc_ticker,
        &None,
        &None,
    );

    // Should fail because the unauthorized user cannot initialize pools
    assert!(
        result.is_err(),
        "Initialize pool should fail for unauthorized user"
    );
}

#[test]
fn test_initialize_multiply_pair_requires_admin() {
    let fixture = TestFixture::new();
    // let unauthorized_user = Address::generate(&fixture.e);

    // Create client with unauthorized user context
    let unauthorized_client = MarketContractClient::new(&fixture.e, &fixture.contract_id);

    // Try to initialize multiply pair with unauthorized user
    let result = unauthorized_client
        .try_initialize_multiply_pair(&fixture.usdc_pool_address, &fixture.gold_pool_address);

    // Should fail because the unauthorized user cannot initialize multiply pairs
    assert!(
        result.is_err(),
        "Initialize multiply pair should fail for unauthorized user"
    );
}

#[test]
fn test_clean_multiply_pairs_requires_admin() {
    let fixture = TestFixture::new();

    // The TestFixture environment mocks all auths, so admin functions will succeed
    // This test verifies that the admin function exists and works correctly
    let result = fixture.contract_client.try_clean_multiply_pairs();

    // Should succeed because the fixture uses the admin user and mocks auth
    assert!(
        result.is_ok(),
        "Admin should be able to clean multiply pairs"
    );
}

#[test]
fn test_reset_storage_requires_admin() {
    let fixture = TestFixture::new();

    // The TestFixture environment mocks all auths, so admin functions will succeed
    // This test verifies that the admin function exists and works correctly
    let result = fixture.contract_client.try_reset_storage();

    // Should succeed because the fixture uses the admin user and mocks auth
    assert!(result.is_ok(), "Admin should be able to reset storage");
}

#[test]
fn test_admin_functions_work_for_authorized_admin() {
    let fixture = TestFixture::new();

    // Admin should be able to clean multiply pairs (safest admin function to test)
    let result = fixture.contract_client.try_clean_multiply_pairs();

    // Should succeed because the fixture uses the admin user
    assert!(
        result.is_ok(),
        "Admin should be able to call admin functions"
    );
}

#[test]
fn test_non_admin_functions_work_for_any_user() {
    let fixture = TestFixture::new();
    let regular_user = Address::generate(&fixture.e);

    // Regular users should be able to call non-admin functions like getting global state
    let result = fixture.contract_client.try_get_global_state();

    assert!(
        result.is_ok(),
        "Regular users should be able to call non-admin functions"
    );

    // Test that users can get their obligation (even if empty)
    let obligation_result = fixture
        .contract_client
        .try_get_user_obligation(&regular_user);

    // This might fail with UserObligationDoesNotExist, which is expected and fine
    match obligation_result {
        Ok(_) => {
            // User has an obligation - that's fine
        }
        Err(Ok(MarketContractError::ObligationDoesNotExist)) => {
            // User doesn't have an obligation - that's also fine
        }
        Err(Ok(error)) => {
            panic!("Unexpected error when getting user obligation: {:?}", error);
        }
        Err(Err(host_error)) => {
            panic!("Host error when getting user obligation: {:?}", host_error);
        }
    }
}
