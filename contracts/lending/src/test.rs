#![cfg(test)]

use {
    crate::{
        constants::{
            LCError, INDIVIDUAL_BUMP, INSTANCE_BUMP, REFLECTOR_TESTNET_ADDRESS, SHARED_BUMP,
        },
        contract::*,
        interest_rate::CompoundRates,
        oracle,
        storage::{DataKey, Obligation, ObligationDeposit, Pool},
    },
    soroban_sdk::{
        symbol_short,
        testutils::{
            storage::{Instance, Persistent},
            Address as _, Ledger,
        },
        token::{StellarAssetClient, TokenClient},
        Address, BytesN, Env, String, Symbol,
    },
};

extern crate std;

// TODO: We can write a declarative macro, which will take the amount of assets you want
// to operate in your test and will generate inner fields like
// token_ticker_1, token_address_2, token_client_n, which would be very convenient to operate with.
// Also, same for the amount of users?
#[allow(unused)]
struct TestEnv<'a> {
    contract_client: LendingContractClient<'a>,
    contract_id: Address,
    asset1: TestAssetSetup<'a>,
    asset2: TestAssetSetup<'a>,
    asset3: TestAssetSetup<'a>,
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
    const DEFAULT_ADMIN_ASSET_MINT_AMOUNT: i128 = 1_000_000;
    const DEFAULT_USER_ASSET_MINT_AMOUNT: i128 = 100_000;

    let token_address = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let asset_client = StellarAssetClient::new(e, &token_address);
    let token_client = TokenClient::new(e, &token_address);

    asset_client.mint(admin, &DEFAULT_ADMIN_ASSET_MINT_AMOUNT); // why can you do this mint at all?
    asset_client.mint(user, &DEFAULT_USER_ASSET_MINT_AMOUNT);

    TestAssetSetup {
        token_address,
        token_client,
        token_ticker: token_ticker.clone(),
    }
}

// TODO: Maybe, accept as a parameter the amount of test assets to be created
// We can implement this as a simple macro. Or just return them as a vector.

// TODO: Check blend test fixtures...
fn setup_test_env(e: &Env) -> TestEnv {
    e.mock_all_auths();

    let contract_admin = Address::generate(e);
    let contract_id = e.register(
        LendingContract,
        (contract_admin.clone(), Option::<i128>::None),
    );
    let contract_client = LendingContractClient::new(e, &contract_id);

    let admin = Address::generate(e);
    let user = Address::generate(e);

    let ticker1 = symbol_short!("TCK1");
    let ticker2 = symbol_short!("TCK2");
    let ticker3 = symbol_short!("TCK3");

    let asset1 = setup_test_asset(e, &admin, &user, &ticker1);
    let asset2 = setup_test_asset(e, &admin, &user, &ticker2);
    let asset3 = setup_test_asset(e, &admin, &user, &ticker3);

    // Registering reflector mock contract is enough.
    // In local tests contracts will call it via the same address as in testnet
    let reflector_address = Address::from_string(&String::from_str(e, REFLECTOR_TESTNET_ADDRESS));
    e.register_at(&reflector_address, oracle::WASM, ());

    TestEnv {
        user,
        admin,
        asset1,
        asset2,
        asset3,
        contract_client,
        contract_id,
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
fn test_deposit() {
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
    let deposited_amount = deposits.get(pool_address).unwrap().amount;

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT)
}

#[test]
fn test_deposit_collateral() {
    const COLLATERAL_DEPOSIT_AMOUNT: i128 = 100;

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

    // Deposit collateral
    contract_client.deposit_collateral(&user, &pool_address, &COLLATERAL_DEPOSIT_AMOUNT);

    // Check obligation
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_collateral_amount = deposits.get(pool_address).unwrap().collateral_amount;

    assert_eq!(deposited_collateral_amount, COLLATERAL_DEPOSIT_AMOUNT);
}

#[test]
fn test_withdraw() {
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
    let deposited_amount = deposits.get(pool_address.clone()).unwrap().amount;

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);

    // Withdraw half
    contract_client.withdraw(&user, &pool_address, &half_deposit);

    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address.clone()).unwrap().amount;
    let Pool { supply, .. } = contract_client.get_pool(&pool_address).unwrap();

    assert_eq!(deposited_amount, half_deposit);
    assert_eq!(supply, half_deposit);
}

#[test]
fn test_withdraw_collateral() {
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
        asset2:
            TestAssetSetup {
                token_address: token_address_2,
                token_ticker: token_ticker_2,
                ..
            },
        user,
        ..
    } = setup_test_env(&e);

    let pool_address = contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    let pool_address_2 =
        contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);

    // Make a plain deposit
    contract_client.deposit(&user, &pool_address, &(DEPOSIT_AMOUNT));

    // Make a collateral deposit by the same token amount
    contract_client.deposit_collateral(&user, &pool_address, &DEPOSIT_AMOUNT);

    // Make a deposit into a different loan pool to ignore health factor issues
    contract_client.deposit(&user, &pool_address_2, &(3 * DEPOSIT_AMOUNT));

    // Borrow some amount to have non-zero borrow interest rate
    contract_client.borrow(&user, &pool_address, &(&DEPOSIT_AMOUNT / 2));

    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_collateral_amount = deposits
        .get(pool_address.clone())
        .unwrap()
        .collateral_amount;
    assert_eq!(deposited_collateral_amount, DEPOSIT_AMOUNT);

    // WARN: For now if the time amount is less than 39 days - the deposit increase is not visible
    // this is an issue for sure
    e.ledger().with_mut(|li| li.timestamp = 39 * 60 * 60 * 24);

    contract_client.add_interest_to_user_obligation(&user, &Some(pool_address.clone()));

    let ObligationDeposit {
        collateral_amount,
        amount,
        ..
    } = contract_client
        .get_user_obligation(&user)
        .unwrap()
        .deposits
        .get(pool_address.clone())
        .unwrap();

    // Since it's not being used for loans - the interest rate isn't accrued for the collateral
    // deposit and its amount in deposit position must be the same
    assert_eq!(collateral_amount, DEPOSIT_AMOUNT);

    // Contrary to a collateral deposit, a plain deposit amount must increase
    assert!(amount > collateral_amount);
}

#[test]
#[should_panic] // TODO: Where possible add specifics in the #[should_panic] attribute
fn test_withdraw_overflow() {
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
    let deposited_amount = deposits.get(pool_address.clone()).unwrap().amount;

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

    let pool_address1 =
        contract_client.initialize_pool(&token_address1, &token_ticker1, &None, &None);
    let pool_address2 =
        contract_client.initialize_pool(&token_address2, &token_ticker2, &None, &None);

    // Deposit token 1 as user
    contract_client.deposit(&user, &pool_address1, &DEPOSIT_AMOUNT);

    // TODO: Add get_deposit(pool_address: PoolAddress) method
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address1.clone()).unwrap().amount;

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);

    // Deposit token 2 as admin
    contract_client.deposit(&admin, &pool_address2, &DEPOSIT_AMOUNT);

    let Obligation { deposits, .. } = contract_client.get_user_obligation(&admin).unwrap();
    let deposited_amount = deposits.get(pool_address2.clone()).unwrap().amount;

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

    let pool_address1 =
        contract_client.initialize_pool(&token_address1, &token_ticker1, &None, &None);
    let pool_address2 =
        contract_client.initialize_pool(&token_address2, &token_ticker2, &None, &None);

    // Deposit token 1 as user
    contract_client.deposit(&user, &pool_address1, &DEPOSIT_AMOUNT);

    // TODO: Add get_deposit(pool_address: PoolAddress) method
    let Obligation { deposits, .. } = contract_client.get_user_obligation(&user).unwrap();
    let deposited_amount = deposits.get(pool_address1.clone()).unwrap().amount;

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);

    // Deposit token 2 as admin
    contract_client.deposit(&admin, &pool_address2, &DEPOSIT_AMOUNT);

    let Obligation { deposits, .. } = contract_client.get_user_obligation(&admin).unwrap();
    let deposited_amount = deposits.get(pool_address2.clone()).unwrap().amount;

    assert_eq!(deposited_amount, DEPOSIT_AMOUNT);

    // Borrow token 2 as user
    contract_client.borrow(&user, &pool_address1, &MAX_HEALTHY_BORROW_AMOUNT);

    // Borrowing more must result in error
    assert!(contract_client
        .try_borrow(&user, &pool_address1, &1)
        .is_err());
}

#[test]
fn test_interest_rates() {
    const DEPOSIT_AMOUNT: i128 = 10_000;

    let e = Env::default();
    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address: token_address_1,
                token_ticker: token_ticker_1,
                ..
            },
        asset2:
            TestAssetSetup {
                token_address: token_address_2,
                token_ticker: token_ticker_2,
                ..
            },
        user,
        admin,
        ..
    } = setup_test_env(&e);

    let pool_address1 =
        contract_client.initialize_pool(&token_address_1, &token_ticker_1, &None, &None);
    let pool_address2 =
        contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);

    // Deposit in a different pool in order to not care about Health Factor
    contract_client.deposit(&user, &pool_address2, &(2 * DEPOSIT_AMOUNT));

    // Deposit to keep pool non-empty
    contract_client.deposit(&admin, &pool_address1, &DEPOSIT_AMOUNT);

    // O% UR
    let CompoundRates {
        borrow_rate_bps: borrow_apy,
        supply_rate_bps: supply_apy,
    } = contract_client.get_apy(&pool_address1);

    assert_eq!(borrow_apy, 320);
    assert_eq!(supply_apy, 0);

    let deposit_increases = [
        (DEPOSIT_AMOUNT * 5) / 10,   // 50% UR
        (DEPOSIT_AMOUNT * 3) / 10,   // 80% UR
        (DEPOSIT_AMOUNT * 15) / 100, // 95% UR
    ];

    // TODO: Decouple from PoolConfig specific values somehow?
    // If `PoolConfig` values are going to change - this test will become broken
    let expected_rates = [(2084, 1042), (3284, 2627), (11326, 10760)];

    for (deposit_increase, (borrow_apy, supply_apy)) in deposit_increases.iter().zip(expected_rates)
    {
        contract_client.borrow(&user, &pool_address1, deposit_increase);

        let CompoundRates {
            borrow_rate_bps,
            supply_rate_bps,
        } = contract_client.get_apy(&pool_address1);

        assert_eq!(borrow_apy, borrow_rate_bps);
        assert_eq!(supply_apy, supply_rate_bps);
    }
}

#[test]
fn test_repay() {
    const DEPOSIT_AMOUNT: i128 = 10_000;

    let e = Env::default();

    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address: token_address_1,
                token_ticker: token_ticker_1,
                ..
            },
        asset2:
            TestAssetSetup {
                token_address: token_address_2,
                token_ticker: token_ticker_2,
                ..
            },
        user,
        admin,
        ..
    } = setup_test_env(&e);

    // This better be in the `setup_test_env`, since when caring about initialization - we cannot invoke this setup at all
    let pool_address1 =
        contract_client.initialize_pool(&token_address_1, &token_ticker_1, &None, &None);
    let pool_address2 =
        contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);

    // Deposit as a different user in order to be able to borrow
    contract_client.deposit(&admin, &pool_address1, &DEPOSIT_AMOUNT);

    // Deposit in a different pool in order to have a collateral present
    contract_client.deposit(&user, &pool_address2, &DEPOSIT_AMOUNT);

    // Borrow
    contract_client.borrow(&user, &pool_address1, &(DEPOSIT_AMOUNT / 3));

    // Check obligation
    let Obligation { borrows, .. } = contract_client.get_user_obligation(&user).unwrap();
    let borrowed_amount = borrows.get(pool_address1.clone()).unwrap().amount;

    assert_eq!(borrowed_amount, DEPOSIT_AMOUNT / 3);

    // Repay
    contract_client.repay(&user, &pool_address1, &(DEPOSIT_AMOUNT / 3));

    // Check obligation
    let Obligation { borrows, .. } = contract_client.get_user_obligation(&user).unwrap();
    assert!(borrows.get(pool_address1.clone()).is_none());
}

#[test]
fn test_repay_with_interest_accrual() {
    const DEPOSIT_AMOUNT: i128 = 10_000;

    let e = Env::default();

    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address: token_address_1,
                token_ticker: token_ticker_1,
                ..
            },
        asset2:
            TestAssetSetup {
                token_address: token_address_2,
                token_ticker: token_ticker_2,
                ..
            },
        user,
        admin,
        ..
    } = setup_test_env(&e);

    let pool_address1 =
        contract_client.initialize_pool(&token_address_1, &token_ticker_1, &None, &None);
    let pool_address2 =
        contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);

    // Deposit as a different user in order to be able to borrow
    contract_client.deposit(&admin, &pool_address1, &DEPOSIT_AMOUNT);

    // Deposit in a different pool in order to have a collateral present
    contract_client.deposit(&user, &pool_address2, &DEPOSIT_AMOUNT);

    // Borrow
    contract_client.borrow(&user, &pool_address1, &(DEPOSIT_AMOUNT / 3));

    // Check obligation
    let Obligation { borrows, .. } = contract_client.get_user_obligation(&user).unwrap();
    let borrowed_amount = borrows.get(pool_address1.clone()).unwrap().amount;

    assert_eq!(borrowed_amount, DEPOSIT_AMOUNT / 3);

    e.ledger().with_mut(|li| li.timestamp = 60 * 60 * 24);

    // Accrue interest in a pool and update the user's obligation
    contract_client.add_interest_to_user_obligation(&user, &Some(pool_address1.clone()));

    // Check obligation
    let Obligation { borrows, .. } = contract_client.get_user_obligation(&user).unwrap();
    let borrowed_amount = borrows.get(pool_address1.clone()).unwrap().amount;

    assert!(borrowed_amount > DEPOSIT_AMOUNT / 3);

    // Partially repay the debt
    contract_client.repay(&user, &pool_address1, &(DEPOSIT_AMOUNT / 10));

    // Move time
    e.ledger().with_mut(|li| li.timestamp = 50 * 60 * 60 * 24);

    // Accrue interest
    // TODO: We better to this whenever we try to read the obligation's data, no?
    contract_client.add_interest_to_user_obligation(&user, &Some(pool_address1.clone()));

    // Check that debt has increased
    let Obligation { borrows, .. } = contract_client.get_user_obligation(&user).unwrap();
    let borrowed_amount = borrows.get(pool_address1.clone()).unwrap().amount;

    let x = (DEPOSIT_AMOUNT / 3) - (DEPOSIT_AMOUNT / 10);
    assert!(borrowed_amount > x);

    // Repay the debt completely
    contract_client.repay(&user, &pool_address1, &borrowed_amount);

    // Check that the borrow position is gone
    let Obligation { borrows, .. } = contract_client.get_user_obligation(&user).unwrap();

    assert!(borrows.get(pool_address1.clone()).is_none());
}

#[test]
fn test_storage_ttl_extension() {
    let e = Env::default();

    let TestEnv {
        contract_client,
        contract_id,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        user,
        ..
    } = setup_test_env(&e);

    // Write something in individual and in shared storages
    contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);

    contract_client.deposit(&user, &token_address, &100);

    // Check the TTL after global storage initialization
    e.as_contract(&contract_id, || {
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(token_address.clone())),
            SHARED_BUMP
        );

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(user.clone())),
            INDIVIDUAL_BUMP
        );
    });

    e.ledger().with_mut(|li| {
        li.sequence_number = 100_000;
    });

    // Check the TTL after global storage initialization
    e.as_contract(&contract_id, || {
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP - 100_000);

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(token_address.clone())),
            SHARED_BUMP - 100_000
        );

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(user.clone())),
            INDIVIDUAL_BUMP - 100_000
        );
    });

    // Extend it and check again
    contract_client.deposit(&user, &token_address, &100);

    // Check the TTL after global storage initialization
    e.as_contract(&contract_id, || {
        // assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(token_address.clone())),
            SHARED_BUMP
        );

        // Threshold hasn't been hit

        // assert_eq!(
        //     e.storage()
        //         .persistent()
        //         .get_ttl(&DataKey::Obligation(user.clone())),
        //     INDIVIDUAL_BUMP
        // );
    });
}

#[test]
fn test_liquidation() {
    const DEPOSIT_AMOUNT: i128 = 1_000;
    let e = Env::default();

    let TestEnv {
        contract_client,
        asset1:
            TestAssetSetup {
                token_address,
                token_ticker,
                ..
            },
        asset2:
            TestAssetSetup {
                token_address: token_address_2,
                token_ticker: token_ticker_2,
                ..
            },
        user,
        admin,
        ..
    } = setup_test_env(&e);

    let pool_address = contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
    let pool_address_2 =
        contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);

    // Provide some liquidity in the first pool
    contract_client.deposit(&admin, &pool_address, &(3 * DEPOSIT_AMOUNT));

    // Deposit in the second pool
    contract_client.deposit(&user, &pool_address_2, &DEPOSIT_AMOUNT);

    // Deposit collateral in the second pool
    contract_client.deposit_collateral(&user, &pool_address_2, &(DEPOSIT_AMOUNT / 4));

    // Borrow a maximal possible amount which will not cause a liquidation
    contract_client.borrow(&user, &pool_address, &(DEPOSIT_AMOUNT));

    // Check pools state
    let pool_1 = contract_client.get_pool(&pool_address).unwrap();

    assert_eq!(pool_1.supply, 3 * DEPOSIT_AMOUNT);
    assert_eq!(pool_1.borrowed, DEPOSIT_AMOUNT);

    let pool_2 = contract_client.get_pool(&pool_address_2).unwrap();

    assert_eq!(pool_2.supply, DEPOSIT_AMOUNT);
    assert_eq!(pool_2.collateral, DEPOSIT_AMOUNT / 4);

    // For now liquidation must fail due to health factor being fine
    assert_eq!(
        contract_client.try_liquidate(&admin, &user, &pool_address, &10),
        Err(Ok(LCError::LiquidatedPositionIsHealthy))
    );

    // Let some time pass => increase the borrowed amount
    // WARN: For now, in order for this test to pass, at least 3 days must pass.
    // This is an issue for sure
    e.ledger().with_mut(|li| li.timestamp = 3 * 60 * 60 * 24);

    // Since liquidation spread is 50%, the liquidation must succeed
    contract_client.liquidate(&admin, &user, &pool_address, &(&DEPOSIT_AMOUNT / 2));

    // Check the pools state
    let pool_1 = contract_client.get_pool(&pool_address).unwrap();

    assert_eq!(pool_1.borrowed, DEPOSIT_AMOUNT / 2);
    assert!(pool_1.supply > DEPOSIT_AMOUNT); // should increase, since interest rate has been accrued

    let pool_2 = contract_client.get_pool(&pool_address_2).unwrap();

    assert!(DEPOSIT_AMOUNT > pool_2.supply); // since collateral has been sold
    assert_eq!(pool_2.collateral, DEPOSIT_AMOUNT / 4);
}

// #[test]
// fn test_liquidation_multiple_collaterals() {
//     const DEPOSIT_AMOUNT: i128 = 1_000;
//     let e = Env::default();

//     let TestEnv {
//         contract_client,
//         asset1:
//             TestAssetSetup {
//                 token_address,
//                 token_ticker,
//                 ..
//             },
//         asset2:
//             TestAssetSetup {
//                 token_address: token_address_2,
//                 token_ticker: token_ticker_2,
//                 ..
//             },
//         asset3:
//             TestAssetSetup {
//                 token_address: token_address_3,
//                 token_ticker: token_ticker_3,
//                 ..
//             },
//         user,
//         admin,
//         ..
//     } = setup_test_env(&e);

//     // Wouldn't it be better to initialize them in the `setup_test_env` right away?
//     let pool_address = contract_client.initialize_pool(&token_address, &token_ticker, &None, &None);
//     let pool_address_2 =
//         contract_client.initialize_pool(&token_address_2, &token_ticker_2, &None, &None);
//     let pool_address_3 =
//         contract_client.initialize_pool(&token_address_3, &token_ticker_3, &None, &None);

//     // Provide some liquidity in the first pool
//     contract_client.deposit(&admin, &pool_address, &(3 * DEPOSIT_AMOUNT));

//     // Deposit in the 2nd pool
//     contract_client.deposit(&user, &pool_address_2, &DEPOSIT_AMOUNT);
//     // Deposit collateral in the 2nd pool
//     contract_client.deposit_collateral(&user, &pool_address_2, &(DEPOSIT_AMOUNT / 4));

//     // Deposit in the 3rd pool
//     contract_client.deposit(&user, &pool_address_3, &(2 * DEPOSIT_AMOUNT));
//     // Deposit collateral in the 3rd pool
//     contract_client.deposit_collateral(&user, &pool_address_3, &(DEPOSIT_AMOUNT / 2));

//     // Borrow a maximal possible amount which will not cause a liquidation
//     contract_client.borrow(&user, &pool_address, &(3 * DEPOSIT_AMOUNT));

//     // Check pools state
//     let pool_1 = contract_client.get_pool(&pool_address).unwrap();

//     assert_eq!(pool_1.supply, 3 * DEPOSIT_AMOUNT);
//     assert_eq!(pool_1.borrowed, 2 * DEPOSIT_AMOUNT);

//     let pool_2 = contract_client.get_pool(&pool_address_2).unwrap();

//     assert_eq!(pool_2.supply, DEPOSIT_AMOUNT);
//     assert_eq!(pool_2.collateral, DEPOSIT_AMOUNT / 4);

//     let pool_3 = contract_client.get_pool(&pool_address_3).unwrap();

//     assert_eq!(pool_3.supply, 2 * DEPOSIT_AMOUNT);
//     assert_eq!(pool_3.collateral, DEPOSIT_AMOUNT / 2);

//     // For now liquidation must fail due to health factor being fine
//     assert_eq!(
//         contract_client.try_liquidate(&admin, &user, &pool_address, &10),
//         Err(Ok(LCError::LiquidatedPositionIsHealthy))
//     );

//     // Let some time pass => increase the borrowed amount
//     // WARN: For now, in order for this test to pass, at least 3 days must pass.
//     // This is an issue for sure
//     e.ledger().with_mut(|li| li.timestamp = 3 * 60 * 60 * 24);

//     // Liquidation must succeed
//     // contract_client.liquidate(&admin, &user, &pool_address, &(&DEPOSIT_AMOUNT / 2));
// }
