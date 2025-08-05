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

#[test]
fn failing() {
    let input = Input {
        commands: [
            TomDeposit(Deposit {
                amount: Amount(1458528137475617424),
                token: BTC,
            }),
            NibblesWithdraw(Withdraw {
                amount: Amount(7668058320836127449),
                token: GOLD,
            }),
            PassTime(PassTime { amount: 195287065 }),
            PassTime(PassTime { amount: 195287282 }),
            PassTime(PassTime { amount: 195287282 }),
            NibblesWithdraw(Withdraw {
                amount: Amount(7668058320838956430),
                token: USDC,
            }),
            NibblesWithdraw(Withdraw {
                amount: Amount(7668058527715977834),
                token: GOLD,
            }),
            PassTime(PassTime { amount: 144439036 }),
            NibblesDepositCollateral(DepositCollateral {
                amount: Amount(5654752612283509176),
                token: GOLD,
            }),
            TomWithdrawFromLeveraged(WithdrawFromLeveraged {
                amount: Amount(5435941496873273144),
                deposit_token: USDC,
                borrow_token: USDC,
            }),
            TomRepay(Repay {
                amount: Amount(894644481270901354),
                token: GOLD,
            }),
            PassTime(PassTime { amount: 195287282 }),
            JerryBorrow(Borrow {
                amount: Amount(2810246167479189503),
                token: GOLD,
            }),
            PassTime(PassTime { amount: 208558949 }),
            NibblesWithdraw(Withdraw {
                amount: Amount(7680209178453502570),
                token: USDC,
            }),
            ButchDepositCollateral(DepositCollateral {
                amount: Amount(10766534864467066879),
                token: GOLD,
            }),
            PassTime(PassTime { amount: 130058565 }),
            TomDepositCollateral(DepositCollateral {
                amount: Amount(7772906986614860479),
                token: BTC,
            }),
            JerryRepay(Repay {
                amount: Amount(10),
                token: BTC,
            }),
            TomRepay(Repay {
                amount: Amount(0),
                token: BTC,
            }),
        ],
    };

    test_fuzzed_issue(&input);
}
