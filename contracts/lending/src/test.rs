#![cfg(test)]

use {
    crate::{
        contract::*,
        oracle,
        storage::{Obligation, Pool},
    },
    soroban_sdk::{
        symbol_short,
        testutils::Address as _,
        token::{StellarAssetClient, TokenClient},
        Address, BytesN, Env, String, Symbol,
    },
};

const REFLECTOR_TESTNET_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

const DEFAULT_ADMIN_ASSET_MINT_AMOUNT: i128 = 1_000_000;
const DEFAULT_USER_ASSET_MINT_AMOUNT: i128 = 100_000;

struct TestEnv<'a> {
    contract_client: LendingContractClient<'a>,
    asset1: TestAssetSetup<'a>,
    asset2: TestAssetSetup<'a>,
    user: Address,
    admin: Address,
}

struct TestAssetSetup<'a> {
    #[allow(unused)]
    token_client: TokenClient<'a>,
    token_address: Address,
    token_ticker: Symbol,
}

fn setup_test_asset<'a>(
    e: &Env,
    admin: &Address,
    user: &Address,
    token_ticker: &Symbol,
) -> TestAssetSetup<'a> {
    let token_address = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let asset_client = StellarAssetClient::new(e, &token_address);
    let token_client = TokenClient::new(e, &token_address);
    asset_client.mint(admin, &DEFAULT_ADMIN_ASSET_MINT_AMOUNT);
    asset_client.mint(user, &DEFAULT_USER_ASSET_MINT_AMOUNT);

    TestAssetSetup {
        token_address,
        token_client,
        token_ticker: token_ticker.clone(),
    }
}

// TODO: Maybe, accept as a parameter the amount of test assets to be created
// We can implement this as a simple macro. Or just return them as a vector.
fn setup_test_env(e: &Env) -> TestEnv {
    e.mock_all_auths();
    let contract_admin = Address::generate(&e);
    let contract_id = e.register(
        LendingContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = LendingContractClient::new(&e, &contract_id);
    let admin = Address::generate(&e);
    let user = Address::generate(&e);
    let ticker1 = symbol_short!("TCK1");
    let ticker2 = symbol_short!("TCK2");

    let asset1 = setup_test_asset(e, &admin, &user, &ticker1);
    let asset2 = setup_test_asset(e, &admin, &user, &ticker2);
    // Registering reflector mock contract is enough.
    // In local tests contracts will call it via the same address as in testnet.
    let reflector_address = Address::from_string(&String::from_str(&e, REFLECTOR_TESTNET_ADDRESS));
    e.register_at(&reflector_address, oracle::WASM, ());

    TestEnv {
        user,
        admin,
        asset1,
        asset2,
        contract_client,
    }
}

#[test]
fn test_pool_initialization() {
    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        ..
    } = setup_test_env(&e);

    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    contract_client.initialize_pool(
        &token_address,
        &token_ticker,
        &Some(BytesN::from_array(&e, &[0; 32])),
        &None,
    );
}

#[test]
fn test_pool_initialization_with_different_name() {
    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        ..
    } = setup_test_env(&e);
    let salt0 = BytesN::from_array(&e, &[0; 32]);
    let salt1 = BytesN::from_array(&e, &[1; 32]);

    assert!(contract_client
        .try_initialize_pool(&token_address, &token_ticker, &Some(salt0), &None)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address, &token_ticker, &Some(salt1), &None)
        .is_ok());
}

#[test]
fn test_pool_not_conflicting_initializations() {
    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address: token_address1,
                token_ticker: token_ticker1,
                ..
            },
        asset2:
            TestAssetSetup {
                token_address: token_address2,
                token_ticker: token_ticker2,
                ..
            },
        ..
    } = setup_test_env(&e);
    let salt = BytesN::from_array(&e, &[0; 32]);

    assert!(contract_client
        .try_initialize_pool(&token_address1, &token_ticker1, &None, &None)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address2, &token_ticker2, &None, &None)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address1, &token_ticker1, &Some(salt.clone()), &None)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address2, &token_ticker2, &Some(salt), &None)
        .is_ok());
}

#[test]
#[should_panic]
fn test_pool_reinitialization_no_salt() {
    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        ..
    } = setup_test_env(&e);

    assert!(contract_client
        .try_initialize_pool(&token_address, &token_ticker, &None, &None)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address, &token_ticker, &None, &None)
        .is_ok());
}

#[test]
#[should_panic]
fn test_pool_reinitialization_with_salt() {
    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        ..
    } = setup_test_env(&e);
    let salt = BytesN::from_array(&e, &[0; 32]);

    assert!(contract_client
        .try_initialize_pool(&token_address, &token_ticker, &Some(salt.clone()), &None)
        .is_ok());

    assert!(contract_client
        .try_initialize_pool(&token_address, &token_ticker, &Some(salt.clone()), &None)
        .is_ok());
}

#[test]
fn test_pool_deposit() {
    const DEPOSIT_AMOUNT: i128 = 100;

    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        user,
        ..
    } = setup_test_env(&e);
    let pool_address = contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
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
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        user,
        ..
    } = setup_test_env(&e);
    let pool_address = contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    // Deposit tokens
    contract_client.deposit(&user, &pool_address, &DEPOSIT_AMOUNT);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap();

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Withdraw half
    contract_client.withdraw(&user, &pool_address, &half_deposit);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap();
    let Pool { supply, .. } = contract_client.get_pool(&pool_address).unwrap();

    assert_eq!(deposited_amount, half_deposit);
    assert_eq!(supply, half_deposit);
}

#[test]
#[should_panic]
fn test_pool_withdraw_overflow() {
    const DEPOSIT_AMOUNT: i128 = 100;

    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        user,
        ..
    } = setup_test_env(&e);
    let pool_address = contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    // Deposit tokens
    contract_client.deposit(&user, &pool_address, &DEPOSIT_AMOUNT);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap();

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Withdraw more than available
    contract_client.withdraw(&user, &pool_address, &(DEPOSIT_AMOUNT + 1));
}

#[test]
fn test_borrow() {
    const DEPOSIT_AMOUNT: i128 = 100;
    let half_deposit_amount = DEPOSIT_AMOUNT / 2;

    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address: token_address1,
                token_ticker: token_ticker1,
                ..
            },
        user,
        admin,
        asset2:
            TestAssetSetup {
                token_address: token_address2,
                token_ticker: token_ticker2,
                ..
            },
        ..
    } = setup_test_env(&e);
    // Deposit token 1 as user
    let pool_address1 =
        contract_client.initialize_pool(&token_address1, &token_ticker1, &None, &None);
    contract_client.deposit(&user, &pool_address1, &DEPOSIT_AMOUNT);
    // TODO: Add get_deposit(pool_address: Address) method
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address1.clone()).unwrap();
    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Deposit token 2 as admin
    let pool_address2 =
        contract_client.initialize_pool(&token_address2, &token_ticker2, &None, &None);
    contract_client.deposit(&admin, &pool_address2, &DEPOSIT_AMOUNT);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&admin).unwrap();
    let deposited_amount = deposits.get(pool_address2.clone()).unwrap();
    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Borrow token 2 as user
    contract_client.borrow(&user, &pool_address1, &half_deposit_amount);
}

#[test]
fn test_borrow_health() {
    const DEPOSIT_AMOUNT: i128 = 100;
    const MAX_HEALTHY_BORROW_AMOUNT: i128 = 80;

    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address: token_address1,
                token_ticker: token_ticker1,
                ..
            },
        user,
        admin,
        asset2:
            TestAssetSetup {
                token_address: token_address2,
                token_ticker: token_ticker2,
                ..
            },
        ..
    } = setup_test_env(&e);
    // Deposit token 1 as user
    let pool_address1 =
        contract_client.initialize_pool(&token_address1, &token_ticker1, &None, &None);
    contract_client.deposit(&user, &pool_address1, &DEPOSIT_AMOUNT);
    // TODO: Add get_deposit(pool_address: Address) method
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address1.clone()).unwrap();
    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Deposit token 2 as admin
    let pool_address2 =
        contract_client.initialize_pool(&token_address2, &token_ticker2, &None, &None);
    contract_client.deposit(&admin, &pool_address2, &DEPOSIT_AMOUNT);
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&admin).unwrap();
    let deposited_amount = deposits.get(pool_address2.clone()).unwrap();
    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);
    // Borrow token 2 as user
    contract_client.borrow(&user, &pool_address1, &MAX_HEALTHY_BORROW_AMOUNT);
    // Borrowing more must result in error
    assert!(contract_client
        .try_borrow(&user, &pool_address1, &1)
        .is_err());
}

#[test]
fn test_default_pool_config_interest_rate_calculation() {
    const DEPOSIT_AMOUNT: i128 = 10_000;

    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address: token_address1,
                token_ticker: token_ticker1,
                ..
            },
        asset2:
            TestAssetSetup {
                token_address: token_address2,
                token_ticker: token_ticker2,
                ..
            },
        user,
        admin,
        ..
    } = setup_test_env(&e);
    let pool_address1 =
        contract_client.initialize_pool(&token_address1, &token_ticker1, &None, &None);
    let pool_address2 =
        contract_client.initialize_pool(&token_address2, &token_ticker2, &None, &None);
    // Deposit tokens
    contract_client.deposit(&user, &pool_address1, &DEPOSIT_AMOUNT);
    contract_client.deposit(&admin, &pool_address2, &(10 * &DEPOSIT_AMOUNT)); // x10 in order to not care about the health factor
    contract_client.deposit(&user, &pool_address2, &(10 * &DEPOSIT_AMOUNT));

    // 0% UR
    let interest_rates = contract_client.get_interest_rates(&pool_address1);
    assert_eq!(interest_rates.borrow_rate_bps, 300);
    assert_eq!(interest_rates.supply_rate_bps, 0);

    // 10% UR
    contract_client.borrow(&user, &pool_address1, &((DEPOSIT_AMOUNT * 10) / 100));
    let interest_rates = contract_client.get_interest_rates(&pool_address1);
    assert_eq!(interest_rates.borrow_rate_bps, 500);
    assert_eq!(interest_rates.supply_rate_bps, 45);

    // 50% UR
    contract_client.borrow(&user, &pool_address1, &((DEPOSIT_AMOUNT * 40) / 100));
    let interest_rates = contract_client.get_interest_rates(&pool_address1);
    assert_eq!(interest_rates.borrow_rate_bps, 1300);
    assert_eq!(interest_rates.supply_rate_bps, 585);

    // 80% UR
    contract_client.borrow(&user, &pool_address1, &((DEPOSIT_AMOUNT * 30) / 100));
    let interest_rates = contract_client.get_interest_rates(&pool_address1);
    assert_eq!(interest_rates.borrow_rate_bps, 1900);
    assert_eq!(interest_rates.supply_rate_bps, 1368);

    // 90% UR
    contract_client.borrow(&user, &pool_address1, &((DEPOSIT_AMOUNT * 10) / 100));
    let interest_rates = contract_client.get_interest_rates(&pool_address1);
    assert_eq!(interest_rates.borrow_rate_bps, 3900);
    assert_eq!(interest_rates.supply_rate_bps, 3159);

    // 95% UR
    contract_client.borrow(&user, &pool_address1, &((DEPOSIT_AMOUNT * 5) / 100));
    let interest_rates = contract_client.get_interest_rates(&pool_address1);
    assert_eq!(interest_rates.borrow_rate_bps, 4900);
    assert_eq!(interest_rates.supply_rate_bps, 4189);

    // 99% UR
    contract_client.borrow(&user, &pool_address1, &((DEPOSIT_AMOUNT * 4) / 100));
    let interest_rates = contract_client.get_interest_rates(&pool_address1);
    assert_eq!(interest_rates.borrow_rate_bps, 5700);
    assert_eq!(interest_rates.supply_rate_bps, 5078);

    // Borrow which implies 100% UR is forbidden
    assert!(contract_client
        .try_borrow(&user, &pool_address1, &((DEPOSIT_AMOUNT * 1) / 100))
        .is_err());
}
