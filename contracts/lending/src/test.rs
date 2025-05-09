#![cfg(test)]

use {
    crate::{
        contract::*,
        storage::{Obligation, Pool},
    },
    soroban_sdk::{testutils::Address as _, token::StellarAssetClient, Address, BytesN, Env},
};

struct TestEnv<'a> {
    contract_client: LendingContractClient<'a>,
    contract_admin: Address,
}

fn setup_test_env(e: &Env) -> TestEnv {
    e.mock_all_auths();
    let contract_admin = Address::generate(&e);
    let contract_id = e.register(LendingContract, (contract_admin.clone(),));
    let contract_client = LendingContractClient::new(&e, &contract_id);

    TestEnv {
        contract_client,
        contract_admin,
    }
}

#[test]
fn test_pool_initialization() {
    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let token_address = Address::generate(&e);
    contract_client.initialize_pool(&token_address, &None, &120);
    contract_client.initialize_pool(
        &token_address,
        &Some(BytesN::from_array(&e, &[0; 32])),
        &120,
    );
}

#[test]
fn test_pool_initialization_with_different_name() {
    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let token_address = Address::generate(&e);
    let salt0 = BytesN::from_array(&e, &[0; 32]);
    let salt1 = BytesN::from_array(&e, &[1; 32]);

    assert!(contract_client
        .try_initialize_pool(&token_address, &Some(salt0), &0)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address, &Some(salt1), &0)
        .is_ok());
}

#[test]
fn test_pool_not_conflicting_initializations() {
    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let token_address1 = Address::generate(&e);
    let token_address2 = Address::generate(&e);
    let salt = BytesN::from_array(&e, &[0; 32]);

    assert!(contract_client
        .try_initialize_pool(&token_address1, &None, &0)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address2, &None, &0)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address1, &Some(salt.clone()), &0)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address2, &Some(salt), &0)
        .is_ok());
}

#[test]
#[should_panic]
fn test_pool_reinitialization_no_salt() {
    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let token_address = Address::generate(&e);

    assert!(contract_client
        .try_initialize_pool(&token_address, &None, &0)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address, &None, &0)
        .is_ok());
}

#[test]
#[should_panic]
fn test_pool_reinitialization_with_salt() {
    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let token_address = Address::generate(&e);
    let salt = BytesN::from_array(&e, &[0; 32]);

    assert!(contract_client
        .try_initialize_pool(&token_address, &Some(salt.clone()), &0)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address, &Some(salt.clone()), &0)
        .is_ok());
}

#[test]
fn test_pool_deposit() {
    const DEPOSIT_AMOUNT: i128 = 100;

    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let user = Address::generate(&e);
    let token_admin = Address::generate(&e);
    // Mint tokens
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_asset_client = StellarAssetClient::new(&e, &token_address);
    token_asset_client.mint(&user, &DEPOSIT_AMOUNT);
    let pool_address = contract_client.initialize_pool(&token_address, &None, &0);
    // Deposit tokens
    contract_client.deposit(&user, &pool_address, &DEPOSIT_AMOUNT);
    // Check obligation
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address).unwrap();

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT)
}

#[test]
fn test_pool_withdraw() {
    const DEPOSIT_AMOUNT: i128 = 100;
    let half_deposit = (DEPOSIT_AMOUNT as f32 / 2_f32) as i128;

    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let user = Address::generate(&e);
    let token_admin = Address::generate(&e);
    // Mint tokens
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_asset_client = StellarAssetClient::new(&e, &token_address);
    token_asset_client.mint(&user, &DEPOSIT_AMOUNT);
    let pool_address = contract_client.initialize_pool(&token_address, &None, &0);
    // Deposit tokens
    contract_client.deposit(&user, &pool_address, &DEPOSIT_AMOUNT);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap();

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Withdraw half
    contract_client.withdraw(&user, &pool_address, &half_deposit);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap();
    let Pool { balance, .. } = contract_client.get_pool(&pool_address).unwrap();

    assert_eq!(deposited_amount, half_deposit);
    assert_eq!(balance, half_deposit);
    // Withdraw half again
    contract_client.withdraw(&user, &pool_address, &half_deposit);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap();
    let Pool { balance, .. } = contract_client.get_pool(&pool_address).unwrap();

    assert_eq!(deposited_amount, 0);
    assert_eq!(balance, 0);
}

#[test]
#[should_panic]
fn test_pool_withdraw_overflow() {
    const DEPOSIT_AMOUNT: i128 = 100;

    let e = Env::default();
    let TestEnv {
        contract_client, ..
    } = setup_test_env(&e);

    let user = Address::generate(&e);
    let token_admin = Address::generate(&e);
    // Mint tokens
    let token_address = e.register_stellar_asset_contract_v2(token_admin).address();
    let token_asset_client = StellarAssetClient::new(&e, &token_address);
    token_asset_client.mint(&user, &DEPOSIT_AMOUNT);
    let pool_address = contract_client.initialize_pool(&token_address, &None, &0);
    // Deposit tokens
    contract_client.deposit(&user, &pool_address, &DEPOSIT_AMOUNT);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap();

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Wilthdraw more than available
    contract_client.withdraw(&user, &pool_address, &(DEPOSIT_AMOUNT + 1));
}
