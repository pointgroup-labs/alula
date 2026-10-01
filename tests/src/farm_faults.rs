//! A farm that refuses a stake push must not trap a lender's funds: the market proceeds and emits
//! `FarmStakePushFailed`. `set_farms_contract` keeps the pools' farm ids, so a repointed farms
//! contract that does not know them refuses every push.
#![cfg(test)]

use farms_interface::Delegatee;
use market::obligation::ObligationKey;
use soroban_sdk::{
    BytesN, Env, contract, contractimpl, symbol_short,
    testutils::{BytesN as _, Events},
};

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture};

/// A farms contract that refuses every delegated stake push, which is what a repointed market sees.
#[contract]
pub struct RefusingFarm;

#[contractimpl]
impl RefusingFarm {
    pub fn set_stake_delegated(
        _e: Env,
        _delegatee: Delegatee,
        _farm_id: BytesN<32>,
        _new_stake: i128,
    ) {
        panic!("this farm does not know that id");
    }
}

#[test]
fn test_a_refusing_farm_does_not_block_a_deposit() {
    let f = TestMarketFixture::new();
    let pool = f.usdc_pool_address.clone();
    let lender = ObligationKey::new(f.users[0].clone());

    let farm = f.e.register(RefusingFarm, ());
    f.contract_client.set_farms_contract(&farm);
    f.contract_client.set_pool_supply_farm(&pool, &BytesN::random(&f.e));

    let deposited = f.contract_client.try_deposit(&lender, &pool, &DEFAULT_DEPOSIT_AMOUNT, &None);
    assert!(deposited.is_ok(), "a refusing farm blocked a deposit: {deposited:?}");
}

#[test]
fn test_a_refusing_farm_does_not_trap_a_withdrawal() {
    let f = TestMarketFixture::new();
    let pool = f.usdc_pool_address.clone();
    let lender = ObligationKey::new(f.users[0].clone());

    // Deposit first, while no farm is wired, so the funds are genuinely in.
    f.contract_client.deposit(&lender, &pool, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let farm = f.e.register(RefusingFarm, ());
    f.contract_client.set_farms_contract(&farm);
    f.contract_client.set_pool_supply_farm(&pool, &BytesN::random(&f.e));

    let withdrawn =
        f.contract_client.try_withdraw(&lender, &pool, &(DEFAULT_DEPOSIT_AMOUNT / 2), &None);
    assert!(withdrawn.is_ok(), "a refusing farm trapped a lender's funds: {withdrawn:?}");
}

/// A farm that accepts, and records what it was given, so the guard cannot be shown to work by simply
/// never pushing anything.
#[contract]
pub struct RecordingFarm;

#[contractimpl]
impl RecordingFarm {
    pub fn set_stake_delegated(
        e: Env,
        _delegatee: Delegatee,
        _farm_id: BytesN<32>,
        new_stake: i128,
    ) {
        e.storage().instance().set(&symbol_short!("last"), &new_stake);
    }

    pub fn last(e: Env) -> i128 {
        e.storage().instance().get(&symbol_short!("last")).unwrap_or(-1)
    }
}

#[test]
fn test_a_working_farm_still_receives_the_stake() {
    let f = TestMarketFixture::new();
    let pool = f.usdc_pool_address.clone();
    let lender = ObligationKey::new(f.users[0].clone());

    let farm = f.e.register(RecordingFarm, ());
    f.contract_client.set_farms_contract(&farm);
    f.contract_client.set_pool_supply_farm(&pool, &BytesN::random(&f.e));

    f.contract_client.deposit(&lender, &pool, &DEFAULT_DEPOSIT_AMOUNT, &None);

    let recorded = RecordingFarmClient::new(&f.e, &farm).last();
    assert!(
        recorded > 0,
        "the guard swallowed a successful push as well: the farm recorded {recorded}"
    );
}

#[test]
fn test_a_refused_push_is_recorded_as_an_event() {
    // The market's event structs are private, so this counts the events one deposit emits: a
    // refused push leaves exactly one more than a deposit with no farm.
    fn events_of_one_deposit(wire_a_refusing_farm: bool) -> usize {
        let f = TestMarketFixture::new();
        let pool = f.usdc_pool_address.clone();
        let lender = ObligationKey::new(f.users[0].clone());
        if wire_a_refusing_farm {
            let farm = f.e.register(RefusingFarm, ());
            f.contract_client.set_farms_contract(&farm);
            f.contract_client.set_pool_supply_farm(&pool, &BytesN::random(&f.e));
        }
        f.contract_client.deposit(&lender, &pool, &DEFAULT_DEPOSIT_AMOUNT, &None);
        // `events().all()` is scoped to the most recent invocation in this SDK, not cumulative, so the
        // count after the deposit is the deposit's own.
        f.e.events().all().filter_by_contract(&f.contract_id).events().len()
    }

    let plain = events_of_one_deposit(false);
    let refused = events_of_one_deposit(true);
    assert_eq!(
        refused,
        plain + 1,
        "the refused push left no record: {refused} events against {plain} for the same deposit \
         with no farm wired"
    );
}
