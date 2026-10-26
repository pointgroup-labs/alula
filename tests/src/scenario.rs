//! State-relative fuzz steps: each step draws its user, pools and amount from the market it runs
//! against, so a liquidation meets a borrower that exists and a position that prices can push
//! under water. The legacy `Command` stays reachable as a rare raw step.

use std::{collections::BTreeMap, fmt};

use arbitrary::{Arbitrary, Unstructured};
use market::{
    constants::BPS_FACTOR,
    error::MCError,
    obligation::{Obligation, ObligationKey},
    oracle::{Prices, get_asset_value_floor},
};
use sep_40_oracle::testutils::Asset;
use soroban_sdk::Address;

use crate::{
    Command, FlashBatch, Liquidate, PassTime, RunCommand, TestMarketFixture, Token,
    make_oracle_prices_different,
};

pub const MAX_STEPS: usize = 32;
// Compounding at the 400% APR cap overflows `total_borrowed` after ~15 idle years (52 GOLD of debt
// reached 1.1e34 units), which `refresh_pool` reports as OverOrUnderflow; time stops short of that.
pub const HORIZON_SECONDS: u64 = 5 * 365 * 24 * 60 * 60;

/// Symmetric in log space, so no token drifts on average.
const PRICE_MOVES_PCT: [i128; 16] =
    [5, 25, 50, 70, 80, 88, 94, 97, 103, 106, 114, 125, 143, 200, 400, 2_000];
/// Unweighted LTVs a shock prices a borrower to: around the 80% close LTV, the 98.5% insolvency
/// LTV, and deep under water.
const SHOCK_LTVS_BPS: [i128; 12] =
    [7_000, 7_900, 8_100, 8_500, 9_000, 9_500, 9_800, 9_900, 10_500, 12_000, 20_000, 100_000];
/// Position values a shock prices to, around the $5 floor.
const SHOCK_CENTS: [i128; 12] = [1, 50, 100, 250, 400, 499, 500, 501, 600, 900, 1_500, 5_000];
// $10^-11 ..= $10^6 at 14 decimals: a shock can reach a far target and no value overflows.
const PRICE_BOUNDS: (i128, i128) = (1_000, 100_000_000_000_000_000_000);

#[derive(Debug)]
pub struct Scenario {
    pub steps: Vec<Step>,
}

#[derive(Debug)]
pub enum Step {
    Supply {
        user: u8,
        token: Token,
        amount: i128,
    },
    AddCollateral {
        user: u8,
        token: Token,
        amount: i128,
    },
    Borrow {
        pick: u8,
        pool: u8,
        portion: Portion,
    },
    Repay {
        pick: u8,
        pool: u8,
        portion: Portion,
    },
    Withdraw {
        pick: u8,
        pool: u8,
        portion: Portion,
    },
    RemoveCollateral {
        pick: u8,
        pool: u8,
        portion: Portion,
    },
    Liquidate(TargetedLiquidation),
    MovePrice {
        token: Token,
        pct: i128,
    },
    /// Reprices one of a borrower's positions so the borrower lands on `target`.
    Shock {
        pick: u8,
        position: u8,
        target: ShockTarget,
    },
    Wait {
        seconds: u64,
    },
    FlashBatch {
        user: u8,
        batch: FlashBatch,
    },
    CoverBadDebt {
        pick: u8,
    },
    ClaimCover {
        pick: u8,
    },
    Raw(Command),
}

#[derive(Debug, Clone, Copy)]
pub enum ShockTarget {
    LtvBps(i128),
    PositionCents(i128),
}

/// A share of the bound a relative step resolves, in basis points. `Max` sends the contract's
/// `i128::MAX` sentinel where the entry point has one.
#[derive(Debug, Clone, Copy)]
pub enum Portion {
    Bps(i128),
    Max,
}

#[derive(Debug)]
pub struct TargetedLiquidation {
    /// Picks among the borrowers the harness sees as liquidatable, when there are any.
    pub prefer_unhealthy: bool,
    pub borrower: u8,
    pub borrow_pool: u8,
    pub collateral_pool: u8,
    pub liquidator: u8,
    /// Of the position's debt; above 10_000 overpays.
    pub repay_bps: i128,
    /// Of the position's collateral, demanded as the minimum seizure.
    pub demand_bps: i128,
    /// Rare: any users, any pools and raw amounts, so malformed calls stay possible.
    pub wild: Option<Liquidate>,
}

// As many steps as the input holds: a short input is a short scenario, not one padded with
// zero-byte defaults.
impl<'a> Arbitrary<'a> for Scenario {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        let mut steps = Vec::new();
        while !u.is_empty() && steps.len() < MAX_STEPS {
            steps.push(u.arbitrary()?);
        }
        Ok(Self { steps })
    }
}

impl<'a> Arbitrary<'a> for Step {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        Ok(match u.int_in_range(0..=99u8)? {
            // Lenders run larger than collateral, so a borrow is bound by health, not by liquidity.
            0..=11 => Step::Supply {
                user: u.arbitrary()?,
                token: u.arbitrary()?,
                amount: log_amount(u, 4..=15)?,
            },
            12..=23 => Step::AddCollateral {
                user: u.arbitrary()?,
                token: u.arbitrary()?,
                amount: log_amount(u, 0..=13)?,
            },
            24..=39 => {
                Step::Borrow { pick: u.arbitrary()?, pool: u.arbitrary()?, portion: u.arbitrary()? }
            }
            40..=43 => {
                Step::Repay { pick: u.arbitrary()?, pool: u.arbitrary()?, portion: u.arbitrary()? }
            }
            44..=47 => Step::Withdraw {
                pick: u.arbitrary()?,
                pool: u.arbitrary()?,
                portion: u.arbitrary()?,
            },
            48..=51 => Step::RemoveCollateral {
                pick: u.arbitrary()?,
                pool: u.arbitrary()?,
                portion: u.arbitrary()?,
            },
            52..=69 => Step::Liquidate(u.arbitrary()?),
            70..=75 => Step::MovePrice { token: u.arbitrary()?, pct: *u.choose(&PRICE_MOVES_PCT)? },
            76..=85 => Step::Shock {
                pick: u.arbitrary()?,
                position: u.arbitrary()?,
                target: if u.int_in_range(0..=3u8)? == 1 {
                    ShockTarget::PositionCents(*u.choose(&SHOCK_CENTS)?)
                } else {
                    ShockTarget::LtvBps(*u.choose(&SHOCK_LTVS_BPS)?)
                },
            },
            86..=90 => Step::Wait {
                seconds: u.int_in_range(1..=9u64)? * 10u64.pow(u.int_in_range(0..=7u32)?),
            },
            91..=93 => Step::FlashBatch { user: u.arbitrary()?, batch: u.arbitrary()? },
            94..=96 => Step::CoverBadDebt { pick: u.arbitrary()? },
            97 => Step::ClaimCover { pick: u.arbitrary()? },
            _ => Step::Raw(u.arbitrary()?),
        })
    }
}

impl<'a> Arbitrary<'a> for Portion {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Half near the bound: a position borrowed to its limit is the one a price move tips over.
        Ok(match u.int_in_range(0..=15u8)? {
            7 => Portion::Max,
            8..=15 => Portion::Bps(u.int_in_range(9_000..=BPS_FACTOR)?),
            _ => Portion::Bps(u.int_in_range(1..=BPS_FACTOR)?),
        })
    }
}

impl<'a> Arbitrary<'a> for TargetedLiquidation {
    fn arbitrary(u: &mut Unstructured<'a>) -> arbitrary::Result<Self> {
        // Rare branches sit off 0x00 and 0xFF, the bytes libFuzzer fills runs with.
        let wild = if u.int_in_range(0..=15u8)? == 7 { Some(u.arbitrary()?) } else { None };

        Ok(Self {
            prefer_unhealthy: u.int_in_range(0..=3u8)? != 0,
            borrower: u.arbitrary()?,
            borrow_pool: u.arbitrary()?,
            collateral_pool: u.arbitrary()?,
            liquidator: u.arbitrary()?,
            // Mostly within the 50% close factor, so a solvent position can be liquidated at all.
            repay_bps: match u.int_in_range(0..=7u8)? {
                2 => u.int_in_range(5_001..=10_000)?,
                5 => u.int_in_range(10_001..=30_000)?,
                _ => u.int_in_range(1..=5_000)?,
            },
            // A seizure is at most about half the position, so a larger demand only gets refused.
            demand_bps: if u.int_in_range(0..=7u8)? == 3 { u.int_in_range(1..=5_000)? } else { 0 },
            wild,
        })
    }
}

/// Token units spread evenly over orders of magnitude.
fn log_amount(
    u: &mut Unstructured,
    exponents: std::ops::RangeInclusive<u32>,
) -> arbitrary::Result<i128> {
    Ok(u.int_in_range(1..=99i128)? * 10i128.pow(u.int_in_range(exponents)?))
}

impl Portion {
    fn of(self, bound: i128, max: i128) -> i128 {
        match self {
            Portion::Max => max,
            Portion::Bps(bps) => scale_bps(bound, bps).max(1),
        }
    }
}

fn scale_bps(amount: i128, bps: i128) -> i128 {
    amount / BPS_FACTOR * bps + amount % BPS_FACTOR * bps / BPS_FACTOR
}

impl Scenario {
    /// Runs the steps on a fresh market, checking the invariants after each one.
    pub fn run(&self, min_collateral_value_cents: i128, tally: &mut ReachTally) {
        let f = fuzz_fixture(min_collateral_value_cents);
        let deadline = f.e.ledger().timestamp() + HORIZON_SECONDS;
        tally.inputs += 1;

        for step in &self.steps {
            if let Some(outcome) = step.run(&f, deadline) {
                tally.record(&outcome);
            }
            f.assert_invariants();
        }
    }
}

/// The market every fuzz target starts from.
pub fn fuzz_fixture<'a>(min_collateral_value_cents: i128) -> TestMarketFixture<'a> {
    let f = TestMarketFixture::new();
    f.e.cost_estimate().budget().reset_unlimited();
    if min_collateral_value_cents != 0 {
        f.set_min_collateral_value_cents(min_collateral_value_cents);
    }
    make_oracle_prices_different(&f.e, &f.oracle_client);

    f
}

impl Step {
    /// Runs the step, never moving time past `deadline`; a liquidation reports how far it got.
    pub fn run(&self, f: &TestMarketFixture, deadline: u64) -> Option<LiquidationOutcome> {
        let users = f.users.len();
        let wait = |seconds: u64| {
            f.pass_time(seconds.min(deadline.saturating_sub(f.e.ledger().timestamp())))
        };

        match self {
            Step::Liquidate(liquidation) => return Some(liquidation.run(f)),
            Step::Raw(Command::AllPassTime(PassTime { amount })) => wait(*amount),
            Step::Raw(command) => match command.liquidation() {
                Some((liquidation, who)) => return Some(liquidation.attempt(f, who)),
                None => command.run(f),
            },
            Step::Supply { user, token, amount } => check(f.contract_client.try_deposit(
                &key(f, *user as usize % users),
                &f.get_pool_address(*token),
                amount,
                &None,
            )),
            Step::AddCollateral { user, token, amount } => {
                check(f.contract_client.try_add_collateral(
                    &key(f, *user as usize % users),
                    &f.get_pool_address(*token),
                    amount,
                    &None,
                ))
            }
            Step::Borrow { pick, pool, portion } => {
                borrow(f, *pick, *pool, *portion);
            }
            Step::Repay { pick, pool, portion } => {
                repay(f, *pick, *pool, *portion);
            }
            Step::Withdraw { pick, pool, portion } => {
                withdraw(f, *pick, *pool, *portion);
            }
            Step::RemoveCollateral { pick, pool, portion } => {
                remove_collateral(f, *pick, *pool, *portion);
            }
            Step::MovePrice { token, pct } => {
                let asset = f.get_pool_address(*token);
                set_price(f, &asset, price_of(f, &asset).saturating_mul(*pct) / 100);
            }
            Step::Shock { pick, position, target } => {
                shock(f, *pick, *position, *target);
            }
            Step::Wait { seconds } => wait(*seconds),
            Step::FlashBatch { user, batch } => batch.run(f, *user as usize % users),
            Step::CoverBadDebt { pick } => {
                if let Some((user, _)) = choose(&holders(f, |o| !o.borrows.is_empty()), *pick) {
                    check(f.contract_client.try_issue_cover_bad_debt(&key(f, user)));
                }
            }
            Step::ClaimCover { pick } => {
                let claimants = holders(f, |o| !o.insurance_fund_requests_ids.is_empty());
                if let Some((user, _)) = choose(&claimants, *pick) {
                    check(f.contract_client.try_claim_cover_bad_debt_results(&key(f, user)));
                }
            }
        }

        None
    }
}

fn borrow(f: &TestMarketFixture, pick: u8, pool: u8, portion: Portion) -> Option<()> {
    let liquid: Vec<Address> = [&f.usdc_pool_address, &f.btc_pool_address, &f.gold_pool_address]
        .into_iter()
        .filter(|address| f.contract_client.get_pool(address).total_available > 0)
        .cloned()
        .collect();
    // A borrow beside a deposit in one pool is refused outright.
    let borrowers: Vec<(usize, Obligation, Vec<Address>)> = holders(f, |o| !o.deposits.is_empty())
        .into_iter()
        .filter_map(|(user, obligation)| {
            let pools: Vec<Address> = liquid
                .iter()
                .filter(|address| !obligation.deposits.contains_key((*address).clone()))
                .cloned()
                .collect();
            (!pools.is_empty()).then_some((user, obligation, pools))
        })
        .collect();
    let (user, obligation, pools) = choose(&borrowers, pick)?;
    let pool_address = choose(&pools, pool)?;
    let pool = f.contract_client.get_pool(&pool_address);

    // The bound the contract itself applies to an `i128::MAX` borrow.
    let bound =
        f.e.as_contract(&f.contract_id, || -> Result<i128, MCError> {
            Ok(obligation
                .compute_max_healthy_debt_added_amount(&f.e, &mut Prices::new(&f.e), &pool)?
                .min(pool.compute_available_utilization_ratio_capped_borrow(&f.e)?)
                .min(pool.reducible_available(&f.e)?))
        })
        .ok()
        .filter(|bound| *bound > 0)?;

    check(f.contract_client.try_borrow(
        &key(f, user),
        &pool_address,
        &portion.of(bound, i128::MAX),
        &None,
    ));
    Some(())
}

fn repay(f: &TestMarketFixture, pick: u8, pool: u8, portion: Portion) -> Option<()> {
    let (user, obligation) = choose(&holders(f, |o| !o.borrows.is_empty()), pick)?;
    let pool_address = choose(&obligation.borrows.keys().iter().collect::<Vec<_>>(), pool)?;
    let position = obligation.borrows.get(pool_address.clone())?;
    let debt = f
        .contract_client
        .get_pool(&pool_address)
        .compute_tokens_from_d_tokens_ceil(&f.e, position.d_tokens)
        .ok()?;

    // The borrower pays the whole amount in and is refunded the excess, so `Max` overpays twice.
    let amount = portion.of(debt, debt.saturating_mul(2).max(1));
    check(f.contract_client.try_repay(&key(f, user), &pool_address, &amount, &None));
    Some(())
}

fn withdraw(f: &TestMarketFixture, pick: u8, pool: u8, portion: Portion) -> Option<()> {
    let suppliers = holders(f, |o| o.deposits.values().iter().any(|d| d.j_tokens > 0));
    let (user, obligation) = choose(&suppliers, pick)?;
    let pools: Vec<Address> =
        obligation.deposits.iter().filter(|(_, d)| d.j_tokens > 0).map(|(a, _)| a).collect();
    let pool_address = choose(&pools, pool)?;
    let position = obligation.deposits.get(pool_address.clone())?;
    let pool = f.contract_client.get_pool(&pool_address);
    let deposit = pool.compute_tokens_from_j_tokens_floor(&f.e, position.j_tokens).ok()?;

    // Under a borrow, an explicit amount above the healthy maximum is refused rather than capped.
    let bound = if obligation.borrows.is_empty() {
        deposit
    } else {
        f.e.as_contract(&f.contract_id, || {
            obligation.compute_max_healthy_collateral_removed_amount(
                &f.e,
                &mut Prices::new(&f.e),
                &pool,
            )
        })
        .ok()?
        .min(deposit)
    };

    check(f.contract_client.try_withdraw(
        &key(f, user),
        &pool_address,
        &portion.of(bound, i128::MAX),
        &None,
    ));
    Some(())
}

fn remove_collateral(f: &TestMarketFixture, pick: u8, pool: u8, portion: Portion) -> Option<()> {
    let holders = holders(f, |o| o.deposits.values().iter().any(|d| d.collateral > 0));
    let (user, obligation) = choose(&holders, pick)?;
    let pools: Vec<Address> =
        obligation.deposits.iter().filter(|(_, d)| d.collateral > 0).map(|(a, _)| a).collect();
    let pool_address = choose(&pools, pool)?;
    let collateral = obligation.deposits.get(pool_address.clone())?.collateral;

    check(f.contract_client.try_remove_collateral(
        &key(f, user),
        &pool_address,
        &portion.of(collateral, i128::MAX),
        &None,
    ));
    Some(())
}

fn shock(f: &TestMarketFixture, pick: u8, position: u8, target: ShockTarget) -> Option<()> {
    let (_, obligation) = choose(&holders(f, |o| !o.borrows.is_empty()), pick)?;
    let positions: Vec<(Address, bool)> = obligation
        .deposits
        .keys()
        .iter()
        .map(|pool| (pool, false))
        .chain(obligation.borrows.keys().iter().map(|pool| (pool, true)))
        .collect();
    let (debt, collateral) =
        f.e.as_contract(&f.contract_id, || -> Result<(i128, i128), MCError> {
            let prices = &mut Prices::new(&f.e);
            Ok((
                obligation.compute_debt_value(&f.e, prices)?,
                obligation.compute_collateral_value(&f.e, prices)?,
            ))
        })
        .ok()?;
    let (debt, collateral) = (debt as f64, collateral as f64);

    // An LTV the picked position cannot reach within the price bounds (other collateral alone covers
    // the debt, say) passes to the next position. Floats suffice: the step only needs to land near.
    let start = position as usize % positions.len();
    for (pool_address, is_debt) in positions.iter().cycle().skip(start).take(positions.len()) {
        let value = position_value(f, &obligation, pool_address, *is_debt)? as f64;
        let wanted = match target {
            ShockTarget::PositionCents(cents) => cents as f64 * 1e12,
            ShockTarget::LtvBps(ltv) if *is_debt => ltv as f64 / 1e4 * collateral - debt + value,
            ShockTarget::LtvBps(ltv) => debt * 1e4 / ltv as f64 - collateral + value,
        };
        let price = price_of(f, pool_address) as f64 * wanted / value;
        if value > 0.0
            && (matches!(target, ShockTarget::PositionCents(_))
                || (PRICE_BOUNDS.0 as f64..=PRICE_BOUNDS.1 as f64).contains(&price))
        {
            set_price(f, pool_address, price as i128);
            return Some(());
        }
    }
    None
}

fn position_value(
    f: &TestMarketFixture,
    obligation: &Obligation,
    pool_address: &Address,
    is_debt: bool,
) -> Option<i128> {
    let pool = f.contract_client.get_pool(pool_address);
    let amount = if is_debt {
        let position = obligation.borrows.get(pool_address.clone())?;
        pool.compute_tokens_from_d_tokens_ceil(&f.e, position.d_tokens).ok()?
    } else {
        let position = obligation.deposits.get(pool_address.clone())?;
        pool.compute_tokens_from_j_tokens_floor(&f.e, position.j_tokens).ok()? + position.collateral
    };

    f.e.as_contract(&f.contract_id, || {
        get_asset_value_floor(
            &f.e,
            &mut Prices::new(&f.e),
            amount,
            &pool.token_address,
            pool.token_decimals,
        )
    })
    .ok()
}

fn price_of(f: &TestMarketFixture, asset: &Address) -> i128 {
    f.oracle_client
        .lastprice(&Asset::Stellar(asset.clone()))
        .expect("fixture prices every pool")
        .price
}

fn set_price(f: &TestMarketFixture, moved: &Address, price: i128) {
    // The oracle feed's positional order, as registered by the fixture.
    let prices = [&f.gold_pool_address, &f.btc_pool_address, &f.usdc_pool_address].map(|asset| {
        if asset == moved {
            price.clamp(PRICE_BOUNDS.0, PRICE_BOUNDS.1)
        } else {
            price_of(f, asset)
        }
    });

    f.oracle_client.set_price_stable(&soroban_sdk::vec![&f.e, prices[0], prices[1], prices[2]]);
}

impl TargetedLiquidation {
    fn run(&self, f: &TestMarketFixture) -> LiquidationOutcome {
        let users = &f.users;

        if let Some(raw) = &self.wild {
            return liquidate(
                f,
                &users[self.liquidator as usize % users.len()],
                &users[self.borrower as usize % users.len()],
                &f.get_pool_address(raw.token),
                &f.get_pool_address(raw.collateral_token),
                raw.repay_amount.0,
                raw.min_collateral_received_amount.0,
            );
        }

        let candidates = holders(f, |o| {
            !o.borrows.is_empty()
                && !o.deposits.is_empty()
                && o.insurance_fund_requests_ids.is_empty()
        });
        let unhealthy: Vec<_> =
            candidates.iter().filter(|(_, o)| looks_liquidatable(f, o)).cloned().collect();
        let eligible =
            if self.prefer_unhealthy && !unhealthy.is_empty() { &unhealthy } else { &candidates };
        let Some((borrower, obligation)) = choose(eligible, self.borrower) else {
            return LiquidationOutcome::NoCandidate;
        };
        let (Some(borrow_pool), Some(collateral_pool)) = (
            choose(&obligation.borrows.keys().iter().collect::<Vec<_>>(), self.borrow_pool),
            choose(&obligation.deposits.keys().iter().collect::<Vec<_>>(), self.collateral_pool),
        ) else {
            unreachable!("candidates hold both positions");
        };
        let liquidators: Vec<usize> = (0..users.len()).filter(|i| *i != borrower).collect();
        let liquidator = liquidators[self.liquidator as usize % liquidators.len()];

        let debt = obligation.borrows.get(borrow_pool.clone()).and_then(|position| {
            f.contract_client
                .get_pool(&borrow_pool)
                .compute_tokens_from_d_tokens_ceil(&f.e, position.d_tokens)
                .ok()
        });
        let collateral = obligation.deposits.get(collateral_pool.clone()).and_then(|position| {
            f.contract_client
                .get_pool(&collateral_pool)
                .compute_tokens_from_j_tokens_floor(&f.e, position.j_tokens)
                .ok()
                .map(|tokens| tokens.saturating_add(position.collateral))
        });

        liquidate(
            f,
            &users[liquidator],
            &users[borrower],
            &borrow_pool,
            &collateral_pool,
            scale_bps(debt.unwrap_or(1), self.repay_bps).max(1),
            scale_bps(collateral.unwrap_or(0), self.demand_bps),
        )
    }
}

/// The health comparison `Obligation::liquidate` makes, approximated from outside; it only steers
/// which borrower a step picks.
fn looks_liquidatable(f: &TestMarketFixture, obligation: &Obligation) -> bool {
    let pool = |address: Address| f.contract_client.get_pool(&address);
    let deposits: Vec<_> = obligation.deposits.iter().map(|(a, p)| (pool(a), p)).collect();
    let borrows: Vec<_> = obligation.borrows.iter().map(|(a, p)| (pool(a), p)).collect();

    f.e.as_contract(&f.contract_id, || -> Result<bool, MCError> {
        let (e, prices) = (&f.e, &mut Prices::new(&f.e));
        let floor = market::obligation::compute_min_collateral_threshold_scaled(e)?;
        let mut capacity = 0_i128;
        for (pool, position) in &deposits {
            let tokens = pool.compute_tokens_from_j_tokens_floor(e, position.j_tokens)?
                + position.collateral;
            let value =
                get_asset_value_floor(e, prices, tokens, &pool.token_address, pool.token_decimals)?;
            capacity += scale_bps(value, pool.config.health_config.close_ltv_bps) - floor;
        }
        let mut debt = 0_i128;
        for (pool, position) in &borrows {
            let tokens = pool.compute_tokens_from_d_tokens_ceil(e, position.d_tokens)?;
            let value =
                get_asset_value_floor(e, prices, tokens, &pool.token_address, pool.token_decimals)?;
            debt += scale_bps(value, pool.config.health_config.liability_factor_bps);
        }
        Ok(debt > capacity)
    })
    .unwrap_or(false)
}

/// Calls `liquidate` and classifies the result for the reach counter.
pub fn liquidate(
    f: &TestMarketFixture,
    liquidator: &Address,
    borrower: &Address,
    borrow_pool: &Address,
    collateral_pool: &Address,
    repay_amount: i128,
    min_collateral_amount: i128,
) -> LiquidationOutcome {
    let well_formed = repay_amount > 0
        && min_collateral_amount >= 0
        && borrow_pool != collateral_pool
        && liquidator != borrower
        && f.obligation(borrower).is_some_and(|o| {
            o.borrows.contains_key(borrow_pool.clone())
                && o.deposits.contains_key(collateral_pool.clone())
                && o.insurance_fund_requests_ids.is_empty()
        });

    let res = f.contract_client.try_liquidate(
        liquidator,
        &ObligationKey::new(borrower.clone()),
        borrow_pool,
        collateral_pool,
        &repay_amount,
        &min_collateral_amount,
    );
    if matches!(res, Err(Ok(MCError::InternalError))) {
        panic!("Internal Error");
    }
    let closed_collateral = res.is_ok()
        && f.obligation(borrower).is_none_or(|o| !o.deposits.contains_key(collateral_pool.clone()));

    LiquidationOutcome::Called {
        well_formed,
        closed_collateral,
        result: match res {
            Ok(_) => Ok(()),
            Err(Ok(error)) => Err(Some(error)),
            Err(Err(_)) => Err(None),
        },
    }
}

#[derive(Debug)]
pub enum LiquidationOutcome {
    /// No user holds a borrow beside a deposit, so there was nothing to call.
    NoCandidate,
    /// The legacy command skips a same-pool pair without a call.
    SamePool,
    /// `well_formed`: positive amounts, distinct pools and users, and both positions on the
    /// borrower, as the harness saw them before the call. `closed_collateral`: the seizure took the
    /// whole collateral position (insolvency or the dust floor). `Err(None)` is a host error.
    Called { well_formed: bool, closed_collateral: bool, result: Result<(), Option<MCError>> },
}

impl LiquidationOutcome {
    /// Whether the call got past the existence checks to the health comparison in
    /// `Obligation::liquidate`. Only errors raised at or after that comparison count; an ambiguous
    /// one (overflow, oracle) counts as not reached.
    pub fn reached_health_check(&self) -> bool {
        matches!(
            self,
            Self::Called {
                well_formed: true,
                result: Ok(())
                    | Err(Some(
                        MCError::ObligationIsHealthy
                            // With well-formed inputs, only the close factor raises it.
                            | MCError::InvalidLiquidationInputs
                            | MCError::LiquidationExcessiveDemandedCollateral
                            | MCError::NonPositiveSharesAmount
                            | MCError::BorrowPositionForAssetExists
                    )),
                ..
            }
        )
    }

    fn label(&self) -> String {
        match self {
            Self::NoCandidate => "not called: no borrower with collateral".into(),
            Self::SamePool => "not called: same pool".into(),
            Self::Called { result: Ok(()), closed_collateral: true, .. } => {
                "liquidated, collateral position closed".into()
            }
            Self::Called { result: Ok(()), .. } => "liquidated".into(),
            Self::Called { result: Err(Some(error)), .. } => format!("{error:?}"),
            Self::Called { result: Err(None), .. } => "host error".into(),
        }
    }
}

#[derive(Debug, Default)]
pub struct ReachTally {
    pub inputs: u32,
    pub calls: u32,
    pub reached: u32,
    pub liquidated: u32,
    pub outcomes: BTreeMap<String, u32>,
}

impl ReachTally {
    pub fn record(&mut self, outcome: &LiquidationOutcome) {
        if let LiquidationOutcome::Called { result, .. } = outcome {
            self.calls += 1;
            self.liquidated += u32::from(result.is_ok());
        }
        self.reached += u32::from(outcome.reached_health_check());
        *self.outcomes.entry(outcome.label()).or_default() += 1;
    }

    pub fn merge(&mut self, other: ReachTally) {
        self.inputs += other.inputs;
        self.calls += other.calls;
        self.reached += other.reached;
        self.liquidated += other.liquidated;
        for (label, count) in other.outcomes {
            *self.outcomes.entry(label).or_default() += count;
        }
    }
}

impl fmt::Display for ReachTally {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{} inputs, {} liquidation calls: {} reached the health check, {} liquidated",
            self.inputs, self.calls, self.reached, self.liquidated
        )?;
        for (label, count) in &self.outcomes {
            writeln!(f, "  {count:>7}  {label}")?;
        }
        Ok(())
    }
}

fn check<T, E>(res: Result<T, Result<MCError, E>>) {
    if matches!(res, Err(Ok(MCError::InternalError))) {
        panic!("Internal Error");
    }
}

fn key(f: &TestMarketFixture, user: usize) -> ObligationKey {
    ObligationKey::new(f.users[user].clone())
}

/// Users whose obligation passes `keep`, with that obligation.
fn holders(f: &TestMarketFixture, keep: impl Fn(&Obligation) -> bool) -> Vec<(usize, Obligation)> {
    f.users
        .iter()
        .enumerate()
        .filter_map(|(i, user)| f.obligation(user).filter(&keep).map(|o| (i, o)))
        .collect()
}

fn choose<T: Clone>(items: &[T], pick: u8) -> Option<T> {
    (!items.is_empty()).then(|| items[pick as usize % items.len()].clone())
}
