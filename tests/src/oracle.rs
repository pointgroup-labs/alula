#![cfg(test)]

use crate::TestMarketFixture;

#[test]
fn test_get_asset_decimals() {
    let TestMarketFixture { contract_client, .. } = TestMarketFixture::new();

    let decimals = contract_client.get_asset_decimals();
    assert_eq!(decimals, 7);
}

#[test]
fn test_get_oracle_price_decimals() {
    let TestMarketFixture { contract_client, .. } = TestMarketFixture::new();

    let decimals = contract_client.get_oracle_price_decimals();
    assert_eq!(decimals, 14);
}

#[test]
fn test_get_pool_asset_oracle_price() {
    let TestMarketFixture { contract_client, usdc_pool_address, .. } = TestMarketFixture::new();

    let price = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert!(price.is_positive());
}
