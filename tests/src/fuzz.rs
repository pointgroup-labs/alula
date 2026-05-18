#![cfg(test)]

use crate::{
    Actor, Amount, Borrow, Command, Deposit, DepositCollateral, Input, Liquidate, Op, PassTime,
    Repay, TestMarketFixture, Token::*, Withdraw, WithdrawCollateral, make_oracle_prices_different,
};

#[allow(unused)]
fn test_fuzzed_issue(input: &Input) {
    let test_fixture = TestMarketFixture::new();
    test_fixture.e.cost_estimate().budget().reset_unlimited();
    make_oracle_prices_different(&test_fixture.e, &test_fixture.oracle_client);

    #[allow(clippy::unused_enumerate_index)]
    for (_idx, command) in input.commands.iter().enumerate() {
        command.run(&test_fixture);
        test_fixture.assert_invariants();
    }
}

#[test]
fn dec_26() {
    test_fuzzed_issue(&Input {
        commands: vec![
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Deposit(Deposit { amount: Amount(289649777), token: GOLD }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Deposit(Deposit { amount: Amount(1635931344), token: GOLD }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::Deposit(Deposit { amount: Amount(1738514335), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Borrow(Borrow { amount: Amount(1503633311), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(1316599749),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 27547829 }) },
            Command {
                actor: Actor::Jerry,
                op: Op::Liquidate(Liquidate {
                    token: GOLD,
                    repay_amount: Amount(4124595096),
                    collateral_token: USDC,
                    min_collateral_received_amount: Amount(2678038431),
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
        ],
    });
}

#[test]
fn test_y() {
    test_fuzzed_issue(&Input {
        commands: vec![
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Deposit(Deposit { amount: Amount(289649777), token: GOLD }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Deposit(Deposit { amount: Amount(1635931344), token: GOLD }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::Deposit(Deposit { amount: Amount(1738514335), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Borrow(Borrow { amount: Amount(1503633311), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 0 }) },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
        ],
    });
}

#[test]
fn test_inconsistent_d_tokens_amount() {
    test_fuzzed_issue(&Input {
        commands: vec![
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Deposit(Deposit { amount: Amount(2678038431), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: USDC }) },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Borrow(Borrow { amount: Amount(2678038431), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 52366013 }) },
            Command {
                actor: Actor::Tom,
                op: Op::Repay(Repay { amount: Amount(3590324223), token: GOLD }),
            },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 6071227 }) },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 6071227 }) },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 6071227 }) },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(1906286495),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
        ],
    });
}

#[test]
fn test_x() {
    test_fuzzed_issue(&Input {
        commands: vec![
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038443),
                    token: BTC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(1197926579),
                    token: BTC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Deposit(Deposit { amount: Amount(2975934976), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678056863),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Borrow(Borrow { amount: Amount(2678038431), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2680867530),
                    token: GOLD,
                }),
            },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 0 }) },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
        ],
    });
}

#[test]
fn test_nov_30() {
    test_fuzzed_issue(&Input {
        commands: vec![
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Deposit(Deposit { amount: Amount(289649777), token: GOLD }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Deposit(Deposit { amount: Amount(1635931344), token: GOLD }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::Deposit(Deposit { amount: Amount(1738514335), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Borrow(Borrow { amount: Amount(1503633311), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038393),
                    token: BTC,
                }),
            },
            Command {
                actor: Actor::Jerry,
                op: Op::Withdraw(Withdraw { amount: Amount(754636781), token: BTC }),
            },
            Command {
                actor: Actor::Tom,
                op: Op::Repay(Repay { amount: Amount(821075880), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
        ],
    });
}

#[test]
fn test_dec_2_1() {
    test_fuzzed_issue(&Input {
        commands: vec![
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::PassTime(PassTime { amount: 0 }) },
            Command {
                actor: Actor::Jerry,
                op: Op::Deposit(Deposit { amount: Amount(1635931344), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(132044232),
                    token: BTC,
                }),
            },
            Command {
                actor: Actor::Tom,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(482789362),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::WithdrawCollateral(WithdrawCollateral {
                    amount: Amount(3321542594),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Borrow(Borrow { amount: Amount(2678038431), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038309),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678026655),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678628255),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
        ],
    });
}

#[test]
fn test_dec_2_2() {
    test_fuzzed_issue(&Input {
        commands: vec![
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: GOLD,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: BTC,
                }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Deposit(Deposit { amount: Amount(1162167621), token: BTC }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Deposit(Deposit { amount: Amount(1162167621), token: BTC }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Deposit(Deposit { amount: Amount(1162167621), token: BTC }),
            },
            Command {
                actor: Actor::Butch,
                op: Op::Deposit(Deposit { amount: Amount(1162167621), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2667577344),
                    token: BTC,
                }),
            },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
            Command { actor: Actor::Tom, op: Op::Repay(Repay { amount: Amount(0), token: BTC }) },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Tom,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Tom,
                op: Op::Borrow(Borrow { amount: Amount(522133279), token: BTC }),
            },
            Command {
                actor: Actor::Tom,
                op: Op::Borrow(Borrow { amount: Amount(522133407), token: USDC }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678026655),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: USDC,
                }),
            },
            Command {
                actor: Actor::Nibbles,
                op: Op::DepositCollateral(DepositCollateral {
                    amount: Amount(2678038431),
                    token: BTC,
                }),
            },
        ],
    });
}
