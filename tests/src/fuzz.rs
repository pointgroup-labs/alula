#![cfg(test)]

use crate::{
    Amount, Borrow, Command::*, Deposit, DepositCollateral, DepositWithLeverage, Input, PassTime,
    Repay, TestMarketFixture, Token::*,
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
            JerryDepositWithLeverage(DepositWithLeverage {
                amount: Amount(3402287818),
                deposit_token: GOLD,
                borrow_token: USDC,
                flash_loan_amount: Amount(2678038431),
                leverage: 2678038431,
            }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
        ],
    };

    test_fuzzed_issue(&input);
}

#[test]
#[ignore]
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
            NibblesDepositWithLeverage(DepositWithLeverage {
                amount: Amount(1402509471),
                deposit_token: USDC,
                borrow_token: USDC,
                flash_loan_amount: Amount(2678026655),
                leverage: 2678038431,
            }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            NibblesDepositCollateral(DepositCollateral { amount: Amount(2678038431), token: USDC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
        ],
    };

    test_fuzzed_issue(&input);
}
