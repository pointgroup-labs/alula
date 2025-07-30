#![cfg(test)]

use crate::TestFixture;

#[test]
fn test_remove_obligation() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    assert!(contract_client.try_get_user_obligation(&user).is_err());

    contract_client.deposit(&user, &usdc_pool_address, &1000);
    assert!(contract_client.try_get_user_obligation(&user).is_ok());

    contract_client.reset_storage();
    assert!(contract_client.try_get_user_obligation(&user).is_err());
}

#[test]
fn test_remove_many_obligations() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user1 = users.get(0).unwrap();
    let user2 = users.get(1).unwrap();

    assert!(contract_client.get_all_obligations().is_empty());

    contract_client.deposit(&user1, &usdc_pool_address, &1000);
    contract_client.deposit(&user2, &usdc_pool_address, &1000);

    assert!(contract_client.try_get_user_obligation(&user1).is_ok());
    assert!(contract_client.try_get_user_obligation(&user2).is_ok());
    assert_eq!(contract_client.get_all_obligations().len(), 2);

    contract_client.reset_storage();

    assert!(contract_client.try_get_user_obligation(&user1).is_err());
    assert!(contract_client.try_get_user_obligation(&user2).is_err());
    assert!(contract_client.get_all_obligations().is_empty());
}

#[test]
fn test_remove_pool() {
    let TestFixture {
        contract_client, ..
    } = TestFixture::new();

    assert_eq!(contract_client.get_all_pools().len(), 3);

    contract_client.reset_storage();

    assert!(contract_client.get_all_pools().is_empty());
}

#[test]
fn test_remove_multiply_pairs() {
    let TestFixture {
        contract_client,
        usdc_pool_address,
        gold_pool_address,
        btc_pool_address,
        ..
    } = TestFixture::new();

    assert!(contract_client.get_all_multiply_pairs().is_empty());

    contract_client.initialize_multiply_pair(&usdc_pool_address, &gold_pool_address);
    contract_client.initialize_multiply_pair(&usdc_pool_address, &btc_pool_address);

    assert_eq!(contract_client.get_all_multiply_pairs().len(), 2);

    contract_client.reset_storage();

    assert!(contract_client.get_all_multiply_pairs().is_empty());
}
