#![cfg(test)]
//! Liquidation prices every deposit of a borrower, so a borrower cannot add to a pool whose price
//! the oracle cannot return. Runs on the aggregated-oracle build with three pools.

extern crate std;

use market::{
    constants::{
        DEFAULT_BAD_DEBT_LOCK_D, DEFAULT_MAX_POSITIONS, DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
    },
    contract::{MarketClient, MarketContract},
    error::MCError,
    obligation::ObligationKey,
    pool::PoolConfig,
    request::{Request, StandardRequest},
    storage::MarketInitParams,
};
use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{
    Address, Env, String, Symbol, Vec, contract, contractimpl,
    testutils::{Address as _, Ledger},
    vec as svec,
};

use crate::setup_test_asset;

mod aggregator_wasm {
    soroban_sdk::contractimport!(file = "../wasms/aggregated_oracle.wasm");
}

/// A heartbeat SEP-40 source quoting per-asset prices stamped now.
#[contract]
pub struct HeartbeatSource;

#[contractimpl]
impl HeartbeatSource {
    pub fn set(e: Env, asset: Address, price: i128) {
        e.storage().instance().set(&asset, &price);
    }
    pub fn drop_asset(e: Env, asset: Address) {
        e.storage().instance().remove(&asset);
    }
    pub fn decimals(_e: Env) -> u32 {
        14
    }
    pub fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
        let Asset::Stellar(a) = asset else { return None };
        let p: Option<i128> = e.storage().instance().get(&a);
        p.map(|price| PriceData { price, timestamp: e.ledger().timestamp() })
    }
    pub fn resolution(_e: Env) -> u32 {
        300
    }
    pub fn base(e: Env) -> Asset {
        Asset::Other(Symbol::new(&e, "USD"))
    }
    pub fn assets(e: Env) -> Vec<Asset> {
        svec![&e]
    }
}

const ONE: i128 = 100_000_000_000_000;
const UNIT: i128 = 10_000_000;
const MAX_DEV_BPS: u32 = 3000;
const WINDOW: u64 = 3600;
const DUST: i128 = 10;

struct Scene<'a> {
    e: Env,
    m: MarketClient<'a>,
    src: HeartbeatSourceClient<'a>,
    gold: Address,
    btc: Address,
    usdc: Address,
    borrower: ObligationKey,
    control: ObligationKey,
    lender: ObligationKey,
    liquidator: Address,
}

fn advance(e: &Env, secs: u64) {
    e.ledger().with_mut(|l| l.timestamp += secs);
}

/// GOLD/BTC/USDC pools on the aggregator; BTC is listed only if `list_btc`. Two identical
/// borrowers hold GOLD collateral and 65 % USDC debt; the lender only supplies USDC.
fn scene<'a>(list_btc: bool) -> Scene<'a> {
    let e = Env::default();
    // The test env sums every call into one budget; on chain each transaction has its own.
    e.cost_estimate().budget().reset_unlimited();
    e.mock_all_auths_allowing_non_root_auth();
    e.ledger().with_mut(|l| {
        l.timestamp = 1_700_000_000;
        l.min_temp_entry_ttl = 500_000;
        l.min_persistent_entry_ttl = 500_000;
        l.max_entry_ttl = 10_000_000;
    });
    let users: std::vec::Vec<Address> = (0..4).map(|_| Address::generate(&e)).collect();
    let admin = Address::generate(&e);
    let gold = setup_test_asset(&e, &admin, &users).token_address;
    let btc = setup_test_asset(&e, &admin, &users).token_address;
    let usdc = setup_test_asset(&e, &admin, &users).token_address;

    let source = e.register(HeartbeatSource, ());
    let src = HeartbeatSourceClient::new(&e, &source);
    for a in [&gold, &btc, &usdc] {
        src.set(a, &ONE);
    }
    let agg_id = e.register(
        aggregator_wasm::WASM,
        (
            admin.clone(),
            Asset::Other(Symbol::new(&e, "USD")),
            14u32,
            360u64,
            svec![
                &e,
                aggregator_wasm::OracleConfigInput {
                    address: source.clone(),
                    is_stellar_data_based: true,
                    is_price_update_periodic: false,
                }
            ],
        ),
    );
    let agg = aggregator_wasm::Client::new(&e, &agg_id);
    agg.add_asset(&Symbol::new(&e, "GOLD"), &gold, &MAX_DEV_BPS, &WINDOW);
    agg.add_asset(&Symbol::new(&e, "USDC"), &usdc, &MAX_DEV_BPS, &WINDOW);
    if list_btc {
        agg.add_asset(&Symbol::new(&e, "BTC"), &btc, &MAX_DEV_BPS, &WINDOW);
    }

    let m_id = e.register(
        MarketContract,
        (
            String::from_str(&e, "market"),
            admin.clone(),
            agg_id.clone(),
            Address::generate(&e),
            Address::generate(&e),
            MarketInitParams {
                max_positions: DEFAULT_MAX_POSITIONS,
                min_collateral_value_cents: 500,
                insolvency_ltv_bps: 9_500,
                update_in_queue_period: DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS,
                is_owned: true,
                bad_debt_lock_d: DEFAULT_BAD_DEBT_LOCK_D,
            },
        ),
    );
    let m = MarketClient::new(&e, &m_id);
    m.update_market_status(&0);
    for a in [&gold, &btc, &usdc] {
        m.queue_in_pool_set(a, &PoolConfig::default());
    }
    advance(&e, DEFAULT_UPDATE_POOL_CONFIG_IN_QUEUE_SECONDS);
    for a in [&gold, &btc, &usdc] {
        m.apply_pool_set(a);
    }

    let borrower = ObligationKey::new(users[0].clone());
    let control = ObligationKey::new(users[1].clone());
    let lender = ObligationKey::new(users[2].clone());
    m.deposit(&lender, &usdc, &(1_000_000 * UNIT), &None);
    for k in [&borrower, &control] {
        m.add_collateral(k, &gold, &(10_000 * UNIT), &None);
        m.borrow(k, &usdc, &(6_500 * UNIT), &None);
    }

    Scene { e, m, src, gold, btc, usdc, borrower, control, lender, liquidator: users[3].clone() }
}

fn flatten<T, C>(
    r: Result<Result<T, C>, Result<MCError, soroban_sdk::InvokeError>>,
) -> Result<(), MCError>
where
    C: core::fmt::Debug,
{
    match r {
        Ok(Ok(_)) => Ok(()),
        Err(Ok(err)) => Err(err),
        Ok(Err(conv)) => panic!("conversion error {conv:?}"),
        Err(Err(invoke)) => panic!("invoke error {invoke:?}"),
    }
}

impl Scene<'_> {
    fn liquidate(&self, k: &ObligationKey) -> Result<(), MCError> {
        flatten(self.m.try_liquidate(
            &self.liquidator,
            k,
            &self.usdc,
            &self.gold,
            &(1_000 * UNIT),
            &0,
        ))
    }

    /// Adds `DUST` of BTC through every entry point: deposit, add_collateral and both batch kinds.
    fn add_btc_dust(&self, k: &ObligationKey) -> [Result<(), MCError>; 4] {
        let request = || StandardRequest { amount: DUST, pool_address: self.btc.clone() };
        [
            flatten(self.m.try_deposit(k, &self.btc, &DUST, &None)),
            flatten(self.m.try_add_collateral(k, &self.btc, &DUST, &None)),
            flatten(self.m.try_submit_requests_batch(
                k,
                &svec![&self.e, Request::Deposit(request())],
                &None,
            )),
            flatten(self.m.try_submit_requests_batch(
                k,
                &svec![&self.e, Request::AddCollateral(request())],
                &None,
            )),
        ]
    }

    fn assert_borrower_dust_refused(&self) {
        assert_eq!(
            self.add_btc_dust(&self.borrower),
            [const { Err(MCError::OracleDoesNotKnowAssetPrice) }; 4]
        );
    }

    fn assert_both_liquidatable(&self) {
        assert_eq!(self.liquidate(&self.control), Ok(()));
        assert_eq!(self.liquidate(&self.borrower), Ok(()));
    }
}

#[test]
fn test_borrower_deposit_refused_while_breaker_withholds_price() {
    let s = scene(true);
    // The breaker compares against the last accepted read, so BTC needs one first.
    assert_eq!(s.m.get_pool_asset_oracle_price(&s.btc), ONE);
    advance(&s.e, 60);
    s.src.set(&s.gold, &(ONE * 81 / 100));
    s.src.set(&s.btc, &(ONE * 65 / 100));
    assert_eq!(
        s.m.try_get_pool_asset_oracle_price(&s.btc),
        Err(Ok(MCError::OracleDoesNotKnowAssetPrice))
    );

    s.assert_borrower_dust_refused();
    s.assert_both_liquidatable();
}

#[test]
fn test_borrower_deposit_refused_into_pool_oracle_does_not_list() {
    let s = scene(false);
    s.assert_borrower_dust_refused();

    s.src.set(&s.gold, &(ONE * 81 / 100));
    s.assert_both_liquidatable();
}

#[test]
fn test_borrower_deposit_refused_while_sources_stop_quoting() {
    let s = scene(true);
    advance(&s.e, 60);
    s.src.drop_asset(&s.btc);
    s.src.set(&s.gold, &(ONE * 81 / 100));

    s.assert_borrower_dust_refused();
    s.assert_both_liquidatable();
}

#[test]
fn test_debt_free_deposit_into_unpriced_pool_succeeds() {
    let s = scene(false);
    assert_eq!(s.add_btc_dust(&s.lender), [const { Ok(()) }; 4]);
}

#[test]
fn test_borrower_deposit_into_priced_pool_succeeds() {
    let s = scene(true);
    assert_eq!(s.add_btc_dust(&s.borrower), [const { Ok(()) }; 4]);
}
