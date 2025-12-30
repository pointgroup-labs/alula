//! Integration tests for the Farms contract
//!
//! Tests cover:
//! - Farm initialization
//! - Reward token setup
//! - Stake/unstake operations
//! - Reward harvesting
//! - Warmup/cooldown periods
//! - Early withdrawal penalties
//! - Delegated stake updates (push model)
//! - Edge cases and error conditions

#![cfg(test)]

use farms::{
    Delegatee, FarmConfig, FarmConfigUpdate, FarmsClient, FarmsContract, LockingMode,
    RewardCurvePoint, RewardScheduleCurve, TimeUnit,
};
use soroban_sdk::{
    Address, IntoVal,
    testutils::Address as _,
    token::{StellarAssetClient, TokenClient},
    vec,
};

use crate::TestMarketFixture;

/// Creates a test fixture that includes both Market and Farms contracts
pub struct TestFarmsFixture<'a> {
    pub market_fixture: TestMarketFixture<'a>,
    pub farms_client: FarmsClient<'a>,
    pub farms_address: Address,
    pub reward_token_client: TokenClient<'a>,
    pub reward_token_address: Address,
    pub reward_vault: Address,
}

impl<'a> TestFarmsFixture<'a> {
    pub fn new() -> Self {
        let market_fixture = TestMarketFixture::new();
        let e = &market_fixture.e;

        // Register Farms contract (constructor now takes: admin, treasury_vault)
        let farms_address = e.register(
            FarmsContract,
            (
                &market_fixture.contract_admin,
                &market_fixture.contract_admin, // Treasury vault = admin for simplicity
            ),
        );
        let farms_client = FarmsClient::new(e, &farms_address);

        // Create a reward token (using USDC for simplicity)
        let reward_admin = Address::generate(e);
        let reward_token_address =
            e.register_stellar_asset_contract_v2(reward_admin.clone()).address();
        let reward_sac = StellarAssetClient::new(e, &reward_token_address);
        let reward_token_client = TokenClient::new(e, &reward_token_address);

        // Create a reward vault
        let reward_vault = Address::generate(e);

        // Mint reward tokens to admin for distribution
        reward_sac.mint(&market_fixture.contract_admin, &1_000_000_000_000);
        reward_sac.mint(&reward_vault, &1_000_000_000_000);

        Self {
            market_fixture,
            farms_client,
            farms_address,
            reward_token_client,
            reward_token_address,
            reward_vault,
        }
    }

    pub fn pass_time(&self, seconds: u64) {
        self.market_fixture.pass_time(seconds);
    }

    pub fn current_timestamp(&self) -> u64 {
        self.market_fixture.e.ledger().timestamp()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Initialization Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_farms_initialization() {
    let fixture = TestFarmsFixture::new();

    let config = fixture.farms_client.get_global_config();

    assert_eq!(config.admin, fixture.market_fixture.contract_admin);
    assert_eq!(config.treasury_fee_bps, 0);
}

#[test]
fn test_create_farm_for_pool() {
    let fixture = TestFarmsFixture::new();
    let _e = &fixture.market_fixture.e;

    let farm_config = FarmConfig {
        delegate_authority: None,
        time_unit: TimeUnit::Seconds,
        deposit_warmup_period: 0,
        withdrawal_cooldown_period: 0,
        locking_mode: LockingMode::None,
        locking_start_ts: 0,
        locking_duration: 0,
        early_withdrawal_penalty_bps: 0,
        deposit_cap: 0,
    };

    // Create a farm
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Verify farm was created
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 0);
    assert_eq!(farm.num_users, 0);
    assert!(!farm.is_frozen);
    assert!(farm.delegate_authority.is_none());
}

#[test]
fn test_initialize_reward_token() {
    let fixture = TestFarmsFixture::new();

    let farm_config = FarmConfig::default();

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Add a reward token
    let reward_index = fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    assert_eq!(reward_index, 0);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_reward_tokens, 1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Staking Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_stake_and_unstake() {
    let fixture = TestFarmsFixture::new();
    let _e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create farm
    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Initialize user
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);

    // Stake
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 1000);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 1000);

    // Unstake
    let net_amount = fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &500);
    assert_eq!(net_amount, 500); // No penalty

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 500);
}

#[test]
fn test_stake_with_warmup() {
    let fixture = TestFarmsFixture::new();
    let _e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create farm with 1 hour warmup
    let farm_config = FarmConfig {
        deposit_warmup_period: 3600, // 1 hour
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Stake should be pending
    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 0);
    assert_eq!(user_state.pending_deposit_stake, 1000);

    // Pass warmup time
    fixture.pass_time(3601);

    // Refresh to activate pending stake
    fixture.farms_client.refresh_user_state(&Delegatee::from(user.clone()), &farm_id);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 1000);
    assert_eq!(user_state.pending_deposit_stake, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Reward Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_reward_accrual_and_harvest() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create farm
    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Add reward token
    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Set reward schedule: 100 tokens per second starting now
    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 100 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    // Fund rewards
    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    // User stakes
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Pass 100 seconds
    fixture.pass_time(100);

    // Check pending rewards (should be ~10,000 = 100 * 100 seconds)
    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    assert!(!pending.is_empty());

    // Harvest
    let initial_balance = fixture.reward_token_client.balance(user);
    let harvested = fixture.farms_client.harvest(&Delegatee::from(user.clone()), &farm_id, &0);
    let final_balance = fixture.reward_token_client.balance(user);

    assert!(harvested > 0);
    assert_eq!(final_balance - initial_balance, harvested);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Penalty Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_early_withdrawal_penalty() {
    let fixture = TestFarmsFixture::new();
    let _e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create farm with continuous locking and 10% penalty
    let farm_config = FarmConfig {
        locking_mode: LockingMode::Continuous,
        locking_duration: 86400,            // 1 day
        early_withdrawal_penalty_bps: 1000, // 10%
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Unstake immediately (should incur 10% penalty)
    let net_amount = fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1000);
    assert_eq!(net_amount, 900); // 1000 - 10% = 900
}

#[test]
fn test_no_penalty_after_lock_expires() {
    let fixture = TestFarmsFixture::new();
    let _e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig {
        locking_mode: LockingMode::Continuous,
        locking_duration: 3600,             // 1 hour
        early_withdrawal_penalty_bps: 1000, // 10%
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Pass lock duration
    fixture.pass_time(3601);

    // Unstake after lock expires (no penalty)
    let net_amount = fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1000);
    assert_eq!(net_amount, 1000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Admin Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_freeze_and_unfreeze_farm() {
    let fixture = TestFarmsFixture::new();

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Freeze
    fixture.farms_client.freeze_farm(&farm_id);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert!(farm.is_frozen);

    // Unfreeze
    fixture.farms_client.unfreeze_farm(&farm_id);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert!(!farm.is_frozen);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Delegated Stake Tests (Push Model)
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_delegated_farm_with_set_stake() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create a delegated farm with Market as the delegate authority
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Verify delegate is set
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert!(farm.delegate_authority.is_some());
    assert_eq!(farm.delegate_authority.clone().unwrap(), fixture.market_fixture.contract_id);

    // Mock the Market contract calling set_stake_delegated
    // In real usage, the Market would call this after a deposit
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);

    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Verify user state was updated (auto-initialized)
    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 1000);

    // Verify farm total was updated
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 1000);
    assert_eq!(farm.num_users, 1);
}

#[test]
fn test_delegated_farm_stake_increase_and_decrease() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create a delegated farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Initial stake
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 1000);

    // Increase stake (e.g., user deposited more)
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1500i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1500);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 1500);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 1500);

    // Decrease stake (e.g., user withdrew)
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                500i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &500);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 500);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 500);
}

#[test]
fn test_delegated_farm_user_count_tracking() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create a delegated farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // First stake should increment user count
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);

    // Full unstake should decrement user count
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                0i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &0);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 0);
}

#[test]
fn test_delegated_farm_deposit_cap() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create a delegated farm with deposit cap
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        deposit_cap: 500, // Cap at 500
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Stake within cap should succeed
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                400i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &400);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 400);

    // Stake exceeding cap should fail
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                600i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);

    let result = fixture.farms_client.try_set_stake_delegated(
        &Delegatee::from(user.clone()),
        &farm_id,
        &600,
    );
    assert!(result.is_err());
}

#[test]
fn test_delegated_farm_frozen_rejects_stake() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create a delegated farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Freeze the farm
    fixture.farms_client.freeze_farm(&farm_id);

    // Attempt to stake on frozen farm should fail
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);

    let result = fixture.farms_client.try_set_stake_delegated(
        &Delegatee::from(user.clone()),
        &farm_id,
        &1000,
    );
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cooldown Period Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_unstake_with_cooldown() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // Create farm with 1 hour cooldown
    let farm_config = FarmConfig {
        withdrawal_cooldown_period: 3600, // 1 hour
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Unstake - should move to pending
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &500);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 500);
    assert_eq!(user_state.pending_withdrawal_stake, 500);

    // Attempt withdraw before cooldown should fail
    let result =
        fixture.farms_client.try_withdraw_unstaked(&Delegatee::from(user.clone()), &farm_id);
    assert!(result.is_err());

    // Pass cooldown time
    fixture.pass_time(3601);

    // Withdraw should succeed now
    let withdrawn =
        fixture.farms_client.withdraw_unstaked(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(withdrawn, 500);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.pending_withdrawal_stake, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Multiple Reward Tokens Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_multiple_reward_tokens() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create farm
    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Create second reward token
    let reward_admin2 = Address::generate(e);
    let reward_token2 = e.register_stellar_asset_contract_v2(reward_admin2.clone()).address();
    let reward_sac2 = StellarAssetClient::new(e, &reward_token2);
    let reward_vault2 = Address::generate(e);
    reward_sac2.mint(&fixture.market_fixture.contract_admin, &1_000_000_000);
    reward_sac2.mint(&reward_vault2, &1_000_000_000);

    // Add two reward tokens
    let idx1 = fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );
    let idx2 = fixture.farms_client.initialize_reward(&farm_id, &reward_token2, &reward_vault2);

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_reward_tokens, 2);

    // Set schedules for both
    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 100 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);
    fixture.farms_client.update_reward_schedule(&farm_id, &1, &schedule);

    // Fund both
    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );
    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &1,
        &1_000_000,
    );

    // User stakes and earns from both
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    fixture.pass_time(100);

    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(pending.len(), 2);
    assert!(pending.get(0).unwrap() > 0);
    assert!(pending.get(1).unwrap() > 0);
}

#[test]
fn test_harvest_all() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create farm with reward
    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);
    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 100 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);
    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    fixture.pass_time(100);

    let initial_balance = fixture.reward_token_client.balance(user);
    let total_harvested =
        fixture.farms_client.harvest_all(&Delegatee::from(user.clone()), &farm_id);
    let final_balance = fixture.reward_token_client.balance(user);

    assert!(total_harvested > 0);
    assert_eq!(final_balance - initial_balance, total_harvested);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Linear Penalty Decay Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_linear_penalty_decay_halfway() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // 10% max penalty, 1000 second lock
    let farm_config = FarmConfig {
        locking_mode: LockingMode::Continuous,
        locking_duration: 1000,
        early_withdrawal_penalty_bps: 1000, // 10%
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Pass 500 seconds (halfway) - should have 5% penalty
    fixture.pass_time(500);

    let net_amount = fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1000);
    // 5% of 1000 = 50, so net = 950
    assert_eq!(net_amount, 950);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Per-Farm Admin Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_per_farm_admin_transfer() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Create a new farm admin
    let new_farm_admin = Address::generate(e);

    // Set pending farm admin
    fixture
        .farms_client
        .update_farm_config(&farm_id, &FarmConfigUpdate::PendingFarmAdmin(new_farm_admin.clone()));

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert!(farm.pending_farm_admin.is_some());
    assert_eq!(farm.pending_farm_admin.clone().unwrap(), new_farm_admin);

    // Accept farm admin
    fixture.farms_client.accept_farm_admin(&farm_id);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert!(farm.farm_admin.is_some());
    assert_eq!(farm.farm_admin.clone().unwrap(), new_farm_admin);
    assert!(farm.pending_farm_admin.is_none());
}

#[test]
fn test_farm_admin_can_update_config() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Transfer admin to new address
    let new_farm_admin = Address::generate(e);
    fixture
        .farms_client
        .update_farm_config(&farm_id, &FarmConfigUpdate::PendingFarmAdmin(new_farm_admin.clone()));
    fixture.farms_client.accept_farm_admin(&farm_id);

    // New farm admin should be able to update config
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &new_farm_admin,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "update_farm_config",
            args: soroban_sdk::vec![
                e,
                farm_id.clone().into_val(e),
                FarmConfigUpdate::DepositCap(5000i128).into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);

    fixture.farms_client.update_farm_config(&farm_id, &FarmConfigUpdate::DepositCap(5000));

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.deposit_cap, 5000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Reward User Once (Airdrop) Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_reward_user_once_airdrop() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create delegated farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Enable reward_user_once
    fixture
        .farms_client
        .update_farm_config(&farm_id, &FarmConfigUpdate::RewardUserOnceEnabled(true));

    // Add reward token
    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Fund rewards
    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    // Initialize user (via delegated stake)
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Airdrop reward to user via delegate
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "reward_user_once",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                0u32.into_val(e),
                500i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.reward_user_once(&Delegatee::from(user.clone()), &farm_id, &0, &500);

    // Check user has pending rewards
    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    assert!(pending.get(0).unwrap() >= 500); // At least the airdrop amount

    // Harvest - use mock_all_auths_allowing_non_root_auth to auto-approve all authorization
    // This is needed because the vault's transfer auth isn't rooted in the harvest call
    e.mock_all_auths_allowing_non_root_auth();
    let initial_balance = fixture.reward_token_client.balance(user);
    let harvested = fixture.farms_client.harvest(&Delegatee::from(user.clone()), &farm_id, &0);
    let final_balance = fixture.reward_token_client.balance(user);

    assert!(harvested >= 500);
    assert_eq!(final_balance - initial_balance, harvested);
}

#[test]
fn test_reward_user_once_disabled_fails() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Create delegated farm (but don't enable reward_user_once)
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);
    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Initialize user
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Attempt airdrop should fail
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "reward_user_once",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                0u32.into_val(e),
                500i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);

    let result = fixture.farms_client.try_reward_user_once(
        &Delegatee::from(user.clone()),
        &farm_id,
        &0,
        &500,
    );
    assert!(result.is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Update Farm Config Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_update_farm_config_variants() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Test DepositWarmupPeriod
    fixture.farms_client.update_farm_config(&farm_id, &FarmConfigUpdate::DepositWarmupPeriod(3600));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.deposit_warmup_period, 3600);

    // Test WithdrawalCooldownPeriod
    fixture
        .farms_client
        .update_farm_config(&farm_id, &FarmConfigUpdate::WithdrawalCooldownPeriod(7200));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.withdrawal_cooldown_period, 7200);

    // Test LockingMode
    fixture
        .farms_client
        .update_farm_config(&farm_id, &FarmConfigUpdate::LockingMode(LockingMode::Continuous));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.locking_mode, LockingMode::Continuous);

    // Test LockingDuration
    fixture.farms_client.update_farm_config(&farm_id, &FarmConfigUpdate::LockingDuration(86400));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.locking_duration, 86400);

    // Test EarlyWithdrawalPenalty
    fixture
        .farms_client
        .update_farm_config(&farm_id, &FarmConfigUpdate::EarlyWithdrawalPenalty(500));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.early_withdrawal_penalty_bps, 500);

    // Test DepositCap
    fixture.farms_client.update_farm_config(&farm_id, &FarmConfigUpdate::DepositCap(1_000_000));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.deposit_cap, 1_000_000);

    // Test DelegateAuthority
    let delegate = Address::generate(e);
    fixture
        .farms_client
        .update_farm_config(&farm_id, &FarmConfigUpdate::DelegateAuthority(Some(delegate.clone())));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.delegate_authority.unwrap(), delegate);

    // Clear delegate
    fixture.farms_client.update_farm_config(&farm_id, &FarmConfigUpdate::DelegateAuthority(None));
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert!(farm.delegate_authority.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Withdraw Slashed Amount Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_withdraw_slashed_amount() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // Create locked farm
    let farm_config = FarmConfig {
        locking_mode: LockingMode::Continuous,
        locking_duration: 86400,
        early_withdrawal_penalty_bps: 1000, // 10%
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Unstake immediately to generate slashed amount
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1000);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.slashed_amount_current, 100); // 10% of 1000
    assert_eq!(farm.slashed_amount_cumulative, 100);

    // Admin withdraws slashed amount
    fixture.farms_client.withdraw_slashed_amount(&farm_id, &50);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.slashed_amount_current, 50);
    assert_eq!(farm.slashed_amount_cumulative, 100); // Cumulative unchanged
}

// ═══════════════════════════════════════════════════════════════════════════════
// Global Admin Transfer Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_global_admin_two_step_transfer() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;

    let new_admin = Address::generate(e);

    // Set pending admin
    fixture.farms_client.set_pending_admin(&new_admin);

    let config = fixture.farms_client.get_global_config();
    assert!(config.pending_admin.is_some());
    assert_eq!(config.pending_admin.unwrap(), new_admin);

    // Accept admin
    fixture.farms_client.accept_admin();

    let config = fixture.farms_client.get_global_config();
    assert_eq!(config.admin, new_admin);
    assert!(config.pending_admin.is_none());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Zero/Boundary Values
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_stake_minimum_amount() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);

    // Stake minimum amount (1)
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.active_stake, 1);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.total_staked, 1);
    assert_eq!(farm.num_users, 1);
}

#[test]
#[should_panic(expected = "#70")] // InvalidAmount
fn test_stake_zero_fails() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);

    // Attempt to stake 0 should fail
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &0);
}

#[test]
fn test_harvest_with_zero_pending_rewards() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // User stakes but no rewards were funded
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    fixture.pass_time(100);

    // Harvest should fail with NoRewardsToHarvest (no schedule, no rewards)
    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(pending.get(0).unwrap(), 0);
}

#[test]
fn test_rewards_accrue_with_empty_farm_then_user_joins() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Set reward schedule BEFORE anyone stakes
    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 100 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    // Fund rewards
    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    // Let 100 seconds pass with NO stakers - rewards should NOT be issued
    fixture.pass_time(100);

    // Now user joins
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Let another 100 seconds pass
    fixture.pass_time(100);

    // User should only get rewards from the time they staked (100 seconds * 100 = 10,000)
    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    let pending_amount = pending.get(0).unwrap();

    // Should be approximately 10,000 (not 20,000)
    assert!(pending_amount > 0);
    assert!(pending_amount <= 10_000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Timing Boundaries
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_withdraw_unstaked_exactly_at_cooldown_end() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig { withdrawal_cooldown_period: 100, ..Default::default() };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Advance exactly to cooldown end
    fixture.pass_time(100);

    // Should succeed exactly at boundary
    let withdrawn =
        fixture.farms_client.withdraw_unstaked(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(withdrawn, 1000);
}

#[test]
#[should_panic(expected = "#51")] // CooldownNotComplete
fn test_withdraw_unstaked_one_second_before_cooldown_end() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig { withdrawal_cooldown_period: 100, ..Default::default() };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Advance to 1 second before cooldown end
    fixture.pass_time(99);

    // Should fail
    fixture.farms_client.withdraw_unstaked(&Delegatee::from(user.clone()), &farm_id);
}

#[test]
fn test_activate_pending_stake_exactly_at_warmup_end() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig { deposit_warmup_period: 100, ..Default::default() };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.pending_deposit_stake, 1000);
    assert_eq!(user_state.active_stake, 0);

    // Advance exactly to warmup end
    fixture.pass_time(100);

    // Refresh should activate pending stake
    fixture.farms_client.refresh_user_state(&Delegatee::from(user.clone()), &farm_id);

    let user_state = fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(user_state.pending_deposit_stake, 0);
    assert_eq!(user_state.active_stake, 1000);
}

#[test]
fn test_penalty_exactly_at_lock_expiry() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig {
        locking_mode: LockingMode::Continuous,
        locking_duration: 100,
        early_withdrawal_penalty_bps: 1000, // 10%
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Advance exactly to lock expiry
    fixture.pass_time(100);

    // Unstake should have no penalty
    let net_amount = fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1000);
    assert_eq!(net_amount, 1000);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.slashed_amount_current, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - User Count Tracking
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_user_count_multiple_stake_unstake_cycles() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);

    // First stake
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &500);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);

    // Stake more (should not increment)
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &500);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);

    // Partial unstake (should not decrement)
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &500);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);

    // Full unstake (should decrement)
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &500);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 0);

    // Re-stake (should increment again)
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);
}

#[test]
fn test_multiple_users_count() {
    let fixture = TestFarmsFixture::new();
    let user1 = &fixture.market_fixture.users[0];
    let user2 = &fixture.market_fixture.users[1];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user1.clone()), &farm_id);
    fixture.farms_client.initialize_user(&Delegatee::from(user2.clone()), &farm_id);

    fixture.farms_client.stake(&Delegatee::from(user1.clone()), &farm_id, &1000);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);
    assert_eq!(farm.total_staked, 1000);

    fixture.farms_client.stake(&Delegatee::from(user2.clone()), &farm_id, &2000);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 2);
    assert_eq!(farm.total_staked, 3000);

    // User1 fully unstakes
    fixture.farms_client.unstake(&Delegatee::from(user1.clone()), &farm_id, &1000);
    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);
    assert_eq!(farm.total_staked, 2000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Error Conditions
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[should_panic(expected = "#20")] // UserAlreadyExists
fn test_double_user_initialization_fails() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id); // Should fail
}

#[test]
#[should_panic(expected = "#10")] // FarmNotFound
fn test_stake_to_nonexistent_farm_fails() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let fake_farm_id = soroban_sdk::BytesN::from_array(e, &[42u8; 32]);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &fake_farm_id, &1000);
}

#[test]
#[should_panic(expected = "#40")] // InsufficientStake
fn test_unstake_more_than_staked_fails() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &1500); // Should fail
}

#[test]
#[should_panic(expected = "#42")] // PendingWithdrawalExists
fn test_double_unstake_without_withdrawal_fails() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig { withdrawal_cooldown_period: 100, ..Default::default() };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // First unstake goes to pending
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &500);

    // Second unstake should fail (must withdraw first)
    fixture.farms_client.unstake(&Delegatee::from(user.clone()), &farm_id, &500);
}

#[test]
#[should_panic(expected = "#32")] // FarmIsDelegated
fn test_stake_on_delegated_farm_fails() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let delegate = Address::generate(e);
    let farm_config = FarmConfig { delegate_authority: Some(delegate), ..Default::default() };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Try to stake directly on delegated farm
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);
}

#[test]
#[should_panic(expected = "#33")] // NotDelegateAuthority
fn test_set_stake_delegated_on_non_delegated_farm_fails() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Non-delegated farm
    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Try to use set_stake_delegated
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);
}

#[test]
#[should_panic(expected = "#34")] // MaxRewardTokensReached
fn test_initialize_more_than_max_reward_tokens_fails() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Try to add 11 reward tokens (max is 10)
    for i in 0..11 {
        let reward_admin = Address::generate(e);
        let reward_token_address =
            e.register_stellar_asset_contract_v2(reward_admin.clone()).address();
        let reward_vault = Address::generate(e);

        fixture.farms_client.initialize_reward(&farm_id, &reward_token_address, &reward_vault);

        // 11th should fail
        if i == 10 {
            panic!("Should have failed before reaching this point");
        }
    }
}

#[test]
#[should_panic(expected = "#21")] // RewardTokenAlreadyExists
fn test_initialize_duplicate_reward_token_fails() {
    let fixture = TestFarmsFixture::new();

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Try to add same token again
    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Reward Curve Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_reward_schedule_starting_in_future() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Schedule starts 100 seconds in the future
    let future_start = fixture.current_timestamp() + 100;
    let schedule = RewardScheduleCurve {
        points: vec![e, RewardCurvePoint { ts_start: future_start, reward_per_time_unit: 100 }],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Wait 50 seconds (still before schedule starts)
    fixture.pass_time(50);

    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    assert_eq!(pending.get(0).unwrap(), 0); // No rewards yet

    // Wait another 100 seconds (50 seconds into reward period)
    fixture.pass_time(100);

    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    let pending_amount = pending.get(0).unwrap();
    // Should have ~50 seconds * 100 = 5000 rewards
    assert!(pending_amount > 0);
}

#[test]
fn test_reward_schedule_rate_drops_to_zero() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Schedule: 100/s for first 100s, then 0
    let now = fixture.current_timestamp();
    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: now, reward_per_time_unit: 100 },
            RewardCurvePoint { ts_start: now + 100, reward_per_time_unit: 0 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    // Wait 200 seconds (100 in reward period, 100 after)
    fixture.pass_time(200);

    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    let pending_amount = pending.get(0).unwrap();

    // Should only have 10,000 rewards (100 seconds * 100)
    assert_eq!(pending_amount, 10_000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Precision and Rounding
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_proportional_rewards_with_unequal_stakes() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user1 = &fixture.market_fixture.users[0];
    let user2 = &fixture.market_fixture.users[1];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 1000 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &10_000_000,
    );

    // User1: 25%, User2: 75%
    fixture.farms_client.initialize_user(&Delegatee::from(user1.clone()), &farm_id);
    fixture.farms_client.initialize_user(&Delegatee::from(user2.clone()), &farm_id);

    fixture.farms_client.stake(&Delegatee::from(user1.clone()), &farm_id, &1000);
    fixture.farms_client.stake(&Delegatee::from(user2.clone()), &farm_id, &3000);

    fixture.pass_time(100);

    let pending1 =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user1.clone()), &farm_id);
    let pending2 =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user2.clone()), &farm_id);

    let rewards1 = pending1.get(0).unwrap();
    let rewards2 = pending2.get(0).unwrap();

    // Total rewards: 100 seconds * 1000 = 100,000
    // User1 should get ~25,000, User2 should get ~75,000
    // Allow for some rounding differences
    assert!((24_000..=26_000).contains(&rewards1));
    assert!((74_000..=76_000).contains(&rewards2));
    assert!(rewards1 + rewards2 <= 100_000);
}

#[test]
fn test_very_small_stake_with_large_rewards() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Large reward rate but within available tokens
    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint {
                ts_start: fixture.current_timestamp(),
                reward_per_time_unit: 1_000_000,
            },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &500_000_000,
    );

    // Very small stake
    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1);

    fixture.pass_time(100);

    // User should get all the rewards (only staker)
    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &farm_id);
    let pending_amount = pending.get(0).unwrap();

    // 100 seconds * 1,000,000 = 100,000,000
    assert_eq!(pending_amount, 100_000_000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - WithExpiry Locking Mode
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_with_expiry_lock_all_users_same_unlock() {
    let fixture = TestFarmsFixture::new();
    let user1 = &fixture.market_fixture.users[0];
    let user2 = &fixture.market_fixture.users[1];

    let now = fixture.current_timestamp();

    // Global lock ends at now + 1000
    let farm_config = FarmConfig {
        locking_mode: LockingMode::WithExpiry,
        locking_start_ts: now,
        locking_duration: 1000,
        early_withdrawal_penalty_bps: 1000,
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_user(&Delegatee::from(user1.clone()), &farm_id);
    fixture.farms_client.initialize_user(&Delegatee::from(user2.clone()), &farm_id);

    // User1 stakes immediately
    fixture.farms_client.stake(&Delegatee::from(user1.clone()), &farm_id, &1000);

    // User2 stakes 500 seconds later
    fixture.pass_time(500);
    fixture.farms_client.stake(&Delegatee::from(user2.clone()), &farm_id, &1000);

    // User2 unstakes immediately after staking (500s into lock, 500s remaining)
    let net_amount2 =
        fixture.farms_client.unstake(&Delegatee::from(user2.clone()), &farm_id, &1000);
    // Penalty should be 5% (50% of 10% max penalty due to linear decay)
    assert_eq!(net_amount2, 950);

    // Wait for lock to expire
    fixture.pass_time(500);

    // User1 can now unstake without penalty
    let net_amount1 =
        fixture.farms_client.unstake(&Delegatee::from(user1.clone()), &farm_id, &1000);
    assert_eq!(net_amount1, 1000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Delegated Farm Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_delegated_farm_set_stake_to_same_value() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Initial stake
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    let farm_before = fixture.farms_client.get_farm(&farm_id);

    // Set to same value (no-op)
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    let farm_after = fixture.farms_client.get_farm(&farm_id);

    // State should be unchanged
    assert_eq!(farm_before.total_staked, farm_after.total_staked);
    assert_eq!(farm_before.num_users, farm_after.num_users);
}

#[test]
fn test_delegated_farm_set_stake_to_zero_removes_user() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };

    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Initial stake
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                1000i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &1000);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 1);

    // Set to zero
    e.mock_auths(&[soroban_sdk::testutils::MockAuth {
        address: &fixture.market_fixture.contract_id,
        invoke: &soroban_sdk::testutils::MockAuthInvoke {
            contract: &fixture.farms_address,
            fn_name: "set_stake_delegated",
            args: soroban_sdk::vec![
                e,
                Delegatee::from(user.clone()).into_val(e),
                farm_id.clone().into_val(e),
                0i128.into_val(e)
            ],
            sub_invokes: &[],
        },
    }]);
    fixture.farms_client.set_stake_delegated(&Delegatee::from(user.clone()), &farm_id, &0);

    let farm = fixture.farms_client.get_farm(&farm_id);
    assert_eq!(farm.num_users, 0);
    assert_eq!(farm.total_staked, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Treasury Fee
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_treasury_fee_on_harvest() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // Update global config with 5% treasury fee
    fixture.farms_client.update_global_config(&farms::GlobalConfigUpdate::TreasuryFeeBps(500));

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 1000 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    fixture.pass_time(100);

    // Harvest
    let initial_user_balance = fixture.reward_token_client.balance(user);
    let initial_treasury_balance =
        fixture.reward_token_client.balance(&fixture.market_fixture.contract_admin);

    e.mock_all_auths_allowing_non_root_auth();
    let harvested = fixture.farms_client.harvest(&Delegatee::from(user.clone()), &farm_id, &0);

    let final_user_balance = fixture.reward_token_client.balance(user);
    let final_treasury_balance =
        fixture.reward_token_client.balance(&fixture.market_fixture.contract_admin);

    // 100 seconds * 1000 = 100,000 total rewards
    // 5% fee = 5,000 to treasury, 95,000 to user
    let user_received = final_user_balance - initial_user_balance;
    let treasury_received = final_treasury_balance - initial_treasury_balance;

    assert_eq!(harvested, user_received);
    assert_eq!(user_received, 95_000);
    assert_eq!(treasury_received, 5_000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests - Min Claim Duration
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
#[should_panic(expected = "#52")] // ClaimTooSoon
fn test_min_claim_duration_prevents_rapid_claims() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Set min claim duration of 100 seconds
    fixture.farms_client.update_farm_config(&farm_id, &FarmConfigUpdate::MinClaimDuration(100));

    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 1000 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    fixture.pass_time(50);

    // First harvest
    e.mock_all_auths_allowing_non_root_auth();
    fixture.farms_client.harvest(&Delegatee::from(user.clone()), &farm_id, &0);

    // Wait only 50 seconds (less than 100)
    fixture.pass_time(50);

    // Second harvest should fail
    fixture.farms_client.harvest(&Delegatee::from(user.clone()), &farm_id, &0);
}

#[test]
fn test_min_claim_duration_allows_claim_after_duration() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    let farm_config = FarmConfig::default();
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    fixture.farms_client.initialize_reward(
        &farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    // Set min claim duration of 100 seconds
    fixture.farms_client.update_farm_config(&farm_id, &FarmConfigUpdate::MinClaimDuration(100));

    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 1000 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &farm_id,
        &0,
        &1_000_000,
    );

    fixture.farms_client.initialize_user(&Delegatee::from(user.clone()), &farm_id);
    fixture.farms_client.stake(&Delegatee::from(user.clone()), &farm_id, &1000);

    fixture.pass_time(100);

    // First harvest
    e.mock_all_auths_allowing_non_root_auth();
    let first_harvest = fixture.farms_client.harvest(&Delegatee::from(user.clone()), &farm_id, &0);
    assert!(first_harvest > 0);

    // Wait 100 seconds (exactly min duration)
    fixture.pass_time(100);

    // Second harvest should succeed
    let second_harvest = fixture.farms_client.harvest(&Delegatee::from(user.clone()), &farm_id, &0);
    assert!(second_harvest > 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Market-Farms Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_market_farms_set_and_get_farms_contract() {
    let fixture = TestFarmsFixture::new();

    // Initially no farms contract configured
    let farms_addr = fixture.market_fixture.contract_client.get_farms_contract();
    assert!(farms_addr.is_none());

    // Set farms contract
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Verify it's set
    let farms_addr = fixture.market_fixture.contract_client.get_farms_contract();
    assert!(farms_addr.is_some());
    assert_eq!(farms_addr.unwrap(), fixture.farms_address);
}

#[test]
fn test_market_farms_set_pool_supply_farm() {
    let fixture = TestFarmsFixture::new();

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated farm with Market as delegate
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Set the farm for USDC pool supply
    fixture
        .market_fixture
        .contract_client
        .set_pool_supply_farm(&fixture.market_fixture.usdc_pool_address, &farm_id);

    // Verify pool has the farm configured
    let pool =
        fixture.market_fixture.contract_client.get_pool(&fixture.market_fixture.usdc_pool_address);
    assert!(pool.farm_supply.is_some());
    assert_eq!(pool.farm_supply.unwrap(), farm_id);
    assert!(pool.farm_debt.is_none());
}

#[test]
fn test_market_farms_set_pool_debt_farm() {
    let fixture = TestFarmsFixture::new();

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Set the farm for USDC pool debt
    fixture
        .market_fixture
        .contract_client
        .set_pool_debt_farm(&fixture.market_fixture.usdc_pool_address, &farm_id);

    // Verify pool has the farm configured
    let pool =
        fixture.market_fixture.contract_client.get_pool(&fixture.market_fixture.usdc_pool_address);
    assert!(pool.farm_supply.is_none());
    assert!(pool.farm_debt.is_some());
    assert_eq!(pool.farm_debt.unwrap(), farm_id);
}

#[test]
fn test_market_farms_clear_pool_farms() {
    let fixture = TestFarmsFixture::new();

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create farms
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let supply_farm_id = fixture.farms_client.initialize_farm(&farm_config);
    let debt_farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Set both farms
    fixture
        .market_fixture
        .contract_client
        .set_pool_supply_farm(&fixture.market_fixture.usdc_pool_address, &supply_farm_id);
    fixture
        .market_fixture
        .contract_client
        .set_pool_debt_farm(&fixture.market_fixture.usdc_pool_address, &debt_farm_id);

    // Verify both are set
    let pool =
        fixture.market_fixture.contract_client.get_pool(&fixture.market_fixture.usdc_pool_address);
    assert!(pool.farm_supply.is_some());
    assert!(pool.farm_debt.is_some());

    // Clear farms
    fixture
        .market_fixture
        .contract_client
        .clear_pool_farms(&fixture.market_fixture.usdc_pool_address);

    // Verify both are cleared
    let pool =
        fixture.market_fixture.contract_client.get_pool(&fixture.market_fixture.usdc_pool_address);
    assert!(pool.farm_supply.is_none());
    assert!(pool.farm_debt.is_none());
}

#[test]
fn test_market_farms_auto_refresh_on_deposit() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated supply farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let supply_farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Configure pool with supply farm
    fixture
        .market_fixture
        .contract_client
        .set_pool_supply_farm(&fixture.market_fixture.usdc_pool_address, &supply_farm_id);

    // User deposits into USDC pool - NO manual refresh_obligation_farms call!
    let deposit_amount = 1000_0000000i128;
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &deposit_amount,
    );

    // Get obligation to see j_tokens
    let obligation = fixture.market_fixture.contract_client.get_user_obligation(user);
    let j_tokens =
        obligation.deposits.get(fixture.market_fixture.usdc_pool_address.clone()).unwrap().j_tokens;

    // Verify farm stake was AUTOMATICALLY updated (no manual refresh needed)
    let user_state =
        fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &supply_farm_id);
    assert_eq!(user_state.active_stake, j_tokens);

    let farm = fixture.farms_client.get_farm(&supply_farm_id);
    assert_eq!(farm.total_staked, j_tokens);
}

#[test]
fn test_market_farms_auto_refresh_on_borrow() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];
    let lender = &fixture.market_fixture.users[1];

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated debt farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let debt_farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Configure USDC pool with debt farm
    fixture
        .market_fixture
        .contract_client
        .set_pool_debt_farm(&fixture.market_fixture.usdc_pool_address, &debt_farm_id);

    // Lender deposits USDC for liquidity
    fixture.market_fixture.contract_client.deposit(
        lender,
        &fixture.market_fixture.usdc_pool_address,
        &10000_0000000i128,
    );

    // User deposits BTC as collateral
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.btc_pool_address,
        &10_0000000i128,
    );

    // User borrows USDC - NO manual refresh_obligation_farms call!
    let borrow_amount = 100_0000000i128;
    fixture.market_fixture.contract_client.borrow(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &borrow_amount,
    );

    // Get obligation to check d_tokens
    let obligation = fixture.market_fixture.contract_client.get_user_obligation(user);
    let d_tokens =
        obligation.borrows.get(fixture.market_fixture.usdc_pool_address.clone()).unwrap().d_tokens;

    // Verify farm stake was AUTOMATICALLY updated
    let user_state =
        fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &debt_farm_id);
    assert_eq!(user_state.active_stake, d_tokens);
}

#[test]
fn test_market_farms_refresh_obligation_syncs_supply_stake() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated supply farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let supply_farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Configure pool with supply farm
    fixture
        .market_fixture
        .contract_client
        .set_pool_supply_farm(&fixture.market_fixture.usdc_pool_address, &supply_farm_id);

    // User deposits into USDC pool
    let deposit_amount = 1000_0000000i128; // 1000 with 7 decimals
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &deposit_amount,
    );

    // Get obligation to see j_tokens
    let obligation = fixture.market_fixture.contract_client.get_user_obligation(user);
    let deposit_position =
        obligation.deposits.get(fixture.market_fixture.usdc_pool_address.clone());
    assert!(deposit_position.is_some());
    let j_tokens = deposit_position.unwrap().j_tokens;
    assert!(j_tokens > 0);

    // Refresh obligation farms
    fixture.market_fixture.contract_client.refresh_obligation_farms(user);

    // Verify farm stake was updated
    let user_state =
        fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &supply_farm_id);
    assert_eq!(user_state.active_stake, j_tokens);

    let farm = fixture.farms_client.get_farm(&supply_farm_id);
    assert_eq!(farm.total_staked, j_tokens);
    assert_eq!(farm.num_users, 1);
}

#[test]
fn test_market_farms_refresh_after_withdraw_updates_stake() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated supply farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let supply_farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Configure pool with supply farm
    fixture
        .market_fixture
        .contract_client
        .set_pool_supply_farm(&fixture.market_fixture.usdc_pool_address, &supply_farm_id);

    // User deposits
    let deposit_amount = 2000_0000000i128;
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &deposit_amount,
    );

    // Refresh farms to sync initial stake
    fixture.market_fixture.contract_client.refresh_obligation_farms(user);

    let obligation = fixture.market_fixture.contract_client.get_user_obligation(user);
    let initial_j_tokens =
        obligation.deposits.get(fixture.market_fixture.usdc_pool_address.clone()).unwrap().j_tokens;

    let user_state =
        fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &supply_farm_id);
    assert_eq!(user_state.active_stake, initial_j_tokens);

    // User withdraws half
    let withdraw_amount = 1000_0000000i128;
    fixture.market_fixture.contract_client.withdraw(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &withdraw_amount,
    );

    // Refresh farms again
    fixture.market_fixture.contract_client.refresh_obligation_farms(user);

    // Verify stake was reduced
    let obligation = fixture.market_fixture.contract_client.get_user_obligation(user);
    let final_j_tokens =
        obligation.deposits.get(fixture.market_fixture.usdc_pool_address.clone()).unwrap().j_tokens;

    let user_state =
        fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &supply_farm_id);
    assert_eq!(user_state.active_stake, final_j_tokens);
    assert!(final_j_tokens < initial_j_tokens);
}

#[test]
fn test_market_farms_refresh_debt_farm_on_borrow() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];
    let lender = &fixture.market_fixture.users[1];

    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated debt farm
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let debt_farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Configure USDC pool with debt farm
    fixture
        .market_fixture
        .contract_client
        .set_pool_debt_farm(&fixture.market_fixture.usdc_pool_address, &debt_farm_id);

    // Lender deposits USDC so there's liquidity to borrow
    let lender_deposit_amount = 10000_0000000i128; // 10,000 USDC
    fixture.market_fixture.contract_client.deposit(
        lender,
        &fixture.market_fixture.usdc_pool_address,
        &lender_deposit_amount,
    );

    // User deposits BTC as collateral
    let collateral_amount = 10_0000000i128; // 10 BTC
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.btc_pool_address,
        &collateral_amount,
    );

    // User borrows USDC (small amount relative to collateral)
    let borrow_amount = 100_0000000i128; // 100 USDC
    fixture.market_fixture.contract_client.borrow(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &borrow_amount,
    );

    // Refresh farms
    fixture.market_fixture.contract_client.refresh_obligation_farms(user);

    // Get obligation to check d_tokens
    let obligation = fixture.market_fixture.contract_client.get_user_obligation(user);
    let borrow_position = obligation.borrows.get(fixture.market_fixture.usdc_pool_address.clone());
    assert!(borrow_position.is_some());
    let d_tokens = borrow_position.unwrap().d_tokens;
    assert!(d_tokens > 0);

    // Verify farm stake
    let user_state =
        fixture.farms_client.get_user_state(&Delegatee::from(user.clone()), &debt_farm_id);
    assert_eq!(user_state.active_stake, d_tokens);
}

#[test]
fn test_market_farms_no_refresh_if_no_farms_configured() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // Set farms contract BUT don't configure any pool farms
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // User deposits
    let deposit_amount = 1000_0000000i128;
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &deposit_amount,
    );

    // Refresh should succeed (no-op, doesn't error)
    fixture.market_fixture.contract_client.refresh_obligation_farms(user);
}

#[test]
fn test_market_farms_refresh_without_farms_contract_succeeds() {
    let fixture = TestFarmsFixture::new();
    let user = &fixture.market_fixture.users[0];

    // DON'T set farms contract

    // User deposits
    let deposit_amount = 1000_0000000i128;
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &deposit_amount,
    );

    // Refresh should succeed (no-op when no farms contract)
    fixture.market_fixture.contract_client.refresh_obligation_farms(user);
}

#[test]
fn test_market_farms_clear_farms_contract() {
    let fixture = TestFarmsFixture::new();

    // Set farms contract
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);
    assert!(fixture.market_fixture.contract_client.get_farms_contract().is_some());

    // Clear it
    fixture.market_fixture.contract_client.clear_farms_contract();
    assert!(fixture.market_fixture.contract_client.get_farms_contract().is_none());
}

#[test]
fn test_market_farms_full_e2e_deposit_refresh_harvest() {
    let fixture = TestFarmsFixture::new();
    let e = &fixture.market_fixture.e;
    let user = &fixture.market_fixture.users[0];

    // === Setup ===
    // Set farms contract on market
    fixture.market_fixture.contract_client.set_farms_contract(&fixture.farms_address);

    // Create a delegated supply farm with Market as delegate
    let farm_config = FarmConfig {
        delegate_authority: Some(fixture.market_fixture.contract_id.clone()),
        ..Default::default()
    };
    let supply_farm_id = fixture.farms_client.initialize_farm(&farm_config);

    // Add reward token and schedule
    fixture.farms_client.initialize_reward(
        &supply_farm_id,
        &fixture.reward_token_address,
        &fixture.reward_vault,
    );

    let schedule = RewardScheduleCurve {
        points: vec![
            e,
            RewardCurvePoint { ts_start: fixture.current_timestamp(), reward_per_time_unit: 1000 },
        ],
    };
    fixture.farms_client.update_reward_schedule(&supply_farm_id, &0, &schedule);

    fixture.farms_client.add_rewards(
        &fixture.market_fixture.contract_admin,
        &supply_farm_id,
        &0,
        &10_000_000,
    );

    // Configure pool with supply farm
    fixture
        .market_fixture
        .contract_client
        .set_pool_supply_farm(&fixture.market_fixture.usdc_pool_address, &supply_farm_id);

    // === User Action: Deposit ===
    let deposit_amount = 1000_0000000i128;
    fixture.market_fixture.contract_client.deposit(
        user,
        &fixture.market_fixture.usdc_pool_address,
        &deposit_amount,
    );

    // Refresh farms to sync stake
    fixture.market_fixture.contract_client.refresh_obligation_farms(user);

    // === Time Passes ===
    fixture.pass_time(100);

    // === Verify Rewards Accrued ===
    let pending =
        fixture.farms_client.get_pending_rewards(&Delegatee::from(user.clone()), &supply_farm_id);
    let pending_amount = pending.get(0).unwrap();
    assert!(pending_amount > 0);

    // === Harvest Rewards ===
    let initial_balance = fixture.reward_token_client.balance(user);
    let harvested =
        fixture.farms_client.harvest(&Delegatee::from(user.clone()), &supply_farm_id, &0);
    let final_balance = fixture.reward_token_client.balance(user);

    assert!(harvested > 0);
    assert_eq!(final_balance - initial_balance, harvested);
}
