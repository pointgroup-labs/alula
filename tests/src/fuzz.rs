#![cfg(test)]

#[allow(unused)]
use crate::{
    assert_invariants, Amount, Borrow,
    Command::{self, *},
    Deposit, DepositWithLeverage, Input, Repay, TestFixture,
    Token::*,
    Withdraw, WithdrawCollateral,
};

#[test]
fn test_transfer_over_balance() {
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

    test_fuzzed_issue(&commands);
}

#[test]
fn test_repay_breaks_token_balance_invariant() {
    let commands = [
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: BTC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: BTC,
        }),
        TomDeposit(Deposit {
            amount: Amount(4702111234474983745),
            token: BTC,
        }),
        TomDeposit(Deposit {
            amount: Amount(4694481606870967846),
            token: BTC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: BTC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: BTC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926861050014314),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        TomRepay(Repay {
            amount: Amount(17672478161302012225),
            token: BTC,
        }),
    ];

    test_fuzzed_issue(&commands);
}

#[test]
fn test_div_by_zero() {
    let commands = [
        TomBorrow(Borrow {
            amount: Amount(2748926567846913574),
            token: GOLD,
        }),
        JerryDepositWithLeverage(DepositWithLeverage {
            amount: Amount(15647235900636014118),
            deposit_token: BTC,
            borrow_token: USDC,
            flash_loan_amount: Amount(7668897261177436522),
            leverage: 1785358954,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        JerryWithdraw(Withdraw {
            amount: Amount(7668058320836127338),
            token: USDC,
        }),
        TomBorrow(Borrow {
            amount: Amount(2748926567846913572),
            token: BTC,
        }),
    ];

    test_fuzzed_issue(&commands);
}

fn test_fuzzed_issue(commands: &[Command]) {
    let test_fixture = TestFixture::new();

    for command in commands {
        command.run(&test_fixture);
        assert_invariants(&test_fixture);
    }
}
