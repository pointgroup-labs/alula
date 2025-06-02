use {
    lending::{
        constants::{
            LCError, INDIVIDUAL_BUMP, INSTANCE_BUMP, LEDGERS_PER_DAY, REFLECTOR_TESTNET_ADDRESS,
            SHARED_BUMP,
        },
        contract::{LendingContract, LendingContractClient},
        oracle,
        storage::{BorrowObligation, DataKey, DepositObligation},
    },
    soroban_sdk::{
        symbol_short,
        testutils::{
            storage::{Instance, Persistent},
            Address as _, Ledger,
        },
        token::{StellarAssetClient, TokenClient},
        vec, Address, Env, String, Vec,
    },
};

pub const DEFAULT_HEALTH_FACTOR_THRESHOLD: i128 = 80;
pub const DEFAULT_ADMIN_ASSET_MINT_AMOUNT: i128 = 1_000_000;
pub const DEFAULT_USER_ASSET_MINT_AMOUNT: i128 = 100_000;
pub const DEFAULT_DEPOSIT_AMOUNT: i128 = DEFAULT_USER_ASSET_MINT_AMOUNT / 2;
#[allow(unused)]
pub const DEFAULT_COLLATERAL_AMOUNT: i128 = DEFAULT_USER_ASSET_MINT_AMOUNT / 2;

#[allow(unused)]
pub struct TestFixture<'a> {
    pub e: Env,
    pub contract_client: LendingContractClient<'a>,
    pub contract_id: Address,
    pub contract_admin: Address,
    // GOLD
    pub gold_client: TokenClient<'a>,
    pub gold_token_address: Address,
    pub gold_admin: Address,
    pub gold_pool_address: Address,
    // BTC
    pub btc_client: TokenClient<'a>,
    pub btc_token_address: Address,
    pub btc_admin: Address,
    pub btc_pool_address: Address,
    // USDC
    pub usdc_client: TokenClient<'a>,
    pub usdc_token_address: Address,
    pub usdc_admin: Address,
    pub usdc_pool_address: Address,
    pub users: Vec<Address>,
}

impl<'a> TestFixture<'a> {
    pub fn new() -> Self {
        let e = Env::default();
        e.mock_all_auths();

        e.ledger().with_mut(|li| {
            li.sequence_number = 0;
            li.max_entry_ttl = INDIVIDUAL_BUMP + 1;
        });

        let contract_admin = Address::generate(&e);
        let contract_id = e.register(
            LendingContract,
            (
                contract_admin.clone(),
                Option::<i128>::Some(DEFAULT_HEALTH_FACTOR_THRESHOLD),
            ),
        );
        let contract_client = LendingContractClient::new(&e, &contract_id);

        let users = vec![
            &e,
            Address::generate(&e),
            Address::generate(&e),
            Address::generate(&e),
        ];

        let usdc_admin = Address::generate(&e);
        let gold_admin = Address::generate(&e);
        let btc_admin = Address::generate(&e);

        // GOLD
        let TestAssetSetup {
            token_client: gold_client,
            token_address: gold_token_address,
        } = setup_test_asset(&e, &gold_admin, &users);
        let gold_pool_address = contract_client.initialize_pool(
            &gold_token_address,
            &symbol_short!("GOLD"),
            &None,
            &None,
        );

        // BTC
        let TestAssetSetup {
            token_client: btc_client,
            token_address: btc_token_address,
        } = setup_test_asset(&e, &btc_admin, &users);
        let btc_pool_address = contract_client.initialize_pool(
            &btc_token_address,
            &symbol_short!("BTC"),
            &None,
            &None,
        );

        // USDC
        let TestAssetSetup {
            token_client: usdc_client,
            token_address: usdc_token_address,
        } = setup_test_asset(&e, &usdc_admin, &users);
        let usdc_pool_address = contract_client.initialize_pool(
            &usdc_token_address,
            &symbol_short!("USDC"),
            &None,
            &None,
        );

        let mock_oracle_address =
            Address::from_string(&String::from_str(&e, REFLECTOR_TESTNET_ADDRESS));
        e.register_at(&mock_oracle_address, oracle::WASM, ());

        Self {
            e,
            contract_client,
            contract_id,
            contract_admin,
            // GOLD
            gold_client,
            gold_token_address,
            gold_admin,
            gold_pool_address,
            // BTC
            btc_client,
            btc_token_address,
            btc_pool_address,
            btc_admin,
            // USDC
            usdc_client,
            usdc_token_address,
            usdc_pool_address,
            usdc_admin,
            users,
        }
    }
}

pub struct TestAssetSetup<'a> {
    token_client: TokenClient<'a>,
    token_address: Address,
}

pub fn setup_test_asset<'a>(e: &Env, admin: &Address, users: &Vec<Address>) -> TestAssetSetup<'a> {
    let token_address = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let asset_client = StellarAssetClient::new(e, &token_address);
    let token_client = TokenClient::new(e, &token_address);

    asset_client.mint(admin, &DEFAULT_ADMIN_ASSET_MINT_AMOUNT);

    for user in users {
        asset_client.mint(&user, &DEFAULT_USER_ASSET_MINT_AMOUNT);
    }

    TestAssetSetup {
        token_address,
        token_client,
    }
}

#[allow(unused)]
pub fn get_deposit_obligation(
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<DepositObligation, LCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_user_obligation(user) else {
        return Err(LCError::ObligationDoesNotExist);
    };

    let deposit = obligation
        .deposits
        .get(pool_address.clone())
        .ok_or(LCError::DepositDoesNotExist)?;

    Ok(deposit)
}

#[allow(unused)]
pub fn get_borrow_obligation(
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<BorrowObligation, LCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_user_obligation(user) else {
        return Err(LCError::ObligationDoesNotExist);
    };

    let borrow = obligation
        .borrows
        .get(pool_address.clone())
        .ok_or(LCError::DepositDoesNotExist)?;

    Ok(borrow)
}

#[test]
fn test_storage_ttl_extension() {
    let TestFixture {
        e,
        contract_client,
        contract_id,
        usdc_pool_address,
        users,
        ..
    } = TestFixture::new();

    let user = users.get(0).unwrap();

    e.as_contract(&contract_id, || {
        // `TestFixture::new()` extends both instance and a specific's pool shared storage
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP
        );
    });

    // Extend individual user's storage
    contract_client.deposit(&user, &usdc_pool_address, &1);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(user.clone())),
            INDIVIDUAL_BUMP
        );
    });

    e.ledger().with_mut(|li| {
        // TODO: Make all shifts depend on the threshold
        // and not on the constant amount of ledgers
        li.sequence_number = 2 * LEDGERS_PER_DAY;
    });

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage().instance().get_ttl(),
            INSTANCE_BUMP - 2 * LEDGERS_PER_DAY
        );
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP - 2 * LEDGERS_PER_DAY
        );

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(user.clone())),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });

    // Extend instance storage
    contract_client.get_global_state();

    e.as_contract(&contract_id, || {
        // Instance's ttl is bumped
        assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);

        // Others aren't bumped
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP - 2 * LEDGERS_PER_DAY
        );

        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(user.clone())),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });

    // Deposit once more to bump shared persistent token storage
    contract_client.deposit(&user, &usdc_pool_address, &1);

    e.as_contract(&contract_id, || {
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
            SHARED_BUMP
        );

        // Individual persistent storage ttl is still the same
        assert_eq!(
            e.storage()
                .persistent()
                .get_ttl(&DataKey::Obligation(user.clone())),
            INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
        );
    });

    // TODO: Add individual storage extension test case
}
