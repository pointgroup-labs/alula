use crate::{
    assert_invariants, Amount, Borrow, Command::*, Deposit, Input, TestFixture, Token::*,
    WithdrawCollateral,
};

#[test]
fn test_fuzzed_issue() {
    let test_fixture = TestFixture::new();

    // Copied from the failing `cargo fuzz` output
    let input = Input {
        commands: [
            TomBorrow(Borrow {
                amount: Amount(2748926567846913574),
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
            JerryWithdrawCollateral(WithdrawCollateral {
                amount: Amount(18446744073694243434),
                token: USDC,
            }),
            TomDeposit(Deposit {
                amount: Amount(5931894172722287186),
                token: BTC,
            }),
            TomDeposit(Deposit {
                amount: Amount(5931894172722287186),
                token: BTC,
            }),
            TomDeposit(Deposit {
                amount: Amount(5980780305148018687),
                token: BTC,
            }),
            JerryDeposit(Deposit {
                amount: Amount(7668058320836127338),
                token: USDC,
            }),
            JerryDeposit(Deposit {
                amount: Amount(18404639832487389734),
                token: GOLD,
            }),
            TomDeposit(Deposit {
                amount: Amount(5931894172722287186),
                token: BTC,
            }),
        ],
    };

    for command in input.commands {
        command.run(&test_fixture);
        assert_invariants(&test_fixture);
    }
}
