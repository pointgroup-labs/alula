use crate::{
    assert_invariants, Amount, Borrow, Command, Deposit, Repay, TestFixture, Token,
    WithdrawCollateral,
};

#[test]
fn test_fuzzed_issue() {
    let test_fixture = TestFixture::new();

    let commands: Vec<Command> = vec![
        Command::TomBorrow(Borrow {
            amount: Amount(3272545084522523242),
            token: Token::USDC,
        }),
        Command::JerryDeposit(Deposit {
            amount: Amount(7668058320836127386),
            token: Token::USDC,
        }),
        Command::JerryWithdrawCollateral(WithdrawCollateral {
            amount: Amount(18446744073709551615),
            token: Token::GOLD,
        }),
        Command::JerryWithdrawCollateral(WithdrawCollateral {
            amount: Amount(18446744069481693183),
            token: Token::GOLD,
        }),
        Command::JerryWithdrawCollateral(WithdrawCollateral {
            amount: Amount(18446744073709551615),
            token: Token::USDC,
        }),
        Command::JerryDeposit(Deposit {
            amount: Amount(7668058320836127338),
            token: Token::USDC,
        }),
        Command::JerryWithdrawCollateral(WithdrawCollateral {
            amount: Amount(0),
            token: Token::BTC,
        }),
        Command::TomRepay(Repay {
            amount: Amount(0),
            token: Token::BTC,
        }),
        Command::TomRepay(Repay {
            amount: Amount(18446744073709551615),
            token: Token::GOLD,
        }),
        Command::JerryWithdrawCollateral(WithdrawCollateral {
            amount: Amount(18446744073709551615),
            token: Token::GOLD,
        }),
    ];

    for command in commands {
        command.run(&test_fixture);
        assert_invariants(&test_fixture);
    }
}
