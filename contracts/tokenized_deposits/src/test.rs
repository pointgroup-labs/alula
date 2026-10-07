#![cfg(test)]

use soroban_sdk::{
    Address, Env, Map, String, contract, contractimpl, contracttype,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
};

use crate::{
    contract::{TokenizedDepositsContract, TokenizedDepositsContractClient},
    market::{DepositPosition, Obligation, ObligationKey, OperationFees, WithdrawResult},
};

// -- Mock market --
//
// Models the parts of the real market the vault depends on: per-obligation `jToken` positions, a
// pool-wide rate that rises as interest accrues, health-gated withdrawals, and permissionless
// liquidation of a single obligation. `jTokens` and assets are tracked as separate quantities so
// that rate movements are observable

#[derive(Clone)]
#[contracttype]
pub enum MockKey {
    Asset,
    TotalJTokens,
    TotalAssets,
    JTokens(ObligationKey),
    Debt(ObligationKey),
}

#[contract]
pub struct MockMarket;

#[contractimpl]
impl MockMarket {
    pub fn __constructor(e: Env, asset: Address) {
        e.storage().instance().set(&MockKey::Asset, &asset);
    }

    // -- Test-only controls --

    // Simulates interest: assets grow while `jTokens` stay fixed, so the rate rises. The harness
    // funds the mock separately, since a contract frame cannot authorize the SAC's `mint`
    pub fn accrue(e: Env, amount: i128) {
        Self::set_total_a(&e, Self::total_a(&e) + amount);
    }

    // Seizes a fraction of one obligation's `jTokens`, mutating only that position. This is the
    // market behaviour whose isolation the vault depends on, and it happens with no notification.
    //
    // The seized assets leave the pool with the liquidator, so `total_assets` drops alongside
    // `total_j_tokens` -- otherwise the seized value would be redistributed to other holders as a
    // windfall, which is the opposite of what a liquidation does
    pub fn liquidate_fraction(e: Env, user: ObligationKey, numerator: i128, denominator: i128) {
        let held = Self::j_of(&e, &user);
        let seized = held * numerator / denominator;
        let (total_j, total_a) = (Self::total_j(&e), Self::total_a(&e));
        let seized_value = if total_j == 0 { 0 } else { seized * total_a / total_j };

        Self::set_j(&e, &user, held - seized);
        Self::set_total_j(&e, total_j - seized);
        Self::set_total_a(&e, total_a - seized_value);
    }

    // -- Market interface --

    pub fn deposit(e: Env, user: ObligationKey, _pool: Address, amount: i128, _r: Option<Address>) {
        let asset = Self::asset(&e);
        // Pull the assets from the vault. This is a nested transfer the vault must pre-authorize,
        // so it exercises the invoker-contract auth path
        TokenClient::new(&e, &asset).transfer(
            &user.user,
            soroban_sdk::MuxedAddress::from(&e.current_contract_address()),
            &amount,
        );

        let (total_j, total_a) = (Self::total_j(&e), Self::total_a(&e));
        // The first deposit fixes the rate at 1:1; later ones convert at the prevailing rate
        let minted = if total_j == 0 || total_a == 0 { amount } else { amount * total_j / total_a };

        Self::set_j(&e, &user, Self::j_of(&e, &user) + minted);
        Self::set_total_j(&e, total_j + minted);
        Self::set_total_a(&e, total_a + amount);
    }

    pub fn simulate_withdraw(
        e: Env,
        user: ObligationKey,
        _pool: Address,
        amount: i128,
        _r: Option<Address>,
    ) -> WithdrawResult {
        let (total_j, total_a) = (Self::total_j(&e), Self::total_a(&e));
        let capped = Self::max_withdrawable(&e, &user).min(amount);
        // The vault recovers the pool rate from this ratio, so it must stay exact
        let burn = if total_a == 0 { 0 } else { capped * total_j / total_a };

        WithdrawResult {
            j_tokens_to_burn: burn,
            deposit_decrease: capped,
            withdrawer_to_receive: capped,
            operation_fees: OperationFees { fee_sum: 0, referrer_fee: 0 },
        }
    }

    pub fn withdraw(e: Env, user: ObligationKey, pool: Address, amount: i128, r: Option<Address>) {
        let result = Self::simulate_withdraw(e.clone(), user.clone(), pool, amount, r);
        if result.deposit_decrease <= 0 {
            panic!("nothing withdrawable");
        }

        Self::set_j(&e, &user, Self::j_of(&e, &user) - result.j_tokens_to_burn);
        Self::set_total_j(&e, Self::total_j(&e) - result.j_tokens_to_burn);
        Self::set_total_a(&e, Self::total_a(&e) - result.deposit_decrease);

        // Paid to the obligation's owner, which is the vault
        let asset = Self::asset(&e);
        TokenClient::new(&e, &asset).transfer(
            &e.current_contract_address(),
            soroban_sdk::MuxedAddress::from(&user.user),
            &result.deposit_decrease,
        );
    }

    pub fn borrow(e: Env, user: ObligationKey, _pool: Address, amount: i128, _r: Option<Address>) {
        let debt = Self::debt_of(&e, &user) + amount;
        // Enforce the same 2x collateral floor that withdrawals respect
        if debt * 2 > Self::value_of(&e, &user) {
            panic!("unhealthy borrow");
        }
        Self::set_debt(&e, &user, debt);

        // The harness pre-funds the mock so it can pay out without minting from a contract frame
        let asset = Self::asset(&e);
        TokenClient::new(&e, &asset).transfer(
            &e.current_contract_address(),
            soroban_sdk::MuxedAddress::from(&user.user),
            &amount,
        );
    }

    pub fn repay(e: Env, user: ObligationKey, _pool: Address, amount: i128, _r: Option<Address>) {
        let debt = Self::debt_of(&e, &user);
        let repaid = amount.min(debt);

        let asset = Self::asset(&e);
        TokenClient::new(&e, &asset).transfer(
            &user.user,
            soroban_sdk::MuxedAddress::from(&e.current_contract_address()),
            &repaid,
        );

        Self::set_debt(&e, &user, debt - repaid);
    }

    pub fn get_user_obligation(e: Env, user: ObligationKey) -> Obligation {
        let mut deposits = Map::new(&e);
        deposits.set(
            e.current_contract_address(),
            DepositPosition {
                j_tokens: Self::j_of(&e, &user),
                collateral: 0,
                originally_deposited: 0,
                last_scarcity_withdraw_ts: 0,
            },
        );

        Obligation {
            deposits,
            borrows: Map::new(&e),
            positions_count: 1,
            insurance_fund_requests_ids: Map::new(&e),
        }
    }

    pub fn refresh_pool(_e: Env, _pool: Address) {}

    // -- Helpers --

    // Withdrawable value, capped by health: a position carrying debt may only draw down to twice
    // that debt, mirroring the real market's LTV gate
    fn max_withdrawable(e: &Env, user: &ObligationKey) -> i128 {
        let value = Self::value_of(e, user);
        let debt = Self::debt_of(e, user);
        if debt == 0 {
            return value;
        }

        let floor = debt * 2;

        if value > floor { value - floor } else { 0 }
    }

    fn value_of(e: &Env, user: &ObligationKey) -> i128 {
        let total_j = Self::total_j(e);
        if total_j == 0 {
            return 0;
        }

        Self::j_of(e, user) * Self::total_a(e) / total_j
    }

    fn asset(e: &Env) -> Address {
        e.storage().instance().get(&MockKey::Asset).unwrap()
    }

    fn total_j(e: &Env) -> i128 {
        e.storage().instance().get(&MockKey::TotalJTokens).unwrap_or(0)
    }

    fn set_total_j(e: &Env, v: i128) {
        e.storage().instance().set(&MockKey::TotalJTokens, &v);
    }

    fn total_a(e: &Env) -> i128 {
        e.storage().instance().get(&MockKey::TotalAssets).unwrap_or(0)
    }

    fn set_total_a(e: &Env, v: i128) {
        e.storage().instance().set(&MockKey::TotalAssets, &v);
    }

    fn j_of(e: &Env, key: &ObligationKey) -> i128 {
        e.storage().instance().get(&MockKey::JTokens(key.clone())).unwrap_or(0)
    }

    fn set_j(e: &Env, key: &ObligationKey, v: i128) {
        e.storage().instance().set(&MockKey::JTokens(key.clone()), &v);
    }

    fn debt_of(e: &Env, key: &ObligationKey) -> i128 {
        e.storage().instance().get(&MockKey::Debt(key.clone())).unwrap_or(0)
    }

    fn set_debt(e: &Env, key: &ObligationKey, v: i128) {
        e.storage().instance().set(&MockKey::Debt(key.clone()), &v);
    }
}

// -- Harness --

struct Setup {
    e: Env,
    vault: Address,
    market: Address,
    asset: Address,
}

fn setup() -> Setup {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let sac = e.register_stellar_asset_contract_v2(admin.clone());
    let asset = sac.address();

    let market = e.register(MockMarket, (asset.clone(),));
    // Lendable reserves, so borrows can be paid out without the mock minting from a contract frame.
    // Held outside the mock's `total_assets` accounting, so it does not affect the share rate
    StellarAssetClient::new(&e, &asset).mint(&market, &1_000_000_0000000);
    // The mock keys its deposit positions by its own address, so the pool address must match
    let pool = market.clone();

    let vault = e.register(
        TokenizedDepositsContract,
        (
            admin,
            asset.clone(),
            market.clone(),
            pool,
            String::from_str(&e, "Tokenized USDC"),
            String::from_str(&e, "tUSDC"),
        ),
    );

    Setup { e, vault, market, asset }
}

impl Setup {
    fn client(&self) -> TokenizedDepositsContractClient<'_> {
        TokenizedDepositsContractClient::new(&self.e, &self.vault)
    }

    fn token(&self) -> TokenClient<'_> {
        TokenClient::new(&self.e, &self.asset)
    }

    fn user_with(&self, amount: i128) -> Address {
        let user = Address::generate(&self.e);
        StellarAssetClient::new(&self.e, &self.asset).mint(&user, &amount);

        user
    }

    fn accrue(&self, amount: i128) {
        // Interest is real money: fund the mock so its books stay backed by actual tokens
        StellarAssetClient::new(&self.e, &self.asset).mint(&self.market, &amount);
        MockMarketClient::new(&self.e, &self.market).accrue(&amount);
    }

    fn liquidate(&self, who: &Address, num: i128, den: i128) {
        let key = self.client().obligation_key_of(who);
        MockMarketClient::new(&self.e, &self.market).liquidate_fraction(&key, &num, &den);
    }
}

// -- Tests --

#[test]
fn test_deposit_credits_shares_to_the_receiver() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    let minted = s.client().deposit(&1_000_0000000, &user, &user);

    assert_eq!(minted, 1_000_0000000);
    assert_eq!(s.client().balance(&user), 1_000_0000000);
    assert_eq!(s.token().balance(&user), 0);
}

#[test]
fn test_share_price_grows_with_accrued_interest() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);
    let before = s.client().assets_of(&user);

    // 10% interest accrues to the pool
    s.accrue(100_0000000);
    let after = s.client().assets_of(&user);

    // The share count is unchanged -- only the price moved
    assert_eq!(s.client().balance(&user), 1_000_0000000);
    assert!(after > before, "expected {after} > {before}");
    assert_eq!(after, 1_100_0000000);
}

#[test]
fn test_liquidation_is_isolated_to_the_affected_participant() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &alice, &alice);
    s.client().deposit(&1_000_0000000, &bob, &bob);
    let bob_before = s.client().assets_of(&bob);

    // Half of Alice's collateral is seized
    s.liquidate(&alice, 1, 2);

    assert_eq!(s.client().balance(&alice), 500_0000000);
    // Bob is entirely untouched -- the whole point of per-participant obligations
    assert_eq!(s.client().balance(&bob), 1_000_0000000);
    assert_eq!(s.client().assets_of(&bob), bob_before);
}

#[test]
fn test_balance_reads_through_so_it_cannot_drift() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);
    // A seizure the vault is never notified about
    s.liquidate(&user, 3, 10);

    // A mirrored ledger would still report the pre-seizure figure here
    assert_eq!(s.client().balance(&user), 700_0000000);
}

#[test]
fn test_redeem_pays_out_and_burns_shares() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);
    s.accrue(100_0000000);

    // 500 shares at a 1.1 rate
    let received = s.client().redeem(&500_0000000, &user, &user);

    assert_eq!(received, 550_0000000);
    assert_eq!(s.token().balance(&user), 550_0000000);
    assert_eq!(s.client().balance(&user), 500_0000000);
}

#[test]
fn test_transfer_moves_value_between_participants() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = Address::generate(&s.e);

    s.client().deposit(&1_000_0000000, &alice, &alice);
    s.client().transfer(&alice, &bob, &400_0000000);

    assert_eq!(s.client().balance(&alice), 600_0000000);
    assert_eq!(s.client().balance(&bob), 400_0000000);
}

#[test]
fn test_transfer_lands_in_the_recipients_own_obligation() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = Address::generate(&s.e);

    s.client().deposit(&1_000_0000000, &alice, &alice);
    s.client().transfer(&alice, &bob, &500_0000000);

    // Transferred shares must be liquidation-isolated too, not pooled with the sender's
    s.liquidate(&alice, 1, 1);

    assert_eq!(s.client().balance(&alice), 0);
    assert_eq!(s.client().balance(&bob), 500_0000000);
}

#[test]
fn test_borrow_against_shares_then_repay() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);
    let borrowed = s.client().borrow(&user, &s.market, &s.asset, &300_0000000);

    assert_eq!(borrowed, 300_0000000);
    assert_eq!(s.token().balance(&user), 300_0000000);
    // Borrowing does not consume shares
    assert_eq!(s.client().balance(&user), 1_000_0000000);

    s.client().repay(&user, &s.market, &s.asset, &300_0000000);

    assert_eq!(s.token().balance(&user), 0);
}

#[test]
fn test_transfer_of_encumbered_shares_is_rejected() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = Address::generate(&s.e);

    s.client().deposit(&1_000_0000000, &alice, &alice);
    s.client().borrow(&alice, &s.market, &s.asset, &400_0000000);

    // 400 of debt locks 800 of collateral, leaving 200 movable. Moving 900 must fail rather than
    // hand Bob a position secretly carrying Alice's debt
    assert!(s.client().try_transfer(&alice, &bob, &900_0000000).is_err());

    assert_eq!(s.client().balance(&alice), 1_000_0000000);
    assert_eq!(s.client().balance(&bob), 0);
}

#[test]
fn test_unencumbered_portion_stays_transferable_while_borrowing() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = Address::generate(&s.e);

    s.client().deposit(&1_000_0000000, &alice, &alice);
    s.client().borrow(&alice, &s.market, &s.asset, &400_0000000);

    // 200 remains free above the 2x floor
    s.client().transfer(&alice, &bob, &150_0000000);

    assert_eq!(s.client().balance(&bob), 150_0000000);
}

#[test]
fn test_borrow_beyond_collateral_is_rejected() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);

    assert!(s.client().try_borrow(&user, &s.market, &s.asset, &600_0000000).is_err());
}

#[test]
fn test_conversions_track_the_pool_rate() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);
    s.accrue(100_0000000);

    // At a 1.1 rate the conversions are mutual inverses
    assert_eq!(s.client().convert_to_assets(&100_0000000, &user), 110_0000000);
    assert_eq!(s.client().convert_to_shares(&110_0000000, &user), 100_0000000);
}

#[test]
fn test_deposit_for_another_receiver() {
    let s = setup();
    let payer = s.user_with(1_000_0000000);
    let receiver = Address::generate(&s.e);

    s.client().deposit(&1_000_0000000, &receiver, &payer);

    assert_eq!(s.client().balance(&receiver), 1_000_0000000);
    assert_eq!(s.client().balance(&payer), 0);
}

#[test]
fn test_later_depositor_does_not_dilute_accrued_yield() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &alice, &alice);
    s.accrue(100_0000000);
    // Bob buys in after the accrual, so he pays the higher price
    s.client().deposit(&1_000_0000000, &bob, &bob);

    // Values are within a stroop of the ideal. The rate is recovered from a simulated withdrawal,
    // so both terms of the ratio are themselves rounded -- a sub-stroop artifact is unavoidable
    // until the market exposes its rate directly
    assert!((s.client().assets_of(&alice) - 1_100_0000000).abs() <= 1);
    assert!((s.client().assets_of(&bob) - 1_000_0000000).abs() <= 1);
    // Bob gets fewer shares than Alice for the same money
    assert!(s.client().balance(&bob) < s.client().balance(&alice));
}

#[test]
fn test_redeeming_more_than_held_is_rejected() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);

    assert!(s.client().try_redeem(&2_000_0000000, &user, &user).is_err());
}

#[test]
fn test_transfer_more_than_held_is_rejected() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = Address::generate(&s.e);

    s.client().deposit(&1_000_0000000, &alice, &alice);

    assert!(s.client().try_transfer(&alice, &bob, &2_000_0000000).is_err());
}

#[test]
fn test_non_positive_amounts_are_rejected() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);

    assert!(s.client().try_deposit(&0, &user, &user).is_err());
    assert!(s.client().try_deposit(&-1, &user, &user).is_err());
    assert!(s.client().try_redeem(&0, &user, &user).is_err());
    assert!(s.client().try_redeem(&-1, &user, &user).is_err());
}

#[test]
fn test_paused_deposits_are_rejected_but_redemptions_stay_open() {
    let s = setup();
    let user = s.user_with(2_000_0000000);

    s.client().deposit(&1_000_0000000, &user, &user);
    s.client().set_deposits_paused(&true);

    assert!(s.client().try_deposit(&500_0000000, &user, &user).is_err());
    // A paused vault must never trap funds
    assert!(s.client().try_redeem(&500_0000000, &user, &user).is_ok());

    s.client().set_deposits_paused(&false);
    assert!(s.client().try_deposit(&500_0000000, &user, &user).is_ok());
}

#[test]
fn test_transfer_from_spends_allowance() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = Address::generate(&s.e);
    let spender = Address::generate(&s.e);

    s.client().deposit(&1_000_0000000, &alice, &alice);
    s.client().approve(&alice, &spender, &400_0000000, &(s.e.ledger().sequence() + 1000));
    s.client().transfer_from(&spender, &alice, &bob, &300_0000000);

    assert_eq!(s.client().balance(&bob), 300_0000000);
    assert_eq!(s.client().allowance(&alice, &spender), 100_0000000);
    // The remaining allowance is enforced
    assert!(s.client().try_transfer_from(&spender, &alice, &bob, &200_0000000).is_err());
}

#[test]
fn test_metadata_matches_the_underlying() {
    let s = setup();

    assert_eq!(s.client().name(), String::from_str(&s.e, "Tokenized USDC"));
    assert_eq!(s.client().symbol(), String::from_str(&s.e, "tUSDC"));
    // Shares are jTokens, denominated in the underlying's decimals
    assert_eq!(s.client().decimals(), s.token().decimals());
}

#[test]
fn test_obligation_keys_are_distinct_per_participant() {
    let s = setup();
    let alice = Address::generate(&s.e);
    let bob = Address::generate(&s.e);

    let alice_key = s.client().obligation_key_of(&alice);
    let bob_key = s.client().obligation_key_of(&bob);

    // Same owner (the vault), different seeds -- that is what isolates liquidation
    assert_eq!(alice_key.user, bob_key.user);
    assert_eq!(alice_key.user, s.vault);
    assert_ne!(alice_key.seed, bob_key.seed);
}
