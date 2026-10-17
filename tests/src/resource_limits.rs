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
//! `liquidate` is the heaviest of the health-sensitive paths: it runs *all four*
//! value aggregators plus `compute_min_collateral_threshold_scaled` before it
//! touches any seizure math, so it pays roughly twice the revaluation work a
//! `borrow` over the same position set does.
//!
//! It is not, however, the binding path. The bad-debt cover flow
//! (`issue_cover_bad_debt` / `claim_cover_bad_debt_results`) costs more per
//! position than a liquidation over the same obligation and is what actually
//! bounds `max_positions` — see `cover_bad_debt_collateral_heavy_pubnet_ceiling`.
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

use controlled_insurance_fund::{
    ControlledInsuranceFundContract, ControlledInsuranceFundContractClient,
};
use farms::{
    DelegatedFarmConfig, Delegation, FarmConfig, FarmsContract, FarmsContractClient,
    OptionalOracle, RewardConfig, RewardCurvePoint, RewardScheduleCurve, RewardType,
};
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
    token::StellarAssetClient,
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

mod insurance_fund_wasm {
    use soroban_sdk::contractimport;

    contractimport!(file = "../wasms/controlled_insurance_fund.wasm");
}

mod farms_wasm {
    use soroban_sdk::contractimport;

    contractimport!(file = "../wasms/farms.wasm");
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
/// `MAX_RESERVES` (10), the hard ceiling `max_positions` may be set to.
const POSITION_SWEEP: [usize; 4] = [3, 5, 7, 10];

/// Counts swept for the collateral-heavy bad-debt shape.
const COLLATERAL_HEAVY_SWEEP: [usize; 4] = [3, 5, 7, 10];

/// Smallest swept position count at which one of the bad-debt cover flow's two
/// calls overruns a pubnet limit, per shape, in `MARKET_BUILDS` order. `None`
/// means the shape survived the whole sweep. Pinning these both records the
/// ceiling and proves every smaller count in the sweep — 10 included — fits.
const EXPECTED_EVEN: [Option<usize>; 2] = [None, None];
const EXPECTED_BORROW_HEAVY: [Option<usize>; 2] = [None, None];
const EXPECTED_COLLATERAL_HEAVY: [Option<usize>; 2] = [None, None];
const EXPECTED_FARMS_BORROW_HEAVY: [Option<usize>; 2] = [None, None];
const EXPECTED_FARMS_LIQUIDATION: [Option<usize>; 2] = [None, None];
const EXPECTED_FARMS_SHARE_LIQUIDATION: [Option<usize>; 2] = [None, None];

/// Funded rewards, and schedule points each, on every farm the farms-attached measurements set up: the
/// caps a delegated farm allows.
const FARM_REWARDS: usize = 4;
const FARM_CURVE_POINTS: u64 = 4;

/// Both builds of the market are swept: the natively-linked contract these
/// tests normally drive, and the deployable wasm that actually runs on pubnet.
const MARKET_BUILDS: [(&str, MarketBuild); 2] =
    [("native", MarketBuild::Native), ("wasm", MarketBuild::Wasm)];

// ---- Oracle ----

// Reuse the canonical mock-oracle address and 14-decimal price scale the rest
// of the suite uses (`tests/src/lib.rs`, `tests/src/borrow.rs`).
const ORACLE_ADDRESS: &str = "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";
const ORACLE_PRICE_DECIMALS: u32 = 14;

// ---- Insurance fund ----

// Pinned like the oracle above rather than generated: `Env`'s contract ids come
// from a counter, so registering the fund with `register_at` keeps every later
// asset/pool address — and therefore the liquidation measurements — identical.
const INSURANCE_FUND_ADDRESS: &str = "CCQ2DINBUGQ2DINBUGQ2DINBUGQ2DINBUGQ2DINBUGQ2DINBUGQ2CNSG";

/// Crashed collateral price ($0.115) that makes the obligation liquidatable at
/// every swept size: close-LTV-weighted collateral is `0.115 * 0.80 * 5_000_000
/// = 460_000` against `500_000` of debt, while the unweighted LTV of 8_695 bps
/// stays under `DEFAULT_INSOLVENCY_LTV_BPS`, so liquidation takes the
/// solvent (close-factor-bounded) branch rather than the bad-debt one.
const CRASHED_COLLATERAL_PRICE: i128 = 11_500_000_000_000;

/// Smallest price the oracle accepts (`get_asset_price` rejects `<= 0`). At
/// 7-decimal amounts every collateral position floors to a value of zero, which
/// is what `require_no_liquidatable_collateral_exists` demands before the market
/// will hand an obligation to the insurance fund.
const DUST_COLLATERAL_PRICE: i128 = 1;

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
    insurance_fund_client: ControlledInsuranceFundContractClient<'a>,
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
        Self::build(build, n_collateral, n_borrow, false)
    }

    /// The same obligation with its collateral held as supply shares, so a liquidation moves supply
    /// stakes as well as debt ones.
    fn with_deposit_collateral(build: MarketBuild, n_collateral: usize, n_borrow: usize) -> Self {
        Self::build(build, n_collateral, n_borrow, true)
    }

    fn build(
        build: MarketBuild,
        n_collateral: usize,
        n_borrow: usize,
        deposit_collateral: bool,
    ) -> Self {
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
            protocol_version: e.ledger().get().protocol_version,
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

        // -- Insurance fund --
        //
        // A real contract rather than a bare address: the bad-debt cover flow
        // calls into it per borrow position and claims tokens back from it. It
        // follows the market's build, because registering the fund natively for
        // the wasm sweep would strip the VM cost out of exactly the loop these
        // measurements are about.
        let insurance_fund = Address::from_str(&e, INSURANCE_FUND_ADDRESS);
        match build {
            MarketBuild::Native => e.register_at(
                &insurance_fund,
                ControlledInsuranceFundContract,
                (contract_admin.clone(),),
            ),
            MarketBuild::Wasm => {
                e.register_at(&insurance_fund, insurance_fund_wasm::WASM, (contract_admin.clone(),))
            }
        };
        let insurance_fund_client = ControlledInsuranceFundContractClient::new(&e, &insurance_fund);

        // All three are minted huge balances by `setup_test_asset`. The fund is
        // deliberately not on this list — it is funded per borrow pool by
        // `fund_insurance_fund`, because minting to it unconditionally shifts
        // every other measurement in this file by ~1%.
        let users =
            vec![borrower_addr.clone(), liquidity_provider_addr.clone(), liquidator_addr.clone()];

        // -- Oracle --
        let oracle = Address::from_str(&e, ORACLE_ADDRESS);
        e.register_at(&oracle, MockPriceOracleWASM, ());
        let oracle_client = MockPriceOracleClient::new(&e, &oracle);

        // -- Market contract --
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
        insurance_fund_client.set_market(&contract_id);
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
            if deposit_collateral {
                contract_client.deposit(&borrower, pool, &collateral_amount, &None);
            } else {
                contract_client.add_collateral(&borrower, pool, &collateral_amount, &None);
            }
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
            insurance_fund_client,
            borrower,
            liquidator: liquidator_addr,
            collateral_pools,
            borrow_pools,
        }
    }

    /// Repricing every collateral asset to `CRASHED_COLLATERAL_PRICE` puts the
    /// obligation under its close-LTV threshold.
    fn crash_collateral_prices(&self) {
        self.reprice_collateral(CRASHED_COLLATERAL_PRICE);
    }

    /// Wipes out the collateral entirely, which is the state the bad-debt cover
    /// flow requires: `require_no_liquidatable_collateral_exists` refuses an
    /// obligation that still has a position worth seizing.
    fn wipe_out_collateral_prices(&self) {
        self.reprice_collateral(DUST_COLLATERAL_PRICE);
    }

    /// Borrow assets stay at $1.
    fn reprice_collateral(&self, collateral_price: i128) {
        let unit_price = 10_i128.pow(ORACLE_PRICE_DECIMALS);
        let mut prices = soroban_sdk::Vec::new(&self.e);
        for _ in 0..self.collateral_pools.len() {
            prices.push_back(collateral_price);
        }
        for _ in 0..self.borrow_pools.len() {
            prices.push_back(unit_price);
        }
        self.oracle_client.set_price_stable(&prices);

        self.e.ledger().with_mut(|li| li.timestamp += 1);
    }

    /// Gives the insurance fund enough of every borrow asset to cover the whole
    /// position, so `claim_cover_bad_debt_results` takes its settling branch.
    fn fund_insurance_fund(&self) {
        for pool in &self.borrow_pools {
            StellarAssetClient::new(&self.e, pool)
                .mint(&self.insurance_fund_client.address, &LIQUIDITY_AMOUNT);
        }
    }

    /// Farms on the given `(pool, is_supply)` slots, each paying `FARM_REWARDS` funded tokens on
    /// `FARM_CURVE_POINTS`-point schedules (the delegated caps), with the borrower's positions enrolled.
    fn attach_farms(
        &self,
        build: MarketBuild,
        slots: &[(Address, bool)],
    ) -> FarmsContractClient<'_> {
        let e = &self.e;
        let admin = Address::generate(e);
        let farms_address = match build {
            MarketBuild::Native => e.register(FarmsContract, (admin.clone(),)),
            MarketBuild::Wasm => e.register(farms_wasm::WASM, (admin.clone(),)),
        };
        let farms = FarmsContractClient::new(e, &farms_address);
        self.contract_client.set_farms_contract(&farms_address);

        let tokens: Vec<Address> = (0..FARM_REWARDS)
            .map(|_| {
                let token = e.register_stellar_asset_contract_v2(admin.clone()).address();
                StellarAssetClient::new(e, &token).mint(&admin, &i128::MAX);
                token
            })
            .collect();

        let mut pools = soroban_sdk::Vec::new(e);
        for (pool, is_supply) in slots {
            pools.push_back(pool.clone());
            let farm_id = farms.initialize_farm(
                &None,
                &FarmConfig {
                    token: pool.clone(),
                    deposit_cap: 0,
                    treasury_fee_bps: 0,
                    min_harvest_delay: 0,
                    min_stake_amount: 0,
                    delegation: Delegation::Delegated(DelegatedFarmConfig {
                        delegate_authority: self.contract_client.address.clone(),
                    }),
                    is_reward_once_enabled: false,
                    is_harvest_permissionless: false,
                    oracle: OptionalOracle::None,
                },
            );
            farms.unfreeze_farm(&farm_id);
            for token in &tokens {
                let mut points = soroban_sdk::Vec::new(e);
                for k in 0..FARM_CURVE_POINTS {
                    points.push_back(RewardCurvePoint {
                        ts_start: e.ledger().timestamp() + k * 24 * 60 * 60,
                        reward_per_time_unit: 1_000,
                    });
                }
                let curve = RewardScheduleCurve { points };
                let index = farms.initialize_reward(
                    &farm_id,
                    &RewardConfig {
                        reward_token: token.clone(),
                        reward_type: RewardType::Proportional,
                        reward_schedule_curve: curve,
                    },
                );
                farms.add_rewards(&admin, &farm_id, &index, &1_000_000_000);
            }
            if *is_supply {
                self.contract_client.set_pool_supply_farm(pool, &farm_id);
            } else {
                self.contract_client.set_pool_debt_farm(pool, &farm_id);
            }
        }
        self.contract_client.refresh_farm_stakes(&self.borrower, &pools);
        farms
    }

    /// A farm on every borrow pool: the side a borrow-heavy cover settles.
    fn debt_farm_slots(&self) -> Vec<(Address, bool)> {
        self.borrow_pools.iter().map(|pool| (pool.clone(), false)).collect()
    }

    /// A farm on the pool of every position: supply farms on the collateral pools, debt farms on the
    /// borrow pools.
    fn every_position_farm_slots(&self) -> Vec<(Address, bool)> {
        self.collateral_pools
            .iter()
            .map(|pool| (pool.clone(), true))
            .chain(self.borrow_pools.iter().map(|pool| (pool.clone(), false)))
            .collect()
    }

    /// Farms on the two pools a liquidation of the first borrow against the first collateral touches.
    fn liquidation_farm_slots(&self) -> Vec<(Address, bool)> {
        vec![(self.borrow_pools[0].clone(), false), (self.collateral_pools[0].clone(), true)]
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

/// Cost of `liquidate` as a function of the borrower's position count, for both
/// the natively-linked contract and the deployable wasm.
///
/// The scenario's health is size-invariant by construction (fixed collateral
/// and debt totals, evenly split), so the only thing moving between rows is the
/// per-position sweep: interest accrual over every pool plus five full value
/// aggregations over every position.
///
/// Measured slope (wasm): ~0.23 MB, ~1.9M instructions and 3 footprint entries
/// per position. At `MAX_RESERVES` positions it takes 12% of `tx_memory_limit`,
/// 6% of CPU and 51 of 400 footprint entries.
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

/// Drives the whole bad-debt cover flow over one obligation and reports each of
/// its two top-level invocations. Soroban budgets per transaction, so the flow
/// is only dangerous if a *single* call blows the ceiling — hence the split.
fn measure_cover_bad_debt(
    build: MarketBuild,
    label: &str,
    n_collateral: usize,
    n_borrow: usize,
) -> (Measurement, Measurement) {
    let total = (n_collateral + n_borrow) as u32;
    let f = PositionFixture::with_positions(build, n_collateral, n_borrow);
    assert_eq!(f.positions_count(), total);

    f.wipe_out_collateral_prices();
    f.fund_insurance_fund();

    f.contract_client.issue_cover_bad_debt(&f.borrower);
    let issue = report(&f, &format!("issue-cover/{label}"), total);

    // `ControlledInsuranceFundContract` always answers `Recorded`, so the issue
    // call leaves one pending request per borrow position, numbered from 0.
    // Approving them all puts the claim on its heaviest (`Ready`) branch, which
    // pays a `get_status`, a `claim_coverage` and two token `balance` calls per
    // request.
    for request_id in 0..n_borrow as u64 {
        // Clamped to the request amount by the fund.
        f.insurance_fund_client.mark_ready(&request_id, &i128::MAX);
    }

    f.contract_client.claim_cover_bad_debt_results(&f.borrower);
    let claim = report(&f, &format!("claim-cover/{label}"), total);

    (issue, claim)
}

/// Sweeps one collateral/borrow split of the bad-debt cover flow and returns the
/// smallest position count at which *either* of its two calls overruns a pubnet
/// limit — i.e. the count at which the obligation can no longer be closed out.
fn sweep_cover_bad_debt(
    build: MarketBuild,
    label: &str,
    sweep: &[usize],
    split: impl Fn(usize) -> (usize, usize),
) -> Option<usize> {
    let mut first_overrun = None;

    for &total in sweep {
        let (n_collateral, n_borrow) = split(total);
        let (issue, claim) = measure_cover_bad_debt(build, label, n_collateral, n_borrow);

        if !(issue.fits_pubnet() && claim.fits_pubnet()) && first_overrun.is_none() {
            first_overrun = Some(total);
        }
    }

    first_overrun
}

/// Ceilings of one bad-debt shape across both market builds, in `MARKET_BUILDS`
/// order.
fn cover_bad_debt_ceilings(
    label_suffix: &str,
    sweep: &[usize],
    split: impl Fn(usize) -> (usize, usize) + Copy,
) -> Vec<Option<usize>> {
    report_header();

    MARKET_BUILDS
        .iter()
        .map(|(build_label, build)| {
            sweep_cover_bad_debt(*build, &format!("{build_label}{label_suffix}"), sweep, split)
        })
        .collect()
}

/// Cost of the bad-debt cover flow at the same even collateral/borrow split
/// `liquidate_scales_within_pubnet_resource_limits` uses, so the two curves are
/// directly comparable. At 10 positions on wasm `issue_cover_bad_debt` costs
/// 21_753_488 memory bytes against `liquidate`'s 16_537_935 and the flash
/// liquidation's 16_920_691, which makes it — not liquidation — the path that
/// bounds `max_positions`.
#[test]
fn cover_bad_debt_even_split_pubnet_ceiling() {
    let ceilings = cover_bad_debt_ceilings("", &POSITION_SWEEP, |total| {
        let n_borrow = total / 2;
        (total - n_borrow, n_borrow)
    });

    assert_eq!(ceilings, EXPECTED_EVEN, "even-split ceiling moved");
}

/// A single collateral pool with every remaining position a borrow, which
/// maximises the per-borrow insurance-fund calls.
///
/// This is the shape the "two cross-contract calls per borrow position" concern
/// points at, and here the *claim* is the expensive half, not the issue: it pays
/// four cross-contract calls per recorded request against the issue's two (19
/// borrows on wasm: 16_942_818 memory bytes against the issue's 12_068_396). It
/// overran at 19 borrows (57_629_267) until protocol 28 began refunding each
/// callee VM's linear memory when its frame pops.
#[test]
fn cover_bad_debt_borrow_heavy_pubnet_ceiling() {
    let ceilings = cover_bad_debt_ceilings("/bh", &POSITION_SWEEP, |total| (1, total - 1));

    assert_eq!(ceilings, EXPECTED_BORROW_HEAVY, "borrow-heavy ceiling moved");
}

/// One borrow — the minimum `require_borrow_exists` accepts — and every
/// remaining position a collateral. The worst of the three shapes and the one
/// that sets the ceiling for the whole flow.
///
/// Collateral positions used to be the expensive ones: the threshold was
/// re-derived inside the per-deposit loop, and the `decimals()` behind it is the
/// one oracle read the market does not cache — ~2.8 MB per deposit on wasm against
/// ~1.7 MB for a borrow. Hoisting it out cut this shape ~40% (10 positions:
/// 63.06% → 39.43% of memory) and removed it as the binding path entirely.
#[test]
fn cover_bad_debt_collateral_heavy_pubnet_ceiling() {
    let ceilings = cover_bad_debt_ceilings("/ch", &COLLATERAL_HEAVY_SWEEP, |total| (total - 1, 1));

    assert_eq!(ceilings, EXPECTED_COLLATERAL_HEAVY, "collateral-heavy ceiling moved");
}

/// `issue_cover_bad_debt` pushes a stake for every position it settles, so with farms attached it is
/// the call farms make heaviest. Measured on the borrow-heavy shape with a farm on the pool of every
/// position, each stake going to zero across every funded reward, against the same obligation with no
/// farms.
///
/// Every reward is its own entry, carrying its schedule, so each push rewrites `FARM_REWARDS + 2`
/// entries. Every swept size fits; at `MAX_RESERVES` the cover takes 95_752 of 132_096 write bytes on
/// wasm, the tightest dimension of any shape.
#[test]
fn issue_cover_with_farms_pubnet_ceiling() {
    report_header();

    let mut ceilings = Vec::with_capacity(MARKET_BUILDS.len());
    for (build_label, build) in MARKET_BUILDS {
        let mut first_overrun = None;
        for total in POSITION_SWEEP {
            let mut issue = Vec::with_capacity(2);
            for with_farms in [false, true] {
                let f = PositionFixture::with_positions(build, 1, total - 1);
                if with_farms {
                    f.attach_farms(build, &f.every_position_farm_slots());
                }
                f.wipe_out_collateral_prices();
                f.fund_insurance_fund();
                f.contract_client.issue_cover_bad_debt(&f.borrower);
                let farms_label = if with_farms { "+farms" } else { "" };
                issue.push(report(&f, &format!("issue/{build_label}{farms_label}"), total as u32));
            }
            let (plain, farmed) = (&issue[0], &issue[1]);
            println!(
                "farms add at {total:>3} positions ({build_label}): cpu +{}  mem +{}  write bytes \
                 +{}",
                farmed.cpu - plain.cpu,
                farmed.mem - plain.mem,
                farmed.write_bytes - plain.write_bytes,
            );
            if !farmed.fits_pubnet() && first_overrun.is_none() {
                first_overrun = Some(total);
            }
        }
        ceilings.push(first_overrun);
    }

    assert_eq!(ceilings, EXPECTED_FARMS_BORROW_HEAVY, "farms-attached ceiling moved");
}

/// A liquidation pushes at most three stakes (the borrower's debt and collateral pools, and the batch's
/// own obligation), so farms add a bounded cost on top of the position sweep. Both the direct and the
/// flash shape are measured with farms on every pool at the delegated caps. Every swept size fits: on
/// wasm at `MAX_RESERVES` the flash shape takes 14% of memory and 42_216 write bytes.
#[test]
fn liquidations_with_farms_pubnet_ceiling() {
    report_header();

    let mut ceilings = Vec::with_capacity(MARKET_BUILDS.len());
    for (build_label, build) in MARKET_BUILDS {
        let mut first_overrun = None;
        for total in POSITION_SWEEP {
            let n_borrow = total / 2;
            for flash in [false, true] {
                let f = PositionFixture::with_positions(build, total - n_borrow, n_borrow);
                f.attach_farms(build, &f.liquidation_farm_slots());
                f.crash_collateral_prices();
                let repay_amount = f.repay_amount();
                if flash {
                    let proxy_swap = f.e.register(MockProxySwap, ());
                    f.e.mock_all_auths_allowing_non_root_auth();
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
                            path: svec![
                                &f.e,
                                f.collateral_pools[0].clone(),
                                f.borrow_pools[0].clone()
                            ],
                            max_amount_in: i128::MAX,
                            amount_out: flash_repayment,
                        }),
                    ];
                    f.contract_client.submit_requests_batch(
                        &ObligationKey::new(f.liquidator.clone()),
                        &batch,
                        &None,
                    );
                } else {
                    f.contract_client.liquidate(
                        &f.liquidator,
                        &f.borrower,
                        &f.borrow_pools[0],
                        &f.collateral_pools[0],
                        &repay_amount,
                        &0,
                    );
                }
                let shape = if flash { "flash-liq" } else { "liquidate" };
                let m = report(&f, &format!("{shape}/{build_label}+farms"), total as u32);
                if !m.fits_pubnet() && first_overrun.is_none() {
                    first_overrun = Some(total);
                }
            }
        }
        ceilings.push(first_overrun);
    }

    assert_eq!(ceilings, EXPECTED_FARMS_LIQUIDATION, "farms-attached liquidation ceiling moved");
}

/// A batched liquidation of collateral held as supply shares moves three stakes: the borrower's debt,
/// the borrower's supply, and the supply credited to the batch's own obligation. Every swept size fits:
/// on wasm at `MAX_RESERVES` it takes 16% of memory and 42_948 write bytes.
#[test]
fn share_collateral_liquidation_with_farms_pubnet_ceiling() {
    report_header();

    let mut ceilings = Vec::with_capacity(MARKET_BUILDS.len());
    for (build_label, build) in MARKET_BUILDS {
        let mut first_overrun = None;
        for total in POSITION_SWEEP {
            let n_borrow = total / 2;
            let f = PositionFixture::with_deposit_collateral(build, total - n_borrow, n_borrow);
            f.attach_farms(build, &f.liquidation_farm_slots());
            f.crash_collateral_prices();
            let liquidator = ObligationKey::new(f.liquidator.clone());
            let batch = svec![
                &f.e,
                Request::Liquidate(LiquidateRequest {
                    borrower_obligation_key: f.borrower.clone(),
                    borrow_pool_address: f.borrow_pools[0].clone(),
                    collateral_pool_address: f.collateral_pools[0].clone(),
                    repay_amount: f.repay_amount(),
                    min_demanded_collateral_amount: 0,
                }),
            ];
            f.contract_client.submit_requests_batch(&liquidator, &batch, &None);
            let m = report(&f, &format!("share-liq/{build_label}+farms"), total as u32);

            let seized = f.contract_client.get_user_obligation(&liquidator).deposits;
            assert!(!seized.is_empty(), "the batch credited no supply shares to move a stake");
            if !m.fits_pubnet() && first_overrun.is_none() {
                first_overrun = Some(total);
            }
        }
        ceilings.push(first_overrun);
    }

    assert_eq!(
        ceilings, EXPECTED_FARMS_SHARE_LIQUIDATION,
        "share-collateral liquidation ceiling moved"
    );
}

/// The largest obligation's cover pushes every farm it touches in the same transaction, with a farm on
/// every position's pool: no stake is left for a resync, and no farm cap is needed.
#[test]
fn the_largest_cover_zeroes_every_farm_stake_at_once() {
    let total = MAX_RESERVES as usize;
    let f = PositionFixture::with_positions(MarketBuild::Wasm, 1, total - 1);
    let farms = f.attach_farms(MarketBuild::Wasm, &f.every_position_farm_slots());
    f.wipe_out_collateral_prices();
    f.fund_insurance_fund();
    f.contract_client.issue_cover_bad_debt(&f.borrower);
    let m = f.last_invocation_cost();
    assert_within_pubnet_limits("issue-cover/wasm+farms", total as u32, &m);

    let delegatee = farms_interface::Delegatee {
        owner: f.borrower.user.clone(),
        seed: f.borrower.seed.clone(),
    };
    for (pool, _) in f.debt_farm_slots() {
        let farm_id = f.contract_client.get_pool_data(&pool).pool.farm_debt.unwrap();
        assert_eq!(farms.get_delegatee_state(&delegatee, &farm_id).active_stake, 0);
    }
}

/// No farm cap: both sides of every pool can be farmed.
#[test]
fn a_market_farms_both_sides_of_every_pool() {
    let f = PositionFixture::with_positions(MarketBuild::Native, 4, 4);
    let slots: Vec<(Address, bool)> = f
        .borrow_pools
        .iter()
        .chain(&f.collateral_pools)
        .flat_map(|pool| [(pool.clone(), true), (pool.clone(), false)])
        .collect();
    f.attach_farms(MarketBuild::Native, &slots);
    for (pool, _) in &slots {
        let data = f.contract_client.get_pool_data(pool).pool;
        assert!(data.farm_supply.is_some() && data.farm_debt.is_some());
    }
}
