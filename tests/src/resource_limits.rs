#![cfg(test)]

//! Resource-limit / scaling tests for an obligation that holds the full
//! "5 borrow + 5 collateral" position set (10 *distinct* positions across
//! 10 distinct pools).
//!
//! Background
//! ----------
//! Every health-sensitive market operation (`borrow`, `add_collateral`,
//! `remove_collateral`, `withdraw`, ...) walks *all* of an obligation's
//! positions: it accrues interest on every borrow/deposit pool and re-values
//! each position through a per-pool oracle cross-contract call plus `I256`
//! math (see `Obligation::compute_*_value*` in `contracts/market`). The cost
//! of a single transaction therefore grows with the number of positions, so a
//! "fat" obligation is the natural worst case for Soroban's per-transaction
//! resource budget.
//!
//! What "resource limits" means here
//! ---------------------------------
//! A default test `Env` is created with the live pubnet budget already
//! installed (`100_000_000` CPU instructions / `40 MiB` of linear memory) and
//! the host re-arms that budget before every top-level invocation. If a call
//! ever blew the budget the host would trap with `"Budget, ExceededLimit"` and
//! the test would fail. Crucially we do **not** call
//! `budget().reset_unlimited()` here (the fuzz harness does, to get out of the
//! budget's way) — keeping the real ceiling armed is the whole point.
//!
//! Caveat: native vs wasm metering
//! -------------------------------
//! These tests drive the natively-linked `MarketContract`, so the budget does
//! not include the wasm VM instantiation / bytecode-execution cost that a real
//! on-chain invocation pays. The host-side work that dominates this contract —
//! oracle cross-contract calls, storage reads/writes, map iteration and `I256`
//! arithmetic — *is* metered, so the numbers below are a meaningful lower
//! bound and a solid regression guard, but not an exact on-chain figure. The
//! assertions are written against the real pubnet ceiling regardless.

use market::{
    constants::{
        DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS, DEFAULT_MAX_POSITIONS,
        DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS, INDIVIDUAL_BUMP,
    },
    contract::{MarketClient, MarketContract},
    obligation::ObligationKey,
    pool::PoolConfig,
    storage::MarketInitParams,
};
use sep_40_oracle::testutils::{Asset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::{
    Address, Env, String, Symbol,
    testutils::{Address as _, Ledger, LedgerInfo},
};

use crate::{get_default_env, setup_test_asset};

// ---- Scenario sizing ----

/// Collateral-only positions (each in its own pool).
const N_COLLATERAL: usize = 5;
/// Borrow positions (each in its own pool, all distinct from the collateral
/// pools), for a total of `N_COLLATERAL + N_BORROW == 10` positions.
const N_BORROW: usize = 5;

/// Collateral deposited into each of the 5 collateral pools (7-decimal SAC).
const COLLATERAL_AMOUNT: i128 = 1_000_000;
/// Liquidity a separate provider seeds into each of the 5 borrow pools so the
/// borrower actually has something to borrow.
const LIQUIDITY_AMOUNT: i128 = 1_000_000;
/// Borrowed per borrow pool. With every asset priced at $1 and a 70% open LTV,
/// total debt value is `5 * BORROW_AMOUNT` against `5 * COLLATERAL_AMOUNT * 0.7`
/// of borrowing power — an obligation-wide LTV of ~14%, comfortably healthy so
/// none of the five borrows trips the health check.
const BORROW_AMOUNT: i128 = 100_000;

// ---- Oracle ----

// Reuse the canonical mock-oracle address and 14-decimal price scale the rest
// of the suite uses (`tests/src/lib.rs`, `tests/src/borrow.rs`).
const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";
const ORACLE_PRICE_DECIMALS: u32 = 14;

// ---- Pubnet per-transaction resource ceiling ----
//
// Mirrors soroban-env-host's `DEFAULT_CPU_INSN_LIMIT` / `DEFAULT_MEM_BYTES_LIMIT`
// (the values `Budget::reset_default()` installs). They are not re-exported by
// the SDK, so we restate them here as the limit our scenario must stay under.
const PUBNET_CPU_INSN_LIMIT: u64 = 100_000_000;
const PUBNET_MEM_BYTES_LIMIT: u64 = 40 * 1024 * 1024;

/// A market wired up with 10 distinct pools and a single borrower whose
/// obligation holds 5 collateral positions and 5 borrow positions.
struct TenPositionFixture<'a> {
    e: Env,
    contract_client: MarketClient<'a>,
    borrower: ObligationKey,
    collateral_pools: Vec<Address>,
    borrow_pools: Vec<Address>,
}

impl TenPositionFixture<'_> {
    fn new() -> Self {
        // NB: `get_default_env` keeps the default (pubnet) budget armed — we
        // deliberately do *not* reset it to unlimited.
        let e = get_default_env();

        // Match the ledger/TTL setup used by `TestMarketFixture` so instance
        // and persistent entry bumps never overrun `max_entry_ttl`.
        e.ledger().set(LedgerInfo {
            timestamp: 1590969600, // June 1, 2020
            protocol_version: 25,
            sequence_number: 1000,
            network_id: Default::default(),
            base_reserve: 10,
            min_temp_entry_ttl: 500000,
            min_persistent_entry_ttl: 500000,
            max_entry_ttl: INDIVIDUAL_BUMP + 1,
        });

        let contract_admin = Address::generate(&e);
        let borrower_addr = Address::generate(&e);
        let liquidity_provider_addr = Address::generate(&e);
        // Both users are minted huge balances by `setup_test_asset`.
        let users = vec![borrower_addr.clone(), liquidity_provider_addr.clone()];

        // -- Oracle --
        let oracle = Address::from_str(&e, ORACLE_ADDRESS);
        e.register_at(&oracle, MockPriceOracleWASM, ());
        let oracle_client = MockPriceOracleClient::new(&e, &oracle);

        // -- Market contract --
        let insurance_fund = Address::generate(&e);
        let market_manager = Address::generate(&e);
        let contract_id = e.register(
            MarketContract,
            (
                String::from_str(&e, "resource_limits_market"),
                contract_admin.clone(),
                oracle.clone(),
                insurance_fund,
                market_manager,
                MarketInitParams {
                    // 20 by default — 10 positions fits with room to spare.
                    max_positions: DEFAULT_MAX_POSITIONS,
                    min_collateral_value_cents: 0i128,
                    insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                    update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                    is_owned: true,
                    bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
                },
            ),
        );
        let contract_client = MarketClient::new(&e, &contract_id);
        // Owned markets start frozen; flip to Active (status code 0).
        contract_client.update_market_status(&0);

        // -- Create 10 distinct pools (5 collateral + 5 borrow) --
        //
        // `pool_assets` accumulates the per-pool oracle assets in the same
        // order we register them, so the positional `set_price_stable` feed
        // below lines up with each pool.
        let mut collateral_pools: Vec<Address> = Vec::with_capacity(N_COLLATERAL);
        let mut borrow_pools: Vec<Address> = Vec::with_capacity(N_BORROW);
        let mut pool_assets = soroban_sdk::Vec::new(&e);

        for _ in 0..N_COLLATERAL {
            let admin = Address::generate(&e);
            let asset = setup_test_asset(&e, &admin, &users);
            register_pool(&e, &contract_client, &asset.token_address);
            pool_assets.push_back(Asset::Stellar(asset.token_address.clone()));
            collateral_pools.push(asset.token_address);
        }
        for _ in 0..N_BORROW {
            let admin = Address::generate(&e);
            let asset = setup_test_asset(&e, &admin, &users);
            register_pool(&e, &contract_client, &asset.token_address);
            pool_assets.push_back(Asset::Stellar(asset.token_address.clone()));
            borrow_pools.push(asset.token_address);
        }

        // -- Oracle data + prices: every pool asset at $1 --
        oracle_client.set_data(
            &contract_admin,
            &Asset::Other(Symbol::new(&e, "USD")),
            &pool_assets,
            &ORACLE_PRICE_DECIMALS,
            &123, // resolution: irrelevant for stable prices
        );
        let unit_price = 10_i128.pow(ORACLE_PRICE_DECIMALS);
        let mut prices = soroban_sdk::Vec::new(&e);
        for _ in 0..(N_COLLATERAL + N_BORROW) {
            prices.push_back(unit_price);
        }
        oracle_client.set_price_stable(&prices);

        let borrower = ObligationKey::new(borrower_addr);
        let liquidity_provider = ObligationKey::new(liquidity_provider_addr);

        // -- Seed each borrow pool with liquidity from the provider --
        for pool in &borrow_pools {
            contract_client.deposit(&liquidity_provider, pool, &LIQUIDITY_AMOUNT, &None);
        }

        // -- 5 collateral positions (distinct pools) --
        for pool in &collateral_pools {
            contract_client.add_collateral(&borrower, pool, &COLLATERAL_AMOUNT, &None);
        }

        // -- 5 borrow positions (distinct pools) --
        //
        // Each borrow re-values the obligation's growing position set, so by
        // the final borrow the host is already metering a full 10-position
        // revaluation against the armed pubnet budget.
        for pool in &borrow_pools {
            contract_client.borrow(&borrower, pool, &BORROW_AMOUNT, &None);
        }

        Self { e, contract_client, borrower, collateral_pools, borrow_pools }
    }
}

/// Queues and applies a default-config pool for `token_address`, advancing the
/// ledger clock past the mandatory queue period.
fn register_pool(e: &Env, contract_client: &MarketClient, token_address: &Address) {
    contract_client.queue_in_pool_set(token_address, &PoolConfig::default());
    e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
    contract_client.apply_pool_set(token_address);
}

/// The obligation really does carry 5 borrow + 5 collateral positions, all in
/// distinct pools. This pins down the exact shape the resource-limit test
/// exercises (and guards against e.g. a pool being silently reused).
#[test]
fn obligation_holds_five_distinct_borrows_and_five_distinct_collaterals() {
    let f = TenPositionFixture::new();

    let obligation = f.contract_client.get_user_obligation(&f.borrower);

    assert_eq!(
        obligation.deposits.len(),
        N_COLLATERAL as u32,
        "expected exactly {N_COLLATERAL} collateral positions",
    );
    assert_eq!(
        obligation.borrows.len(),
        N_BORROW as u32,
        "expected exactly {N_BORROW} borrow positions",
    );
    assert_eq!(
        obligation.positions_count,
        (N_COLLATERAL + N_BORROW) as u32,
        "expected 10 total positions",
    );

    // Each collateral pool is a deposit position and never doubles as a borrow.
    for pool in &f.collateral_pools {
        assert!(
            obligation.deposits.contains_key(pool.clone()),
            "collateral pool {pool:?} missing from deposits",
        );
        assert!(
            !obligation.borrows.contains_key(pool.clone()),
            "collateral pool {pool:?} unexpectedly also a borrow",
        );
    }
    // ...and symmetrically for borrow pools.
    for pool in &f.borrow_pools {
        assert!(
            obligation.borrows.contains_key(pool.clone()),
            "borrow pool {pool:?} missing from borrows",
        );
        assert!(
            !obligation.deposits.contains_key(pool.clone()),
            "borrow pool {pool:?} unexpectedly also a deposit",
        );
    }
}

/// Building all 10 positions and then performing the heaviest position-scaling
/// operation an obligation of this size can trigger stays comfortably within
/// the pubnet per-transaction resource budget.
///
/// The build phase already exercises the limits: each of the five borrows is a
/// top-level call metered against the armed pubnet budget over an ever-larger
/// position set, so reaching this test body at all means none of them tripped
/// `"Budget, ExceededLimit"`. We then probe once more and assert the headroom
/// explicitly.
#[test]
fn ten_positions_stay_within_pubnet_resource_limits() {
    let f = TenPositionFixture::new();

    // Sanity-check the precondition the measurement relies on.
    let obligation = f.contract_client.get_user_obligation(&f.borrower);
    assert_eq!(obligation.positions_count, (N_COLLATERAL + N_BORROW) as u32);

    // Borrowing a token amount from an *existing* borrow pool keeps the
    // position count at 10 while forcing a full sweep: interest accrual across
    // all 10 pools plus an oracle-backed revaluation of all 10 positions, in a
    // single top-level invocation. The host re-arms the pubnet budget at the
    // start of this call, so the budget snapshot afterwards reflects exactly
    // this operation.
    const PROBE_BORROW_AMOUNT: i128 = 1_000;
    f.contract_client.borrow(&f.borrower, &f.borrow_pools[0], &PROBE_BORROW_AMOUNT, &None);

    let budget = f.e.cost_estimate().budget();
    let cpu = budget.cpu_instruction_cost();
    let mem = budget.memory_bytes_cost();

    // The probe must actually do work (guards against measuring a no-op).
    assert!(cpu > 0, "expected the 10-position borrow to consume CPU instructions");
    assert!(mem > 0, "expected the 10-position borrow to consume memory");

    // ...and it must fit under the real pubnet ceiling the host enforces.
    assert!(
        cpu < PUBNET_CPU_INSN_LIMIT,
        "10-position borrow used {cpu} CPU instructions, exceeding the pubnet limit of \
         {PUBNET_CPU_INSN_LIMIT}",
    );
    assert!(
        mem < PUBNET_MEM_BYTES_LIMIT,
        "10-position borrow used {mem} memory bytes, exceeding the pubnet limit of \
         {PUBNET_MEM_BYTES_LIMIT}",
    );
}
