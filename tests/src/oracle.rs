#![cfg(test)]

use crate::{TestMarketFixture, make_oracle_prices_negative, make_oracle_prices_zero};
use market::error::MCError;

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

#[test]
fn test_zero_oracle_price_is_not_accepted() {
    let TestMarketFixture { e, contract_client, usdc_pool_address, oracle_client, .. } =
        TestMarketFixture::new();

    let price = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert!(price.is_positive());

    make_oracle_prices_zero(&e, &oracle_client);

    assert_eq!(
        contract_client.try_get_pool_asset_oracle_price(&usdc_pool_address),
        Err(Ok(MCError::NonPositiveOraclePrice))
    );
}

#[test]
fn test_negative_oracle_price_is_not_accepted() {
    let TestMarketFixture { e, contract_client, usdc_pool_address, oracle_client, .. } =
        TestMarketFixture::new();

    let price = contract_client.get_pool_asset_oracle_price(&usdc_pool_address);
    assert!(price.is_positive());

    make_oracle_prices_negative(&e, &oracle_client);

    assert_eq!(
        contract_client.try_get_pool_asset_oracle_price(&usdc_pool_address),
        Err(Ok(MCError::NonPositiveOraclePrice))
    );
}
