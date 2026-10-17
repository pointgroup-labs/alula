extern crate std;

use sep_40_oracle::{Asset, PriceData};
use soroban_sdk::{
    Address, BytesN, Env, I256, Map, Symbol, contract, contractimpl, contracttype, map as smap,
    testutils::{Address as _, Ledger},
};

use crate::{
    chainlink::{ProxyReadError, Round},
    contract::{ChainlinkSep40AdapterContract, ChainlinkSep40AdapterContractClient},
};

#[contracttype]
enum MockKey {
    Round(BytesN<32>),
    Frozen,
}

/// Chainlink's proxy as far as the adapter sees it: one round per id, or a panic while frozen.
#[contract]
struct MockProxy;

#[contractimpl]
impl MockProxy {
    pub fn set_round(e: Env, data_id: BytesN<32>, answer: I256, timestamp: u64) {
        e.storage()
            .instance()
            .set(&MockKey::Round(data_id), &Round { answer, round_id: 1, timestamp });
    }

    pub fn set_frozen(e: Env, frozen: bool) {
        e.storage().instance().set(&MockKey::Frozen, &frozen);
    }

    pub fn latest_round(
        e: Env,
        data_id: BytesN<32>,
        _decimals: u32,
    ) -> Result<Round, ProxyReadError> {
        assert!(!e.storage().instance().get(&MockKey::Frozen).unwrap_or(false), "feed frozen");
        e.storage().instance().get(&MockKey::Round(data_id)).ok_or(ProxyReadError::NoDataPresent)
    }
}

struct Setup<'a> {
    e: Env,
    proxy: MockProxyClient<'a>,
    adapter: ChainlinkSep40AdapterContractClient<'a>,
    token: Address,
    data_id: BytesN<32>,
}

const DECIMALS: u32 = 14;

fn setup<'a>() -> Setup<'a> {
    let e = Env::default();
    e.ledger().set_timestamp(1_800_000_000);
    let proxy = MockProxyClient::new(&e, &e.register(MockProxy, ()));
    let token = Address::generate(&e);
    let data_id = BytesN::from_array(&e, &[7; 32]);
    proxy.set_round(&data_id, &I256::from_i128(&e, 18_265_440_093_400), &1_799_999_000);
    let adapter = ChainlinkSep40AdapterContractClient::new(
        &e,
        &e.register(
            ChainlinkSep40AdapterContract,
            (proxy.address.clone(), DECIMALS, smap![&e, (token.clone(), data_id.clone())]),
        ),
    );
    Setup { e, proxy, adapter, token, data_id }
}

#[test]
fn test_lastprice_is_the_latest_round() {
    let s = setup();
    assert_eq!(
        pt(s.adapter.lastprice(&Asset::Stellar(s.token.clone()))),
        Some((18_265_440_093_400, 1_799_999_000))
    );
    assert!(matches!(s.adapter.base(), Asset::Other(usd) if usd == Symbol::new(&s.e, "USD")));
    assert_eq!(s.adapter.decimals(), DECIMALS);
    assert!(matches!(s.adapter.assets().first(), Some(Asset::Stellar(t)) if t == s.token));
}

/// A staleness limit equal to the 24 h heartbeat would drop a quiet feed every day while its next round lands.
#[test]
fn test_resolution_outlasts_the_heartbeat() {
    assert!(setup().adapter.resolution() > 24 * 60 * 60);
}

#[test]
fn test_unknown_or_unpriceable_answers_are_none() {
    let s = setup();
    assert_eq!(pt(s.adapter.lastprice(&Asset::Stellar(Address::generate(&s.e)))), None);
    assert_eq!(pt(s.adapter.lastprice(&Asset::Other(Symbol::new(&s.e, "XLM")))), None);

    for answer in [
        I256::from_i128(&s.e, 0),
        I256::from_i128(&s.e, -1),
        I256::from_i128(&s.e, i128::MAX).add(&I256::from_i32(&s.e, 1)),
    ] {
        s.proxy.set_round(&s.data_id, &answer, &1_799_999_000);
        assert_eq!(pt(s.adapter.lastprice(&Asset::Stellar(s.token.clone()))), None);
    }
}

#[test]
fn test_a_frozen_feed_is_none_not_a_failure() {
    let s = setup();
    s.proxy.set_frozen(&true);
    assert_eq!(pt(s.adapter.lastprice(&Asset::Stellar(s.token.clone()))), None);
}

#[test]
#[should_panic(expected = "Error(Contract, #50)")]
fn test_constructor_refuses_an_id_the_proxy_does_not_know() {
    let s = setup();
    let unknown = BytesN::from_array(&s.e, &[9; 32]);
    s.e.register(
        ChainlinkSep40AdapterContract,
        (s.proxy.address.clone(), DECIMALS, smap![&s.e, (Address::generate(&s.e), unknown)]),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #1)")]
fn test_constructor_refuses_more_than_18_decimals() {
    let s = setup();
    s.e.register(
        ChainlinkSep40AdapterContract,
        (s.proxy.address.clone(), 19_u32, smap![&s.e, (s.token.clone(), s.data_id.clone())]),
    );
}

#[test]
#[should_panic(expected = "Error(Contract, #2)")]
fn test_constructor_refuses_an_empty_feed_map() {
    let s = setup();
    s.e.register(
        ChainlinkSep40AdapterContract,
        (s.proxy.address.clone(), DECIMALS, Map::<Address, BytesN<32>>::new(&s.e)),
    );
}

fn pt(p: Option<PriceData>) -> Option<(i128, u64)> {
    p.map(|p| (p.price, p.timestamp))
}
