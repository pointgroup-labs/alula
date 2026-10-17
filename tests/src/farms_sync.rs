//! The farms controller's stakes follow the market's positions: supply farms hold j-tokens, debt farms
//! d-tokens, and debt under a bad-debt request holds nothing.
#![cfg(test)]

use farms::{
    DelegatedFarmConfig, Delegation, FarmConfig, FarmsContract, FarmsContractClient,
    OptionalOracle, RewardConfig, RewardCurvePoint, RewardScheduleCurve, RewardType,
};
use farms_interface::Delegatee;
use market::{
    constants::{DEFAULT_BAD_DEBT_LOCK_D, SECONDS_IN_YEAR},
    obligation::ObligationKey,
    request::{LiquidateRequest, Request, StandardRequest},
};
use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, BytesN, map as smap, testutils::Ledger, vec as svec};

use crate::{DEFAULT_DEPOSIT_AMOUNT, TestMarketFixture, get_obligation_d_tokens_as_tokens};

const ONE_TOKEN: i128 = 10_000_000;

fn pools(f: &TestMarketFixture) -> [Address; 3] {
    [f.usdc_pool_address.clone(), f.gold_pool_address.clone(), f.btc_pool_address.clone()]
}

#[derive(Clone, Copy, Debug)]
enum Side {
    Supply,
    Debt,
}

const SIDES: [Side; 2] = [Side::Supply, Side::Debt];

/// A farms contract delegated to the market, with a funded proportional reward on a supply and a debt
/// farm of every pool.
struct Farms<'a> {
    client: FarmsContractClient<'a>,
    ids: Vec<(Address, u8, BytesN<32>)>,
}

impl Farms<'_> {
    fn id(&self, pool: &Address, side: Side) -> BytesN<32> {
        let (_, _, id) = self.ids.iter().find(|(p, s, _)| p == pool && *s == side as u8).unwrap();
        id.clone()
    }
}

fn attach_farms<'a>(f: &TestMarketFixture<'a>) -> Farms<'a> {
    let client =
        FarmsContractClient::new(&f.e, &f.e.register(FarmsContract, (f.contract_admin.clone(),)));
    f.contract_client.set_farms_contract(&client.address);
    let reward = f.usdc_token_address.clone();
    f.usdc_sac.mint(&f.contract_admin, &(6 * 1_000 * ONE_TOKEN));
    let mut ids = Vec::new();
    for pool in pools(f) {
        for side in SIDES {
            let farm_id = client.initialize_farm(
                &None,
                &FarmConfig {
                    token: pool.clone(),
                    deposit_cap: 0,
                    treasury_fee_bps: 0,
                    min_harvest_delay: 0,
                    min_stake_amount: 0,
                    delegation: Delegation::Delegated(DelegatedFarmConfig {
                        delegate_authority: f.contract_id.clone(),
                    }),
                    is_reward_once_enabled: false,
                    is_harvest_permissionless: false,
                    oracle: OptionalOracle::None,
                },
            );
            client.unfreeze_farm(&farm_id);
            let index = client.initialize_reward(
                &farm_id,
                &RewardConfig {
                    reward_token: reward.clone(),
                    reward_type: RewardType::Proportional,
                    reward_schedule_curve: RewardScheduleCurve {
                        points: svec![
                            &f.e,
                            RewardCurvePoint {
                                ts_start: f.e.ledger().timestamp(),
                                reward_per_time_unit: 1_000,
                            }
                        ],
                    },
                },
            );
            client.add_rewards(&f.contract_admin, &farm_id, &index, &(1_000 * ONE_TOKEN));
            match side {
                Side::Supply => f.contract_client.set_pool_supply_farm(&pool, &farm_id),
                Side::Debt => f.contract_client.set_pool_debt_farm(&pool, &farm_id),
            }
            ids.push((pool.clone(), side as u8, farm_id));
        }
    }
    Farms { client, ids }
}

fn delegatee(key: &ObligationKey) -> Delegatee {
    Delegatee { owner: key.user.clone(), seed: key.seed.clone() }
}

fn farm_stake(farms: &Farms, key: &ObligationKey, pool: &Address, side: Side) -> i128 {
    match farms.client.try_get_delegatee_state(&delegatee(key), &farms.id(pool, side)) {
        Ok(Ok(state)) => state.active_stake,
        _ => 0,
    }
}

fn market_stake(f: &TestMarketFixture, key: &ObligationKey, pool: &Address, side: Side) -> i128 {
    let Ok(Ok(obligation)) = f.contract_client.try_get_user_obligation(key) else {
        return 0;
    };
    match side {
        Side::Supply => obligation.deposits.get(pool.clone()).map(|d| d.j_tokens).unwrap_or(0),
        Side::Debt => {
            if obligation.insurance_fund_requests_ids.keys().iter().any(|(p, _)| p == *pool) {
                0
            } else {
                obligation.borrows.get(pool.clone()).map(|b| b.d_tokens).unwrap_or(0)
            }
        }
    }
}

fn assert_synced(f: &TestMarketFixture, farms: &Farms, keys: &[ObligationKey], when: &str) {
    for key in keys {
        for pool in pools(f) {
            for side in SIDES {
                assert_eq!(
                    farm_stake(farms, key, &pool, side),
                    market_stake(f, key, &pool, side),
                    "{when}: {side:?} farm of {pool:?} disagrees with {key:?}"
                );
            }
        }
    }
}

struct Rng(u64);

impl Rng {
    fn below(&mut self, n: u64) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0 % n
    }

    fn pick<'a, T>(&mut self, items: &'a [T]) -> &'a T {
        &items[self.below(items.len() as u64) as usize]
    }
}

/// Successful operations per kind, so the random test fails if it stops exercising the market.
#[derive(Debug, Default)]
struct Tally {
    supply: u32,
    debt: u32,
    liquidations: u32,
    batches: u32,
    covers_issued: u32,
    covers_claimed: u32,
}

/// BTC, GOLD and USDC prices, in the oracle's feed order, as a percent of $1. The market refuses a
/// zero price, so 0 % becomes the smallest positive one.
fn set_prices(f: &TestMarketFixture, percent: [i128; 3]) {
    let price = |p: i128| (1_00000000000000_i128 * p / 100).max(1);
    f.oracle_client.set_price_stable(&svec![
        &f.e,
        price(percent[0]),
        price(percent[1]),
        price(percent[2])
    ]);
    f.pass_time(1);
}

fn random_batch(
    f: &TestMarketFixture,
    rng: &mut Rng,
    keys: &[ObligationKey],
) -> soroban_sdk::Vec<Request> {
    let pools = pools(f);
    let mut requests = svec![&f.e];
    for _ in 0..=rng.below(2) {
        let pool_address = rng.pick(&pools).clone();
        let amount = 1 + rng.below(ONE_TOKEN as u64 / 4) as i128;
        let standard = StandardRequest { amount, pool_address };
        requests.push_back(match rng.below(5) {
            0 => Request::Deposit(standard),
            1 => Request::Withdraw(standard),
            2 => Request::Borrow(standard),
            3 => Request::Repay(standard),
            _ => Request::Liquidate(LiquidateRequest {
                borrower_obligation_key: rng.pick(keys).clone(),
                borrow_pool_address: rng.pick(&pools).clone(),
                collateral_pool_address: rng.pick(&pools).clone(),
                repay_amount: amount,
                min_demanded_collateral_amount: 0,
            }),
        });
    }
    requests
}

#[test]
fn random_market_sequences_keep_farm_stakes_in_step() {
    let mut tally = Tally::default();
    for seed in 1..=24_u64 {
        let mut rng = Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1);
        let f = TestMarketFixture::new();
        f.e.cost_estimate().budget().reset_unlimited();
        let queue_period = f.contract_client.get_global_state().update_in_queue_period;
        f.contract_client.queue_in_market_update(&10, &1, &DEFAULT_BAD_DEBT_LOCK_D);
        f.pass_time(queue_period);
        f.contract_client.apply_market_update();
        let farms = attach_farms(&f);

        let seeded =
            ObligationKey::new_with_seed(f.users[0].clone(), BytesN::from_array(&f.e, &[7; 32]));
        let keys = [
            ObligationKey::new(f.users[0].clone()),
            ObligationKey::new(f.users[1].clone()),
            ObligationKey::new(f.users[2].clone()),
            seeded,
        ];
        let lender = ObligationKey::new(f.users[3].clone());
        for pool in pools(&f) {
            f.contract_client.deposit(&lender, &pool, &(100 * ONE_TOKEN), &None);
        }
        // Positions at their borrowing limit, so price moves make them liquidatable or bad debt.
        let (usdc, gold, btc) =
            (f.usdc_pool_address.clone(), f.gold_pool_address.clone(), f.btc_pool_address.clone());
        for (key, collateral, borrow) in [
            (&keys[0], gold.clone(), usdc.clone()),
            (&keys[1], btc.clone(), usdc.clone()),
            (&keys[2], gold.clone(), btc.clone()),
            (&keys[3], usdc.clone(), gold.clone()),
        ] {
            f.contract_client.deposit(key, &collateral, &ONE_TOKEN, &None);
            f.contract_client.borrow(key, &borrow, &i128::MAX, &None);
        }
        let mut open_requests = 0_u64;

        for step in 0..40 {
            let key = rng.pick(&keys).clone();
            let pool = rng.pick(&pools(&f)).clone();
            let amount = 1 + rng.below(ONE_TOKEN as u64) as i128;
            let op = rng.below(12);
            match op {
                0 => {
                    tally.supply +=
                        f.contract_client.try_deposit(&key, &pool, &amount, &None).is_ok() as u32
                }
                1 => {
                    tally.supply +=
                        f.contract_client.try_add_collateral(&key, &pool, &amount, &None).is_ok()
                            as u32
                }
                2 => {
                    let amount = if rng.below(3) == 0 { i128::MAX } else { amount / 2 };
                    tally.supply +=
                        f.contract_client.try_withdraw(&key, &pool, &amount, &None).is_ok() as u32;
                }
                3 | 4 => {
                    let amount = if rng.below(3) == 0 { i128::MAX } else { amount / 4 };
                    tally.debt +=
                        f.contract_client.try_borrow(&key, &pool, &amount, &None).is_ok() as u32;
                }
                5 => {
                    tally.debt +=
                        f.contract_client.try_repay(&key, &pool, &amount, &None).is_ok() as u32
                }
                6 => f.pass_time(rng.below(60 * 24 * 60 * 60)),
                7 => set_prices(&f, [50 + rng.below(51) as i128, 50 + rng.below(51) as i128, 100]),
                8 => {
                    let Ok(Ok(obligation)) = f.contract_client.try_get_user_obligation(&key) else {
                        continue;
                    };
                    let (borrows, deposits) =
                        (obligation.borrows.keys(), obligation.deposits.keys());
                    if borrows.is_empty() || deposits.is_empty() {
                        continue;
                    }
                    let borrow_pool = borrows.get_unchecked(rng.below(borrows.len() as u64) as u32);
                    let collateral_pool =
                        deposits.get_unchecked(rng.below(deposits.len() as u64) as u32);
                    let liquidator = &f.users[1 + rng.below(3) as usize];
                    if *liquidator == key.user {
                        continue;
                    }
                    let repay =
                        obligation.borrows.get_unchecked(borrow_pool.clone()).d_tokens / 3 + 1;
                    tally.liquidations += f
                        .contract_client
                        .try_liquidate(liquidator, &key, &borrow_pool, &collateral_pool, &repay, &0)
                        .is_ok() as u32;
                }
                9 => {
                    let requests = random_batch(&f, &mut rng, &keys);
                    tally.batches +=
                        f.contract_client.try_submit_requests_batch(&key, &requests, &None).is_ok()
                            as u32;
                }
                10 => {
                    // Wipe one collateral asset so cover can be issued on whoever is left with only debt.
                    set_prices(&f, if rng.below(2) == 0 { [100, 0, 100] } else { [0, 100, 100] });
                    if f.contract_client.try_issue_cover_bad_debt(&key).is_ok() {
                        tally.covers_issued += 1;
                        open_requests += f
                            .contract_client
                            .get_user_obligation(&key)
                            .insurance_fund_requests_ids
                            .len() as u64;
                    }
                    set_prices(&f, [100, 100, 100]);
                }
                _ => {
                    let pending = matches!(
                        f.contract_client.try_get_user_obligation(&key),
                        Ok(Ok(ref o)) if !o.insurance_fund_requests_ids.is_empty()
                    );
                    for request_id in 0..open_requests {
                        let _ = f.controlled_insurance_fund_client.try_mark_ready(&request_id, &0);
                    }
                    let claimed = f.contract_client.try_claim_cover_bad_debt_results(&key).is_ok();
                    tally.covers_claimed += (pending && claimed) as u32;
                }
            }
            assert_synced(
                &f,
                &farms,
                &keys,
                &format!("seed {seed} step {step} op {op} on {key:?}"),
            );
        }
    }

    println!("{tally:?}");
    // Measured 118 / 52 / 9 / 16 / 14 / 3; the floors catch a driver that stops reaching the market.
    assert!(
        tally.supply >= 80
            && tally.debt >= 30
            && tally.liquidations >= 5
            && tally.batches >= 10
            && tally.covers_issued >= 5
            && tally.covers_claimed >= 1,
        "the random sequences stopped exercising the market: {tally:?}"
    );
}

#[test]
fn issuing_bad_debt_cover_zeroes_both_sides() {
    let f = TestMarketFixture::new();
    let farms = attach_farms(&f);
    let (borrower, lender, liquidator) =
        (f.users[0].clone(), f.users[1].clone(), f.users[2].clone());
    let borrower_key = ObligationKey::new(borrower.clone());
    let keys = [
        borrower_key.clone(),
        ObligationKey::new(lender.clone()),
        ObligationKey::new(liquidator.clone()),
    ];

    // A one-cent floor over $1 positions lets the borrower keep a dust deposit that the cover sweeps.
    let queue_period = f.contract_client.get_global_state().update_in_queue_period;
    f.contract_client.queue_in_market_update(&10, &1, &DEFAULT_BAD_DEBT_LOCK_D);
    f.e.ledger().with_mut(|li| li.timestamp += queue_period);
    f.contract_client.apply_market_update();

    f.contract_client.set_take_rate_fees_beneficiaries(
        &f.usdc_pool_address,
        &smap![&f.e, (f.insurance_fund.clone(), 10_000)],
    );
    f.contract_client.add_collateral(&borrower_key, &f.gold_pool_address, &ONE_TOKEN, &None);
    f.contract_client.deposit(&borrower_key, &f.btc_pool_address, &10, &None);
    f.contract_client.deposit(&keys[1], &f.usdc_pool_address, &ONE_TOKEN, &None);
    f.contract_client.borrow(&borrower_key, &f.usdc_pool_address, &i128::MAX, &None);
    assert_synced(&f, &farms, &keys, "after borrowing");

    f.e.ledger().with_mut(|li| li.timestamp += 5 * SECONDS_IN_YEAR);
    f.contract_client.refresh_pool(&f.usdc_pool_address);
    let debt = get_obligation_d_tokens_as_tokens(
        &f.e,
        &f.contract_client,
        &borrower,
        &f.usdc_pool_address,
    )
    .unwrap();
    f.contract_client.liquidate(
        &liquidator,
        &borrower_key,
        &f.usdc_pool_address,
        &f.gold_pool_address,
        &debt.fixed_mul_ceil(90, 100).unwrap(),
        &ONE_TOKEN,
    );
    f.contract_client.distribute_all_pools_fees();
    assert_synced(&f, &farms, &keys, "after the liquidation");
    assert!(farm_stake(&farms, &borrower_key, &f.btc_pool_address, Side::Supply) > 0);

    f.contract_client.issue_cover_bad_debt(&borrower_key);
    let obligation = f.contract_client.get_user_obligation(&borrower_key);
    assert!(
        obligation.borrows.get(f.usdc_pool_address.clone()).is_some(),
        "the request was settled at once, so the deferred path went untested"
    );
    assert_eq!(farm_stake(&farms, &borrower_key, &f.usdc_pool_address, Side::Debt), 0);
    assert_eq!(farm_stake(&farms, &borrower_key, &f.btc_pool_address, Side::Supply), 0);
    assert_synced(&f, &farms, &keys, "after issuing cover");

    let covered = f.usdc_token_client.balance(&f.insurance_fund);
    f.controlled_insurance_fund_client.mark_ready(&0, &covered);
    f.contract_client.claim_cover_bad_debt_results(&borrower_key);
    assert_synced(&f, &farms, &keys, "after claiming the cover");

    f.contract_client.refresh_farm_stakes(&borrower_key, &svec![&f.e, f.usdc_pool_address.clone()]);
    assert_synced(&f, &farms, &keys, "after a resync of the settled obligation");
}

#[test]
fn a_batched_liquidation_credits_the_seeded_obligation_farm() {
    let f = TestMarketFixture::new();
    let farms = attach_farms(&f);
    let (borrower, lender, liquidator) =
        (f.users[0].clone(), f.users[1].clone(), f.users[2].clone());
    let seeded = ObligationKey::new_with_seed(liquidator, BytesN::from_array(&f.e, &[7u8; 32]));
    let keys = [ObligationKey::new(borrower.clone()), ObligationKey::new(lender), seeded.clone()];

    f.contract_client.deposit(&keys[1], &f.usdc_pool_address, &(2 * DEFAULT_DEPOSIT_AMOUNT), &None);
    f.contract_client.deposit(&keys[0], &f.gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    f.contract_client.borrow(
        &keys[0],
        &f.usdc_pool_address,
        &((DEFAULT_DEPOSIT_AMOUNT * 65) / 100),
        &None,
    );
    f.pass_time(3 * 365 * 24 * 60 * 60);
    f.contract_client.refresh_pool(&f.usdc_pool_address);
    f.contract_client.refresh_pool(&f.gold_pool_address);
    f.e.ledger().with_mut(|li| li.timestamp += 1);

    let batch = svec![
        &f.e,
        Request::Liquidate(LiquidateRequest {
            borrower_obligation_key: keys[0].clone(),
            borrow_pool_address: f.usdc_pool_address.clone(),
            collateral_pool_address: f.gold_pool_address.clone(),
            repay_amount: 10_000,
            min_demanded_collateral_amount: 0,
        }),
    ];
    f.contract_client.submit_requests_batch(&seeded, &batch, &None);

    assert!(farm_stake(&farms, &seeded, &f.gold_pool_address, Side::Supply) > 0);
    assert_synced(&f, &farms, &keys, "after the batched liquidation");
}

#[test]
fn resync_enrolls_old_positions_and_zeroes_removed_ones() {
    let f = TestMarketFixture::new();
    let key = ObligationKey::new(f.users[0].clone());
    let lender = ObligationKey::new(f.users[1].clone());
    let all = pools(&f);

    f.contract_client.deposit(&lender, &f.usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    f.contract_client.deposit(&key, &f.gold_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    f.contract_client.borrow(&key, &f.usdc_pool_address, &(DEFAULT_DEPOSIT_AMOUNT / 10), &None);

    let farms = attach_farms(&f);
    assert_eq!(farm_stake(&farms, &key, &f.gold_pool_address, Side::Supply), 0);
    f.contract_client
        .refresh_farm_stakes(&key, &svec![&f.e, all[0].clone(), all[1].clone(), all[2].clone()]);
    f.contract_client.refresh_farm_stakes(&lender, &svec![&f.e, all[0].clone()]);
    assert_synced(&f, &farms, &[key.clone(), lender.clone()], "after enrolling");

    // A position closed while its pool was detached keeps a stale stake until resynced.
    let gold_supply = farms.id(&f.gold_pool_address, Side::Supply);
    f.contract_client.repay(&key, &f.usdc_pool_address, &DEFAULT_DEPOSIT_AMOUNT, &None);
    f.contract_client.clear_pool_farms(&f.gold_pool_address);
    f.contract_client.withdraw(&key, &f.gold_pool_address, &i128::MAX, &None);
    f.contract_client.set_pool_supply_farm(&f.gold_pool_address, &gold_supply);
    assert!(farm_stake(&farms, &key, &f.gold_pool_address, Side::Supply) > 0);

    f.contract_client.refresh_farm_stakes(&key, &svec![&f.e, f.gold_pool_address.clone()]);
    assert_synced(&f, &farms, &[key, lender], "after resyncing a removed obligation");
}
