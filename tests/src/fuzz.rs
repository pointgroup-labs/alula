#![cfg(test)]

use crate::{
    Amount, Borrow, Command::*, Deposit, DepositCollateral, Input, PassTime, Repay,
    TestMarketFixture, Token::*,
};

#[allow(unused)]
fn test_fuzzed_issue(input: &Input) {
    let test_fixture = TestMarketFixture::new();
    test_fixture.e.cost_estimate().budget().reset_unlimited();

    for command in &input.commands {
        command.run(&test_fixture);
        test_fixture.assert_invariants();
    }
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
