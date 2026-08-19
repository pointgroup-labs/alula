#![cfg(test)]

//! Resource-limit / scaling tests for "fat" obligations — position sets spread
//! across many distinct pools.
//!
//! Background
//! ----------
//! Every health-sensitive market operation (`borrow`, `add_collateral`,
//! `remove_collateral`, `withdraw`, `liquidate`, ...) walks *all* of an
//! obligation's positions: it accrues interest on every borrow/deposit pool and
//! re-values each position through a per-pool oracle cross-contract call plus
//! `I256` math (see `Obligation::compute_*_value*` in `contracts/market`). The
//! cost of a single transaction therefore grows with the number of positions, so
//! a "fat" obligation is the natural worst case for Soroban's per-transaction
//! resource budget, and `max_positions` is the governance knob that bounds it.
//!
//! `liquidate` is the heaviest of those paths: `Obligation::liquidate` runs *all
//! four* value aggregators plus `compute_min_collateral_threshold_scaled` before
//! it touches any seizure math, so it pays roughly twice the revaluation work a
//! `borrow` over the same position set does.
//!
//! What "resource limits" means here
//! ---------------------------------
//! A Soroban transaction has to fit four independent per-transaction ceilings,
//! not just CPU: instructions, linear memory, total footprint ledger entries and
//! the write set. Each measured invocation is checked against all of them by
//! `assert_within_pubnet_limits`, and every test prints the full row so the
//! curve — not just a pass/fail — is on the record.
//!
//! The fixture measures rather than traps, because both of the host's built-in
//! traps misfire here for reasons that have nothing to do with the contract:
//!
//! * Alongside the real budget the test host keeps a *shadow* budget of the same
//!   size for its debug/observation bookkeeping, and a liquidation over ~7
//!   positions saturates the shadow **memory** dimension (measured: 41_943_190
//!   shadow bytes against 11_948_732 real ones), after which the host's own
//!   `get_authenticated_authorizations` panics with `"Budget, ExceededLimit"`.
//!   The fixture lifts the limits past it; charges are limit-independent, so the
//!   measurements do not move.
//! * The SDK's `enforce_resource_limits` panics *inside* the invocation, before
//!   the offending row can be printed, and its own footprint walk is charged to
//!   the real budget — inflating the very number being measured (10-position
//!   `borrow`: 3_544_460 memory bytes with it on, 2_146_560 with it off). The
//!   fixture disables it and re-imposes the same limits on the measurement.
//!
//! Caveat: native vs wasm metering
//! -------------------------------
//! These tests drive the natively-linked `MarketContract`, so the budget does
//! not include the wasm VM instantiation / bytecode-execution cost that a real
//! on-chain invocation pays. The host-side work that dominates this contract —
//! oracle cross-contract calls, storage reads/writes, map iteration and `I256`
//! arithmetic — *is* metered, so the numbers below are a meaningful lower
//! bound and a solid regression guard, but not an exact on-chain figure.
//! Both sweeps therefore also run against `wasms/market.wasm` — the artifact
//! that actually runs on pubnet — so the multiplier is measured rather than
//! assumed, and the wasm rows are the ones a governance decision should read.

use market::{
    constants::{
        DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_INSOLVENCY_LTV_BPS,
        DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS, INDIVIDUAL_BUMP, MAX_RESERVES,
    },
    contract::{MarketClient, MarketContract},
    obligation::ObligationKey,
    pool::PoolConfig,
    request::{LiquidateRequest, Request, StandardRequest, SwapForExactTokensRequest},
    storage::MarketInitParams,
};
use sep_40_oracle::testutils::{Asset, MockPriceOracleClient, MockPriceOracleWASM};
use soroban_sdk::{
    Address, Env, String, Symbol,
    testutils::{Address as _, Ledger, LedgerInfo},
    vec as svec,
};

use crate::{batch_flash_swap::MockProxySwap, get_default_env, setup_test_asset};

mod market_wasm {
    use soroban_sdk::contractimport;

    // The deployable artifact, i.e. what actually runs on pubnet. Registering it
    // alongside the native contract is how `liquidate_native_vs_wasm_market`
    // measures the VM instantiation + bytecode execution these tests otherwise
    // skip.
    contractimport!(file = "../wasms/market.wasm");
}

// ---- Scenario sizing ----

/// Collateral-only positions (each in its own pool).
const N_COLLATERAL: usize = 5;
/// Borrow positions (each in its own pool, all distinct from the collateral
/// pools), for a total of `N_COLLATERAL + N_BORROW == 10` positions.
const N_BORROW: usize = 5;

/// Collateral spread evenly over the collateral pools (7-decimal SAC). Held
/// constant across position counts so an obligation's *health* is identical at
/// every size and only the per-position sweep cost varies.
const COLLATERAL_TOTAL: i128 = 5_000_000;
/// Debt spread evenly over the borrow pools, likewise size-invariant.
const BORROW_TOTAL: i128 = 500_000;
/// Liquidity a separate provider seeds into each borrow pool. Sized to cover
/// `BORROW_TOTAL` even when there is a single borrow pool.
const LIQUIDITY_AMOUNT: i128 = 1_000_000;

/// Position counts swept by the liquidation scaling tests. Ends at
/// `MAX_RESERVES` (25), the hard ceiling `max_positions` may be set to.
const POSITION_SWEEP: [usize; 7] = [3, 5, 7, 10, 15, 20, 25];

/// Both builds of the market are swept: the natively-linked contract these
/// tests normally drive, and the deployable wasm that actually runs on pubnet.
const MARKET_BUILDS: [(&str, MarketBuild); 2] =
    [("native", MarketBuild::Native), ("wasm", MarketBuild::Wasm)];

// ---- Oracle ----

// Reuse the canonical mock-oracle address and 14-decimal price scale the rest
// of the suite uses (`tests/src/lib.rs`, `tests/src/borrow.rs`).
const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";
const ORACLE_PRICE_DECIMALS: u32 = 14;

/// Crashed collateral price ($0.115) that makes the obligation liquidatable at
/// every swept size: close-LTV-weighted collateral is `0.115 * 0.80 * 5_000_000
/// = 460_000` against `500_000` of debt, while the unweighted LTV of 8_695 bps
/// stays under `DEFAULT_INSOLVENCY_LTV_BPS`, so liquidation takes the
/// solvent (close-factor-bounded) branch rather than the bad-debt one.
const CRASHED_COLLATERAL_PRICE: i128 = 11_500_000_000_000;

// ---- Pubnet per-transaction resource ceilings ----
//
// Read off the live network's `ConfigSettingEntry`s on 2026-08-19 (protocol 27):
// `contract_compute_v0.{tx_max_instructions, tx_memory_limit}`,
// `contract_ledger_cost_v0.{tx_max_write_ledger_entries, tx_max_write_bytes}`
// and `contract_ledger_cost_ext_v0.tx_max_footprint_entries`.
//
// Deliberately *not* taken from the SDK's `InvocationResourceLimits::mainnet()`,
// which is a hardcoded snapshot the SDK itself flags as needing manual updates
// and is stale on three of these five (100/400 footprint entries,
// 50/200 write entries, 600M/400M instructions).
const PUBNET_CPU_INSN_LIMIT: u64 = 400_000_000;
const PUBNET_MEM_BYTES_LIMIT: u64 = 41_943_040;
const PUBNET_FOOTPRINT_ENTRY_LIMIT: u32 = 400;
const PUBNET_WRITE_ENTRY_LIMIT: u32 = 200;
const PUBNET_WRITE_BYTES_LIMIT: u32 = 132_096;

/// Limits armed for the fixture. Large enough that the host's shadow
/// bookkeeping (see the module doc) never trips; the pubnet ceiling is enforced
/// by `assert_within_pubnet_limits` on the measured numbers instead.
const MEASUREMENT_LIMIT: u64 = 1_000 * PUBNET_CPU_INSN_LIMIT;

/// Which build of the market contract the fixture registers.
#[derive(Clone, Copy, PartialEq)]
enum MarketBuild {
    Native,
    Wasm,
}

/// A market wired up with `n_collateral + n_borrow` distinct pools and a single
/// borrower whose obligation holds one position in each.
struct PositionFixture<'a> {
    e: Env,
    contract_client: MarketClient<'a>,
    oracle_client: MockPriceOracleClient<'a>,
    borrower: ObligationKey,
    liquidator: Address,
    collateral_pools: Vec<Address>,
    borrow_pools: Vec<Address>,
}

impl PositionFixture<'_> {
    fn new() -> Self {
        Self::with_positions(MarketBuild::Native, N_COLLATERAL, N_BORROW)
    }

    fn with_positions(build: MarketBuild, n_collateral: usize, n_borrow: usize) -> Self {
        assert!(n_collateral >= 1 && n_borrow >= 1);
        assert!(n_collateral + n_borrow <= MAX_RESERVES as usize);

        let e = get_default_env();
        // See the module doc: the host's shadow budget saturates on its own
        // debug bookkeeping long before the contract approaches the real
        // ceiling, so we lift both out of the way and enforce the pubnet
        // ceiling with `assert_within_pubnet_limits` on the measured numbers.
        e.cost_estimate().budget().reset_limits(MEASUREMENT_LIMIT, MEASUREMENT_LIMIT);
        // The host's own mainnet-limit check panics before the offending row can
        // be printed, which is useless for a sweep that is trying to find where
        // the ceiling is. `assert_within_pubnet_limits` re-imposes it.
        e.cost_estimate().disable_resource_limits();

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
        let liquidator_addr = Address::generate(&e);
        // All three are minted huge balances by `setup_test_asset`.
        let users =
            vec![borrower_addr.clone(), liquidity_provider_addr.clone(), liquidator_addr.clone()];

        // -- Oracle --
        let oracle = Address::from_str(&e, ORACLE_ADDRESS);
        e.register_at(&oracle, MockPriceOracleWASM, ());
        let oracle_client = MockPriceOracleClient::new(&e, &oracle);

        // -- Market contract --
        let insurance_fund = Address::generate(&e);
        let market_manager = Address::generate(&e);
        let init_args = (
            String::from_str(&e, "resource_limits_market"),
            contract_admin.clone(),
            oracle.clone(),
            insurance_fund,
            market_manager,
            MarketInitParams {
                // Raised to the hard ceiling so the sweep, not the guard, is
                // what bounds this fixture.
                max_positions: MAX_RESERVES,
                min_collateral_value_cents: 0i128,
                insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned: true,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        );
        let contract_id = match build {
            MarketBuild::Native => e.register(MarketContract, init_args),
            MarketBuild::Wasm => e.register(market_wasm::WASM, init_args),
        };
        let contract_client = MarketClient::new(&e, &contract_id);
        // Owned markets start frozen; flip to Active (status code 0).
        contract_client.update_market_status(&0);

        // -- Create the distinct pools (collateral first, then borrow) --
        //
        // `pool_assets` accumulates the per-pool oracle assets in the same
        // order we register them, so the positional `set_price_stable` feed
        // below lines up with each pool.
        let mut collateral_pools: Vec<Address> = Vec::with_capacity(n_collateral);
        let mut borrow_pools: Vec<Address> = Vec::with_capacity(n_borrow);
        let mut pool_assets = soroban_sdk::Vec::new(&e);

        for _ in 0..n_collateral {
            let admin = Address::generate(&e);
            let asset = setup_test_asset(&e, &admin, &users);
            register_pool(&e, &contract_client, &asset.token_address);
            pool_assets.push_back(Asset::Stellar(asset.token_address.clone()));
            collateral_pools.push(asset.token_address);
        }
        for _ in 0..n_borrow {
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
        for _ in 0..(n_collateral + n_borrow) {
            prices.push_back(unit_price);
        }
        oracle_client.set_price_stable(&prices);

        let borrower = ObligationKey::new(borrower_addr);
        let liquidity_provider = ObligationKey::new(liquidity_provider_addr);

        // -- Seed each borrow pool with liquidity from the provider --
        for pool in &borrow_pools {
            contract_client.deposit(&liquidity_provider, pool, &LIQUIDITY_AMOUNT, &None);
        }

        // -- Collateral positions (distinct pools) --
        let collateral_amount = COLLATERAL_TOTAL / n_collateral as i128;
        for pool in &collateral_pools {
            contract_client.add_collateral(&borrower, pool, &collateral_amount, &None);
        }

        // -- Borrow positions (distinct pools) --
        //
        // Each borrow re-values the obligation's growing position set, so by
        // the final borrow the host is already metering a full-size
        // revaluation against the armed pubnet budget.
        let borrow_amount = BORROW_TOTAL / n_borrow as i128;
        for pool in &borrow_pools {
            contract_client.borrow(&borrower, pool, &borrow_amount, &None);
        }

        Self {
            e,
            contract_client,
            oracle_client,
            borrower,
            liquidator: liquidator_addr,
            collateral_pools,
            borrow_pools,
        }
    }

    /// Repricing every collateral asset to `CRASHED_COLLATERAL_PRICE` puts the
    /// obligation under its close-LTV threshold. The extra ledger second evicts
    /// the market's per-timestamp oracle price cache (`market::oracle`), which
    /// would otherwise keep serving the pre-crash prices.
    fn crash_collateral_prices(&self) {
        let unit_price = 10_i128.pow(ORACLE_PRICE_DECIMALS);
        let mut prices = soroban_sdk::Vec::new(&self.e);
        for _ in 0..self.collateral_pools.len() {
            prices.push_back(CRASHED_COLLATERAL_PRICE);
        }
        for _ in 0..self.borrow_pools.len() {
            prices.push_back(unit_price);
        }
        self.oracle_client.set_price_stable(&prices);

        self.e.ledger().with_mut(|li| li.timestamp += 1);
    }

    /// A repay that stays under the 50% close factor of the targeted borrow
    /// position at every swept size.
    fn repay_amount(&self) -> i128 {
        BORROW_TOTAL / self.borrow_pools.len() as i128 / 10
    }

    fn positions_count(&self) -> u32 {
        self.contract_client.get_user_obligation(&self.borrower).positions_count
    }

    /// Resources metered for the last top-level invocation (the host resets the
    /// counters at the start of each one).
    fn last_invocation_cost(&self) -> Measurement {
        let budget = self.e.cost_estimate().budget();
        let r = self.e.cost_estimate().resources();

        Measurement {
            cpu: budget.cpu_instruction_cost(),
            mem: budget.memory_bytes_cost(),
            footprint_entries: r.disk_read_entries + r.memory_read_entries + r.write_entries,
            write_entries: r.write_entries,
            write_bytes: r.write_bytes,
        }
    }
}

struct Measurement {
    cpu: u64,
    mem: u64,
    footprint_entries: u32,
    write_entries: u32,
    write_bytes: u32,
}

impl Measurement {
    fn fits_pubnet(&self) -> bool {
        self.cpu < PUBNET_CPU_INSN_LIMIT
            && self.mem < PUBNET_MEM_BYTES_LIMIT
            && self.footprint_entries <= PUBNET_FOOTPRINT_ENTRY_LIMIT
            && self.write_entries <= PUBNET_WRITE_ENTRY_LIMIT
            && self.write_bytes <= PUBNET_WRITE_BYTES_LIMIT
    }
}

/// Queues and applies a default-config pool for `token_address`, advancing the
/// ledger clock past the mandatory queue period.
fn register_pool(e: &Env, contract_client: &MarketClient, token_address: &Address) {
    contract_client.queue_in_pool_set(token_address, &PoolConfig::default());
    e.ledger().with_mut(|li| li.timestamp += DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
    contract_client.apply_pool_set(token_address);
}

fn assert_within_pubnet_limits(label: &str, positions: u32, m: &Measurement) {
    assert!(m.cpu > 0 && m.mem > 0, "{label} at {positions} positions metered nothing");
    assert!(
        m.fits_pubnet(),
        "{label} at {positions} positions exceeds a pubnet transaction limit: cpu \
         {}/{PUBNET_CPU_INSN_LIMIT}, mem {}/{PUBNET_MEM_BYTES_LIMIT}, footprint entries \
         {}/{PUBNET_FOOTPRINT_ENTRY_LIMIT}, write entries {}/{PUBNET_WRITE_ENTRY_LIMIT}, write \
         bytes {}/{PUBNET_WRITE_BYTES_LIMIT}",
        m.cpu,
        m.mem,
        m.footprint_entries,
        m.write_entries,
        m.write_bytes,
    );
}

fn report_header() {
    println!(
        "{:<18} {:>3} {:>10} {:>7} {:>10} {:>7} {:>8} {:>7} {:>8} {:>5}",
        "op", "pos", "cpu", "%cpu", "mem", "%mem", "footprnt", "wr_ent", "wr_bytes", "fits",
    );
}

/// Prints one table row for the last top-level invocation and hands the
/// measurement back.
fn report(f: &PositionFixture, label: &str, positions: u32) -> Measurement {
    let m = f.last_invocation_cost();

    println!(
        "{label:<18} {positions:>3} {:>10} {:>6.2}% {:>10} {:>6.2}% {:>4}/{:<3} {:>3}/{:<3} {:>8} \
         {:>5}",
        m.cpu,
        100.0 * m.cpu as f64 / PUBNET_CPU_INSN_LIMIT as f64,
        m.mem,
        100.0 * m.mem as f64 / PUBNET_MEM_BYTES_LIMIT as f64,
        m.footprint_entries,
        PUBNET_FOOTPRINT_ENTRY_LIMIT,
        m.write_entries,
        PUBNET_WRITE_ENTRY_LIMIT,
        m.write_bytes,
        m.fits_pubnet(),
    );

    m
}

/// The obligation really does carry 5 borrow + 5 collateral positions, all in
/// distinct pools. This pins down the exact shape the resource-limit test
/// exercises (and guards against e.g. a pool being silently reused).
#[test]
fn obligation_holds_five_distinct_borrows_and_five_distinct_collaterals() {
    let f = PositionFixture::new();

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

/// A `borrow` over a 10-position obligation stays comfortably within the pubnet
/// per-transaction resource limits. Kept as the baseline the liquidation numbers
/// are read against: at the same position count `liquidate` costs ~2.5x on CPU
/// and ~7x on memory, which is the dimension that binds.
#[test]
fn ten_positions_stay_within_pubnet_resource_limits() {
    let f = PositionFixture::new();

    // Sanity-check the precondition the measurement relies on.
    assert_eq!(f.positions_count(), (N_COLLATERAL + N_BORROW) as u32);

    // Borrowing from an *existing* borrow pool keeps the position count at 10
    // while forcing a full sweep: interest accrual across all 10 pools plus a
    // revaluation of all 10 positions, in a single top-level invocation. The
    // host resets the counters at the start of it, so the snapshot afterwards
    // reflects exactly this operation.
    const PROBE_BORROW_AMOUNT: i128 = 1_000;
    f.contract_client.borrow(&f.borrower, &f.borrow_pools[0], &PROBE_BORROW_AMOUNT, &None);

    let positions = (N_COLLATERAL + N_BORROW) as u32;
    report_header();
    let m = report(&f, "borrow", positions);
    assert_within_pubnet_limits("borrow", positions, &m);
}

/// Cost of `liquidate` as a function of the borrower's position count — the
/// measurement that bounds `max_positions` — for both the natively-linked
/// contract and the deployable wasm.
///
/// The scenario's health is size-invariant by construction (fixed collateral
/// and debt totals, evenly split), so the only thing moving between rows is the
/// per-position sweep: interest accrual over every pool plus five full value
/// aggregations over every position.
///
/// Measured slope (wasm): ~1.49 MB, ~2.7M instructions and 5 footprint entries
/// per position. Memory is the constraint — at `MAX_RESERVES` positions it sits
/// at ~93% of `tx_memory_limit` while CPU is at ~18% and the footprint at ~37%.
#[test]
fn liquidate_scales_within_pubnet_resource_limits() {
    report_header();

    let mut by_build = Vec::with_capacity(MARKET_BUILDS.len());
    for (build_label, build) in MARKET_BUILDS {
        let mut costs = Vec::with_capacity(POSITION_SWEEP.len());
        for total in POSITION_SWEEP {
            let n_borrow = total / 2;
            let n_collateral = total - n_borrow;
            let f = PositionFixture::with_positions(build, n_collateral, n_borrow);
            assert_eq!(f.positions_count(), total as u32);

            f.crash_collateral_prices();
            f.contract_client.liquidate(
                &f.liquidator,
                &f.borrower,
                &f.borrow_pools[0],
                &f.collateral_pools[0],
                &f.repay_amount(),
                &0,
            );

            let label = format!("liquidate/{build_label}");
            let m = report(&f, &label, total as u32);
            assert_within_pubnet_limits(&label, total as u32, &m);
            costs.push(m);
        }
        by_build.push(costs);
    }

    for (i, total) in POSITION_SWEEP.iter().enumerate() {
        let (native, wasm) = (&by_build[0][i], &by_build[1][i]);
        println!(
            "wasm/native at {total:>3} positions: cpu {:.2}x  mem {:.2}x",
            wasm.cpu as f64 / native.cpu as f64,
            wasm.mem as f64 / native.mem as f64,
        );
    }
}

/// The flash-liquidation path: a single `submit_requests_batch` that flash
/// borrows the repay amount, liquidates, and swaps seized collateral back to
/// cover the flash repayment — the shape a liquidation bot with no inventory
/// actually submits. It is the heaviest liquidation shape the market exposes,
/// and the one K2's audit reports breaking at 2+ reserves.
#[test]
fn flash_liquidate_scales_within_pubnet_resource_limits() {
    report_header();

    for (build_label, build) in MARKET_BUILDS {
        for total in POSITION_SWEEP {
            let n_borrow = total / 2;
            let n_collateral = total - n_borrow;
            let f = PositionFixture::with_positions(build, n_collateral, n_borrow);
            let proxy_swap = f.e.register(MockProxySwap, ());
            // The swap provider calls `require_auth` from a nested frame, which
            // plain `mock_all_auths` rejects.
            f.e.mock_all_auths_allowing_non_root_auth();

            f.crash_collateral_prices();

            let repay_amount = f.repay_amount();
            // Flash repayment is principal plus the 0.01% fee, rounded up.
            let flash_repayment = repay_amount + (repay_amount + 9_999) / 10_000;
            let batch = svec![
                &f.e,
                Request::FlashBorrow(StandardRequest {
                    amount: repay_amount,
                    pool_address: f.borrow_pools[0].clone(),
                }),
                Request::Liquidate(LiquidateRequest {
                    borrower_obligation_key: f.borrower.clone(),
                    borrow_pool_address: f.borrow_pools[0].clone(),
                    collateral_pool_address: f.collateral_pools[0].clone(),
                    repay_amount,
                    min_demanded_collateral_amount: 0,
                }),
                Request::SwapForExactTokens(SwapForExactTokensRequest {
                    swap_provider: proxy_swap,
                    path: svec![&f.e, f.collateral_pools[0].clone(), f.borrow_pools[0].clone()],
                    max_amount_in: i128::MAX,
                    amount_out: flash_repayment,
                }),
            ];

            f.contract_client.submit_requests_batch(
                &ObligationKey::new(f.liquidator.clone()),
                &batch,
                &None,
            );

            let label = format!("flash-liq/{build_label}");
            let m = report(&f, &label, total as u32);
            assert_within_pubnet_limits(&label, total as u32, &m);
        }
    }
}
