extern crate std;

use sep_40_oracle::Asset;
use soroban_sdk::{
    Address, Env, Map, Symbol, U256, contract, contractimpl, contracttype, map as smap,
    testutils::{Address as _, Ledger},
};

use crate::{
    contract::{RedStoneSep40AdapterContract, RedStoneSep40AdapterContractClient},
    redstone::RSPriceData,
};

#[contracttype]
enum MockKey {
    Decimals,
    Data,
}

/// One RedStone price feed: its decimals are fixed at deploy, like the real feed's constant.
#[contract]
struct MockFeed;

#[contractimpl]
impl MockFeed {
    pub fn __constructor(e: Env, decimals: u64) {
        e.storage().instance().set(&MockKey::Decimals, &decimals);
    }

    pub fn set_price(e: Env, price: u128, timestamp_s: u64) {
        let data = RSPriceData {
            package_timestamp: timestamp_s * 1_000,
            price: U256::from_u128(&e, price),
            write_timestamp: timestamp_s * 1_000,
        };
        e.storage().instance().set(&MockKey::Data, &data);
    }

    pub fn read_price_data(e: Env) -> RSPriceData {
        e.storage().instance().get(&MockKey::Data).unwrap()
    }

    pub fn decimals(e: Env) -> u64 {
        e.storage().instance().get(&MockKey::Decimals).unwrap()
    }
}

const NOW: u64 = 1_800_000_000;
const DECIMALS: u32 = 14;

fn env() -> Env {
    let e = Env::default();
    e.mock_all_auths();
    e.ledger().set_timestamp(NOW);
    e
}

fn feed(e: &Env, decimals: u64) -> MockFeedClient<'_> {
    MockFeedClient::new(e, &e.register(MockFeed, (decimals,)))
}

fn adapter<'a>(e: &Env, feeds: Map<Address, Address>) -> RedStoneSep40AdapterContractClient<'a> {
    let usd = Asset::Other(Symbol::new(e, "USD"));
    let id = e.register(
        RedStoneSep40AdapterContract,
        (Address::generate(e), usd, DECIMALS, 3_600_u32, feeds),
    );
    RedStoneSep40AdapterContractClient::new(e, &id)
}

fn price(adapter: &RedStoneSep40AdapterContractClient, token: &Address) -> Option<i128> {
    adapter.lastprice(&Asset::Stellar(token.clone())).map(|p| p.price)
}

#[test]
fn test_each_feed_is_rescaled_from_its_own_decimals() {
    let e = env();
    let (eight, eighteen, six) = (feed(&e, 8), feed(&e, 18), feed(&e, 6));
    let (a, b, c) = (Address::generate(&e), Address::generate(&e), Address::generate(&e));
    let adapter = adapter(
        &e,
        smap![&e, (a.clone(), eight.address.clone()), (b.clone(), eighteen.address.clone())],
    );
    adapter.add_feed(&c, &six.address);

    eight.set_price(&150_000_000, &NOW);
    eighteen.set_price(&1_500_000_000_000_000_000, &NOW);
    six.set_price(&1_500_000, &NOW);
    let one_and_a_half = 150_000_000_000_000;
    assert_eq!(price(&adapter, &a), Some(one_and_a_half));
    assert_eq!(price(&adapter, &b), Some(one_and_a_half));
    assert_eq!(price(&adapter, &c), Some(one_and_a_half));
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_the_constructor_refuses_a_feed_wider_than_18_decimals() {
    let e = env();
    adapter(&e, smap![&e, (Address::generate(&e), feed(&e, 19).address.clone())]);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_add_feed_refuses_a_feed_wider_than_18_decimals() {
    let e = env();
    adapter(&e, Map::new(&e)).add_feed(&Address::generate(&e), &feed(&e, 19).address);
}

#[test]
#[should_panic(expected = "Error(Contract, #15)")]
fn test_add_feed_refuses_a_feed_without_decimals() {
    let e = env();
    adapter(&e, Map::new(&e)).add_feed(&Address::generate(&e), &Address::generate(&e));
}
