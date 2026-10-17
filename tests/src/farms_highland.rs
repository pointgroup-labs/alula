//! One regression per farms finding of the Oct-2026 review: each scenario failed, froze a farm or let a
//! config change reach pending funds before the fix.
#![cfg(test)]

use farms::{
    DelegatedFarmConfig, Delegation, FCError, FarmConfig, FarmsContract, FarmsContractClient,
    LockingMode, NonDelegatedFarmConfig, OptionalOracle, RewardConfig, RewardCurvePoint,
    RewardScheduleCurve, RewardType,
};
use farms_interface::Delegatee;
use soroban_sdk::{
    Address, BytesN, Env,
    testutils::{Address as _, Ledger},
    token::{StellarAssetClient, TokenClient},
    vec as svec,
};

const DAY: u64 = 24 * 60 * 60;

struct Setup<'a> {
    e: Env,
    farms: FarmsContractClient<'a>,
    admin: Address,
    market: Address,
    staked: Address,
    reward: Address,
}

fn setup<'a>() -> Setup<'a> {
    let e = Env::default();
    e.mock_all_auths();
    e.cost_estimate().budget().reset_unlimited();
    e.ledger().set_timestamp(1_000);
    let admin = Address::generate(&e);
    let farms = FarmsContractClient::new(&e, &e.register(FarmsContract, (admin.clone(),)));
    let token = || {
        let address = e.register_stellar_asset_contract_v2(admin.clone()).address();
        StellarAssetClient::new(&e, &address).mint(&admin, &i128::MAX);
        address
    };
    let (staked, reward) = (token(), token());
    Setup { market: Address::generate(&e), e, farms, admin, staked, reward }
}

fn delegated(s: &Setup) -> Delegation {
    Delegation::Delegated(DelegatedFarmConfig { delegate_authority: s.market.clone() })
}

fn standalone(warmup: u64, cooldown: u64) -> Delegation {
    Delegation::NonDelegated(NonDelegatedFarmConfig {
        locking_ts: 0,
        locking_duration: 0,
        locking_mode: LockingMode::None,
        deposit_warmup_period: warmup,
        withdrawal_cooldown_period: cooldown,
        early_withdrawal_penalty_bps: 0,
    })
}

fn config(
    s: &Setup,
    delegation: Delegation,
    min_stake_amount: i128,
    deposit_cap: i128,
) -> FarmConfig {
    FarmConfig {
        token: s.staked.clone(),
        deposit_cap,
        treasury_fee_bps: 0,
        min_harvest_delay: 0,
        min_stake_amount,
        delegation,
        is_reward_once_enabled: false,
        is_harvest_permissionless: false,
        oracle: OptionalOracle::None,
    }
}

fn farm(s: &Setup, config: FarmConfig) -> BytesN<32> {
    let farm_id = s.farms.initialize_farm(&None, &config);
    s.farms.unfreeze_farm(&farm_id);
    farm_id
}

fn reward_config(s: &Setup, reward_type: RewardType, rate: i128) -> RewardConfig {
    RewardConfig {
        reward_token: s.reward.clone(),
        reward_type,
        reward_schedule_curve: RewardScheduleCurve {
            points: svec![
                &s.e,
                RewardCurvePoint { ts_start: s.e.ledger().timestamp(), reward_per_time_unit: rate }
            ],
        },
    }
}

/// A proportional reward paying `rate` per second from now, funded with `budget`.
fn funded_reward(s: &Setup, farm_id: &BytesN<32>, rate: i128, budget: i128) -> u32 {
    let index =
        s.farms.initialize_reward(farm_id, &reward_config(s, RewardType::Proportional, rate));
    s.farms.add_rewards(&s.admin, farm_id, &index, &budget);
    index
}

fn user(s: &Setup) -> Delegatee {
    let owner = Address::generate(&s.e);
    StellarAssetClient::new(&s.e, &s.staked).mint(&owner, &1_000_000_000);
    Delegatee::new(owner)
}

fn pass(s: &Setup, seconds: u64) {
    s.e.ledger().with_mut(|li| li.timestamp += seconds);
}

/// #2: a staker counts once whatever it stakes and positions split freely by seed, so an equal share
/// per staker is capturable on any farm; `Constant` is refused everywhere.
#[test]
fn constant_rewards_are_refused() {
    let s = setup();
    let delegated_farm = farm(&s, config(&s, delegated(&s), 0, 0));
    let standalone_farm = farm(&s, config(&s, standalone(0, 0), 1, 0));

    for farm_id in [&delegated_farm, &standalone_farm] {
        assert_eq!(
            s.farms.try_initialize_reward(farm_id, &reward_config(&s, RewardType::Constant, 1)),
            Err(Ok(FCError::InvalidConfig))
        );
        s.farms.initialize_reward(farm_id, &reward_config(&s, RewardType::Proportional, 1));
    }
}

/// Every stake push rewrites each reward with its schedule, so delegated farms cap both to keep a
/// bad-debt cover over many positions inside a transaction.
#[test]
fn delegated_farms_cap_rewards_and_schedule_points() {
    let s = setup();
    let farm_id = farm(&s, config(&s, delegated(&s), 0, 0));
    let curve = |points: u64| RewardScheduleCurve {
        points: (0..points)
            .map(|k| RewardCurvePoint { ts_start: 1_000 + k * DAY, reward_per_time_unit: 1 })
            .fold(svec![&s.e], |mut v, p| {
                v.push_back(p);
                v
            }),
    };
    let reward = |curve| RewardConfig {
        reward_token: Address::generate(&s.e),
        reward_type: RewardType::Proportional,
        reward_schedule_curve: curve,
    };

    assert_eq!(
        s.farms.try_initialize_reward(&farm_id, &reward(curve(5))),
        Err(Ok(FCError::InvalidRewardScheduleCurve))
    );
    for _ in 0..4 {
        s.farms.initialize_reward(&farm_id, &reward(curve(4)));
    }
    assert_eq!(
        s.farms.try_initialize_reward(&farm_id, &reward(curve(1))),
        Err(Ok(FCError::MaxFarmNumRewardsReached))
    );
    assert_eq!(
        s.farms.try_set_reward_schedule(&farm_id, &0, &curve(5)),
        Err(Ok(FCError::InvalidRewardScheduleCurve))
    );
}

/// #4: a 1-unit stake alone in the farm drives the index past i128; a later normal stake and every
/// refresh used to fail on `stake × index`. The amounts are exact to the unit under the old rounding.
#[test]
fn a_dust_stake_alone_does_not_block_later_stakes() {
    let s = setup();
    let farm_id = farm(&s, config(&s, delegated(&s), 0, 0));
    let index = funded_reward(&s, &farm_id, 10_i128.pow(22), 2 * 10_i128.pow(24));
    let (dust, whale) = (user(&s), user(&s));

    s.farms.set_stake_delegated(&dust, &farm_id, &1);
    pass(&s, 100);
    s.farms.set_stake_delegated(&whale, &farm_id, &10_i128.pow(20));
    pass(&s, 1);
    s.farms.set_stake_delegated(&whale, &farm_id, &(10_i128.pow(20) + 1));

    assert_eq!(s.farms.harvest(&dust, &farm_id, &index), 10_i128.pow(24) + 99);
    assert_eq!(s.farms.harvest(&whale, &farm_id, &index), 10_i128.pow(22) - 100);
}

/// #4: a partial unstake may not leave less than `min_stake_amount` behind.
#[test]
fn a_partial_unstake_cannot_leave_dust() {
    let s = setup();
    let farm_id = farm(&s, config(&s, standalone(0, 0), 100, 0));
    let alice = user(&s);
    s.farms.stake(&alice, &farm_id, &101);

    assert_eq!(s.farms.try_unstake(&alice, &farm_id, &100), Err(Ok(FCError::InvalidAmount)));
    s.farms.unstake(&alice, &farm_id, &101);
}

/// #4: a deposit still in warm-up can be taken back instead of being trapped until activation.
#[test]
fn a_warm_up_deposit_can_be_taken_back() {
    let s = setup();
    let farm_id = farm(&s, config(&s, standalone(DAY, 0), 0, 0));
    let alice = user(&s);
    let staked = TokenClient::new(&s.e, &s.staked);
    let before = staked.balance(&alice.owner);

    s.farms.stake(&alice, &farm_id, &1_000);
    assert_eq!(s.farms.withdraw_pending_deposit(&alice, &farm_id), 1_000);

    assert_eq!(staked.balance(&alice.owner), before);
    assert_eq!(s.farms.get_farm(&farm_id).farm.total_pending_deposits, 0);
    assert_eq!(
        s.farms.try_withdraw_pending_deposit(&alice, &farm_id),
        Err(Ok(FCError::NoPendingDeposit))
    );
}

/// #5: an extreme rate used to overflow `rate × elapsed` on every refresh, freezing stakes and the
/// schedule itself; it now pays out the funded budget and the schedule can be replaced.
#[test]
fn an_extreme_rate_pays_the_budget_instead_of_freezing() {
    let s = setup();
    let farm_id = farm(&s, config(&s, delegated(&s), 0, 0));
    let index = funded_reward(&s, &farm_id, i128::MAX, 1_000);
    let alice = user(&s);
    s.farms.set_stake_delegated(&alice, &farm_id, &10);
    pass(&s, 2);

    s.farms.set_stake_delegated(&alice, &farm_id, &20);
    assert_eq!(s.farms.harvest(&alice, &farm_id, &index), 1_000);
    s.farms.set_reward_schedule(
        &farm_id,
        &index,
        &RewardScheduleCurve {
            points: svec![
                &s.e,
                RewardCurvePoint { ts_start: s.e.ledger().timestamp(), reward_per_time_unit: 1 }
            ],
        },
    );
}

/// #7: while deposits wait out warm-up (nothing staked yet), the locking terms they were made under
/// cannot change — including the warm-up period itself.
#[test]
fn locking_terms_are_fixed_while_deposits_wait() {
    let s = setup();
    let initial = config(&s, standalone(DAY, 0), 0, 0);
    let farm_id = farm(&s, initial.clone());
    let alice = user(&s);
    s.farms.stake(&alice, &farm_id, &1_000);

    let mut locked = initial.clone();
    locked.delegation = Delegation::NonDelegated(NonDelegatedFarmConfig {
        locking_ts: 0,
        locking_duration: 365 * DAY,
        locking_mode: LockingMode::Continuous,
        deposit_warmup_period: DAY,
        withdrawal_cooldown_period: 0,
        early_withdrawal_penalty_bps: 10_000,
    });
    let shorter_warmup = config(&s, standalone(DAY / 2, 0), 0, 0);
    for update in [&locked, &shorter_warmup] {
        assert_eq!(
            s.farms.try_update_farm_config(&farm_id, update),
            Err(Ok(FCError::InvalidFarmConfigUpdate))
        );
    }

    s.farms.withdraw_pending_deposit(&alice, &farm_id);
    s.farms.update_farm_config(&farm_id, &locked);
}

/// Only pending deposits pin the warm-up period: it can still be retuned on a farm whose stake is active.
#[test]
fn the_warm_up_period_stays_tunable_without_pending_deposits() {
    let s = setup();
    let farm_id = farm(&s, config(&s, standalone(0, 0), 0, 0));
    let alice = user(&s);
    s.farms.stake(&alice, &farm_id, &1_000);

    s.farms.update_farm_config(&farm_id, &config(&s, standalone(DAY, 0), 0, 0));
}

/// #7: a withdrawal waiting out its cooldown also pins the terms.
#[test]
fn cooldown_withdrawals_pin_the_terms_until_paid_out() {
    let s = setup();
    let initial = config(&s, standalone(0, DAY), 0, 0);
    let farm_id = farm(&s, initial);
    let alice = user(&s);
    s.farms.stake(&alice, &farm_id, &1_000);
    s.farms.unstake(&alice, &farm_id, &1_000);

    let shorter_cooldown = config(&s, standalone(0, DAY / 2), 0, 0);
    assert_eq!(
        s.farms.try_update_farm_config(&farm_id, &shorter_cooldown),
        Err(Ok(FCError::InvalidFarmConfigUpdate))
    );

    pass(&s, DAY);
    s.farms.withdraw_unstaked(&alice, &farm_id);
    assert_eq!(s.farms.get_farm(&farm_id).farm.total_pending_withdrawals, 0);
    s.farms.update_farm_config(&farm_id, &shorter_cooldown);
}

/// #8: one refresh issuing ~1e21 units used to overflow `rewards × 1e18` before the division.
#[test]
fn a_large_refresh_issues_without_overflow() {
    let s = setup();
    let farm_id = farm(&s, config(&s, delegated(&s), 0, 0));
    let index = funded_reward(&s, &farm_id, 10_i128.pow(17), 10_i128.pow(21));
    let alice = user(&s);
    s.farms.set_stake_delegated(&alice, &farm_id, &10_000_000);
    pass(&s, 10_000);

    assert_eq!(s.farms.harvest(&alice, &farm_id, &index), 10_i128.pow(21));
}

/// The deposit cap counts deposits still in warm-up, so they cannot oversubscribe it and then fail to
/// activate.
#[test]
fn the_deposit_cap_counts_pending_deposits() {
    let s = setup();
    let farm_id = farm(&s, config(&s, standalone(DAY, 0), 0, 1_000));
    let (alice, bob) = (user(&s), user(&s));
    s.farms.stake(&alice, &farm_id, &600);

    assert_eq!(s.farms.try_stake(&bob, &farm_id, &600), Err(Ok(FCError::DepositCapExceeded)));
    pass(&s, DAY);
    s.farms.refresh_delegatee_state(&alice, &farm_id);
    assert_eq!(s.farms.get_farm(&farm_id).farm.total_staked, 600);
}

/// Lifetime issuance can pass i128 through refills; the cumulative counter used to overflow on every
/// refresh after that, freezing the farm again.
#[test]
fn lifetime_issuance_past_i128_keeps_refreshing() {
    let s = setup();
    let farm_id = farm(&s, config(&s, delegated(&s), 0, 0));
    let index = funded_reward(&s, &farm_id, i128::MAX, i128::MAX);
    let alice = user(&s);
    s.farms.set_stake_delegated(&alice, &farm_id, &1);
    pass(&s, 1);
    assert_eq!(s.farms.harvest(&alice, &farm_id, &index), i128::MAX);
    // Make room in alice's balance, which the token caps at i128::MAX, for the next payout.
    TokenClient::new(&s.e, &s.reward).transfer(&alice.owner, &s.admin, &i128::MAX);
    s.farms.add_rewards(&s.admin, &farm_id, &index, &1);
    pass(&s, 1);
    s.farms.set_stake_delegated(&alice, &farm_id, &2);
    assert_eq!(s.farms.harvest(&alice, &farm_id, &index), 1);
}

/// A dust stake can push the index far enough that `stake × index` leaves even I256 for a large
/// stake; the tally of that stake still fits.
#[test]
fn a_huge_stake_after_a_huge_index_still_fits() {
    let s = setup();
    let farm_id = farm(&s, config(&s, delegated(&s), 0, 0));
    let index = funded_reward(&s, &farm_id, 10_i128.pow(30), 10_i128.pow(30));
    let (dust, whale) = (user(&s), user(&s));
    s.farms.set_stake_delegated(&dust, &farm_id, &1);
    pass(&s, 1);

    s.farms.set_stake_delegated(&whale, &farm_id, &10_i128.pow(30));
    assert_eq!(s.farms.harvest(&dust, &farm_id, &index), 10_i128.pow(30));
}

/// The treasury fee on a large harvest used to overflow `pending × fee_bps` before dividing.
#[test]
fn a_treasury_fee_on_a_large_harvest_does_not_overflow() {
    let s = setup();
    let mut with_fee = config(&s, delegated(&s), 0, 0);
    with_fee.treasury_fee_bps = 2_000;
    let farm_id = farm(&s, with_fee);
    let index = funded_reward(&s, &farm_id, 10_i128.pow(36), 10_i128.pow(36));
    let alice = user(&s);
    s.farms.set_stake_delegated(&alice, &farm_id, &1);
    pass(&s, 1);

    assert_eq!(
        s.farms.get_pending_rewards(&alice, &farm_id),
        svec![&s.e, (s.reward.clone(), 8 * 10_i128.pow(35))]
    );
    assert_eq!(s.farms.harvest(&alice, &farm_id, &index), 8 * 10_i128.pow(35));
}

/// A position's reward debt used to be `stake × index`, whose size grows with the farm's lifetime: after
/// a dust stake has taken i128::MAX twice, a full-size stake overflowed I256 a few rounds later even
/// though it was owed only one round.
#[test]
fn a_full_size_stake_on_a_long_lived_index_still_settles() {
    let s = setup();
    let farm_id = farm(&s, config(&s, delegated(&s), 0, 0));
    let index = funded_reward(&s, &farm_id, i128::MAX, i128::MAX);
    let alice = user(&s);
    let reward = TokenClient::new(&s.e, &s.reward);
    let round = || {
        pass(&s, 1);
        assert_eq!(s.farms.harvest(&alice, &farm_id, &index), i128::MAX);
        reward.transfer(&alice.owner, &s.admin, &i128::MAX);
        s.farms.add_rewards(&s.admin, &farm_id, &index, &i128::MAX);
    };

    s.farms.set_stake_delegated(&alice, &farm_id, &1);
    round();
    round();
    s.farms.set_stake_delegated(&alice, &farm_id, &0);
    s.farms.set_stake_delegated(&alice, &farm_id, &i128::MAX);
    for _ in 0..5 {
        round();
    }
}

/// A lock that ends past `u64::MAX` used to overflow on every early exit once funds were in, and active
/// stake then pinned the very terms that would fix it.
#[test]
fn a_lock_ending_past_the_clock_is_refused() {
    let s = setup();
    let mut config = config(&s, standalone(0, 0), 0, 0);
    config.delegation = Delegation::NonDelegated(NonDelegatedFarmConfig {
        locking_ts: u64::MAX,
        locking_duration: 1,
        locking_mode: LockingMode::WithExpiry,
        deposit_warmup_period: 0,
        withdrawal_cooldown_period: 0,
        early_withdrawal_penalty_bps: 100,
    });

    assert_eq!(s.farms.try_initialize_farm(&None, &config), Err(Ok(FCError::InvalidConfig)));
}

/// The lifetime slash total only reports; recycling slashed funds past i128 used to block every
/// penalized exit.
#[test]
fn slash_totals_past_i128_do_not_block_exits() {
    let s = setup();
    let mut config = config(&s, standalone(0, 0), 0, 0);
    config.delegation = Delegation::NonDelegated(NonDelegatedFarmConfig {
        locking_ts: 0,
        locking_duration: 365 * DAY,
        locking_mode: LockingMode::Continuous,
        deposit_warmup_period: 0,
        withdrawal_cooldown_period: 0,
        early_withdrawal_penalty_bps: 10_000,
    });
    let farm_id = farm(&s, config);
    let alice = Delegatee::new(Address::generate(&s.e));
    StellarAssetClient::new(&s.e, &s.staked).mint(&alice.owner, &(i128::MAX - 1));

    let staked = TokenClient::new(&s.e, &s.staked);
    // Clear the admin's own balance, which the token caps at i128::MAX, so it can take the slash.
    staked.transfer(&s.admin, Address::generate(&s.e), &i128::MAX);

    s.farms.stake(&alice, &farm_id, &(i128::MAX - 1));
    s.farms.unstake(&alice, &farm_id, &(i128::MAX - 1));
    s.farms.withdraw_slashed(&farm_id, &(i128::MAX - 1));
    staked.transfer(&s.admin, &alice.owner, &2);
    s.farms.stake(&alice, &farm_id, &2);

    assert_eq!(s.farms.unstake(&alice, &farm_id, &2), 0);
}
