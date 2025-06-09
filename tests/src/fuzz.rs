use crate::{Amount, Borrow, Command, Deposit, Repay, TestFixture, Token, WithdrawCollateral};

use {
    soroban_sdk::{testutils::Address as _, Address},
    std::i128,
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

fn assert_invariants(fixture: &TestFixture) {
    let TestFixture {
        e,
        contract_client,
        gold_sac,
        gold_pool_address,
        btc_sac,
        btc_pool_address,
        usdc_sac,
        usdc_pool_address,
        ..
    } = fixture;

    let usdc_pool = contract_client.get_pool(&usdc_pool_address).unwrap();
    let gold_pool = contract_client.get_pool(&gold_pool_address).unwrap();
    let btc_pool = contract_client.get_pool(&btc_pool_address).unwrap();

    // All data on all pools is non-negative
    assert!(usdc_pool.total_supply >= 0);
    assert!(usdc_pool.total_borrowed >= 0);
    assert!(usdc_pool.total_collateral >= 0);

    assert!(gold_pool.total_supply >= 0);
    assert!(gold_pool.total_borrowed >= 0);
    assert!(gold_pool.total_collateral >= 0);

    assert!(btc_pool.total_supply >= 0);
    assert!(btc_pool.total_borrowed >= 0);
    assert!(btc_pool.total_collateral >= 0);

    // // Total deposited amount is always not smaller than total borrowed amount
    let usdc_pool_available = usdc_pool
        .total_supply
        .checked_sub(usdc_pool.total_borrowed)
        .unwrap();
    assert!(usdc_pool_available >= 0);

    let gold_pool_available = gold_pool
        .total_supply
        .checked_sub(gold_pool.total_borrowed)
        .unwrap();
    assert!(gold_pool_available >= 0);

    let btc_pool_available = btc_pool
        .total_supply
        .checked_sub(btc_pool.total_borrowed)
        .unwrap();
    assert!(btc_pool_available >= 0);

    // // You can always borrow available amount if it is healthy
    let new_borrower = Address::generate(e);

    let usdc_mint_amount = usdc_pool_available.saturating_mul(2).saturating_add(1);
    let btc_mint_amount = btc_pool_available.saturating_mul(2).saturating_add(1);
    let gold_mint_amount = gold_pool_available.saturating_mul(2).saturating_add(1);

    let max = i128::max(usdc_mint_amount, btc_mint_amount);
    let max = i128::max(gold_mint_amount, max);

    usdc_sac.mint(&new_borrower, &max);
    btc_sac.mint(&new_borrower, &max);
    gold_sac.mint(&new_borrower, &max);

    if btc_pool_available > 0 {
        contract_client.deposit_collateral(&new_borrower, &usdc_pool_address, &(max));
        contract_client.borrow(&new_borrower, &btc_pool_address, &btc_pool_available);
        contract_client.repay(&new_borrower, &btc_pool_address, &btc_pool_available);
        contract_client.withdraw_collateral(&new_borrower, &usdc_pool_address, &(max));
    }

    if gold_pool_available > 0 {
        contract_client.deposit_collateral(&new_borrower, &usdc_pool_address, &(max));

        contract_client.borrow(&new_borrower, &gold_pool_address, &gold_pool_available);
        contract_client.repay(&new_borrower, &gold_pool_address, &gold_pool_available);
        contract_client.withdraw_collateral(&new_borrower, &usdc_pool_address, &(max));
    }

    if usdc_pool_available > 0 {
        contract_client.deposit_collateral(&new_borrower, &gold_pool_address, &(max));

        contract_client.borrow(&new_borrower, &usdc_pool_address, &usdc_pool_available);
        contract_client.repay(&new_borrower, &usdc_pool_address, &usdc_pool_available);
        contract_client.withdraw_collateral(&new_borrower, &gold_pool_address, &(max));
    }
}
