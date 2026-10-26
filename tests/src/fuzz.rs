#![cfg(test)]

use std::{
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    sync::atomic::{AtomicUsize, Ordering},
};

use arbitrary::{Arbitrary, Unstructured};

use crate::{
    Amount, BatchOp, Borrow,
    Command::*,
    Deposit, DepositCollateral, FlashBatch, Input, Liquidate, PassTime, Repay, TestMarketFixture,
    Token::*,
    Withdraw, WithdrawCollateral, make_oracle_prices_different,
    scenario::{ReachTally, Scenario, fuzz_fixture},
};

#[allow(unused)]
fn test_fuzzed_issue(input: &Input) {
    let test_fixture = TestMarketFixture::new();
    test_fixture.e.cost_estimate().budget().reset_unlimited();
    make_oracle_prices_different(&test_fixture.e, &test_fixture.oracle_client);

    for (ind, command) in input.commands.iter().enumerate() {
        command.run(&test_fixture);
        test_fixture.assert_invariants();
    }
}

/// A flash borrow followed, in the same transaction, by a deposit into the flashed pool: the shape
/// that repriced shares. Pinned as a deterministic regression for the flash command family.
#[test]
fn flash_batch_sep_28() {
    test_fuzzed_issue(&Input {
        commands: core::array::from_fn(|i| match i {
            0 => NibblesDeposit(Deposit { amount: Amount(3_000_000_000), token: USDC }),
            1 => JerryDeposit(Deposit { amount: Amount(2_000_000_000), token: GOLD }),
            2 => TomFlashBatch(FlashBatch {
                flash_token: USDC,
                flash_pct: 90,
                ops: [
                    BatchOp::Deposit(Deposit { amount: Amount(500_000_000), token: USDC }),
                    BatchOp::Deposit(Deposit { amount: Amount(100_000_000), token: USDC }),
                ],
            }),
            3 => ButchFlashBatch(FlashBatch {
                flash_token: GOLD,
                flash_pct: 75,
                ops: [
                    BatchOp::Deposit(Deposit { amount: Amount(200_000_000), token: GOLD }),
                    BatchOp::AddCollateral(DepositCollateral {
                        amount: Amount(10_000_000),
                        token: GOLD,
                    }),
                ],
            }),
            _ => AllPassTime(PassTime { amount: 3600 }),
        }),
    });
}

#[test]
fn dec_26() {
    test_fuzzed_issue(&Input {
        commands: [
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            ButchDeposit(Deposit { amount: Amount(289649777), token: GOLD }),
            JerryDeposit(Deposit { amount: Amount(1635931344), token: GOLD }),
            NibblesDeposit(Deposit { amount: Amount(1738514335), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            JerryBorrow(Borrow { amount: Amount(1503633311), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(1316599749), token: USDC }),
            AllPassTime(PassTime { amount: 27547829 }),
            JerryLiquidate(Liquidate {
                token: GOLD,
                repay_amount: Amount(4124595096),
                collateral_token: USDC,
                min_collateral_received_amount: Amount(2678038431),
            }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
        ],
    });
}

#[test]
fn test_y() {
    let input = Input {
        commands: [
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            ButchDeposit(Deposit { amount: Amount(289649777), token: GOLD }),
            JerryDeposit(Deposit { amount: Amount(1635931344), token: GOLD }),
            NibblesDeposit(Deposit { amount: Amount(1738514335), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            JerryBorrow(Borrow { amount: Amount(1503633311), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            AllPassTime(PassTime { amount: 0 }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
        ],
    };

    test_fuzzed_issue(&input);
}

#[test]
fn test_inconsistent_d_tokens_amount() {
    let input = Input {
        commands: [
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            JerryDeposit(Deposit { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
            TomRepay(Repay { amount: Amount(0), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            JerryBorrow(Borrow { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            AllPassTime(PassTime { amount: 52366013 }),
            TomRepay(Repay { amount: Amount(3590324223), token: GOLD }),
            AllPassTime(PassTime { amount: 6071227 }),
            AllPassTime(PassTime { amount: 6071227 }),
            AllPassTime(PassTime { amount: 6071227 }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(1906286495), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
        ],
    };

    test_fuzzed_issue(&input);
}

#[test]
fn test_x() {
    let input = Input {
        commands: [
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038443), token: BTC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(1197926579), token: BTC }),
            JerryDeposit(Deposit { amount: Amount(2975934976), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678056863), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            JerryBorrow(Borrow { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2680867530), token: GOLD }),
            AllPassTime(PassTime { amount: 0 }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
        ],
    };

    test_fuzzed_issue(&input);
}

#[test]
fn test_nov_30() {
    let input = Input {
        commands: [
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            ButchDeposit(Deposit { amount: Amount(289649777), token: GOLD }),
            JerryDeposit(Deposit { amount: Amount(1635931344), token: GOLD }),
            NibblesDeposit(Deposit { amount: Amount(1738514335), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            JerryBorrow(Borrow { amount: Amount(1503633311), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038393), token: BTC }),
            JerryWithdraw(Withdraw { amount: Amount(754636781), token: BTC }),
            TomRepay(Repay { amount: Amount(821075880), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
        ],
    };

    test_fuzzed_issue(&input);
}

#[test]
fn test_dec_2_1() {
    test_fuzzed_issue(&Input {
        commands: [
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            AllPassTime(PassTime { amount: 0 }),
            JerryDeposit(Deposit { amount: Amount(1635931344), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            ButchDepositCollateral(DepositCollateral { amount: Amount(132044232), token: BTC }),
            TomDepositCollateral(DepositCollateral { amount: Amount(482789362), token: USDC }),
            NibblesWithdrawCollateral(WithdrawCollateral {
                amount: Amount(3321542594),
                token: USDC,
            }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            ButchBorrow(Borrow { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038309), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678026655), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678628255), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
        ],
    });
}

#[test]
fn test_dec_2_2() {
    test_fuzzed_issue(&Input {
        commands: [
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: GOLD }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: BTC }),
            ButchDeposit(Deposit { amount: Amount(1162167621), token: BTC }),
            ButchDeposit(Deposit { amount: Amount(1162167621), token: BTC }),
            ButchDeposit(Deposit { amount: Amount(1162167621), token: BTC }),
            ButchDeposit(Deposit { amount: Amount(1162167621), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2667577344), token: BTC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomBorrow(Borrow { amount: Amount(522133279), token: BTC }),
            TomBorrow(Borrow { amount: Amount(522133407), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678026655), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: BTC }),
        ],
    });
}

const REACH_SEEDS: usize = 128;
const REACH_INPUT_LEN: usize = 320;

/// Replays scenarios and counts how far their liquidations get. `FUZZ_REACH_CORPUS=<dir>` replays a
/// libFuzzer corpus instead of the seeded set, at the floor in `FUZZ_REACH_FLOOR_CENTS` (default 0);
/// `FUZZ_REACH_LEGACY=1` decodes it as the legacy `Input`, for a baseline.
#[test]
fn liquidation_reach() {
    let Ok(dir) = std::env::var("FUZZ_REACH_CORPUS") else {
        for cents in [0, 500] {
            let tally = replay(&seeded_inputs(), cents, false);
            println!("seeded set, floor {cents} cents: {tally}");
            assert_substantial_reach(&tally);
        }
        return;
    };

    let cents = std::env::var("FUZZ_REACH_FLOOR_CENTS").map_or(0, |v| v.parse().unwrap());
    let legacy = std::env::var("FUZZ_REACH_LEGACY").is_ok_and(|v| v == "1");
    let mut paths: Vec<_> =
        std::fs::read_dir(&dir).unwrap().map(|entry| entry.unwrap().path()).collect();
    paths.sort();
    let inputs: Vec<_> = paths
        .iter()
        .map(|path| (path.display().to_string(), std::fs::read(path).unwrap()))
        .collect();

    let tally = replay(&inputs, cents, legacy);
    println!("{dir}, floor {cents} cents, legacy {legacy}: {tally}");
    // A corpus is kept for coverage, so its malformed calls are over-represented and only
    // inertness is asserted; the seeded set above carries the ratios.
    if !legacy {
        assert!(tally.reached > 0 && tally.liquidated > 0, "the corpus never liquidated");
    }
}

fn assert_substantial_reach(tally: &ReachTally) {
    assert!(tally.calls >= tally.inputs, "under one liquidation call per input");
    assert!(tally.reached * 2 >= tally.calls, "under half the calls reached the health check");
    assert!(tally.liquidated * 10 >= tally.calls, "under a tenth of the calls liquidated");
}

fn seeded_inputs() -> Vec<(String, Vec<u8>)> {
    // splitmix64
    let mut state = 0x9E37_79B9_7F4A_7C15_u64;
    let mut next_byte = move || {
        state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        (z ^ (z >> 31)) as u8
    };

    (0..REACH_SEEDS)
        .map(|i| (format!("seed {i}"), (0..REACH_INPUT_LEN).map(|_| next_byte()).collect()))
        .collect()
}

/// Replays on unnamed worker threads, which also keeps the SDK from writing a test snapshot per
/// input.
fn replay(inputs: &[(String, Vec<u8>)], cents: i128, legacy: bool) -> ReachTally {
    let next = AtomicUsize::new(0);
    let workers = std::thread::available_parallelism().map_or(1, |n| n.get()).min(16);

    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..workers)
            .map(|_| {
                scope.spawn(|| {
                    let mut tally = ReachTally::default();
                    while let Some((name, bytes)) = inputs.get(next.fetch_add(1, Ordering::Relaxed))
                    {
                        if let Err(panic) = catch_unwind(AssertUnwindSafe(|| {
                            replay_one(bytes, cents, legacy, &mut tally)
                        })) {
                            eprintln!("replaying {name} failed");
                            resume_unwind(panic);
                        }
                    }
                    tally
                })
            })
            .collect();

        handles.into_iter().fold(ReachTally::default(), |mut total, handle| {
            total.merge(handle.join().unwrap_or_else(|panic| resume_unwind(panic)));
            total
        })
    })
}

/// Decodes as libfuzzer-sys does, skipping what the target would skip.
fn replay_one(bytes: &[u8], cents: i128, legacy: bool, tally: &mut ReachTally) {
    if !legacy {
        if let Ok(scenario) = Scenario::arbitrary_take_rest(Unstructured::new(bytes)) {
            scenario.run(cents, tally);
        }
        return;
    }

    if bytes.len() < Input::size_hint(0).0 {
        return;
    }
    let Ok(input) = Input::arbitrary_take_rest(Unstructured::new(bytes)) else {
        return;
    };
    let f = fuzz_fixture(cents);
    tally.inputs += 1;
    for command in &input.commands {
        match command.liquidation() {
            Some((liquidation, who)) => tally.record(&liquidation.attempt(&f, who)),
            None => command.run(&f),
        }
        f.assert_invariants();
    }
}
