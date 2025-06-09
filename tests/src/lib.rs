use {
    arbitrary::Unstructured,
    lending::{
        constants::{LCError, INDIVIDUAL_BUMP, REFLECTOR_TESTNET_ADDRESS},
        contract::{LendingContract, LendingContractClient},
        oracle,
        storage::{BorrowObligation, DepositObligation},
    },
    soroban_sdk::{
        symbol_short,
        testutils::{arbitrary::Arbitrary, Address as _, Ledger},
        token::{StellarAssetClient, TokenClient},
        vec, Address, Env, String, Vec,
    },
};

pub const DEFAULT_HEALTH_FACTOR_THRESHOLD: i128 = 80;
pub const DEFAULT_ADMIN_ASSET_MINT_AMOUNT: i128 = 1_000_000;
pub const DEFAULT_USER_ASSET_MINT_AMOUNT: i128 = 100_000;
pub const DEFAULT_DEPOSIT_AMOUNT: i128 = DEFAULT_USER_ASSET_MINT_AMOUNT / 2;
#[allow(unused)]
pub const DEFAULT_COLLATERAL_AMOUNT: i128 = DEFAULT_USER_ASSET_MINT_AMOUNT / 2;

#[derive(Arbitrary, Debug, Clone, Copy)]
pub enum Token {
    BTC,
    USDC,
    GOLD,
}

#[allow(unused)]
pub struct TestFixture<'a> {
    pub e: Env,
    pub contract_client: LendingContractClient<'a>,
    pub contract_id: Address,
    pub contract_admin: Address,
    // GOLD
    pub gold_sac: StellarAssetClient<'a>,
    pub gold_token_client: TokenClient<'a>,
    pub gold_token_address: Address,
    pub gold_admin: Address,
    pub gold_pool_address: Address,
    // BTC
    pub btc_sac: StellarAssetClient<'a>,
    pub btc_token_client: TokenClient<'a>,
    pub btc_token_address: Address,
    pub btc_admin: Address,
    pub btc_pool_address: Address,
    // USDC
    pub usdc_sac: StellarAssetClient<'a>,
    pub usdc_token_client: TokenClient<'a>,
    pub usdc_token_address: Address,
    pub usdc_admin: Address,
    pub usdc_pool_address: Address,
    pub users: Vec<Address>,
}

impl Default for TestFixture<'_> {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture<'_> {
    pub fn new() -> Self {
        let e = Env::default();
        e.mock_all_auths();

        e.ledger().with_mut(|li| {
            li.sequence_number = 0;
            li.max_entry_ttl = INDIVIDUAL_BUMP + 1;
        });

        let contract_admin = Address::generate(&e);
        let contract_id = e.register(
            LendingContract,
            (
                contract_admin.clone(),
                Option::<i128>::Some(DEFAULT_HEALTH_FACTOR_THRESHOLD),
            ),
        );
        let contract_client = LendingContractClient::new(&e, &contract_id);

        let users = vec![
            &e,
            Address::generate(&e),
            Address::generate(&e),
            Address::generate(&e),
        ];

        let usdc_admin = Address::generate(&e);
        let gold_admin = Address::generate(&e);
        let btc_admin = Address::generate(&e);

        // GOLD
        let TestAssetSetup {
            sac_client: gold_sac,
            token_client: gold_token_client,
            token_address: gold_token_address,
        } = setup_test_asset(&e, &gold_admin, &users);
        let gold_pool_address = contract_client.initialize_pool(
            &gold_token_address,
            &symbol_short!("GOLD"),
            &None,
            &None,
        );

        // BTC
        let TestAssetSetup {
            sac_client: btc_sac,
            token_client: btc_token_client,
            token_address: btc_token_address,
        } = setup_test_asset(&e, &btc_admin, &users);
        let btc_pool_address = contract_client.initialize_pool(
            &btc_token_address,
            &symbol_short!("BTC"),
            &None,
            &None,
        );

        // USDC
        let TestAssetSetup {
            sac_client: usdc_sac,
            token_client: usdc_token_client,
            token_address: usdc_token_address,
        } = setup_test_asset(&e, &usdc_admin, &users);
        let usdc_pool_address = contract_client.initialize_pool(
            &usdc_token_address,
            &symbol_short!("USDC"),
            &None,
            &None,
        );

        let mock_oracle_address =
            Address::from_string(&String::from_str(&e, REFLECTOR_TESTNET_ADDRESS));
        e.register_at(&mock_oracle_address, oracle::WASM, ());

        Self {
            e,
            contract_client,
            contract_id,
            contract_admin,
            // GOLD
            gold_sac,
            gold_token_client,
            gold_token_address,
            gold_admin,
            gold_pool_address,
            // BTC
            btc_sac,
            btc_token_client,
            btc_token_address,
            btc_pool_address,
            btc_admin,
            // USDC
            usdc_sac,
            usdc_token_client,
            usdc_token_address,
            usdc_pool_address,
            usdc_admin,
            users,
        }
    }

    pub fn get_pool_address(&self, token: Token) -> Address {
        let address = match token {
            Token::BTC => &self.btc_pool_address,
            Token::USDC => &self.usdc_pool_address,
            Token::GOLD => &self.gold_pool_address,
        };

        address.clone()
    }

    pub fn get_token_client(&self, token: Token) -> &TokenClient {
        match token {
            Token::BTC => &self.btc_token_client,
            Token::USDC => &self.usdc_token_client,
            Token::GOLD => &self.gold_token_client,
        }
    }
}

pub struct TestAssetSetup<'a> {
    token_client: TokenClient<'a>,
    token_address: Address,
    sac_client: StellarAssetClient<'a>,
}

pub fn setup_test_asset<'a>(e: &Env, admin: &Address, users: &Vec<Address>) -> TestAssetSetup<'a> {
    let token_address = e
        .register_stellar_asset_contract_v2(admin.clone())
        .address();
    let sac_client = StellarAssetClient::new(e, &token_address);
    let token_client = TokenClient::new(e, &token_address);

    sac_client.mint(admin, &DEFAULT_USER_ASSET_MINT_AMOUNT);

    for user in users {
        sac_client.mint(&user, &DEFAULT_USER_ASSET_MINT_AMOUNT);
    }

    TestAssetSetup {
        token_address,
        token_client,
        sac_client,
    }
}

#[derive(Arbitrary, Debug)]
pub enum Command {
    TomRepay(Repay),
    JerryRepay(Repay),

    TomBorrow(Borrow),
    JerryBorrow(Borrow),

    TomDeposit(Deposit),
    JerryDeposit(Deposit),

    TomWithdraw(Withdraw),
    JerryWithdraw(Withdraw),

    TomLiquidate(Liquidate),
    JerryLiquidate(Liquidate),

    TomDepositCollateral(DepositCollateral),
    JerryDepositCollateral(DepositCollateral),

    TomWithdrawCollateral(WithdrawCollateral),
    JerryWithdrawCollateral(WithdrawCollateral),
    // PassTime(),
}

impl Command {
    pub fn run(&self, test_fixture: &TestFixture) {
        match self {
            Command::TomRepay(command) => command.run(test_fixture, 0),
            Command::TomBorrow(command) => command.run(test_fixture, 0),
            Command::TomDeposit(command) => command.run(test_fixture, 0),
            Command::TomWithdraw(command) => command.run(test_fixture, 0),
            Command::TomLiquidate(command) => command.run(test_fixture, 0),
            Command::TomDepositCollateral(command) => command.run(test_fixture, 0),
            Command::TomWithdrawCollateral(command) => command.run(test_fixture, 0),

            Command::JerryRepay(command) => command.run(test_fixture, 1),
            Command::JerryBorrow(command) => command.run(test_fixture, 1),
            Command::JerryDeposit(command) => command.run(test_fixture, 1),
            Command::JerryWithdraw(command) => command.run(test_fixture, 1),
            Command::JerryLiquidate(command) => command.run(test_fixture, 1),
            Command::JerryDepositCollateral(command) => command.run(test_fixture, 1),
            Command::JerryWithdrawCollateral(command) => command.run(test_fixture, 1),
        }
    }
}

#[derive(Arbitrary, Debug)]
pub struct Amount(
    #[arbitrary(with = |u: &mut Unstructured| u.int_in_range(0..=(u64::MAX as i128)))] pub i128,
);

pub fn assert_invariants(fixture: &TestFixture) {
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

    let usdc_pool = contract_client.get_pool(usdc_pool_address).unwrap();
    let gold_pool = contract_client.get_pool(gold_pool_address).unwrap();
    let btc_pool = contract_client.get_pool(btc_pool_address).unwrap();

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

    // Total deposited amount is always not smaller than total borrowed amount
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

    // You can always borrow and repay the available amount
    let new_borrower = Address::generate(e);

    let max_collateral_amount = i128::max(usdc_pool_available, btc_pool_available);
    let max_collateral_amount = i128::max(gold_pool_available, max_collateral_amount);

    usdc_sac.mint(&new_borrower, &max_collateral_amount);
    btc_sac.mint(&new_borrower, &max_collateral_amount);
    gold_sac.mint(&new_borrower, &max_collateral_amount);

    if btc_pool_available > 0 {
        contract_client.deposit_collateral(
            &new_borrower,
            usdc_pool_address,
            &max_collateral_amount,
        );
        contract_client.borrow(&new_borrower, btc_pool_address, &btc_pool_available);
        contract_client.repay(&new_borrower, btc_pool_address, &btc_pool_available);
        contract_client.withdraw_collateral(
            &new_borrower,
            usdc_pool_address,
            &(max_collateral_amount),
        );
    }

    if gold_pool_available > 0 {
        contract_client.deposit_collateral(
            &new_borrower,
            usdc_pool_address,
            &max_collateral_amount,
        );

        contract_client.borrow(&new_borrower, gold_pool_address, &gold_pool_available);
        contract_client.repay(&new_borrower, gold_pool_address, &gold_pool_available);
        contract_client.withdraw_collateral(
            &new_borrower,
            usdc_pool_address,
            &max_collateral_amount,
        );
    }

    if usdc_pool_available > 0 {
        contract_client.deposit_collateral(
            &new_borrower,
            gold_pool_address,
            &max_collateral_amount,
        );
        contract_client.borrow(&new_borrower, usdc_pool_address, &usdc_pool_available);
        contract_client.repay(&new_borrower, usdc_pool_address, &usdc_pool_available);
        contract_client.withdraw_collateral(
            &new_borrower,
            gold_pool_address,
            &max_collateral_amount,
        );
    }
}

#[derive(Arbitrary, Debug)]
pub struct Input {
    pub commands: [Command; 10],
}

#[derive(Arbitrary, Debug)]
pub struct Repay {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Borrow {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Deposit {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Withdraw {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]

pub struct DepositCollateral {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct WithdrawCollateral {
    pub amount: Amount,
    pub token: Token,
}

#[derive(Arbitrary, Debug)]
pub struct Liquidate {
    pub amount: Amount,
    pub token: Token,
}

impl Borrow {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_borrow(&user, &pool_address, &self.amount.0);
    }
}

impl Deposit {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_deposit(&user, &pool_address, &self.amount.0);
    }
}

impl DepositCollateral {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_deposit_collateral(&user, &pool_address, &self.amount.0);
    }
}

impl WithdrawCollateral {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_withdraw_collateral(&user, &pool_address, &self.amount.0);
    }
}

impl Withdraw {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_withdraw(&user, &pool_address, &self.amount.0);
    }
}

impl Repay {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let user = users.get(who).unwrap();
        let _ = contract_client.try_withdraw(&user, &pool_address, &self.amount.0);
    }
}

impl Liquidate {
    pub fn run(&self, test_fixture: &TestFixture, who: u32) {
        let pool_address = test_fixture.get_pool_address(self.token);
        let TestFixture {
            contract_client,
            users,
            ..
        } = test_fixture;

        let (liquidator, borrower) = if who == 0 {
            (users.get(0).unwrap(), users.get(1).unwrap())
        } else {
            (users.get(1).unwrap(), users.get(0).unwrap())
        };

        let _ =
            contract_client.try_liquidate(&liquidator, &borrower, &pool_address, &self.amount.0);
    }
}

#[allow(unused)]
pub fn get_deposit_obligation(
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<DepositObligation, LCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_user_obligation(user) else {
        return Err(LCError::ObligationDoesNotExist);
    };

    let deposit = obligation
        .deposits
        .get(pool_address.clone())
        .ok_or(LCError::DepositDoesNotExist)?;

    Ok(deposit)
}

#[allow(unused)]
pub fn get_borrow_obligation(
    contract_client: &LendingContractClient,
    user: &Address,
    pool_address: &Address,
) -> Result<BorrowObligation, LCError> {
    let Ok(Ok(obligation)) = contract_client.try_get_user_obligation(user) else {
        return Err(LCError::ObligationDoesNotExist);
    };

    let borrow = obligation
        .borrows
        .get(pool_address.clone())
        .ok_or(LCError::DepositDoesNotExist)?;

    Ok(borrow)
}

#[cfg(test)]
mod tests {
    use {
        super::*,
        lending::{
            constants::{INDIVIDUAL_BUMP, INSTANCE_BUMP, LEDGERS_PER_DAY, SHARED_BUMP},
            storage::DataKey,
        },
        soroban_sdk::testutils::{
            storage::{Instance, Persistent},
            Ledger,
        },
    };

    #[test]
    fn test_storage_ttl_extension() {
        let TestFixture {
            e,
            contract_client,
            contract_id,
            usdc_pool_address,
            users,
            ..
        } = TestFixture::new();

        let user = users.get(0).unwrap();

        e.as_contract(&contract_id, || {
            // `TestFixture::new()` extends both instance and a specific's pool shared storage
            assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);
            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
                SHARED_BUMP
            );
        });

        // Extend individual user's storage
        contract_client.deposit(&user, &usdc_pool_address, &1);

        e.as_contract(&contract_id, || {
            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Obligation(user.clone())),
                INDIVIDUAL_BUMP
            );
        });

        e.ledger().with_mut(|li| {
            // TODO: Make all shifts depend on the threshold
            // and not on the constant amount of ledgers
            li.sequence_number = 2 * LEDGERS_PER_DAY;
        });

        e.as_contract(&contract_id, || {
            assert_eq!(
                e.storage().instance().get_ttl(),
                INSTANCE_BUMP - 2 * LEDGERS_PER_DAY
            );
            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
                SHARED_BUMP - 2 * LEDGERS_PER_DAY
            );

            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Obligation(user.clone())),
                INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
            );
        });

        // Extend instance storage
        contract_client.get_global_state();

        e.as_contract(&contract_id, || {
            // Instance's ttl is bumped
            assert_eq!(e.storage().instance().get_ttl(), INSTANCE_BUMP);

            // Others aren't bumped
            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
                SHARED_BUMP - 2 * LEDGERS_PER_DAY
            );

            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Obligation(user.clone())),
                INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
            );
        });

        // Deposit once more to bump shared persistent token storage
        contract_client.deposit(&user, &usdc_pool_address, &1);

        e.as_contract(&contract_id, || {
            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Pool(usdc_pool_address.clone())),
                SHARED_BUMP
            );

            // Individual persistent storage ttl is still the same
            assert_eq!(
                e.storage()
                    .persistent()
                    .get_ttl(&DataKey::Obligation(user.clone())),
                INDIVIDUAL_BUMP - 2 * LEDGERS_PER_DAY
            );
        });

        // TODO: Add individual storage extension test case
    }
}

#[cfg(test)]
mod borrow;
#[cfg(test)]
mod deposit;
#[cfg(test)]
mod fuzz;
#[cfg(test)]
mod initialize;
#[cfg(test)]
mod interest_rates;
#[cfg(test)]
mod liquidate;
#[cfg(test)]
mod repay;
#[cfg(test)]
mod withdraw;
