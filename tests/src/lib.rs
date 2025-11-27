mod bad_debt;
mod borrow;
mod deposit;
mod fees;
mod fuzz;
mod initialize;
mod interest_rates;
mod leverage;
mod liquidate;
mod market_manager;
mod misc;
mod repay;
mod requests_batching;
mod security;
mod storage_extension;
mod swap;
mod update;
mod withdraw;

use std::ops::{Add, Sub};

use arbitrary::Unstructured;
use market::{
    constants::{
        BPS_FACTOR, DEFAULT_INSOLVENCY_LTV_BPS, DEFAULT_MAX_POSITIONS,
        DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS, INDIVIDUAL_BUMP, ROUTER_ADDRESS,
    },
    contract::{MarketClient, MarketContract},
    error::MCError,
    math_utils::MathUtils,
    obligation::{BorrowPosition, DepositPosition},
    pool::{PoolConfig, PoolFeeConfig},
    soroswap_router as router,
};
use sep_40_oracle::testutils::{Asset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{
    Address, Env, Symbol,
    testutils::{Address as _, Ledger, LedgerInfo, arbitrary::Arbitrary},
    token::{self, StellarAssetClient, TokenClient},
};

pub const DEFAULT_DEPOSIT_AMOUNT: i128 = 50_000;
pub const DEFAULT_COLLATERAL_AMOUNT: i128 = DEFAULT_DEPOSIT_AMOUNT;
pub const DEFAULT_ADMIN_ASSET_MINT_AMOUNT: i128 = i128::MAX / 1024;
pub const DEFAULT_USER_ASSET_MINT_AMOUNT: i128 = DEFAULT_ADMIN_ASSET_MINT_AMOUNT;

const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

#[derive(Arbitrary, Debug, Clone, Copy)]
pub enum Token {
    BTC,
    USDC,
    GOLD,
}

pub struct TestMarketFixture<'a> {
    pub e: Env,
    pub contract_client: MarketClient<'a>,
    pub contract_id: Address,
    pub contract_admin: Address,
    pub users: Vec<Address>,
    // Oracle
    pub oracle_client: MockPriceOracleClient<'a>,
    pub oracle_address: Address,
    // Swap Router
    pub router_client: router::Client<'a>,
    pub router_address: Address,
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
impl TestMarketFixture<'_> {
    pub fn new() -> Self {
        let pool_config = Default::default();

        Self::new_with_pool_config(pool_config)
    }

    fn new_with_pool_config(pool_config: PoolConfig) -> Self {
        let e = get_default_env();
        e.mock_all_auths_allowing_non_root_auth(); // TODO: Test flash loans on testnet

        e.ledger().set(LedgerInfo {
            timestamp: 1590969600, // June 1, 2020
            protocol_version: 23,
            sequence_number: 1000,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 500000,
            min_persistent_entry_ttl: 500000,
            max_entry_ttl: INDIVIDUAL_BUMP + 1,
        });

        let users = vec![
            Address::generate(&e),
            Address::generate(&e),
            Address::generate(&e),
            Address::generate(&e),
        ];

        // Configure USDC SAC first, since it's used in the oracle as a base asset
        let usdc_admin = Address::generate(&e);
        let TestAssetSetup {
            sac_client: usdc_sac,
            token_client: usdc_token_client,
            token_address: usdc_token_address,
        } = setup_test_asset(&e, &usdc_admin, &users);

        let oracle_address = Address::from_str(&e, ORACLE_ADDRESS);
        e.register_at(&oracle_address, MockPriceOracleWASM, ());
        let oracle_client = MockPriceOracleClient::new(&e, &oracle_address);

        // Register Market contract
        let contract_admin = Address::generate(&e);
        let market_manager_address = Address::generate(&e);
        let contract_name = soroban_sdk::String::from_str(&e, "market_contract");
        let contract_id = e.register(
            MarketContract,
            (
                contract_name,
                contract_admin.clone(),
                oracle_address.clone(),
                market_manager_address,
                DEFAULT_MAX_POSITIONS,
                0i128,
                DEFAULT_INSOLVENCY_LTV_BPS,
                Some(DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS),
            ),
        );
        let contract_client = MarketClient::new(&e, &contract_id);

        contract_client.update_market_status(&0);

        let router_address = Address::from_str(&e, ROUTER_ADDRESS);
        e.register_at(&router_address, router::WASM, (usdc_token_address.clone(),));
        let router_client = router::Client::new(&e, &router_address);

        // GOLD
        let gold_admin = Address::generate(&e);
        let TestAssetSetup {
            sac_client: gold_sac,
            token_client: gold_token_client,
            token_address: gold_token_address,
        } = setup_test_asset(&e, &gold_admin, &users);
        let gold_pool_address =
            contract_client.initialize_pool(&gold_token_address, &None, &Some(pool_config));

        // BTC
        let btc_admin = Address::generate(&e);
        let TestAssetSetup {
            sac_client: btc_sac,
            token_client: btc_token_client,
            token_address: btc_token_address,
        } = setup_test_asset(&e, &btc_admin, &users);
        let btc_pool_address =
            contract_client.initialize_pool(&btc_token_address, &None, &Some(pool_config));

        // USDC
        let usdc_pool_address =
            contract_client.initialize_pool(&usdc_token_address, &None, &Some(pool_config));

        contract_client.initialize_multiply_pair(&gold_pool_address, &usdc_pool_address);

        oracle_client.set_data(
            &contract_admin,
            &Asset::Other(Symbol::new(&e, "USD")),
            &soroban_sdk::vec![
                &e,
                Asset::Stellar(gold_pool_address.clone()),
                Asset::Stellar(btc_pool_address.clone()),
                Asset::Stellar(usdc_pool_address.clone()),
            ],
            &14,
            &123, // NB: Resolution is irrelevant because of using the stable prices
        );
        make_oracle_prices_equal(&e, &oracle_client);

        Self {
            e,
            contract_client,
            contract_id,
            contract_admin,
            // Oracle
            oracle_client,
            oracle_address,
            // Swap router
            router_client,
            router_address,
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

    pub fn get_token_sac(&self, token: Token) -> &StellarAssetClient<'_> {
        match token {
            Token::BTC => &self.btc_sac,
            Token::USDC => &self.usdc_sac,
            Token::GOLD => &self.gold_sac,
        }
    }

    pub fn pass_time(&self, seconds: u64) {
        self.e.ledger().with_mut(|li| {
            li.timestamp = li.timestamp.saturating_add(seconds);
        });
    }

    pub fn assert_invariants(&self) {
        let TestMarketFixture {
            e,
            contract_client,
            contract_id,
            gold_sac,
            gold_pool_address,
            btc_sac,
            btc_pool_address,
            usdc_sac,
            usdc_pool_address,
            users,
            ..
        } = self;

        let pools = vec![
            contract_client.get_pool(usdc_pool_address),
            contract_client.get_pool(btc_pool_address),
            contract_client.get_pool(gold_pool_address),
        ];
        let clients =
            pools.iter().map(|pool| token::Client::new(e, &pool.token_address)).collect::<Vec<_>>();

        // -- Pool data must be non-negative --

        for pool in &pools {
            assert!(pool.total_borrowed >= 0);
            assert!(pool.total_collateral >= 0);
            assert!(pool.total_j_tokens >= 0);
            assert!(pool.total_d_tokens >= 0);
        }

        // -- Contract's token balances shouldn't be smaller than the corresponding `available` + fees values on pools --

        let token_balances: Vec<i128> =
            clients.iter().map(|client| client.balance(contract_id)).collect();

        for (pool, &token_balance) in pools.iter().zip(token_balances.iter()) {
            // Calculate the Total Liabilities of the protocol (User Liquidity + Admin Revenue)
            let expected_minimum_balance = pool
                .total_available
                .checked_add(pool.accumulated_market_fees)
                .expect("Overflow in invariant calc")
                .checked_add(pool.accumulated_host_fees)
                .expect("Overflow in invariant calc")
                .checked_add(pool.accumulated_reserve_fees)
                .expect("Overflow in invariant calc");

            assert!(
                token_balance >= expected_minimum_balance,
                "INSOLVENCY DETECTED in Pool {:?}: Physical Balance ({}) < Net Available + Fees \
                 ({})",
                pool.pool_address,
                token_balance,
                expected_minimum_balance
            );
        }

        // -- It must be always possible to borrow what's available on the pool --

        let new_borrower = Address::generate(e);

        let collateral_amount = pools
            .iter()
            .max_by(|x, y| x.total_available.cmp(&y.total_available))
            .unwrap()
            .total_available()
            .unwrap();

        usdc_sac.mint(&new_borrower, &(2 * collateral_amount));
        btc_sac.mint(&new_borrower, &(2 * collateral_amount));
        gold_sac.mint(&new_borrower, &(2 * collateral_amount));

        let multiply_pairs = contract_client.get_all_multiply_pairs();

        for pool in &pools {
            let (mut j_tokens_obligations_sum, mut d_tokens_obligations_sum) = (0_i128, 0_i128);

            for user in users {
                if let Ok(Ok(obligation)) = contract_client.try_get_user_obligation(user) {
                    if let Some(deposit_position) =
                        obligation.deposits.get(pool.pool_address.clone())
                    {
                        j_tokens_obligations_sum += deposit_position.j_tokens;
                    }

                    if let Some(borrow_position) = obligation.borrows.get(pool.pool_address.clone())
                    {
                        d_tokens_obligations_sum += borrow_position.d_tokens;
                    }
                }

                if let Ok(Ok(earn_obligation)) = contract_client.try_get_earn_user_obligation(user)
                {
                    if let Some(deposit_position) =
                        earn_obligation.deposits.get(pool.pool_address.clone())
                    {
                        j_tokens_obligations_sum += deposit_position.j_tokens;
                    }

                    if let Some(borrow_position) =
                        earn_obligation.borrows.get(pool.pool_address.clone())
                    {
                        d_tokens_obligations_sum += borrow_position.d_tokens;
                    }
                }

                for mp in &multiply_pairs {
                    if mp.deposit_pool == pool.pool_address {
                        if let Ok(Ok(mp_obligation)) = contract_client
                            .try_get_multiply_pair_obligation(
                                user,
                                &mp.deposit_pool,
                                &mp.borrow_pool,
                            )
                            && let Some(deposit_position) =
                                mp_obligation.deposits.get(pool.pool_address.clone())
                        {
                            j_tokens_obligations_sum += deposit_position.j_tokens;
                        }
                    } else if mp.borrow_pool == pool.pool_address
                        && let Ok(Ok(mp_obligation)) = contract_client
                            .try_get_multiply_pair_obligation(
                                user,
                                &mp.deposit_pool,
                                &mp.borrow_pool,
                            )
                        && let Some(borrow_position) =
                            mp_obligation.borrows.get(pool.pool_address.clone())
                    {
                        d_tokens_obligations_sum += borrow_position.d_tokens;
                    }
                }
            }

            contract_client.refresh_pool(&pool.pool_address);
            let pool = contract_client.get_pool(&pool.pool_address);

            assert_eq!(pool.total_j_tokens, j_tokens_obligations_sum);
            assert_eq!(pool.total_d_tokens, d_tokens_obligations_sum);

            let available_borrow = pool.compute_available_utilization_ratio_cap_borrow(e).unwrap();

            contract_client.add_collateral(&new_borrower, &pool.token_address, &collateral_amount);
            contract_client.borrow(&new_borrower, &pool.token_address, &available_borrow);

            contract_client.repay(&new_borrower, &pool.token_address, &available_borrow);
            contract_client.remove_collateral(
                &new_borrower,
                &pool.token_address,
                &collateral_amount,
            );

            if let Ok(Ok(mut obligation)) = contract_client.try_get_user_obligation(&new_borrower) {
                // NB: If borrower's obligation still not closed due to unfavorable roundings - remove it manually to preserve invariants
                let Some(borrow_position) = obligation.borrows.get(pool.pool_address.clone())
                else {
                    continue;
                };

                let mut pool = contract_client.get_pool(&pool.pool_address);
                pool.adjust_total_d_tokens(
                    e,
                    borrow_position.d_tokens.checked_neg().map_over_or_underflow().unwrap(),
                )
                .unwrap();
                e.as_contract(contract_id, || {
                    obligation.try_remove_borrow_position(e, &pool.pool_address.clone()).unwrap();
                    pool.set(e);
                })
            }
        }
    }
}

// ---- Fuzzing suite ----

pub trait RunCommand {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize);
}

// TODO: Macro?
#[derive(Arbitrary, Debug)]
pub enum Command {
    TomRepay(Repay),
    JerryRepay(Repay),
    ButchRepay(Repay),
    NibblesRepay(Repay),

    TomBorrow(Borrow),
    JerryBorrow(Borrow),
    ButchBorrow(Borrow),
    NibblesBorrow(Borrow),

    TomDeposit(Deposit),
    JerryDeposit(Deposit),
    ButchDeposit(Deposit),
    NibblesDeposit(Deposit),

    TomWithdraw(Withdraw),
    JerryWithdraw(Withdraw),
    ButchWithdraw(Withdraw),
    NibblesWithdraw(Withdraw),

    TomLiquidate(Liquidate),
    JerryLiquidate(Liquidate),
    ButchLiquidate(Liquidate),
    NibblesLiquidate(Liquidate),

    TomDepositCollateral(DepositCollateral),
    JerryDepositCollateral(DepositCollateral),
    ButchDepositCollateral(DepositCollateral),
    NibblesDepositCollateral(DepositCollateral),

    TomWithdrawCollateral(WithdrawCollateral),
    JerryWithdrawCollateral(WithdrawCollateral),
    ButchWithdrawCollateral(WithdrawCollateral),
    NibblesWithdrawCollateral(WithdrawCollateral),

    TomDepositWithLeverage(DepositWithLeverage),
    JerryDepositWithLeverage(DepositWithLeverage),
    ButchDepositWithLeverage(DepositWithLeverage),
    NibblesDepositWithLeverage(DepositWithLeverage),

    TomWithdrawFromLeveraged(WithdrawFromLeveraged),
    JerryWithdrawFromLeveraged(WithdrawFromLeveraged),
    ButchWithdrawFromLeveraged(WithdrawFromLeveraged),
    NibblesWithdrawFromLeveraged(WithdrawFromLeveraged),

    AllPassTime(PassTime),
}

impl Command {
    pub fn run(&self, test_fixture: &TestMarketFixture) {
        use Command::*;

        match self {
            // Tom
            TomRepay(command) => command.run(test_fixture, 0),
            TomBorrow(command) => command.run(test_fixture, 0),
            TomDeposit(command) => command.run(test_fixture, 0),
            TomWithdraw(command) => command.run(test_fixture, 0),
            TomLiquidate(command) => command.run(test_fixture, 0),
            TomDepositCollateral(command) => command.run(test_fixture, 0),
            TomWithdrawCollateral(command) => command.run(test_fixture, 0),
            TomDepositWithLeverage(command) => command.run(test_fixture, 0),
            TomWithdrawFromLeveraged(command) => command.run(test_fixture, 0),
            // Jerry
            JerryRepay(command) => command.run(test_fixture, 1),
            JerryBorrow(command) => command.run(test_fixture, 1),
            JerryDeposit(command) => command.run(test_fixture, 1),
            JerryWithdraw(command) => command.run(test_fixture, 1),
            JerryLiquidate(command) => command.run(test_fixture, 1),
            JerryDepositCollateral(command) => command.run(test_fixture, 1),
            JerryWithdrawCollateral(command) => command.run(test_fixture, 1),
            JerryDepositWithLeverage(command) => command.run(test_fixture, 1),
            JerryWithdrawFromLeveraged(command) => command.run(test_fixture, 1),
            // Butch
            ButchRepay(command) => command.run(test_fixture, 2),
            ButchBorrow(command) => command.run(test_fixture, 2),
            ButchDeposit(command) => command.run(test_fixture, 2),
            ButchWithdraw(command) => command.run(test_fixture, 2),
            ButchLiquidate(command) => command.run(test_fixture, 2),
            ButchDepositCollateral(command) => command.run(test_fixture, 2),
            ButchWithdrawCollateral(command) => command.run(test_fixture, 2),
            ButchDepositWithLeverage(command) => command.run(test_fixture, 2),
            ButchWithdrawFromLeveraged(command) => command.run(test_fixture, 2),
            // Nibbles
            NibblesRepay(command) => command.run(test_fixture, 3),
            NibblesBorrow(command) => command.run(test_fixture, 3),
            NibblesDeposit(command) => command.run(test_fixture, 3),
            NibblesWithdraw(command) => command.run(test_fixture, 3),
            NibblesLiquidate(command) => command.run(test_fixture, 3),
            NibblesDepositCollateral(command) => command.run(test_fixture, 3),
            NibblesWithdrawCollateral(command) => command.run(test_fixture, 3),
            NibblesDepositWithLeverage(command) => command.run(test_fixture, 3),
            NibblesWithdrawFromLeveraged(command) => command.run(test_fixture, 3),
            // All
            AllPassTime(command) => command.run(test_fixture, 0),
        }
    }
}

#[derive(Arbitrary, Debug)]
pub struct Amount(
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=(u32::MAX as i128)))] pub i128,
);

#[derive(Arbitrary, Debug)]
pub struct Input {
    pub commands: [Command; 20],
}

#[derive(Arbitrary, Debug)]
pub struct PassTime {
    // up to 2 years
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=(2 * 365 * 24 * 60 * 60)))]
    pub amount: u64,
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
    pub token: Token,
    pub repay_amount: Amount,
    pub collateral_token: Token,
    pub min_collateral_received_amount: Amount,
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
pub struct WithdrawFromLeveraged {
    pub amount: Amount,
    pub deposit_token: Token,
    pub borrow_token: Token,
}

impl RunCommand for PassTime {
    fn run(&self, test_fixture: &TestMarketFixture, _who: usize) {
        test_fixture.pass_time(self.amount);
    }
}

impl RunCommand for Borrow {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;
        let _ = contract_client.try_borrow(&users[who], &pool_address, &self.amount.0);
    }
}

impl RunCommand for Deposit {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let _ = contract_client.try_deposit(&users[who], &pool_address, &self.amount.0);
    }
}

impl RunCommand for DepositCollateral {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let _ = contract_client.try_add_collateral(&users[who], &pool_address, &self.amount.0);
    }
}

impl RunCommand for WithdrawCollateral {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let _ = contract_client.try_remove_collateral(&users[who], &pool_address, &self.amount.0);
    }
}

impl RunCommand for Withdraw {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let _ = contract_client.try_withdraw(&users[who], &pool_address, &self.amount.0);
    }
}

impl RunCommand for Repay {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let _ = contract_client.try_repay(&users[who], &pool_address, &self.amount.0);
    }
}

impl RunCommand for Liquidate {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let borrow_pool_address = test_fixture.get_pool_address(self.token);
        let collateral_pool_address = test_fixture.get_pool_address(self.collateral_token);

        if borrow_pool_address != collateral_pool_address {
            let TestMarketFixture { contract_client, users, .. } = test_fixture;
            let (liquidator, borrower) = (&users[who], &users[(who + 1) % users.len()]);

            let _ = contract_client.try_liquidate(
                liquidator,
                borrower,
                &None,
                &borrow_pool_address,
                &collateral_pool_address,
                &self.repay_amount.0,
                &self.min_collateral_received_amount.0,
            );
        }
    }
}

impl RunCommand for DepositWithLeverage {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let deposit_pool_address = test_fixture.get_pool_address(self.deposit_token);
        let borrow_pool_address = test_fixture.get_pool_address(self.borrow_token);

        if deposit_pool_address != borrow_pool_address {
            let TestMarketFixture { contract_client, users, .. } = test_fixture;

            let (flash_loan_provider, lender) = (&users[who], &users[(who + 1) % users.len()]);

            contract_client.deposit(
                flash_loan_provider,
                &borrow_pool_address,
                &self.flash_loan_amount.0,
            );

            let _ = contract_client.try_deposit_with_leverage(
                lender,
                &deposit_pool_address,
                &borrow_pool_address,
                &false,
                &self.amount.0,
                &self.leverage,
            );
        }
    }
}

impl RunCommand for WithdrawFromLeveraged {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let deposit_pool_address = test_fixture.get_pool_address(self.deposit_token);
        let borrow_pool_address = test_fixture.get_pool_address(self.borrow_token);

        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let _ = contract_client.try_withdraw_from_leveraged(
            &users[who],
            &deposit_pool_address,
            &borrow_pool_address,
            &self.amount.0,
        );
    }
}

// ---- Helpers that encapsulate access to inner structures ----

// -- Obligation --

// - Direct accessors -
pub fn get_obligation_j_tokens(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_deposit_position(contract_client, user, pool_address)?;

    Ok(deposit_position.j_tokens)
}

pub fn get_earn_obligation_j_tokens(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position =
        get_earn_obligation_deposit_position(contract_client, user, pool_address)?;

    Ok(deposit_position.j_tokens)
}

pub fn get_multiply_pair_obligation_j_tokens(
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_multiply_pair_deposit_position(
        contract_client,
        user,
        deposit_pool_address,
        borrow_pool_address,
    )?;

    Ok(deposit_position.j_tokens)
}

pub fn get_obligation_d_tokens(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_borrow_position(contract_client, user, pool_address)?;

    Ok(deposit_position.d_tokens)
}

pub fn get_earn_obligation_d_tokens(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    // NB: This is expected to always return Err(MCError::BorrowDoesNotExist)
    let borrow_position = get_earn_obligation_borrow_position(contract_client, user, pool_address)?;

    Ok(borrow_position.d_tokens)
}

pub fn get_multiply_pair_obligation_d_tokens(
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<i128, MCError> {
    let borrow_position = get_multiply_pair_borrow_position(
        contract_client,
        user,
        deposit_pool_address,
        borrow_pool_address,
    )?;

    Ok(borrow_position.d_tokens)
}

pub fn get_obligation_borrowed(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let borrow_position = get_borrow_position(contract_client, user, pool_address)?;

    Ok(borrow_position.originally_borrowed)
}

pub fn get_multiply_pair_obligation_borrowed(
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<i128, MCError> {
    let borrow_position = get_multiply_pair_borrow_position(
        contract_client,
        user,
        deposit_pool_address,
        borrow_pool_address,
    )?;

    Ok(borrow_position.originally_borrowed)
}

pub fn get_obligation_deposited(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_deposit_position(contract_client, user, pool_address)?;

    Ok(deposit_position.originally_deposited)
}

pub fn get_earn_obligation_deposited(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position =
        get_earn_obligation_deposit_position(contract_client, user, pool_address)?;

    Ok(deposit_position.originally_deposited)
}

pub fn get_multiply_pair_obligation_deposited(
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_multiply_pair_deposit_position(
        contract_client,
        user,
        deposit_pool_address,
        borrow_pool_address,
    )?;

    Ok(deposit_position.originally_deposited)
}

pub fn get_obligation_collateral(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_deposit_position(contract_client, user, pool_address)?;

    Ok(deposit_position.collateral)
}

pub fn get_multiply_pair_obligation_collateral(
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_multiply_pair_deposit_position(
        contract_client,
        user,
        deposit_pool_address,
        borrow_pool_address,
    )?;

    Ok(deposit_position.collateral)
}

// - Indirect accessors -

pub fn get_obligation_d_tokens_as_tokens(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(pool_address);
    let d_tokens = get_obligation_d_tokens(contract_client, user, pool_address)?;

    let deposited_tokens = pool.compute_tokens_from_d_tokens_floor(e, d_tokens)?;

    Ok(deposited_tokens)
}

pub fn get_obligation_unpaid_interest(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let total_debt = get_obligation_d_tokens_as_tokens(e, contract_client, user, pool_address)?;
    let initially_borrowed = get_obligation_borrowed(contract_client, user, pool_address)?;

    if total_debt < initially_borrowed {
        return Err(MCError::InternalError);
    }
    let unpaid_interest = total_debt - initially_borrowed;

    Ok(unpaid_interest)
}

pub fn get_obligation_j_tokens_as_tokens(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(pool_address);
    let j_tokens = get_obligation_j_tokens(contract_client, user, pool_address)?;

    let deposited_tokens = pool.compute_tokens_from_j_tokens_floor(e, j_tokens)?;

    Ok(deposited_tokens)
}

pub fn get_earn_obligation_j_tokens_as_tokens(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(pool_address);
    let j_tokens = get_earn_obligation_j_tokens(contract_client, user, pool_address)?;

    let deposited_tokens = pool.compute_tokens_from_j_tokens_floor(e, j_tokens)?;

    Ok(deposited_tokens)
}

pub fn get_multiply_pair_obligation_j_tokens_as_tokens(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(deposit_pool_address);
    let j_tokens = get_multiply_pair_obligation_j_tokens(
        contract_client,
        user,
        deposit_pool_address,
        borrow_pool_address,
    )?;

    let deposited_tokens = pool.compute_tokens_from_j_tokens_floor(e, j_tokens)?;

    Ok(deposited_tokens)
}

pub fn compute_user_obligation_debt_value(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
) -> i128 {
    let obligation = contract_client.get_user_obligation(user);

    e.as_contract(&contract_client.address, || obligation.compute_debt_value(e).unwrap())
}

pub fn compute_user_obligation_collateral_value(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
) -> i128 {
    let obligation = contract_client.get_user_obligation(user);

    e.as_contract(&contract_client.address, || obligation.compute_collateral_value(e).unwrap())
}

pub fn compute_multiply_pair_obligation_debt_value(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> i128 {
    let obligation = contract_client.get_multiply_pair_obligation(
        user,
        deposit_pool_address,
        borrow_pool_address,
    );

    e.as_contract(&contract_client.address, || obligation.compute_debt_value(e).unwrap())
}

pub fn compute_multiply_pair_obligation_collateral_value(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> i128 {
    let obligation = contract_client.get_multiply_pair_obligation(
        user,
        deposit_pool_address,
        borrow_pool_address,
    );

    e.as_contract(&contract_client.address, || obligation.compute_collateral_value(e).unwrap())
}

// - Inner struct accessors -

pub fn get_deposit_position(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<DepositPosition, MCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_user_obligation(user) else {
        return Err(MCError::ObligationDoesNotExist);
    };

    let deposit =
        obligation.deposits.get(pool_address.clone()).ok_or(MCError::DepositDoesNotExist)?;

    Ok(deposit)
}

pub fn get_earn_obligation_deposit_position(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<DepositPosition, MCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_earn_user_obligation(user) else {
        return Err(MCError::ObligationDoesNotExist);
    };

    let deposit =
        obligation.deposits.get(pool_address.clone()).ok_or(MCError::DepositDoesNotExist)?;

    Ok(deposit)
}

pub fn get_multiply_pair_deposit_position(
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<DepositPosition, MCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_multiply_pair_obligation(
        user,
        deposit_pool_address,
        borrow_pool_address,
    ) else {
        return Err(MCError::ObligationDoesNotExist);
    };

    let deposit = obligation
        .deposits
        .get(deposit_pool_address.clone())
        .ok_or(MCError::DepositDoesNotExist)?;

    Ok(deposit)
}

pub fn get_borrow_position(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<BorrowPosition, MCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_user_obligation(user) else {
        return Err(MCError::ObligationDoesNotExist);
    };

    let borrow = obligation.borrows.get(pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?;

    Ok(borrow)
}

pub fn get_earn_obligation_borrow_position(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<BorrowPosition, MCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_earn_user_obligation(user) else {
        return Err(MCError::ObligationDoesNotExist);
    };

    // NB: Expected that this always returns `Err(MCError::BorrowDoesNotExist)`
    let borrow = obligation.borrows.get(pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?;

    Ok(borrow)
}

pub fn get_multiply_pair_borrow_position(
    contract_client: &MarketClient,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) -> Result<BorrowPosition, MCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_multiply_pair_obligation(
        user,
        deposit_pool_address,
        borrow_pool_address,
    ) else {
        return Err(MCError::ObligationDoesNotExist);
    };

    let borrow =
        obligation.borrows.get(borrow_pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?;

    Ok(borrow)
}

// -- Pool --

pub fn get_pool_total_j_tokens(contract_client: &MarketClient, pool_address: &Address) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.total_j_tokens
}

pub fn get_pool_total_d_tokens(contract_client: &MarketClient, pool_address: &Address) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.total_d_tokens
}

pub fn get_pool_total_borrowed(contract_client: &MarketClient, pool_address: &Address) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.total_borrowed
}

pub fn get_pool_total_supply(
    contract_client: &MarketClient,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(pool_address);
    let total_supply = pool.total_supply()?;

    Ok(total_supply)
}

pub fn get_pool_total_available(contract_client: &MarketClient, pool_address: &Address) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.total_available
}

pub fn get_pool_total_collateral(contract_client: &MarketClient, pool_address: &Address) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.total_collateral
}

pub fn get_pool_accumulated_host_fees(
    contract_client: &MarketClient,
    pool_address: &Address,
) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.accumulated_host_fees
}

pub fn get_pool_accumulated_market_fees(
    contract_client: &MarketClient,
    pool_address: &Address,
) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.accumulated_market_fees
}

pub fn get_pool_accumulated_reserve_fees(
    contract_client: &MarketClient,
    pool_address: &Address,
) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.accumulated_reserve_fees
}

pub fn get_pool_available_reserve_fees(
    contract_client: &MarketClient,
    pool_address: &Address,
) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    i128::min(pool.total_available, pool.accumulated_reserve_fees)
}

pub fn compute_pool_collateral_value(
    e: &Env,
    contract_client: &MarketClient,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(pool_address);

    e.as_contract(&contract_client.address, || pool.compute_total_collateral_value(e))
}

pub fn compute_pool_debt_value(
    e: &Env,
    contract_client: &MarketClient,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(pool_address);

    e.as_contract(&contract_client.address, || pool.compute_total_debt_value(e))
}

// - PoolConfig -

pub fn get_pool_fee_config(
    contract_client: &MarketClient,
    pool_address: &Address,
) -> PoolFeeConfig {
    let pool = contract_client.get_pool(pool_address);

    pool.config.fee_config
}

// ---- MISC ----

pub fn make_oracle_prices_different(e: &Env, oracle_client: &MockPriceOracleClient) {
    oracle_client.set_price_stable(&soroban_sdk::vec![
        e,
        50_00000000000000, // BTC
        3_00000000000000,  // GOLD
        1_00000000000000,  // USDC
    ]);
}

pub fn make_oracle_prices_equal(e: &Env, oracle_client: &MockPriceOracleClient) {
    oracle_client.set_price_stable(&soroban_sdk::vec![
        e,
        1_00000000000000, // BTC
        1_00000000000000, // GOLD
        1_00000000000000, // USDC
    ]);
}

pub struct TestAssetSetup<'a> {
    pub token_client: TokenClient<'a>,
    pub token_address: Address,
    pub sac_client: StellarAssetClient<'a>,
}

pub fn setup_test_asset<'a>(e: &Env, admin: &Address, users: &Vec<Address>) -> TestAssetSetup<'a> {
    let token_address = e.register_stellar_asset_contract_v2(admin.clone()).address();
    let sac_client = StellarAssetClient::new(e, &token_address);
    let token_client = TokenClient::new(e, &token_address);

    sac_client.mint(admin, &DEFAULT_ADMIN_ASSET_MINT_AMOUNT);

    for user in users {
        sac_client.mint(user, &DEFAULT_USER_ASSET_MINT_AMOUNT);
    }

    TestAssetSetup { token_address, token_client, sac_client }
}

pub fn setup_market_client<'a>(e: &Env, is_owned: bool) -> MarketClient<'a> {
    let contract_name = soroban_sdk::String::from_str(e, "market_contract");
    let contract_admin = Address::generate(e);
    let oracle = Address::generate(e);

    let contract_id = e.register(
        MarketContract,
        (
            contract_name,
            contract_admin.clone(),
            oracle,
            contract_admin,
            DEFAULT_MAX_POSITIONS,
            0i128,
            DEFAULT_INSOLVENCY_LTV_BPS,
            if is_owned { Some(DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS) } else { None },
        ),
    );

    let client = MarketClient::new(e, &contract_id);

    if is_owned {
        client.update_market_status(&0);
    }

    client
}

pub fn register_random_sac(e: &Env) -> Address {
    let token_admin = Address::generate(e);

    e.register_stellar_asset_contract_v2(token_admin).address()
}

pub fn get_default_env() -> Env {
    let e = Env::default();
    e.mock_all_auths();

    e
}

pub fn assert_approx_eq_abs<T>(a: T, b: T, delta: T)
where
    T: PartialOrd + Add<Output = T> + Sub<Output = T> + Copy + core::fmt::Debug,
{
    assert!(
        a >= b - delta && a <= b + delta,
        "assertion failed: `(left != right)` (left: `{:?}`, right: `{:?}`, delta: `{:?}`)",
        a,
        b,
        delta
    );
}

/// Asserts that `a` is approximately equal to `b` within a relative error of `delta`
///
/// # Arguments
/// * `delta_bps` - percentage represented in basis points such that 15% is 15_00
pub fn assert_approx_eq_rel(a: i128, b: i128, delta_bps: i128) {
    let abs_delta = b.fixed_mul_floor(delta_bps, BPS_FACTOR).unwrap();
    assert_approx_eq_abs(a, b, abs_delta);
}

pub fn get_amount_scaled_down(amount: i128, scale_bps: i128) -> i128 {
    amount.checked_sub(amount.fixed_mul_floor(scale_bps, BPS_FACTOR).unwrap()).unwrap()
}

pub fn get_amount_scaled_up(amount: i128, scale_bps: i128) -> i128 {
    amount.checked_add(amount.fixed_mul_ceil(scale_bps, BPS_FACTOR).unwrap()).unwrap()
}
