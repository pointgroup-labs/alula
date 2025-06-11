#![cfg(test)]

#[allow(unused)]
use crate::{
    assert_invariants, Amount, Borrow, Command::*, Deposit, Input, Repay, TestFixture, Token::*,
    Withdraw, WithdrawCollateral,
};

#[test]
fn test_fuzzed_issue() {
    let test_fixture = TestFixture::new();

    // Copied from the failing `cargo fuzz` output
    let commands = [
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: BTC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: BTC,
        }),
        JerryWithdrawCollateral(WithdrawCollateral {
            amount: Amount(18446744073709491750),
            token: BTC,
        }),
        JerryRepay(Repay {
            amount: Amount(2748926567846913657),
            token: USDC,
        }),
        TomWithdraw(Withdraw {
            amount: Amount(8753160913407277433),
            token: USDC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: BTC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926861050014314),
            token: BTC,
        }),
        JerryBorrow(Borrow {
            amount: Amount(4702111411259206250),
            token: USDC,
        }),
        JerryDeposit(Deposit {
            amount: Amount(7668058027634803009),
            token: BTC,
        }),
        JerryBorrow(Borrow {
            amount: Amount(4702111234474983745),
            token: BTC,
        }),
    ];

    for command in commands {
        command.run(&test_fixture);
        assert_invariants(&test_fixture);
    }
}
