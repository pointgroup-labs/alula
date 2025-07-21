#![deny(clippy::absurd_extreme_comparisons)]

mod borrow;
mod deposit;
mod fuzz;
mod initialize;
mod interest_rates;
mod leverage;
mod liquidate;
mod repay;
mod swap;
mod withdraw;

use {
    arbitrary::Unstructured,
    lending::{
        constants::{INDIVIDUAL_BUMP, REFLECTOR_TESTNET_ADDRESS, SOROSWAP_ROUTER_TESTNET_ADDRESS},
        contract::{LendingContract, LendingContractClient},
        obligation::{BorrowObligation, DepositObligation},
        oracle,
        pool::PoolConfig,
        soroswap_router, LCError,
    },
    soroban_sdk::{
        symbol_short,
        testutils::{arbitrary::Arbitrary, Address as _, EnvTestConfig, Ledger},
        token::{self, StellarAssetClient, TokenClient},
        vec, Address, Env, Vec,
    },
};

pub const DEFAULT_DEPOSIT_AMOUNT: i128 = 50_000;
pub const DEFAULT_HEALTH_FACTOR_THRESHOLD: i128 = 80;
pub const DEFAULT_ADMIN_ASSET_MINT_AMOUNT: i128 = i128::MAX / 2;
pub const DEFAULT_USER_ASSET_MINT_AMOUNT: i128 = DEFAULT_ADMIN_ASSET_MINT_AMOUNT;
pub const DEFAULT_COLLATERAL_AMOUNT: i128 = DEFAULT_DEPOSIT_AMOUNT;

#[derive(Arbitrary, Debug, Clone, Copy)]
pub enum Token {
    BTC,
    USDC,
    GOLD,
}

pub struct TestFixture<'a> {
    pub e: Env,
    pub contract_client: LendingContractClient<'a>,
    pub contract_id: Address,
    pub contract_admin: Address,
    pub users: Vec<Address>,
    // Oracle
    pub oracle_client: oracle::Client<'a>,
    pub oracle_address: Address,
    // Swap Router
    pub soroswap_router_client: soroswap_router::Client<'a>,
    pub soroswap_router_address: Address,
    // GOLD
    pub gold_sac: StellarAssetClient<'a>,
    pub gold_token_client: TokenClient<'a>,
    pub gold_token_address: Address,
    pub gold_pool_address: Address,
    pub gold_admin: Address,
    // BTC
    pub btc_sac: StellarAssetClient<'a>,
    pub btc_token_client: TokenClient<'a>,
    pub btc_token_address: Address,
    pub btc_pool_address: Address,
    pub btc_admin: Address,
    // USDC
    pub usdc_sac: StellarAssetClient<'a>,
    pub usdc_token_client: TokenClient<'a>,
    pub usdc_token_address: Address,
    pub usdc_pool_address: Address,
    pub usdc_admin: Address,
}

#[allow(clippy::new_without_default)]
impl TestFixture<'_> {
    pub fn new() -> Self {
        let pool_config = Default::default();

        Self::new_with_pool_config(pool_config)
    }

    fn new_with_pool_config(pool_config: PoolConfig) -> Self {
        let e = Env::new_with_config(EnvTestConfig {
            capture_snapshot_at_drop: false,
        });
        e.mock_all_auths();
        // TODO: Think more about what sometimes happens in tests
        // when this is opted out
        e.mock_all_auths_allowing_non_root_auth();

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

        let oracle_address = Address::from_str(&e, REFLECTOR_TESTNET_ADDRESS);
        e.register_at(&oracle_address, oracle::WASM, ());
        let oracle_client = oracle::Client::new(&e, &oracle_address);

        let soroswap_router_address = Address::from_str(&e, SOROSWAP_ROUTER_TESTNET_ADDRESS);
        e.register_at(&soroswap_router_address, soroswap_router::WASM, ());
        let soroswap_router_client = soroswap_router::Client::new(&e, &soroswap_router_address);

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
            sac_client: gold_sac,
            token_client: gold_token_client,
            token_address: gold_token_address,
        } = setup_test_asset(&e, &gold_admin, &users);
        let gold_pool_address = contract_client.initialize_pool(
            &gold_token_address,
            &symbol_short!("GOLD"),
            &None,
            &Some(pool_config),
        );

        // BTC
        let TestAssetSetup {
            sac_client: btc_sac,
            token_client: btc_token_client,
            token_address: btc_token_address,
        } = setup_test_asset(&e, &btc_admin, &users);
        let btc_pool_address = contract_client.initialize_pool(
            &btc_token_address,
            &symbol_short!("BTC"),
            &None,
            &Some(pool_config),
        );

        // USDC
        let TestAssetSetup {
            sac_client: usdc_sac,
            token_client: usdc_token_client,
            token_address: usdc_token_address,
        } = setup_test_asset(&e, &usdc_admin, &users);
        let usdc_pool_address = contract_client.initialize_pool(
            &usdc_token_address,
            &symbol_short!("USDC"),
            &None,
            &Some(pool_config),
        );

        Self {
            e,
            contract_client,
            contract_id,
            contract_admin,
            // Oracle
            oracle_client,
            oracle_address,
            // Swap router
            soroswap_router_client,
            soroswap_router_address,
            // GOLD
            gold_sac,
            gold_token_client,
            gold_token_address,
            gold_pool_address,
            gold_admin,
            // BTC
            btc_sac,
            btc_token_client,
            btc_token_address,
            btc_pool_address,
            btc_admin,
            // USDC
            usdc_sac,
            usdc_token_client,
            usdc_token_address,
            usdc_pool_address,
            usdc_admin,
            users,
        }
    }

    pub fn get_pool_address(&self, token: Token) -> Address {
        let address = match token {
            Token::BTC => &self.btc_pool_address,
            Token::USDC => &self.usdc_pool_address,
            Token::GOLD => &self.gold_pool_address,
        };

        address.clone()
    }

    pub fn get_token_client(&self, token: Token) -> &TokenClient<'_> {
        match token {
            Token::BTC => &self.btc_token_client,
            Token::USDC => &self.usdc_token_client,
            Token::GOLD => &self.gold_token_client,
        }
    }
}

pub struct TestAssetSetup<'a> {
    pub token_client: TokenClient<'a>,
    pub token_address: Address,
    pub sac_client: StellarAssetClient<'a>,
}

pub fn setup_test_asset<'a>(e: &Env, admin: &Address, users: &Vec<Address>) -> TestAssetSetup<'a> {
    let token_address = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let sac_client = StellarAssetClient::new(e, &token_address);
    let token_client = TokenClient::new(e, &token_address);

    sac_client.mint(admin, &DEFAULT_ADMIN_ASSET_MINT_AMOUNT);

    for user in users {
        sac_client.mint(&user, &DEFAULT_USER_ASSET_MINT_AMOUNT);
    }

    TestAssetSetup {
        token_address,
        token_client,
        sac_client,
    }
}

#[derive(Arbitrary, Debug)]
pub enum Command {
    TomRepay(Repay),
    JerryRepay(Repay),

    TomBorrow(Borrow),
    JerryBorrow(Borrow),

    TomDeposit(Deposit),
    JerryDeposit(Deposit),

    TomWithdraw(Withdraw),
    JerryWithdraw(Withdraw),

    TomLiquidate(Liquidate),
    JerryLiquidate(Liquidate),

    TomDepositCollateral(DepositCollateral),
    JerryDepositCollateral(DepositCollateral),

    TomWithdrawCollateral(WithdrawCollateral),
    JerryWithdrawCollateral(WithdrawCollateral),

    TomDepositWithLeverage(DepositWithLeverage),
    JerryDepositWithLeverage(DepositWithLeverage),

    TomDeleverageAndWithdraw(DeleverageAndWithdraw),
    JerryDeleverageAndWithdraw(DeleverageAndWithdraw),
    // PassTime(),
}

impl Command {
    pub fn run(&self, test_fixture: &TestFixture) {
        match self {
            Command::TomRepay(command) => command.run(test_fixture, 0),
            Command::TomBorrow(command) => command.run(test_fixture, 0),
            Command::TomDeposit(command) => command.run(test_fixture, 0),
            Command::TomWithdraw(command) => command.run(test_fixture, 0),
            Command::TomLiquidate(command) => command.run(test_fixture, 0),
            Command::TomDepositCollateral(command) => command.run(test_fixture, 0),
            Command::TomWithdrawCollateral(command) => command.run(test_fixture, 0),
            Command::TomDepositWithLeverage(command) => command.run(test_fixture, 0),
            Command::TomDeleverageAndWithdraw(command) => command.run(test_fixture, 0),

            Command::JerryRepay(command) => command.run(test_fixture, 1),
            Command::JerryBorrow(command) => command.run(test_fixture, 1),
            Command::JerryDeposit(command) => command.run(test_fixture, 1),
            Command::JerryWithdraw(command) => command.run(test_fixture, 1),
            Command::JerryLiquidate(command) => command.run(test_fixture, 1),
            Command::JerryDepositCollateral(command) => command.run(test_fixture, 1),
            Command::JerryWithdrawCollateral(command) => command.run(test_fixture, 1),
            Command::JerryDepositWithLeverage(command) => command.run(test_fixture, 1),
            Command::JerryDeleverageAndWithdraw(command) => command.run(test_fixture, 1),
        }
    }
}

#[derive(Arbitrary, Debug)]
pub struct Amount(
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=(u64::MAX as i128)))] pub i128,
);

pub fn assert_invariants(fixture: &TestFixture) {
    let TestFixture {
        e,
        contract_client,
        contract_id,
        gold_sac,
        gold_pool_address,
        gold_token_address,
        btc_sac,
        btc_pool_address,
        btc_token_address,
        usdc_sac,
        usdc_pool_address,
        usdc_token_address,
        users,
        ..
    } = fixture;

    // Get all pools
    let usdc_pool = contract_client.get_pool(usdc_pool_address);
    let gold_pool = contract_client.get_pool(gold_pool_address);
    let btc_pool = contract_client.get_pool(btc_pool_address);

    // Basic non-negative invariants
    // All data on all pools must be non-negative
    assert!(
        usdc_pool.total_shares >= 0,
        "USDC pool total_supply must be non-negative"
    );
    assert!(
        usdc_pool.total_borrowed >= 0,
        "USDC pool total_borrowed must be non-negative"
    );
    assert!(
        usdc_pool.total_collateral >= 0,
        "USDC pool total_collateral must be non-negative"
    );

    assert!(
        gold_pool.total_shares >= 0,
        "GOLD pool total_supply must be non-negative"
    );
    assert!(
        gold_pool.total_borrowed >= 0,
        "GOLD pool total_borrowed must be non-negative"
    );
    assert!(
        gold_pool.total_collateral >= 0,
        "GOLD pool total_collateral must be non-negative"
    );

    assert!(
        btc_pool.total_shares >= 0,
        "BTC pool total_supply must be non-negative"
    );
    assert!(
        btc_pool.total_borrowed >= 0,
        "BTC pool total_borrowed must be non-negative"
    );
    assert!(
        btc_pool.total_collateral >= 0,
        "BTC pool total_collateral must be non-negative"
    );

    // Token balance invariants
    // Contract's token balances should match the total supply and collateral in each pool
    let usdc_token_client = token::Client::new(e, usdc_token_address);
    let gold_token_client = token::Client::new(e, gold_token_address);
    let btc_token_client = token::Client::new(e, btc_token_address);

    let usdc_contract_balance = usdc_token_client.balance(contract_id);
    let gold_contract_balance = gold_token_client.balance(contract_id);
    let btc_contract_balance = btc_token_client.balance(contract_id);

    let usdc_expected_balance = usdc_pool.available + usdc_pool.total_collateral;
    let gold_expected_balance = gold_pool.available + gold_pool.total_collateral;
    let btc_expected_balance = btc_pool.available + btc_pool.total_collateral;

    assert_eq!(
        usdc_contract_balance, usdc_expected_balance,
        "USDC contract balance must match pool totals"
    );
    assert_eq!(
        gold_contract_balance, gold_expected_balance,
        "GOLD contract balance must match pool totals"
    );
    assert_eq!(
        btc_contract_balance, btc_expected_balance,
        "BTC contract balance must match pool totals"
    );

    // Health factor invariants
    // Check that all users with obligations have a health factor above the threshold
    for user in users.iter() {
        let obligation_result = contract_client.try_get_user_obligation(&user);
        if let Ok(Ok(obligation)) = obligation_result {
            // If user has an obligation, check that it's healthy
            let is_healthy =
                e.as_contract(contract_id, || obligation.is_healthy(e).unwrap_or(true));
            assert!(is_healthy, "User obligation must be healthy");
        }
    }

    // Functional invariants
    // You can always borrow and repay the available amount
    let new_borrower = Address::generate(e);

    let collateral_amount = 2 * i128::max(
        gold_pool.available,
        i128::max(usdc_pool.available, btc_pool.available),
    );

    usdc_sac.mint(&new_borrower, &collateral_amount);
    btc_sac.mint(&new_borrower, &collateral_amount);
    gold_sac.mint(&new_borrower, &collateral_amount);

    // Test borrowing and repaying for each pool if there are available funds
    if btc_pool.available > 0 {
        let available_borrow = btc_pool.compute_available_borrow(e).unwrap();

        contract_client.add_collateral(&new_borrower, usdc_pool_address, &collateral_amount);
        contract_client.borrow(&new_borrower, btc_pool_address, &available_borrow);

        // Verify the borrowed amount is reflected in the user's obligation
        let obligation = contract_client.get_user_obligation(&new_borrower);
        let borrow_obligation = obligation
            .borrows
            .get(btc_pool_address.clone())
            .expect("Borrow obligation must be present");

        assert_eq!(
            borrow_obligation.borrowed, available_borrow,
            "Borrowed amount must match in user obligation"
        );

        contract_client.repay(&new_borrower, btc_pool_address, &available_borrow);
        contract_client.remove_collateral(&new_borrower, usdc_pool_address, &collateral_amount);
    }

    if gold_pool.available > 0 {
        let available_borrow = gold_pool.compute_available_borrow(e).unwrap();

        contract_client.add_collateral(&new_borrower, usdc_pool_address, &collateral_amount);
        contract_client.borrow(&new_borrower, gold_pool_address, &available_borrow);

        // Verify the borrowed amount is reflected in the user's obligation
        let obligation = contract_client.get_user_obligation(&new_borrower);
        let borrow_obligation = obligation
            .borrows
            .get(gold_pool_address.clone())
            .expect("Borrow obligation must be present");

        assert_eq!(
            borrow_obligation.borrowed, available_borrow,
            "Borrowed amount must match in user obligation"
        );

        contract_client.repay(&new_borrower, gold_pool_address, &available_borrow);
        contract_client.remove_collateral(&new_borrower, usdc_pool_address, &collateral_amount);
    }

    if usdc_pool.available > 0 {
        let available_borrow = usdc_pool.compute_available_borrow(e).unwrap();

        contract_client.add_collateral(&new_borrower, gold_pool_address, &collateral_amount);
        contract_client.borrow(&new_borrower, usdc_pool_address, &available_borrow);

        // Verify the borrowed amount is reflected in the user's obligation
        let obligation = contract_client.get_user_obligation(&new_borrower);
        let borrow_obligation = obligation
            .borrows
            .get(usdc_pool_address.clone())
            .expect("Borrow obligation must be present");

        assert_eq!(
            borrow_obligation.borrowed, available_borrow,
            "Borrowed amount must match in user obligation"
        );

        contract_client.repay(&new_borrower, usdc_pool_address, &available_borrow);
        contract_client.remove_collateral(&new_borrower, gold_pool_address, &collateral_amount);
    }

    // 6. Interest rate invariants
    // Verify that interest rates are calculated correctly
    let usdc_apy = contract_client.get_apy(usdc_pool_address);
    let gold_apy = contract_client.get_apy(gold_pool_address);
    let btc_apy = contract_client.get_apy(btc_pool_address);

    // Borrow interest rates should be non-negative
    assert!(
        usdc_apy.borrow_bps > 0,
        "USDC borrow rate must be non-negative"
    );
    assert!(
        gold_apy.borrow_bps > 0,
        "GOLD borrow rate must be non-negative"
    );
    assert!(
        btc_apy.borrow_bps > 0,
        "BTC borrow rate must be non-negative"
    );

    // Borrow rate should be greater than or equal to deposit rate
    assert!(
        usdc_apy.borrow_bps >= usdc_apy.supply_bps,
        "USDC borrow rate must be >= deposit rate"
    );
    assert!(
        gold_apy.borrow_bps >= gold_apy.supply_bps,
        "GOLD borrow rate must be >= deposit rate"
    );
    assert!(
        btc_apy.borrow_bps >= btc_apy.supply_bps,
        "BTC borrow rate must be >= deposit rate"
    );
}

#[derive(Arbitrary, Debug)]
pub struct Input {
    pub commands: [Command; 10],
}

#[derive(Arbitrary, Debug)]
pub struct Repay {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Borrow {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Deposit {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Withdraw {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]

pub struct DepositCollateral {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct WithdrawCollateral {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Liquidate {
    pub amount: Amount,
    pub token: Token,
    pub collateral_token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct DepositWithLeverage {
    pub amount: Amount,
    pub deposit_token: Token,
    pub borrow_token: Token,
    pub flash_loan_amount: Amount,
    pub leverage: u32,
}

#[derive(Arbitrary, Debug)]
pub struct DeleverageAndWithdraw {
    pub amount: Amount,
    pub deposit_token: Token,
    pub borrow_token: Token,
}

impl Borrow {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_borrow(&user, &pool_address, &self.amount.0);
    }
}

impl Deposit {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_deposit(&user, &pool_address, &self.amount.0);
    }
}

impl DepositCollateral {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_add_collateral(&user, &pool_address, &self.amount.0);
    }
}

impl WithdrawCollateral {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_remove_collateral(&user, &pool_address, &self.amount.0);
    }
}

impl Withdraw {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_withdraw(&user, &pool_address, &self.amount.0);
    }
}

impl Repay {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_repay(&user, &pool_address, &self.amount.0);
    }
}

impl Liquidate {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let collateral_pool_address = test_fixture.get_pool_address(self.collateral_token);

        if pool_address != collateral_pool_address {
            let TestFixture {
                contract_client,
                users,
                ..
            } = test_fixture;

            let (liquidator, borrower) = if who == 0 {
                (users.get(0).unwrap(), users.get(1).unwrap())
            } else {
                (users.get(1).unwrap(), users.get(0).unwrap())
            };

            let _ = contract_client.try_liquidate(
                &liquidator,
                &borrower,
                &pool_address,
                &collateral_pool_address,
                &self.amount.0,
            );
        }
    }
}

impl DepositWithLeverage {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let deposit_pool_address = test_fixture.get_pool_address(self.deposit_token);
        let borrow_pool_address = test_fixture.get_pool_address(self.borrow_token);

        if deposit_pool_address != borrow_pool_address {
            let TestFixture {
                contract_client,
                users,
                ..
            } = test_fixture;

            let (flash_loan_provider, lender) = if who == 0 {
                (users.get(0).unwrap(), users.get(1).unwrap())
            } else {
                (users.get(1).unwrap(), users.get(0).unwrap())
            };

            contract_client.deposit(
                &flash_loan_provider,
                &borrow_pool_address,
                &self.flash_loan_amount.0,
            );

            let _ = contract_client.try_deposit_with_leverage(
                &lender,
                &deposit_pool_address,
                &borrow_pool_address,
                &self.amount.0,
                &self.leverage,
            );
        }
    }
}

impl DeleverageAndWithdraw {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let deposit_pool_address = test_fixture.get_pool_address(self.deposit_token);
        let borrow_pool_address = test_fixture.get_pool_address(self.borrow_token);

        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_deleverage_and_withdraw(
            &user,
            &deposit_pool_address,
            &borrow_pool_address,
            &self.amount.0,
        );
    }
}

#[allow(unused)]
pub fn get_obligation_shares(
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, LCError> {
    let deposit_obligation = get_deposit_obligation(contract_client, user, pool_address)?;

    Ok(deposit_obligation.shares)
}

#[allow(unused)]
pub fn get_obligation_tokens_from_shares(
    e: &Env,
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, LCError> {
    let shares = get_obligation_shares(contract_client, user, pool_address)?;

    let pool = contract_client.get_pool(pool_address);

    pool.compute_tokens_from_shares(e, shares)
}

#[allow(unused)]
pub fn get_obligation_borrowed(
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, LCError> {
    let borrow_obligation = get_borrow_obligation(contract_client, user, pool_address)?;

    Ok(borrow_obligation.borrowed)
}

#[allow(unused)]
pub fn get_obligation_collateral(
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, LCError> {
    let deposit_obligation = get_deposit_obligation(contract_client, user, pool_address)?;

    Ok(deposit_obligation.collateral)
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
        .ok_or(LCError::BorrowDoesNotExist)?;

    Ok(borrow)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        lending::{
            constants::{BPS_FACTOR, INDIVIDUAL_BUMP, INSTANCE_BUMP, LEDGERS_PER_DAY, SHARED_BUMP},
            storage::DataKey,
        },
        soroban_fixed_point_math::FixedPoint,
        soroban_sdk::testutils::{
            storage::{Instance, Persistent},
            Ledger,
        },
    };

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

    pub fn get_amount_scaled_down(amount: i128, scale_bps: i128) -> i128 {
        amount
            .checked_sub(amount.fixed_div_floor(BPS_FACTOR, scale_bps).unwrap())
            .unwrap()
    }

    pub fn get_amount_scaled_up(amount: i128, scale_bps: i128) -> i128 {
        amount
            .checked_add(amount.fixed_div_floor(BPS_FACTOR, scale_bps).unwrap())
            .unwrap()
    }
}
