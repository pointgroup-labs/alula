mod bad_debt;
mod batch_flash_swap;
mod borrow;
mod deposit;
mod fees;
mod fuzz;
mod initialize;
mod interest_rates;
mod liquidate;
mod market_manager;
mod misc;
mod oracle;
mod repay;
mod storage_extension;
mod update;
mod withdraw;

use std::ops::{Add, Sub};

use arbitrary::Unstructured;
use controlled_insurance_fund::ControlledInsuranceFundContractClient;
use insurance_fund_interface::InsuranceFundClient;
use market::{
    constants::{
        BPS_FACTOR, DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS, DEFAULT_MAX_POSITIONS,
        DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS, INDIVIDUAL_BUMP,
    },
    contract::{MarketClient, MarketContract, MarketContractClient},
    error::MCError,
    math_utils::MathUtils,
    obligation::{BorrowPosition, DepositPosition, ObligationKey},
    pool::{PoolConfig, PoolFeeConfig},
    storage::MarketInitParams,
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
    pub full_contract_client: MarketContractClient<'a>,
    pub contract_id: Address,
    pub contract_admin: Address,
    pub users: Vec<Address>,
    // Oracle
    pub oracle_client: MockPriceOracleClient<'a>,
    pub oracle: Address,
    // Insurance Fund
    pub controlled_insurance_fund_client: ControlledInsuranceFundContractClient<'a>,
    pub insurance_fund_client: InsuranceFundClient<'a>,
    pub insurance_fund: Address,
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
        e.mock_all_auths_allowing_non_root_auth();

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
            Address::generate(&e),
        ];

        // Configure USDC SAC first, since it's used in the oracle as a base asset
        let usdc_admin = Address::generate(&e);
        let TestAssetSetup {
            sac_client: usdc_sac,
            token_client: usdc_token_client,
            token_address: usdc_token_address,
        } = setup_test_asset(&e, &usdc_admin, &users);

        let oracle = Address::from_str(&e, ORACLE_ADDRESS);
        e.register_at(&oracle, MockPriceOracleWASM, ());
        let oracle_client = MockPriceOracleClient::new(&e, &oracle);

        let contract_admin = Address::generate(&e);

        let insurance_fund = e.register(
            controlled_insurance_fund::ControlledInsuranceFundContract,
            (contract_admin.clone(),),
        );
        let controlled_insurance_fund_client =
            controlled_insurance_fund::ControlledInsuranceFundContractClient::new(
                &e,
                &insurance_fund,
            );
        let insurance_fund_client = InsuranceFundClient::new(&e, &insurance_fund);

        // Register Market contract
        let market_manager_address = Address::generate(&e);
        let contract_name = soroban_sdk::String::from_str(&e, "market_contract");
        let market_contract_id = e.register(
            MarketContract,
            (
                contract_name,
                contract_admin.clone(),
                oracle.clone(),
                insurance_fund.clone(),
                market_manager_address,
                MarketInitParams {
                    max_positions: DEFAULT_MAX_POSITIONS,
                    min_collateral_value_cents: 0i128,
                    insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                    update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                    is_owned: true,
                    bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
                },
            ),
        );
        let contract_client = MarketClient::new(&e, &market_contract_id);
        let full_contract_client = MarketContractClient::new(&e, &market_contract_id);

        controlled_insurance_fund_client.set_market(&market_contract_id);
        contract_client.update_market_status(&0);

        // GOLD
        let gold_admin = Address::generate(&e);
        let TestAssetSetup {
            sac_client: gold_sac,
            token_client: gold_token_client,
            token_address: gold_token_address,
        } = setup_test_asset(&e, &gold_admin, &users);
        contract_client.queue_in_pool_set(&gold_token_address, &pool_config);
        e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
        contract_client.apply_pool_set(&gold_token_address);
        let gold_pool_address = gold_token_address.clone();

        // BTC
        let btc_admin = Address::generate(&e);
        let TestAssetSetup {
            sac_client: btc_sac,
            token_client: btc_token_client,
            token_address: btc_token_address,
        } = setup_test_asset(&e, &btc_admin, &users);
        contract_client.queue_in_pool_set(&btc_token_address, &pool_config);
        e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
        contract_client.apply_pool_set(&btc_token_address);
        let btc_pool_address = btc_token_address.clone();

        // USDC
        contract_client.queue_in_pool_set(&usdc_token_address, &pool_config);
        e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
        contract_client.apply_pool_set(&usdc_token_address);
        let usdc_pool_address = usdc_token_address.clone();

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
            full_contract_client,
            contract_id: market_contract_id,
            contract_admin,
            // Oracle
            oracle_client,
            oracle,
            // Insurance Fund
            controlled_insurance_fund_client,
            insurance_fund_client,
            insurance_fund,
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
            gold_pool_address,
            btc_pool_address,
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
            assert!(pool.total_supply().unwrap() >= 0);
            assert!(pool.total_available().unwrap() >= 0);
        }

        // -- Contract's token balances shouldn't be smaller than the corresponding `available` + both fee buckets --

        let token_balances: Vec<i128> =
            clients.iter().map(|client| client.balance(contract_id)).collect();

        for (pool, &token_balance) in pools.iter().zip(token_balances.iter()) {
            let expected_minimum_balance = pool
                .total_available
                .checked_add(pool.operation_fees_sum)
                .and_then(|v| v.checked_add(pool.take_rate_fees_sum))
                .expect("Overflow in invariant calc");

            assert!(
                token_balance >= expected_minimum_balance,
                "INSOLVENCY DETECTED in Pool {:?}: Physical Balance ({}) < Net Available + \
                 Operation Fees + Take-Rate Fees ({})",
                pool.pool_address,
                token_balance,
                expected_minimum_balance
            );
        }

        // -- Per-user token/collateral accounting and borrow-floor checks --

        for pool in pools {
            let (mut j_tokens_obligations_sum, mut d_tokens_obligations_sum) = (0_i128, 0_i128);
            let mut collateral_obligations_sum = 0_i128;

            for user in users {
                if let Ok(Ok(obligation)) =
                    contract_client.try_get_user_obligation(&ObligationKey::new(user.clone()))
                {
                    if let Some(deposit_position) =
                        obligation.deposits.get(pool.pool_address.clone())
                    {
                        j_tokens_obligations_sum += deposit_position.j_tokens;
                        collateral_obligations_sum += deposit_position.collateral;
                    }

                    if let Some(borrow_position) = obligation.borrows.get(pool.pool_address.clone())
                    {
                        d_tokens_obligations_sum += borrow_position.d_tokens;

                        assert!(
                            get_obligation_unpaid_interest(
                                e,
                                contract_client,
                                user,
                                &pool.pool_address
                            )
                            .is_ok(),
                            "BORROW FLOOR VIOLATED for user {:?} in pool {:?}: current debt < \
                             originally_borrowed",
                            user,
                            pool.pool_address
                        );
                    }
                }
            }

            contract_client.refresh_pool(&pool.pool_address);
            let pool_snapshot_1 = contract_client.get_pool(&pool.pool_address);
            contract_client.refresh_pool(&pool.pool_address);
            let pool_snapshot_2 = contract_client.get_pool(&pool.pool_address);

            assert_eq!(pool_snapshot_1, pool_snapshot_2);
            // Use the post-refresh snapshot for the sum checks below.
            let pool = pool_snapshot_2;

            assert_eq!(pool.total_j_tokens, j_tokens_obligations_sum);
            assert_eq!(pool.total_d_tokens, d_tokens_obligations_sum);

            assert_eq!(
                pool.total_collateral, collateral_obligations_sum,
                "COLLATERAL CONSERVATION VIOLATED in pool {:?}: pool.total_collateral ({}) != sum \
                 of user collateral ({})",
                pool.pool_address, pool.total_collateral, collateral_obligations_sum
            );
        }
    }
}

// ---- Fuzzing suite ----

pub trait RunCommand {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize);
}

/// Which user is performing the operation.
#[derive(Arbitrary, Debug, Clone, Copy)]
pub enum Actor {
    Tom,
    Jerry,
    Butch,
    Nibbles,
}

impl Actor {
    pub fn index(self) -> usize {
        match self {
            Actor::Tom => 0,
            Actor::Jerry => 1,
            Actor::Butch => 2,
            Actor::Nibbles => 3,
        }
    }
}

/// The operation to perform (actor-independent).
#[derive(Arbitrary, Debug)]
pub enum Op {
    Repay(Repay),
    Borrow(Borrow),
    Deposit(Deposit),
    Withdraw(Withdraw),
    Liquidate(Liquidate),
    DepositCollateral(DepositCollateral),
    WithdrawCollateral(WithdrawCollateral),
    PassTime(PassTime),
}

/// A single fuzz command: who does what.
#[derive(Arbitrary, Debug)]
pub struct Command {
    pub actor: Actor,
    pub op: Op,
}

impl Command {
    pub fn run(&self, test_fixture: &TestMarketFixture) {
        let who = self.actor.index();
        match &self.op {
            Op::Repay(cmd) => cmd.run(test_fixture, who),
            Op::Borrow(cmd) => cmd.run(test_fixture, who),
            Op::Deposit(cmd) => cmd.run(test_fixture, who),
            Op::Withdraw(cmd) => cmd.run(test_fixture, who),
            Op::Liquidate(cmd) => cmd.run(test_fixture, who),
            Op::DepositCollateral(cmd) => cmd.run(test_fixture, who),
            Op::WithdrawCollateral(cmd) => cmd.run(test_fixture, who),
            Op::PassTime(cmd) => cmd.run(test_fixture, who),
        }
    }
}

#[derive(Debug)]
pub struct Amount(pub i128);

impl<'a> arbitrary::Arbitrary<'a> for Amount {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        const MAX: u64 = u64::MAX / 1_000_000u64;
        let tag = u.int_in_range::<u8>(0..=7)?;
        let v: i128 = match tag {
            0 => 0,
            1 => 1,
            2 => BPS_FACTOR,
            3 => u32::MAX as i128,
            4 => MAX as i128,
            _ => u.int_in_range::<u64>(0..=MAX)? as i128,
        };

        Ok(Amount(v))
    }
}

/// Variable-length command sequence, capped at 64 entries so interest accrual and liquidation
/// cascades have enough runway without making individual iterations too expensive.
#[derive(Debug)]
pub struct Input {
    pub commands: Vec<Command>,
}

impl<'a> arbitrary::Arbitrary<'a> for Input {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        const MAX_COMMANDS: usize = 64;
        let len = u.int_in_range::<usize>(0..=MAX_COMMANDS)?;
        let commands =
            (0..len).map(|_| Command::arbitrary(u)).collect::<arbitrary::Result<Vec<_>>>()?;
        Ok(Input { commands })
    }
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

// ---- Fuzz error-checking helper ----
//
// Each operation has an explicit whitelist of MCError variants that are normal business outcomes
// (e.g. "can't borrow when under-collateralised").  Any error NOT on the whitelist — including
// every host-level panic — is treated as a bug and will fail the fuzz target immediately.
//
// Maintenance note: if you add a new MCError variant and a fuzz run starts failing on an input
// that exercises a legitimate new rejection path, add the variant to the relevant whitelist(s)
// below.  Do NOT add InternalError or OverOrUnderflow to any whitelist.

/// Whitelisted errors for each operation.  Returns `true` when `e` is a known, expected
/// business-logic rejection for `op` and the fuzz run should continue silently.
fn is_expected_fuzz_error(op: &'static str, e: &MCError) -> bool {
    match op {
        "borrow" => matches!(
            e,
            MCError::InvalidInputAmount
                | MCError::BorrowForbiddenOnMarket
                | MCError::MarketIsFrozen
                | MCError::TooManyPositions
                | MCError::MinCollateralValueIsNotMet
                | MCError::NonPositiveSharesAmount
                | MCError::PoolDoesNotExist
                | MCError::NotEnoughPoolFunds
                | MCError::OperationForbiddenOnPool
                | MCError::PoolBadDebtLocked
                | MCError::ObligationDoesNotExist
                | MCError::UnhealthyOperation
                | MCError::PoolUtilizationRatioCapExceeded
                | MCError::OracleDoesNotKnowAssetPrice
                | MCError::OracleStalePrice
                | MCError::NonPositiveOraclePrice
                | MCError::DepositPositionForAssetExists
        ),
        "deposit" => matches!(
            e,
            MCError::InvalidInputAmount
                | MCError::DepositForbiddenOnMarket
                | MCError::MarketIsFrozen
                | MCError::TooManyPositions
                | MCError::NonPositiveSharesAmount
                | MCError::PoolDoesNotExist
                | MCError::OperationForbiddenOnPool
                | MCError::PoolBadDebtLocked
                | MCError::PoolSupplyLimitExceeded
                | MCError::OracleDoesNotKnowAssetPrice
                | MCError::OracleStalePrice
                | MCError::NonPositiveOraclePrice
        ),
        "deposit_collateral" => matches!(
            e,
            MCError::InvalidInputAmount
                | MCError::DepositForbiddenOnMarket
                | MCError::MarketIsFrozen
                | MCError::TooManyPositions
                | MCError::NonPositiveSharesAmount
                | MCError::PoolDoesNotExist
                | MCError::OperationForbiddenOnPool
                | MCError::PoolBadDebtLocked
                | MCError::AssetCannotBeUsedAsCollateral
                | MCError::OracleDoesNotKnowAssetPrice
                | MCError::OracleStalePrice
                | MCError::NonPositiveOraclePrice
        ),
        "withdraw" => matches!(
            e,
            MCError::InvalidInputAmount
                | MCError::MarketIsFrozen
                | MCError::NonPositiveSharesAmount
                | MCError::PoolDoesNotExist
                | MCError::NotEnoughPoolFunds
                | MCError::OperationForbiddenOnPool
                | MCError::PoolBadDebtLocked
                | MCError::ObligationDoesNotExist
                | MCError::DepositPositionDoesNotExist
                | MCError::WithdrawScarcityOverLimit
                | MCError::ScarcityCooldownPeriod
                | MCError::UnhealthyOperation
                | MCError::OracleDoesNotKnowAssetPrice
                | MCError::OracleStalePrice
                | MCError::NonPositiveOraclePrice
        ),
        "withdraw_collateral" => matches!(
            e,
            MCError::InvalidInputAmount
                | MCError::MarketIsFrozen
                | MCError::NonPositiveSharesAmount
                | MCError::PoolDoesNotExist
                | MCError::OperationForbiddenOnPool
                | MCError::PoolBadDebtLocked
                | MCError::ObligationDoesNotExist
                | MCError::DepositPositionDoesNotExist
                | MCError::UnhealthyOperation
                | MCError::OracleDoesNotKnowAssetPrice
                | MCError::OracleStalePrice
                | MCError::NonPositiveOraclePrice
        ),
        "repay" => matches!(
            e,
            MCError::InvalidInputAmount
                | MCError::MarketIsFrozen
                | MCError::NonPositiveSharesAmount
                | MCError::PoolDoesNotExist
                | MCError::OperationForbiddenOnPool
                | MCError::PoolBadDebtLocked
                | MCError::ObligationDoesNotExist
                | MCError::BorrowPositionDoesNotExist
        ),
        "liquidate" => matches!(
            e,
            MCError::InvalidInputAmount
                | MCError::MarketIsFrozen
                | MCError::NonPositiveSharesAmount
                | MCError::PoolDoesNotExist
                | MCError::OperationForbiddenOnPool
                | MCError::PoolBadDebtLocked
                | MCError::ObligationDoesNotExist
                | MCError::DepositPositionDoesNotExist
                | MCError::BorrowPositionDoesNotExist
                | MCError::InvalidLiquidationInputs
                | MCError::ObligationIsHealthy
                | MCError::ObligationContainsOpenCoverBadDebtRequests
                | MCError::AssetCannotBeUsedAsCollateral
                | MCError::LiquidationExcessiveDemandedCollateral
                | MCError::SwapSlippageExceeded
                | MCError::OracleDoesNotKnowAssetPrice
                | MCError::OracleStalePrice
                | MCError::NonPositiveOraclePrice
        ),
        _ => false,
    }
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
        let res = contract_client.try_borrow(
            &ObligationKey::new(users[who].clone()),
            &pool_address,
            &self.amount.0,
            &None,
        );
        match res {
            Ok(_) => {}
            Err(Ok(e)) if is_expected_fuzz_error("borrow", &e) => {}
            Err(Ok(e)) => panic!("unexpected MCError in fuzz op `borrow`: {e:?}"),
            Err(Err(host_err)) => panic!("host error in fuzz op `borrow`: {host_err:?}"),
        }
    }
}

impl RunCommand for Deposit {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let res = contract_client.try_deposit(
            &ObligationKey::new(users[who].clone()),
            &pool_address,
            &self.amount.0,
            &None,
        );
        match res {
            Ok(_) => {}
            Err(Ok(e)) if is_expected_fuzz_error("deposit", &e) => {}
            Err(Ok(e)) => panic!("unexpected MCError in fuzz op `deposit`: {e:?}"),
            Err(Err(host_err)) => panic!("host error in fuzz op `deposit`: {host_err:?}"),
        }
    }
}

impl RunCommand for DepositCollateral {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let res = contract_client.try_add_collateral(
            &ObligationKey::new(users[who].clone()),
            &pool_address,
            &self.amount.0,
            &None,
        );
        match res {
            Ok(_) => {}
            Err(Ok(e)) if is_expected_fuzz_error("deposit_collateral", &e) => {}
            Err(Ok(e)) => panic!("unexpected MCError in fuzz op `deposit_collateral`: {e:?}"),
            Err(Err(host_err)) => {
                panic!("host error in fuzz op `deposit_collateral`: {host_err:?}")
            }
        }
    }
}

impl RunCommand for WithdrawCollateral {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let res = contract_client.try_remove_collateral(
            &ObligationKey::new(users[who].clone()),
            &pool_address,
            &self.amount.0,
            &None,
        );
        match res {
            Ok(_) => {}
            Err(Ok(e)) if is_expected_fuzz_error("withdraw_collateral", &e) => {}
            Err(Ok(e)) => panic!("unexpected MCError in fuzz op `withdraw_collateral`: {e:?}"),
            Err(Err(host_err)) => {
                panic!("host error in fuzz op `withdraw_collateral`: {host_err:?}")
            }
        }
    }
}

impl RunCommand for Withdraw {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let res = contract_client.try_withdraw(
            &ObligationKey::new(users[who].clone()),
            &pool_address,
            &self.amount.0,
            &None,
        );
        match res {
            Ok(_) => {}
            Err(Ok(e)) if is_expected_fuzz_error("withdraw", &e) => {}
            Err(Ok(e)) => panic!("unexpected MCError in fuzz op `withdraw`: {e:?}"),
            Err(Err(host_err)) => panic!("host error in fuzz op `withdraw`: {host_err:?}"),
        }
    }
}

impl RunCommand for Repay {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestMarketFixture { contract_client, users, .. } = test_fixture;

        let res = contract_client.try_repay(
            &ObligationKey::new(users[who].clone()),
            &pool_address,
            &self.amount.0,
            &None,
        );
        match res {
            Ok(_) => {}
            Err(Ok(e)) if is_expected_fuzz_error("repay", &e) => {}
            Err(Ok(e)) => panic!("unexpected MCError in fuzz op `repay`: {e:?}"),
            Err(Err(host_err)) => panic!("host error in fuzz op `repay`: {host_err:?}"),
        }
    }
}

impl RunCommand for Liquidate {
    fn run(&self, test_fixture: &TestMarketFixture, who: usize) {
        let borrow_pool_address = test_fixture.get_pool_address(self.token);
        let collateral_pool_address = test_fixture.get_pool_address(self.collateral_token);

        if borrow_pool_address != collateral_pool_address {
            let TestMarketFixture { contract_client, users, .. } = test_fixture;
            let (liquidator, borrower) = (&users[who], &users[(who + 1) % users.len()]);

            let res = contract_client.try_liquidate(
                liquidator,
                &ObligationKey::new(borrower.clone()),
                &borrow_pool_address,
                &collateral_pool_address,
                &self.repay_amount.0,
                &self.min_collateral_received_amount.0,
            );
            match res {
                Ok(_) => {}
                Err(Ok(e)) if is_expected_fuzz_error("liquidate", &e) => {}
                Err(Ok(e)) => panic!("unexpected MCError in fuzz op `liquidate`: {e:?}"),
                Err(Err(host_err)) => panic!("host error in fuzz op `liquidate`: {host_err:?}"),
            }
        }
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

pub fn get_obligation_d_tokens(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_borrow_position(contract_client, user, pool_address)?;

    Ok(deposit_position.d_tokens)
}

pub fn get_obligation_initially_borrowed(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let borrow_position = get_borrow_position(contract_client, user, pool_address)?;

    Ok(borrow_position.originally_borrowed)
}

pub fn get_obligation_originally_deposited(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let deposit_position = get_deposit_position(contract_client, user, pool_address)?;

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

// - Indirect accessors -

pub fn get_obligation_d_tokens_as_tokens(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let pool = contract_client.get_pool(pool_address);
    let d_tokens = get_obligation_d_tokens(contract_client, user, pool_address)?;

    let deposited_tokens = pool.compute_tokens_from_d_tokens_ceil(e, d_tokens)?;

    Ok(deposited_tokens)
}

pub fn get_obligation_unpaid_interest(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<i128, MCError> {
    let total_debt = get_obligation_d_tokens_as_tokens(e, contract_client, user, pool_address)?;
    let initially_borrowed =
        get_obligation_initially_borrowed(contract_client, user, pool_address)?;

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

pub fn compute_unparameterized_ltv_bps(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
) -> Result<i128, MCError> {
    let debt_value = compute_user_obligation_debt_value(e, contract_client, user);
    let collateral_value = compute_user_obligation_collateral_value(e, contract_client, user);

    debt_value.fixed_div_ceil(collateral_value, BPS_FACTOR).map_over_or_underflow()
}

pub fn compute_user_obligation_debt_value(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
) -> i128 {
    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));

    e.as_contract(&contract_client.address, || obligation.compute_debt_value(e).unwrap())
}

pub fn compute_user_obligation_collateral_value(
    e: &Env,
    contract_client: &MarketClient,
    user: &Address,
) -> i128 {
    let obligation = contract_client.get_user_obligation(&ObligationKey::new(user.clone()));

    e.as_contract(&contract_client.address, || obligation.compute_collateral_value(e).unwrap())
}

// - Inner struct accessors -

pub fn get_deposit_position(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<DepositPosition, MCError> {
    let Ok(Ok(obligation)) =
        contract_client.try_get_user_obligation(&ObligationKey::new(user.clone()))
    else {
        return Err(MCError::ObligationDoesNotExist);
    };

    let deposit = obligation
        .deposits
        .get(pool_address.clone())
        .ok_or(MCError::DepositPositionDoesNotExist)?;

    Ok(deposit)
}

pub fn get_borrow_position(
    contract_client: &MarketClient,
    user: &Address,
    pool_address: &Address,
) -> Result<BorrowPosition, MCError> {
    let Ok(Ok(obligation)) =
        contract_client.try_get_user_obligation(&ObligationKey::new(user.clone()))
    else {
        return Err(MCError::ObligationDoesNotExist);
    };

    let borrow =
        obligation.borrows.get(pool_address.clone()).ok_or(MCError::BorrowPositionDoesNotExist)?;

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

pub fn get_pool_operation_fees_sum(contract_client: &MarketClient, pool_address: &Address) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.operation_fees_sum
}

pub fn get_pool_take_rate_fees_sum(contract_client: &MarketClient, pool_address: &Address) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.take_rate_fees_sum
}

pub fn get_pool_utilization_ratio_bps(
    contract_client: &MarketClient,
    pool_address: &Address,
) -> i128 {
    let pool = contract_client.get_pool(pool_address);

    pool.compute_utilization_ratio_bps().unwrap()
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

pub fn make_oracle_prices_zero(e: &Env, oracle_client: &MockPriceOracleClient) {
    oracle_client.set_price_stable(&soroban_sdk::vec![e, 0, 0, 0,]);
}

pub fn make_oracle_prices_negative(e: &Env, oracle_client: &MockPriceOracleClient) {
    oracle_client.set_price_stable(&soroban_sdk::vec![e, -1, -1, -1,]);
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
    let insurance_fund = Address::generate(e);

    let contract_id = e.register(
        MarketContract,
        (
            contract_name,
            contract_admin.clone(),
            oracle,
            insurance_fund,
            contract_admin,
            MarketInitParams {
                max_positions: DEFAULT_MAX_POSITIONS,
                min_collateral_value_cents: 0i128,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
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
