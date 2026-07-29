#![cfg(test)]
#![allow(clippy::inconsistent_digit_grouping)]
extern crate std;

use market::obligation::{Obligation, ObligationKey, OperationFees, WithdrawResult};
use soroban_sdk::{
    Address, Env, Map, MuxedAddress, String, Symbol, contract, contractimpl,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
};

use crate::{
    constants::{MAX_DECIMALS, PREFERRED_DECIMALS_OFFSET},
    contract::{TokenizedDepositsContract, TokenizedDepositsContractClient},
    vault::TokenizedVaultClient,
};

// A stand-in for the lending market.
//
// It reproduces the two behaviours the vault actually depends on: it custodies the deposited
// asset, and the amount it will pay back grows as interest accrues. Interest is injected in tests
// by minting straight into the mock, which mirrors how the real market's holdings grow
#[contract]
pub struct MockMarket;

#[contractimpl]
impl MockMarket {
    pub fn __constructor(e: Env, asset: Address) {
        e.storage().instance().set(&Symbol::new(&e, "asset"), &asset);
    }

    fn asset(e: &Env) -> Address {
        e.storage().instance().get(&Symbol::new(e, "asset")).unwrap()
    }

    fn principal(e: &Env, user: &Address) -> i128 {
        e.storage().instance().get(&user.clone()).unwrap_or(0)
    }

    fn set_principal(e: &Env, user: &Address, amount: i128) {
        e.storage().instance().set(&user.clone(), &amount);
    }

    fn total_principal(e: &Env) -> i128 {
        e.storage().instance().get(&Symbol::new(e, "total")).unwrap_or(0)
    }

    fn set_total_principal(e: &Env, amount: i128) {
        e.storage().instance().set(&Symbol::new(e, "total"), &amount);
    }

    // What a depositor can withdraw: their principal plus their pro-rata share of any surplus
    // the mock holds beyond the outstanding principal
    fn withdrawable(e: &Env, user: &Address) -> i128 {
        let principal = Self::principal(e, user);
        let total = Self::total_principal(e);
        if total == 0 || principal == 0 {
            return 0;
        }

        let held = TokenClient::new(e, &Self::asset(e)).balance(&e.current_contract_address());

        principal + (held - total).max(0) * principal / total
    }

    pub fn deposit(
        e: Env,
        user: ObligationKey,
        _pool_address: Address,
        amount: i128,
        _referrer: Option<Address>,
    ) {
        user.user.require_auth();

        let asset = Self::asset(&e);
        let market = e.current_contract_address();
        TokenClient::new(&e, &asset).transfer(&user.user, MuxedAddress::from(&market), &amount);

        Self::set_principal(&e, &user.user, Self::principal(&e, &user.user) + amount);
        Self::set_total_principal(&e, Self::total_principal(&e) + amount);
    }

    pub fn withdraw(
        e: Env,
        user: ObligationKey,
        _pool_address: Address,
        amount: i128,
        _referrer: Option<Address>,
    ) {
        user.user.require_auth();

        let available = Self::withdrawable(&e, &user.user);
        let paid = amount.min(available);

        let principal = Self::principal(&e, &user.user);
        let principal_consumed = paid.min(principal);

        Self::set_principal(&e, &user.user, principal - principal_consumed);
        Self::set_total_principal(&e, Self::total_principal(&e) - principal_consumed);

        let asset = Self::asset(&e);
        let market = e.current_contract_address();
        TokenClient::new(&e, &asset).transfer(&market, MuxedAddress::from(&user.user), &paid);
    }

    pub fn simulate_withdraw(
        e: Env,
        user: ObligationKey,
        _pool_address: Address,
        amount: i128,
        _referrer: Option<Address>,
    ) -> WithdrawResult {
        let paid = amount.min(Self::withdrawable(&e, &user.user));

        WithdrawResult {
            j_tokens_to_burn: paid,
            deposit_decrease: paid,
            withdrawer_to_receive: paid,
            operation_fees: OperationFees { fee_sum: 0, referrer_fee: 0 },
        }
    }

    pub fn get_user_obligation(e: Env, _user: ObligationKey) -> Obligation {
        Obligation {
            deposits: Map::new(&e),
            borrows: Map::new(&e),
            positions_count: 0,
            insurance_fund_requests_ids: Map::new(&e),
        }
    }

    pub fn refresh_pool(_e: Env, _pool_address: Address) {}
}

struct Setup {
    e: Env,
    admin: Address,
    asset: Address,
    market: Address,
    vault: Address,
}

fn setup() -> Setup {
    let e = Env::default();
    e.mock_all_auths();

    let admin = Address::generate(&e);
    let asset_admin = Address::generate(&e);
    let asset = e.register_stellar_asset_contract_v2(asset_admin).address();

    let market = e.register(MockMarket, (asset.clone(),));
    let pool = Address::generate(&e);

    let vault = e.register(
        TokenizedDepositsContract,
        (
            admin.clone(),
            asset.clone(),
            market.clone(),
            pool,
            String::from_str(&e, "Alula USDC Vault"),
            String::from_str(&e, "avUSDC"),
        ),
    );

    Setup { e, admin, asset, market, vault }
}

impl Setup {
    fn vault_client(&self) -> TokenizedVaultClient<'_> {
        TokenizedVaultClient::new(&self.e, &self.vault)
    }

    fn admin_client(&self) -> TokenizedDepositsContractClient<'_> {
        TokenizedDepositsContractClient::new(&self.e, &self.vault)
    }

    fn shares(&self) -> TokenClient<'_> {
        TokenClient::new(&self.e, &self.vault)
    }

    fn asset_client(&self) -> TokenClient<'_> {
        TokenClient::new(&self.e, &self.asset)
    }

    // Simulates interest accruing in the market
    fn accrue(&self, amount: i128) {
        StellarAssetClient::new(&self.e, &self.asset).mint(&self.market, &amount);
    }

    fn user_with(&self, amount: i128) -> Address {
        let user = Address::generate(&self.e);
        StellarAssetClient::new(&self.e, &self.asset).mint(&user, &amount);
        user
    }
}

// -- Setup & metadata --

#[test]
fn test_metadata() {
    let s = setup();

    // Share decimals are the asset's decimals plus the virtual offset
    assert_eq!(s.shares().decimals(), 7 + PREFERRED_DECIMALS_OFFSET.min(MAX_DECIMALS - 7));
    assert_eq!(s.shares().symbol(), String::from_str(&s.e, "avUSDC"));
    assert_eq!(s.vault_client().query_asset(), s.asset);
    assert_eq!(s.admin_client().admin(), s.admin);
    assert_eq!(s.vault_client().total_assets(), 0);
}

// -- The core property: share price rises, share balance does not --

#[test]
fn test_share_balance_is_fixed_while_value_grows() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    let shares = s.vault_client().deposit(&1_000_0000000, &user, &user, &user);

    assert_eq!(s.shares().balance(&user), shares);
    assert_eq!(s.vault_client().convert_to_assets(&shares), 1_000_0000000);

    // 10% interest accrues in the market
    s.accrue(100_0000000);

    // The share balance is untouched -- this is the whole point of the vault model
    assert_eq!(s.shares().balance(&user), shares);
    // ...but each share is now worth more. Conversions floor, so allow a single stroop
    assert!((s.vault_client().convert_to_assets(&shares) - 1_100_0000000).abs() <= 1);
    assert_eq!(s.vault_client().total_assets(), 1_100_0000000);
}

#[test]
fn test_late_depositor_pays_higher_share_price() {
    let s = setup();
    let early = s.user_with(1_000_0000000);
    let late = s.user_with(1_000_0000000);

    let early_shares = s.vault_client().deposit(&1_000_0000000, &early, &early, &early);

    // Share price doubles
    s.accrue(1_000_0000000);

    let late_shares = s.vault_client().deposit(&1_000_0000000, &late, &late, &late);

    // The late depositor gets roughly half the shares for the same assets. The virtual offset
    // perturbs the ratio very slightly, so compare proportionally rather than exactly
    assert!(late_shares < early_shares);
    assert!(
        (late_shares - early_shares / 2).abs() * 1_000_000 < early_shares,
        "late {} vs early/2 {}",
        late_shares,
        early_shares / 2
    );

    // Both are worth what was put in; the early depositor keeps their gain
    assert!((s.vault_client().convert_to_assets(&late_shares) - 1_000_0000000).abs() <= 1);
    assert!((s.vault_client().convert_to_assets(&early_shares) - 2_000_0000000).abs() <= 1);
}

#[test]
fn test_yield_is_not_diluted_by_later_deposits() {
    let s = setup();
    let early = s.user_with(1_000_0000000);
    let late = s.user_with(5_000_0000000);

    let early_shares = s.vault_client().deposit(&1_000_0000000, &early, &early, &early);
    s.accrue(500_0000000);

    let before = s.vault_client().convert_to_assets(&early_shares);
    s.vault_client().deposit(&5_000_0000000, &late, &late, &late);
    let after = s.vault_client().convert_to_assets(&early_shares);

    // A large late deposit must not move an existing holder's claim
    assert!((after - before).abs() <= 1, "before {} after {}", before, after);
}

// -- Round trips --

#[test]
fn test_deposit_then_redeem_returns_principal_plus_yield() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    let shares = s.vault_client().deposit(&1_000_0000000, &user, &user, &user);
    assert_eq!(s.asset_client().balance(&user), 0);

    s.accrue(250_0000000);

    let assets = s.vault_client().redeem(&shares, &user, &user, &user);

    assert!((assets - 1_250_0000000).abs() <= 1);
    assert_eq!(s.asset_client().balance(&user), assets);
    assert_eq!(s.shares().balance(&user), 0);
}

#[test]
fn test_withdraw_burns_the_right_share_count() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    let shares = s.vault_client().deposit(&1_000_0000000, &user, &user, &user);
    s.accrue(1_000_0000000); // price doubles

    // Withdrawing half the value should cost roughly a quarter of the shares
    let burned = s.vault_client().withdraw(&500_0000000, &user, &user, &user);

    assert_eq!(s.asset_client().balance(&user), 500_0000000);
    assert!(
        (burned - shares / 4).abs() * 1_000_000 < shares,
        "burned {} vs shares/4 {}",
        burned,
        shares / 4
    );
    assert_eq!(s.shares().balance(&user), shares - burned);
}

#[test]
fn test_mint_charges_the_previewed_amount() {
    let s = setup();
    let user = s.user_with(2_000_0000000);

    s.vault_client().deposit(&1_000_0000000, &user, &user, &user);
    s.accrue(500_0000000);

    let target_shares = 100_0000000;
    let quoted = s.vault_client().preview_mint(&target_shares);

    let before = s.asset_client().balance(&user);
    let paid = s.vault_client().mint(&target_shares, &user, &user, &user);

    assert_eq!(paid, quoted);
    assert_eq!(before - s.asset_client().balance(&user), paid);
}

// -- Preview/actual agreement, as required by the standard --

#[test]
fn test_previews_match_actual_results() {
    let s = setup();
    let user = s.user_with(3_000_0000000);

    s.vault_client().deposit(&1_000_0000000, &user, &user, &user);
    // A rate that does not divide evenly, to exercise rounding
    s.accrue(333_3333333);

    let predicted = s.vault_client().preview_deposit(&500_0000000);
    let actual = s.vault_client().deposit(&500_0000000, &user, &user, &user);
    assert_eq!(predicted, actual);

    let predicted = s.vault_client().preview_withdraw(&200_0000000);
    let actual = s.vault_client().withdraw(&200_0000000, &user, &user, &user);
    assert_eq!(predicted, actual);

    // Redeem a share amount large enough to be worth a non-zero number of assets. Shares carry
    // the virtual offset's extra decimals, so they are much finer-grained than the underlying
    let some_shares = s.shares().balance(&user) / 10;
    let predicted = s.vault_client().preview_redeem(&some_shares);
    let actual = s.vault_client().redeem(&some_shares, &user, &user, &user);
    assert_eq!(predicted, actual);
}

// -- Rounding direction: the invariant that protects existing holders --

#[test]
fn test_round_trip_never_profits_the_user() {
    let s = setup();
    let victim = s.user_with(10_000_0000000);
    let other = s.user_with(1_000_0000000);

    s.vault_client().deposit(&1_000_0000000, &other, &other, &other);
    s.accrue(777_7777777);

    let before = s.asset_client().balance(&victim);

    // Deposit and immediately redeem, repeatedly. Rounding must never pay out more than went in
    for _ in 0..25 {
        let shares = s.vault_client().deposit(&13_0000001, &victim, &victim, &victim);
        s.vault_client().redeem(&shares, &victim, &victim, &victim);
    }

    let after = s.asset_client().balance(&victim);
    assert!(after <= before, "user gained {} through rounding", after - before);
}

#[test]
fn test_inflation_attack_is_not_profitable() {
    let s = setup();
    let attacker = s.user_with(10_000_0000000);
    let victim = s.user_with(1_000_0000000);

    // Classic setup: attacker takes a minimal position, then donates to inflate the share price
    let attacker_shares = s.vault_client().deposit(&1, &attacker, &attacker, &attacker);
    s.accrue(5_000_0000000); // the "donation"

    // The victim must still receive a non-zero, fairly priced number of shares
    let victim_shares = s.vault_client().deposit(&1_000_0000000, &victim, &victim, &victim);
    assert!(victim_shares > 0, "victim was rounded down to zero shares");

    // The victim keeps essentially all of their deposit
    let victim_value = s.vault_client().convert_to_assets(&victim_shares);
    assert!(victim_value > 999_0000000, "victim only kept {}", victim_value);

    // And the attacker cannot capture the victim's deposit on top of their own donation
    let attacker_value = s.vault_client().convert_to_assets(&attacker_shares);
    assert!(
        attacker_value < 5_000_0000000 + 1,
        "attacker captured {} on a 5000 donation",
        attacker_value
    );
}

// -- Share transferability --

#[test]
fn test_shares_are_transferable_and_carry_their_value() {
    let s = setup();
    let alice = s.user_with(1_000_0000000);
    let bob = Address::generate(&s.e);

    let shares = s.vault_client().deposit(&1_000_0000000, &alice, &alice, &alice);
    s.accrue(500_0000000);

    let half = shares / 2;
    s.shares().transfer(&alice, MuxedAddress::from(&bob), &half);

    assert_eq!(s.shares().balance(&alice), shares - half);
    assert_eq!(s.shares().balance(&bob), half);

    // Bob can redeem the value that came with the shares, without ever having deposited
    let assets = s.vault_client().redeem(&half, &bob, &bob, &bob);
    assert!((assets - 750_0000000).abs() <= 1);
    assert_eq!(s.asset_client().balance(&bob), assets);
}

#[test]
fn test_operator_needs_share_allowance_to_redeem() {
    let s = setup();
    let owner = s.user_with(1_000_0000000);
    let operator = Address::generate(&s.e);

    let shares = s.vault_client().deposit(&1_000_0000000, &owner, &owner, &owner);

    // Without an allowance the operator cannot move the owner's shares
    assert!(s.vault_client().try_redeem(&shares, &operator, &owner, &operator).is_err());

    s.shares().approve(&owner, &operator, &shares, &(s.e.ledger().sequence() + 1_000));
    let assets = s.vault_client().redeem(&shares, &operator, &owner, &operator);

    assert_eq!(s.asset_client().balance(&operator), assets);
    assert_eq!(s.shares().balance(&owner), 0);
}

// -- Limits --

#[test]
fn test_max_withdraw_is_bounded_by_holdings() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.vault_client().deposit(&1_000_0000000, &user, &user, &user);
    s.accrue(100_0000000);

    assert!((s.vault_client().max_withdraw(&user) - 1_100_0000000).abs() <= 1);

    // Someone with no shares can withdraw nothing
    let stranger = Address::generate(&s.e);
    assert_eq!(s.vault_client().max_withdraw(&stranger), 0);
}

#[test]
fn test_max_redeem_equals_balance_when_liquidity_suffices() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    let shares = s.vault_client().deposit(&1_000_0000000, &user, &user, &user);

    assert_eq!(s.vault_client().max_redeem(&user), shares);
}

#[test]
fn test_withdrawing_more_than_owned_fails() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.vault_client().deposit(&1_000_0000000, &user, &user, &user);

    assert!(s.vault_client().try_withdraw(&2_000_0000000, &user, &user, &user).is_err());
}

// -- Administration --

#[test]
fn test_pause_blocks_deposits_but_never_withdrawals() {
    let s = setup();
    let user = s.user_with(2_000_0000000);

    let shares = s.vault_client().deposit(&1_000_0000000, &user, &user, &user);

    s.admin_client().set_deposits_paused(&true);

    assert_eq!(s.vault_client().max_deposit(&user), 0);
    assert!(s.vault_client().try_deposit(&100_0000000, &user, &user, &user).is_err());

    // Funds must never be trapped by a pause
    let assets = s.vault_client().redeem(&shares, &user, &user, &user);
    assert!((assets - 1_000_0000000).abs() <= 1);

    s.admin_client().set_deposits_paused(&false);
    assert!(s.vault_client().try_deposit(&100_0000000, &user, &user, &user).is_ok());
}

#[test]
fn test_two_step_admin_rotation() {
    let s = setup();
    let new_admin = Address::generate(&s.e);

    s.admin_client().propose_new_admin(&new_admin);
    // Not effective until claimed
    assert_eq!(s.admin_client().admin(), s.admin);

    s.admin_client().accept_proposed_admin();
    assert_eq!(s.admin_client().admin(), new_admin);

    assert!(s.admin_client().try_accept_proposed_admin().is_err());
}

#[test]
fn test_admin_holds_no_shares_and_cannot_mint() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    let shares = s.vault_client().deposit(&1_000_0000000, &user, &user, &user);

    // The admin has no mint, burn or transfer authority over shares. The only way to extract
    // value is to hold shares, which the admin does not
    assert_eq!(s.shares().balance(&s.admin), 0);
    assert_eq!(s.shares().balance(&user), shares);
}

// -- Aggregate consistency --

#[test]
fn test_sum_of_claims_never_exceeds_total_assets() {
    let s = setup();

    let mut users = std::vec::Vec::new();
    for i in 1..=5i128 {
        let u = s.user_with(1_000_0000000 * i);
        s.vault_client().deposit(&(1_000_0000000 * i), &u, &u, &u);
        s.accrue(37_0000001);
        users.push(u);
    }

    let claims: i128 =
        users.iter().map(|u| s.vault_client().convert_to_assets(&s.shares().balance(u))).sum();

    let total = s.vault_client().total_assets();

    // Solvency: the vault must always be able to honor every claim it has issued
    assert!(claims <= total, "claims {} exceed assets {}", claims, total);
}

#[test]
fn test_redeeming_everyone_drains_the_vault_without_shortfall() {
    let s = setup();

    let mut users = std::vec::Vec::new();
    for i in 1..=4i128 {
        let u = s.user_with(500_0000000 * i);
        s.vault_client().deposit(&(500_0000000 * i), &u, &u, &u);
        users.push(u);
    }

    s.accrue(613_0000007);

    // Every holder must be able to exit; the last one out must not find the vault short
    for u in &users {
        let shares = s.shares().balance(u);
        let assets = s.vault_client().redeem(&shares, u, u, u);
        assert!(assets > 0);
    }

    assert_eq!(s.vault_client().total_supply(), 0);
}

#[test]
fn test_dust_redemption_is_rejected_rather_than_paying_zero() {
    let s = setup();
    let user = s.user_with(1_000_0000000);

    s.vault_client().deposit(&1_000_0000000, &user, &user, &user);

    // Shares are finer-grained than the underlying by the virtual offset, so a small share
    // amount is worth less than one stroop. Silently accepting it would burn the shares and pay
    // out nothing
    assert_eq!(s.vault_client().preview_redeem(&1), 0);
    assert!(s.vault_client().try_redeem(&1, &user, &user, &user).is_err());

    // The holder's position is untouched by the rejected attempt
    assert_eq!(s.vault_client().max_redeem(&user), s.shares().balance(&user));
}

#[test]
fn test_smallest_deposit_still_mints_shares_after_heavy_accrual() {
    let s = setup();
    // Fund one stroop above the initial deposit so the dust deposit below is affordable
    let user = s.user_with(1_000_0000001);

    s.vault_client().deposit(&1_000_0000000, &user, &user, &user);
    // Inflate the share price by five orders of magnitude
    s.accrue(100_000_000_0000000);

    // The virtual offset makes shares far finer-grained than the underlying, so even a
    // single stroop deposited against a hugely appreciated share price still buys shares.
    // Without the offset this would round to zero and silently donate the assets
    assert!(s.vault_client().preview_deposit(&1) > 0);

    let before = s.shares().balance(&user);
    s.vault_client().deposit(&1, &user, &user, &user);
    assert!(s.shares().balance(&user) > before);
}
