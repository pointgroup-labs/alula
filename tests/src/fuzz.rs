#![cfg(test)]

use soroban_sdk::{testutils::Address as _, Address};

use crate::{
    Amount, Borrow,
    Command::{self, *},
    Deposit, DepositCollateral, DepositWithLeverage, Input, Liquidate, PassTime, Repay,
    TestFixture,
    Token::*,
    Withdraw, WithdrawCollateral, WithdrawFromLeveraged,
};

fn test_fuzzed_issue(input: &Input) {
    let test_fixture = TestFixture::new();
    test_fixture.e.cost_estimate().budget().reset_unlimited();

    for command in &input.commands {
        command.run(&test_fixture);
        test_fixture.assert_invariants();
    }
}
