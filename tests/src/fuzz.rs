#![cfg(test)]

use crate::{
    Amount, Borrow, Command::*, Deposit, Input, Repay, TestMarketFixture, Token::*, Withdraw,
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
fn new_one() {
    let input = Input {
        commands: [
            JerryBorrow(Borrow { amount: Amount(640034342), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(640034342), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(4294967295), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(640034342), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(640036390), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(640034342), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(640034342), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(640034342), token: BTC }),
            JerryBorrow(Borrow { amount: Amount(640034410), token: USDC }),
            NibblesWithdraw(Withdraw { amount: Amount(1785358954), token: USDC }),
            NibblesWithdraw(Withdraw { amount: Amount(1785358954), token: USDC }),
            NibblesWithdraw(Withdraw { amount: Amount(1785358954), token: USDC }),
            NibblesWithdraw(Withdraw { amount: Amount(1785358954), token: BTC }),
            JerryDeposit(Deposit { amount: Amount(1094795585), token: BTC }),
            JerryDeposit(Deposit { amount: Amount(1094795585), token: BTC }),
            JerryDeposit(Deposit { amount: Amount(1094795585), token: BTC }),
            JerryDeposit(Deposit { amount: Amount(1094795585), token: BTC }),
            JerryDeposit(Deposit { amount: Amount(1094795585), token: BTC }),
            JerryDeposit(Deposit { amount: Amount(1094795585), token: BTC }),
            TomRepay(Repay { amount: Amount(0), token: BTC }),
        ],
    };

    test_fuzzed_issue(&input);
}
