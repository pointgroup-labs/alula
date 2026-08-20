mod mock_pool;

use soroban_sdk::{
    Address, Env, IntoVal, Map, Vec,
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    token::{StellarAssetClient, TokenClient},
    vec,
};

use crate::{
    ASPError, AquaSwapProviderContract, AquaSwapProviderContractClient,
    tests::mock_pool::{MockAquaPool, MockAquaPoolClient},
};

const FEE_BPS: u128 = 30;

const POOL_AB_RESERVE_A: i128 = 1_000_000;
const POOL_AB_RESERVE_B: i128 = 100_000;
const POOL_BC_RESERVE_B: i128 = 100_000;
const POOL_BC_RESERVE_C: i128 = 10_000_000;

const USER_BALANCE_A: i128 = 100_000;
const SWAP_AMOUNT: i128 = 10_000;

struct TestFixture<'a> {
    e: Env,
    user: Address,
    token_a: Address,
    token_b: Address,
    token_c: Address,
    pool_ab: Address,
    pool_bc: Address,
    provider_id: Address,
    provider: AquaSwapProviderContractClient<'a>,
}

impl TestFixture<'_> {
    /// A -> B -> C, priced so that a hop through B yields far fewer B than the
    /// C it finally buys. That gap is what separates a final-leg slippage
    /// bound from a per-hop one.
    fn new() -> Self {
        let e = Env::default();
        e.mock_all_auths();

        let admin = Address::generate(&e);
        let user = Address::generate(&e);

        let token_a = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let token_b = e.register_stellar_asset_contract_v2(admin.clone()).address();
        let token_c = e.register_stellar_asset_contract_v2(admin.clone()).address();

        let pool_ab =
            e.register(MockAquaPool, (vec![&e, token_a.clone(), token_b.clone()], FEE_BPS));
        let pool_bc =
            e.register(MockAquaPool, (vec![&e, token_b.clone(), token_c.clone()], FEE_BPS));

        StellarAssetClient::new(&e, &token_a).mint(&pool_ab, &POOL_AB_RESERVE_A);
        StellarAssetClient::new(&e, &token_b).mint(&pool_ab, &POOL_AB_RESERVE_B);
        StellarAssetClient::new(&e, &token_b).mint(&pool_bc, &POOL_BC_RESERVE_B);
        StellarAssetClient::new(&e, &token_c).mint(&pool_bc, &POOL_BC_RESERVE_C);

        // The user holds no B: a chained swap that spends more B than the
        // previous hop produced cannot silently succeed.
        StellarAssetClient::new(&e, &token_a).mint(&user, &USER_BALANCE_A);

        let mut pools = Map::new(&e);
        pools.set((token_a.clone(), token_b.clone()), pool_ab.clone());
        pools.set((token_b.clone(), token_c.clone()), pool_bc.clone());

        let provider_id = e.register(AquaSwapProviderContract, (admin, pools));
        let provider = AquaSwapProviderContractClient::new(&e, &provider_id);

        TestFixture { e, user, token_a, token_b, token_c, pool_ab, pool_bc, provider_id, provider }
    }

    fn balance(&self, token: &Address, who: &Address) -> i128 {
        TokenClient::new(&self.e, token).balance(who)
    }
}

#[test]
fn test_single_hop_swap_exact() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone()];

    let quoted = f.provider.get_amount_out(&path, &SWAP_AMOUNT);
    let received = f.provider.swap_exact(&f.user, &path, &SWAP_AMOUNT, &quoted);

    assert_eq!(received, quoted);
    assert_eq!(f.balance(&f.token_a, &f.user), USER_BALANCE_A - SWAP_AMOUNT);
    assert_eq!(f.balance(&f.token_b, &f.user), received);
}

#[test]
fn test_multi_hop_matches_two_sequential_swaps() {
    let chained = {
        let f = TestFixture::new();
        let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

        f.provider.swap_exact(&f.user, &path, &SWAP_AMOUNT, &0)
    };

    let sequential = {
        let f = TestFixture::new();
        let first = f.provider.swap_exact(
            &f.user,
            &vec![&f.e, f.token_a.clone(), f.token_b.clone()],
            &SWAP_AMOUNT,
            &0,
        );

        f.provider.swap_exact(
            &f.user,
            &vec![&f.e, f.token_b.clone(), f.token_c.clone()],
            &first,
            &0,
        )
    };

    assert_eq!(chained, sequential);
    assert!(chained > 0);
}

#[test]
fn test_multi_hop_leaves_no_intermediate_dust() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

    let received = f.provider.swap_exact(&f.user, &path, &SWAP_AMOUNT, &0);

    assert_eq!(f.balance(&f.token_a, &f.user), USER_BALANCE_A - SWAP_AMOUNT);
    assert_eq!(f.balance(&f.token_b, &f.user), 0);
    assert_eq!(f.balance(&f.token_c, &f.user), received);
}

#[test]
fn test_get_amount_out_folds_across_hops() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

    let quoted = f.provider.get_amount_out(&path, &SWAP_AMOUNT);

    let hop_ab =
        f.provider.get_amount_out(&vec![&f.e, f.token_a.clone(), f.token_b.clone()], &SWAP_AMOUNT);
    let hop_bc =
        f.provider.get_amount_out(&vec![&f.e, f.token_b.clone(), f.token_c.clone()], &hop_ab);

    assert_eq!(quoted, hop_bc);
    assert_eq!(quoted, f.provider.swap_exact(&f.user, &path, &SWAP_AMOUNT, &0));
}

#[test]
fn test_min_amount_out_bounds_the_final_leg_only() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

    let intermediate =
        f.provider.get_amount_out(&vec![&f.e, f.token_a.clone(), f.token_b.clone()], &SWAP_AMOUNT);
    let final_out = f.provider.get_amount_out(&path, &SWAP_AMOUNT);

    // A per-hop bound would reject this: the A -> B leg delivers far less than
    // `min_amount_out`, while the route as a whole clears it.
    let min_amount_out = final_out / 2;
    assert!(min_amount_out > intermediate);

    let received = f.provider.swap_exact(&f.user, &path, &SWAP_AMOUNT, &min_amount_out);

    assert_eq!(received, final_out);
}

#[test]
fn test_min_amount_out_enforced_on_the_route_output() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

    let final_out = f.provider.get_amount_out(&path, &SWAP_AMOUNT);

    assert!(f.provider.try_swap_exact(&f.user, &path, &SWAP_AMOUNT, &(final_out + 1)).is_err());
    assert_eq!(f.balance(&f.token_a, &f.user), USER_BALANCE_A);
    assert_eq!(f.balance(&f.token_c, &f.user), 0);
}

#[test]
fn test_unregistered_intermediate_pair_fails_with_pool_not_found() {
    let f = TestFixture::new();
    let token_d = Address::generate(&f.e);
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), token_d];

    assert_eq!(
        f.provider.try_swap_exact(&f.user, &path, &SWAP_AMOUNT, &0),
        Err(Ok(ASPError::PoolNotFound.into()))
    );
    assert_eq!(f.balance(&f.token_a, &f.user), USER_BALANCE_A);
    assert_eq!(f.balance(&f.token_b, &f.user), 0);
}

#[test]
fn test_short_path_rejected() {
    let f = TestFixture::new();

    assert_eq!(
        f.provider.try_get_amount_out(&vec![&f.e, f.token_a.clone()], &SWAP_AMOUNT),
        Err(Ok(ASPError::InvalidPath.into()))
    );
    assert_eq!(
        f.provider.try_swap_exact(&f.user, &Vec::new(&f.e), &SWAP_AMOUNT, &0),
        Err(Ok(ASPError::InvalidPath.into()))
    );
}

#[test]
fn test_swap_for_exact_single_hop() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone()];
    let amount_out = 500i128;

    let quoted = f.provider.get_amount_in(&path, &amount_out);
    let spent = f.provider.swap_for_exact(&f.user, &path, &quoted, &amount_out);

    assert_eq!(spent, quoted);
    assert_eq!(f.balance(&f.token_b, &f.user), amount_out);
    assert_eq!(f.balance(&f.token_a, &f.user), USER_BALANCE_A - spent);
}

#[test]
fn test_get_amount_in_folds_backwards_across_hops() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];
    let amount_out = 50_000i128;

    let quoted = f.provider.get_amount_in(&path, &amount_out);

    let hop_bc =
        f.provider.get_amount_in(&vec![&f.e, f.token_b.clone(), f.token_c.clone()], &amount_out);
    let hop_ab =
        f.provider.get_amount_in(&vec![&f.e, f.token_a.clone(), f.token_b.clone()], &hop_bc);

    assert_eq!(quoted, hop_ab);
}

#[test]
fn test_multi_hop_swap_for_exact_delivers_exactly_and_leaves_no_dust() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];
    let amount_out = 50_000i128;

    let quoted = f.provider.get_amount_in(&path, &amount_out);
    let spent = f.provider.swap_for_exact(&f.user, &path, &quoted, &amount_out);

    assert_eq!(spent, quoted);
    assert_eq!(f.balance(&f.token_c, &f.user), amount_out);
    assert_eq!(f.balance(&f.token_b, &f.user), 0);
    assert_eq!(f.balance(&f.token_a, &f.user), USER_BALANCE_A - spent);
}

#[test]
fn test_multi_hop_swap_for_exact_enforces_max_amount_in() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];
    let amount_out = 50_000i128;

    let quoted = f.provider.get_amount_in(&path, &amount_out);

    assert!(f.provider.try_swap_for_exact(&f.user, &path, &(quoted - 1), &amount_out).is_err());
    assert_eq!(f.balance(&f.token_a, &f.user), USER_BALANCE_A);
}

/// The route is sized from quotes, so a hop that charges more than it quoted
/// must revert rather than top itself up from the caller's own holding of the
/// intermediate token — which `max_amount_in`, measured on `path[0]`, misses.
#[test]
fn test_multi_hop_swap_for_exact_never_spends_caller_intermediate_balance() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];
    let amount_out = 50_000i128;

    StellarAssetClient::new(&f.e, &f.token_b).mint(&f.user, &10_000);
    MockAquaPoolClient::new(&f.e, &f.pool_bc).set_strict_receive_surcharge(&1);

    let quoted = f.provider.get_amount_in(&path, &amount_out);

    assert!(f.provider.try_swap_for_exact(&f.user, &path, &quoted, &amount_out).is_err());
    assert_eq!(f.balance(&f.token_b, &f.user), 10_000);
}

/// A multi-hop call needs no extra signature from the caller — it stays one
/// `swap_exact` authorization — but its sub-invocation tree must list every
/// hop. Callers that hand-build auth entries instead of taking them from
/// simulation have to widen the tree per hop.
#[test]
fn test_multi_hop_auth_tree_covers_every_hop() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

    let hop_ab =
        f.provider.get_amount_out(&vec![&f.e, f.token_a.clone(), f.token_b.clone()], &SWAP_AMOUNT);

    let received = with_multi_hop_auth_tree(&f, &path, hop_ab, |tree| {
        f.provider.mock_auths(&[MockAuth { address: &f.user, invoke: tree }]).swap_exact(
            &f.user,
            &path,
            &SWAP_AMOUNT,
            &0,
        )
    });

    assert!(received > 0);
}

/// The intermediate hop's `transfer` entry carries the amount the previous hop
/// produced, so a route signed against stale pool state fails as an auth error
/// rather than as slippage. Single-hop entries only ever carry `amount_in`.
#[test]
fn test_auth_tree_intermediate_amount_must_be_exact() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

    let hop_ab =
        f.provider.get_amount_out(&vec![&f.e, f.token_a.clone(), f.token_b.clone()], &SWAP_AMOUNT);

    let result = with_multi_hop_auth_tree(&f, &path, hop_ab + 1, |tree| {
        f.provider.mock_auths(&[MockAuth { address: &f.user, invoke: tree }]).try_swap_exact(
            &f.user,
            &path,
            &SWAP_AMOUNT,
            &0,
        )
    });

    assert!(result.is_err());
}

/// Builds the tree a caller must authorize for `A -> B -> C`, with the second
/// hop sized at `hop_ab`. Borrowed throughout, hence the callback.
fn with_multi_hop_auth_tree<R>(
    f: &TestFixture<'_>,
    path: &Vec<Address>,
    hop_ab: i128,
    run: impl FnOnce(&MockAuthInvoke) -> R,
) -> R {
    let transfer_a = MockAuthInvoke {
        contract: &f.token_a,
        fn_name: "transfer",
        args: (&f.user, &f.pool_ab, SWAP_AMOUNT).into_val(&f.e),
        sub_invokes: &[],
    };
    let transfer_b = MockAuthInvoke {
        contract: &f.token_b,
        fn_name: "transfer",
        args: (&f.user, &f.pool_bc, hop_ab).into_val(&f.e),
        sub_invokes: &[],
    };

    let swap_ab_subs = [transfer_a];
    let swap_ab = MockAuthInvoke {
        contract: &f.pool_ab,
        fn_name: "swap",
        args: (&f.user, 0u32, 1u32, SWAP_AMOUNT as u128, 0u128).into_val(&f.e),
        sub_invokes: &swap_ab_subs,
    };
    let swap_bc_subs = [transfer_b];
    let swap_bc = MockAuthInvoke {
        contract: &f.pool_bc,
        fn_name: "swap",
        args: (&f.user, 0u32, 1u32, hop_ab as u128, 0u128).into_val(&f.e),
        sub_invokes: &swap_bc_subs,
    };

    let both_hops = [swap_ab, swap_bc];
    let full = MockAuthInvoke {
        contract: &f.provider_id,
        fn_name: "swap_exact",
        args: (&f.user, path.clone(), SWAP_AMOUNT, 0i128).into_val(&f.e),
        sub_invokes: &both_hops,
    };

    run(&full)
}

#[test]
fn test_auth_tree_missing_a_hop_is_rejected() {
    let f = TestFixture::new();
    let path = vec![&f.e, f.token_a.clone(), f.token_b.clone(), f.token_c.clone()];

    let transfer_a = MockAuthInvoke {
        contract: &f.token_a,
        fn_name: "transfer",
        args: (&f.user, &f.pool_ab, SWAP_AMOUNT).into_val(&f.e),
        sub_invokes: &[],
    };
    let swap_ab_subs = [transfer_a];
    let swap_ab = MockAuthInvoke {
        contract: &f.pool_ab,
        fn_name: "swap",
        args: (&f.user, 0u32, 1u32, SWAP_AMOUNT as u128, 0u128).into_val(&f.e),
        sub_invokes: &swap_ab_subs,
    };

    let first_hop_only = [swap_ab];
    let partial = MockAuthInvoke {
        contract: &f.provider_id,
        fn_name: "swap_exact",
        args: (&f.user, path.clone(), SWAP_AMOUNT, 0i128).into_val(&f.e),
        sub_invokes: &first_hop_only,
    };

    assert!(
        f.provider
            .mock_auths(&[MockAuth { address: &f.user, invoke: &partial }])
            .try_swap_exact(&f.user, &path, &SWAP_AMOUNT, &0)
            .is_err()
    );
}
