// #![cfg(test)]
// #![allow(clippy::inconsistent_digit_grouping)]

// use farms::{
//     DelegatedFarmConfig, Delegation, FarmConfig, FarmingKey, FarmsClient, FarmsContract,
//     LockingMode, NonDelegatedFarmConfig, OptionalOracle, OracleConfig, RewardCurvePoint,
//     RewardScheduleCurve, RewardType,
// };
// use soroban_sdk::{
//     Address, Env,
//     testutils::{Address as _, Ledger, LedgerInfo},
//     token::{StellarAssetClient, TokenClient},
//     vec,
// };

// const MINT_AMOUNT: i128 = 1_000_000_0000000;

// struct TestFarmsSetup<'a> {
//     e: Env,
//     admin: Address,
//     stake_token: Address,
//     stake_token_client: TokenClient<'a>,
//     reward_token: Address,
//     reward_token_client: TokenClient<'a>,
//     users: [Address; 3],
// }

// impl TestFarmsSetup<'_> {
//     fn new() -> Self {
//         let e = Env::default();
//         e.mock_all_auths_allowing_non_root_auth();
//         e.ledger().set(LedgerInfo {
//             timestamp: 1_000_000,
//             protocol_version: 23,
//             sequence_number: 1000,
//             network_id: Default::default(),
//             base_reserve: 10,
//             min_temp_entry_ttl: 500_000,
//             min_persistent_entry_ttl: 500_000,
//             max_entry_ttl: 500_001,
//         });

//         let admin = Address::generate(&e);

//         let stake_admin = Address::generate(&e);
//         let stake_token = e.register_stellar_asset_contract_v2(stake_admin.clone()).address();
//         let stake_sac = StellarAssetClient::new(&e, &stake_token);
//         let stake_token_client = TokenClient::new(&e, &stake_token);

//         let reward_admin = Address::generate(&e);
//         let reward_token = e.register_stellar_asset_contract_v2(reward_admin.clone()).address();
//         let reward_sac = StellarAssetClient::new(&e, &reward_token);
//         let reward_token_client = TokenClient::new(&e, &reward_token);

//         let users = [Address::generate(&e), Address::generate(&e), Address::generate(&e)];

//         for user in &users {
//             stake_sac.mint(user, &MINT_AMOUNT);
//         }
//         stake_sac.mint(&admin, &MINT_AMOUNT);
//         reward_sac.mint(&admin, &MINT_AMOUNT);

//         Self { e, admin, stake_token, stake_token_client, reward_token, reward_token_client, users }
//     }

//     fn deploy_farm(&self, config: FarmConfig) -> FarmsClient<'_> {
//         let address = self.e.register(FarmsContract, (config,));
//         FarmsClient::new(&self.e, &address)
//     }

//     fn pass_time(&self, seconds: u64) {
//         self.e.ledger().with_mut(|li| {
//             li.timestamp = li.timestamp.saturating_add(seconds);
//         });
//     }

//     fn current_ts(&self) -> u64 {
//         self.e.ledger().timestamp()
//     }

//     fn fk(&self, user: &Address) -> FarmingKey {
//         FarmingKey::new(user.clone())
//     }

//     fn default_non_delegated_config(&self) -> FarmConfig {
//         FarmConfig {
//             token: self.stake_token.clone(),
//             admin: self.admin.clone(),
//             deposit_cap: 0,
//             treasury_fee_bps: 0,
//             min_harvest_delay: 0,
//             min_stake_amount: 0,
//             delegation: Delegation::NonDelegated(NonDelegatedFarmConfig {
//                 locking_ts: 0,
//                 locking_duration: 0,
//                 locking_mode: LockingMode::None,
//                 deposit_warmup_period: 0,
//                 withdrawal_cooldown_period: 0,
//                 early_withdrawal_penalty_bps: 0,
//             }),
//             is_reward_once_enabled: false,
//             is_harvest_permissionless: false,
//             proposed_admin: None,
//             oracle: OptionalOracle::None,
//         }
//     }

//     fn default_delegated_config(&self, delegate: &Address) -> FarmConfig {
//         FarmConfig {
//             token: self.stake_token.clone(),
//             admin: self.admin.clone(),
//             deposit_cap: 0,
//             treasury_fee_bps: 0,
//             min_harvest_delay: 0,
//             min_stake_amount: 0,
//             delegation: Delegation::Delegated(DelegatedFarmConfig {
//                 delegate_authority: delegate.clone(),
//                 second_delegate_authority: None,
//             }),
//             is_reward_once_enabled: false,
//             is_harvest_permissionless: false,
//             proposed_admin: None,
//             oracle: OptionalOracle::None,
//         }
//     }

//     fn setup_rewards(&self, client: &FarmsClient<'_>) {
//         client.initialize_reward(&self.reward_token, &RewardType::Proportional);

//         client.add_rewards(&(100_000_0000000_i128), &self.admin, &self.reward_token);

//         let schedule = RewardScheduleCurve {
//             points: vec![
//                 &self.e,
//                 RewardCurvePoint { ts_start: self.current_ts(), reward_per_time_unit: 1_0000000 },
//             ],
//         };
//         client.update_reward_schedule(&self.reward_token, &schedule);
//     }

//     fn create_farm_with_rewards(&self, config: FarmConfig) -> FarmsClient<'_> {
//         let client = self.deploy_farm(config);
//         client.unfreeze_farm();
//         self.setup_rewards(&client);
//         client
//     }
// }

// #[test]
// fn test_initialize_farm() {
//     let s = TestFarmsSetup::new();
//     let config = s.default_non_delegated_config();

//     let client = s.deploy_farm(config);
//     let farm = client.get_farm();

//     assert_eq!(farm.total_staked, 0);
//     assert_eq!(farm.num_users, 0);
//     assert!(farm.is_frozen);
//     assert_eq!(farm.config.token, s.stake_token);
//     assert_eq!(farm.config.admin, s.admin);
// }

// #[test]
// fn test_initialize_delegated_farm() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let config = s.default_delegated_config(&delegate);

//     let client = s.deploy_farm(config);
//     let farm = client.get_farm();

//     assert!(matches!(farm.config.delegation, Delegation::Delegated(_)));
//     assert!(farm.is_frozen);
// }

// #[test]
// fn test_freeze_and_unfreeze() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());

//     assert!(client.get_farm().is_frozen);

//     client.unfreeze_farm();
//     assert!(!client.get_farm().is_frozen);

//     client.freeze_farm();
//     assert!(client.get_farm().is_frozen);
// }

// #[test]
// fn test_stake_on_frozen_farm_fails() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());

//     let user = &s.users[0];
//     let result = client.try_stake(&s.fk(user), &100_0000000);
//     assert!(result.is_err());
// }

// #[test]
// fn test_stake_basic() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     let amount = 100_0000000_i128;

//     let balance_before = s.stake_token_client.balance(user);
//     client.stake(&fk, &amount);
//     let balance_after = s.stake_token_client.balance(user);

//     assert_eq!(balance_before - balance_after, amount);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, amount);

//     let farm = client.get_farm();
//     assert_eq!(farm.total_staked, amount);
//     assert_eq!(farm.num_users, 1);
// }

// #[test]
// fn test_stake_zero_fails() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let result = client.try_stake(&s.fk(&s.users[0]), &0);
//     assert!(result.is_err());
// }

// #[test]
// fn test_stake_below_min_amount_fails() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.min_stake_amount = 100;

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let result = client.try_stake(&s.fk(&s.users[0]), &50);
//     assert!(result.is_err());

//     client.stake(&s.fk(&s.users[0]), &100);
//     let pos = client.get_farming_position(&s.fk(&s.users[0]));
//     assert_eq!(pos.active_stake, 100);
// }

// #[test]
// fn test_deposit_cap() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.deposit_cap = 200_0000000;

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     client.stake(&s.fk(&s.users[0]), &150_0000000);

//     let result = client.try_stake(&s.fk(&s.users[1]), &100_0000000);
//     assert!(result.is_err());

//     client.stake(&s.fk(&s.users[1]), &50_0000000);
//     let farm = client.get_farm();
//     assert_eq!(farm.total_staked, 200_0000000);
// }

// #[test]
// fn test_unstake_basic() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &1000_0000000);

//     client.unstake(&500_0000000, &fk);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, 500_0000000);
//     assert_eq!(pos.pending_withdrawal_stake, 500_0000000);

//     let farm = client.get_farm();
//     assert_eq!(farm.total_staked, 500_0000000);

//     let balance_before = s.stake_token_client.balance(user);
//     let withdrawn = client.withdraw_unstaked(&fk);
//     let balance_after = s.stake_token_client.balance(user);

//     assert_eq!(withdrawn, 500_0000000);
//     assert_eq!(balance_after - balance_before, 500_0000000);
// }

// #[test]
// fn test_unstake_full_withdrawal_below_min() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.min_stake_amount = 100_0000000;

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);

//     let result = client.try_unstake(&50_0000000, &fk);
//     assert!(result.is_err());

//     client.unstake(&100_0000000, &fk);
//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, 0);
//     assert_eq!(pos.pending_withdrawal_stake, 100_0000000);
// }

// #[test]
// fn test_unstake_insufficient_stake_fails() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);

//     let result = client.try_unstake(&200_0000000, &fk);
//     assert!(result.is_err());
// }

// #[test]
// fn test_pending_withdrawal_blocks_second_unstake() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &1000_0000000);
//     client.unstake(&300_0000000, &fk);

//     let result = client.try_unstake(&300_0000000, &fk);
//     assert!(result.is_err());

//     client.withdraw_unstaked(&fk);
//     client.unstake(&300_0000000, &fk);
//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, 400_0000000);
// }

// #[test]
// fn test_unstake_with_cooldown() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.withdrawal_cooldown_period = 3600;
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &1000_0000000);
//     client.unstake(&500_0000000, &fk);

//     let result = client.try_withdraw_unstaked(&fk);
//     assert!(result.is_err());

//     s.pass_time(3601);

//     let withdrawn = client.withdraw_unstaked(&fk);
//     assert_eq!(withdrawn, 500_0000000);
// }

// #[test]
// fn test_early_withdrawal_penalty_continuous() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.locking_mode = LockingMode::Continuous;
//         nd.locking_duration = 1000;
//         nd.early_withdrawal_penalty_bps = 1000; // 10%
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &1000_0000000);

//     client.unstake(&1000_0000000, &fk);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.pending_withdrawal_stake, 900_0000000);

//     let farm = client.get_farm();
//     assert_eq!(farm.current_slashed_amount, 100_0000000);
// }

// #[test]
// fn test_penalty_decay_halfway() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.locking_mode = LockingMode::Continuous;
//         nd.locking_duration = 1000;
//         nd.early_withdrawal_penalty_bps = 1000; // 10%
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &1000_0000000);

//     s.pass_time(500);
//     client.unstake(&1000_0000000, &fk);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.pending_withdrawal_stake, 950_0000000);
// }

// #[test]
// fn test_no_penalty_after_lock_expires() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.locking_mode = LockingMode::Continuous;
//         nd.locking_duration = 1000;
//         nd.early_withdrawal_penalty_bps = 1000;
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &1000_0000000);

//     s.pass_time(1001);
//     client.unstake(&1000_0000000, &fk);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.pending_withdrawal_stake, 1000_0000000);

//     let farm = client.get_farm();
//     assert_eq!(farm.current_slashed_amount, 0);
// }

// #[test]
// fn test_warmup_period() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.deposit_warmup_period = 3600;
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, 0);
//     assert_eq!(pos.pending_deposit_stake, 100_0000000);

//     s.pass_time(1800);
//     let result = client.try_refresh_farming_position(&fk);
//     assert!(result.is_err());

//     s.pass_time(1801);
//     client.refresh_farming_position(&fk);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, 100_0000000);
//     assert_eq!(pos.pending_deposit_stake, 0);
// }

// #[test]
// fn test_reward_accrual_and_harvest() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);

//     s.pass_time(100);

//     let pending = client.get_pending_rewards(&fk);
//     assert_eq!(pending.len(), 1);
//     let (token, amount) = pending.get(0).unwrap();
//     assert_eq!(token, s.reward_token);
//     assert_eq!(amount, 100_0000000);

//     let balance_before = s.reward_token_client.balance(user);
//     let harvested = client.harvest(&s.reward_token, &fk);
//     let balance_after = s.reward_token_client.balance(user);

//     assert_eq!(harvested, 100_0000000);
//     assert_eq!(balance_after - balance_before, 100_0000000);

//     let result = client.try_harvest(&s.reward_token, &fk);
//     assert!(result.is_err());
// }

// #[test]
// fn test_harvest_with_treasury_fee() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.treasury_fee_bps = 1000; // 10%

//     let client = s.create_farm_with_rewards(config);

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     let harvested = client.harvest(&s.reward_token, &fk);
//     assert_eq!(harvested, 90_0000000);
// }

// #[test]
// fn test_harvest_all() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     let balance_before = s.reward_token_client.balance(user);
//     client.harvest_all(&fk);
//     let balance_after = s.reward_token_client.balance(user);

//     assert_eq!(balance_after - balance_before, 100_0000000);
// }

// #[test]
// fn test_multiple_stakers_proportional() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let fk0 = s.fk(&s.users[0]);
//     let fk1 = s.fk(&s.users[1]);

//     client.stake(&fk0, &300_0000000);
//     client.stake(&fk1, &100_0000000);

//     s.pass_time(100);

//     let h0 = client.harvest(&s.reward_token, &fk0);
//     let h1 = client.harvest(&s.reward_token, &fk1);

//     assert_eq!(h0, 75_0000000);
//     assert_eq!(h1, 25_0000000);
// }

// #[test]
// fn test_min_harvest_delay() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.min_harvest_delay = 600;

//     let client = s.create_farm_with_rewards(config);

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     client.harvest(&s.reward_token, &fk);

//     s.pass_time(100);
//     let result = client.try_harvest(&s.reward_token, &fk);
//     assert!(result.is_err());

//     s.pass_time(500);
//     let h = client.harvest(&s.reward_token, &fk);
//     assert!(h > 0);
// }

// #[test]
// fn test_delegated_set_stake() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let config = s.default_delegated_config(&delegate);

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.set_stake_delegated(&delegate, &fk, &1000_0000000);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, 1000_0000000);

//     let farm = client.get_farm();
//     assert_eq!(farm.total_staked, 1000_0000000);
//     assert_eq!(farm.num_users, 1);

//     client.set_stake_delegated(&delegate, &fk, &1500_0000000);
//     assert_eq!(client.get_farm().total_staked, 1500_0000000);

//     client.set_stake_delegated(&delegate, &fk, &500_0000000);
//     assert_eq!(client.get_farm().total_staked, 500_0000000);

//     client.set_stake_delegated(&delegate, &fk, &0);
//     let farm = client.get_farm();
//     assert_eq!(farm.total_staked, 0);
//     assert_eq!(farm.num_users, 0);
// }

// #[test]
// fn test_delegated_deposit_cap() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let mut config = s.default_delegated_config(&delegate);
//     config.deposit_cap = 500_0000000;

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.set_stake_delegated(&delegate, &fk, &400_0000000);

//     let result = client.try_set_stake_delegated(&delegate, &fk, &600_0000000);
//     assert!(result.is_err());
// }

// #[test]
// fn test_delegated_frozen_farm_rejects() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let config = s.default_delegated_config(&delegate);
//     let client = s.deploy_farm(config);

//     let result = client.try_set_stake_delegated(&delegate, &s.fk(&s.users[0]), &100_0000000);
//     assert!(result.is_err());
// }

// #[test]
// fn test_delegated_user_count_tracking() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let config = s.default_delegated_config(&delegate);
//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk0 = s.fk(&s.users[0]);
//     let fk1 = s.fk(&s.users[1]);

//     client.set_stake_delegated(&delegate, &fk0, &100_0000000);
//     assert_eq!(client.get_farm().num_users, 1);

//     client.set_stake_delegated(&delegate, &fk1, &200_0000000);
//     assert_eq!(client.get_farm().num_users, 2);

//     client.set_stake_delegated(&delegate, &fk0, &300_0000000);
//     assert_eq!(client.get_farm().num_users, 2);

//     client.set_stake_delegated(&delegate, &fk0, &0);
//     assert_eq!(client.get_farm().num_users, 1);

//     client.set_stake_delegated(&delegate, &fk1, &0);
//     assert_eq!(client.get_farm().num_users, 0);
// }

// #[test]
// fn test_non_delegated_rejects_set_stake_delegated() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let caller = Address::generate(&s.e);
//     let result = client.try_set_stake_delegated(&caller, &s.fk(&s.users[0]), &100_0000000);
//     assert!(result.is_err());
// }

// #[test]
// fn test_propose_accept_admin() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());

//     let new_admin = Address::generate(&s.e);
//     client.propose_admin(&new_admin);

//     let farm = client.get_farm();
//     assert_eq!(farm.config.proposed_admin, Some(new_admin.clone()));

//     client.accept_admin();

//     let farm = client.get_farm();
//     assert_eq!(farm.config.admin, new_admin);
//     assert!(farm.config.proposed_admin.is_none());
// }

// #[test]
// fn test_withdraw_unused_rewards() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let recipient = Address::generate(&s.e);
//     let withdraw_amount = 1000_0000000_i128;

//     let balance_before = s.reward_token_client.balance(&recipient);
//     client.withdraw_unused(&withdraw_amount, &recipient, &s.reward_token);
//     let balance_after = s.reward_token_client.balance(&recipient);

//     assert_eq!(balance_after - balance_before, withdraw_amount);
// }

// #[test]
// fn test_withdraw_slashed() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.locking_mode = LockingMode::Continuous;
//         nd.locking_duration = 1000;
//         nd.early_withdrawal_penalty_bps = 1000; // 10%
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &1000_0000000);

//     client.unstake(&1000_0000000, &fk);

//     let farm = client.get_farm();
//     assert_eq!(farm.current_slashed_amount, 100_0000000);

//     let recipient = Address::generate(&s.e);
//     let balance_before = s.stake_token_client.balance(&recipient);
//     client.withdraw_slashed(&100_0000000, &recipient);
//     let balance_after = s.stake_token_client.balance(&recipient);

//     assert_eq!(balance_after - balance_before, 100_0000000);

//     let farm = client.get_farm();
//     assert_eq!(farm.current_slashed_amount, 0);
// }

// #[test]
// fn test_withdraw_treasury_fees() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.treasury_fee_bps = 1000; // 10%

//     let client = s.create_farm_with_rewards(config);

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     client.harvest(&s.reward_token, &fk);

//     let recipient = Address::generate(&s.e);
//     let balance_before = s.reward_token_client.balance(&recipient);
//     client.withdraw_treasury_fees(&10_0000000, &recipient, &s.reward_token);
//     let balance_after = s.reward_token_client.balance(&recipient);

//     assert_eq!(balance_after - balance_before, 10_0000000);
// }

// #[test]
// fn test_reward_once() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let mut config = s.default_delegated_config(&delegate);
//     config.is_reward_once_enabled = true;

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     client.initialize_reward(&s.reward_token, &RewardType::Proportional);
//     client.add_rewards(&10_000_0000000, &s.admin, &s.reward_token);

//     let fk = s.fk(&s.users[0]);
//     client.set_stake_delegated(&delegate, &fk, &100_0000000);

//     client.reward_once(&500_0000000, &s.reward_token, &fk);

//     let pending = client.get_pending_rewards(&fk);
//     let (_, amount) = pending.get(0).unwrap();
//     assert!(amount >= 500_0000000);

//     let harvested = client.harvest(&s.reward_token, &fk);
//     assert!(harvested >= 500_0000000);
// }

// #[test]
// fn test_reward_once_disabled_fails() {
//     let s = TestFarmsSetup::new();
//     let config = s.default_non_delegated_config();
//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     client.initialize_reward(&s.reward_token, &RewardType::Proportional);
//     client.add_rewards(&10_000_0000000, &s.admin, &s.reward_token);

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);

//     let result = client.try_reward_once(&500_0000000, &s.reward_token, &fk);
//     assert!(result.is_err());
// }

// #[test]
// fn test_constant_reward_type() {
//     let s = TestFarmsSetup::new();
//     let config = s.default_non_delegated_config();
//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     client.initialize_reward(&s.reward_token, &RewardType::Constant);
//     client.add_rewards(&100_000_0000000, &s.admin, &s.reward_token);

//     let schedule = RewardScheduleCurve {
//         points: vec![
//             &s.e,
//             RewardCurvePoint { ts_start: s.current_ts(), reward_per_time_unit: 2_0000000 },
//         ],
//     };
//     client.update_reward_schedule(&s.reward_token, &schedule);

//     let fk0 = s.fk(&s.users[0]);
//     let fk1 = s.fk(&s.users[1]);

//     client.stake(&fk0, &100_0000000);
//     client.stake(&fk1, &900_0000000);

//     s.pass_time(100);

//     let h0 = client.harvest(&s.reward_token, &fk0);
//     let h1 = client.harvest(&s.reward_token, &fk1);

//     assert_eq!(h0, h1);
//     assert_eq!(h0, 100_0000000);
// }

// #[test]
// fn test_pending_rewards_query_is_readonly() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     let farm_before = client.get_farm();

//     let pending1 = client.get_pending_rewards(&fk);
//     let pending2 = client.get_pending_rewards(&fk);

//     let (_, a1) = pending1.get(0).unwrap();
//     let (_, a2) = pending2.get(0).unwrap();
//     assert_eq!(a1, a2);
//     assert_eq!(a1, 100_0000000);

//     let farm_after = client.get_farm();
//     assert_eq!(farm_before.total_staked, farm_after.total_staked);
//     assert_eq!(farm_before.num_users, farm_after.num_users);

//     let harvested = client.harvest(&s.reward_token, &fk);
//     assert_eq!(harvested, 100_0000000);
// }

// #[test]
// fn test_num_users_tracks_correctly_through_stake_unstake() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let fk0 = s.fk(&s.users[0]);
//     let fk1 = s.fk(&s.users[1]);

//     client.stake(&fk0, &100_0000000);
//     assert_eq!(client.get_farm().num_users, 1);

//     client.stake(&fk1, &100_0000000);
//     assert_eq!(client.get_farm().num_users, 2);

//     client.stake(&fk0, &50_0000000);
//     assert_eq!(client.get_farm().num_users, 2);

//     client.unstake(&150_0000000, &fk0);
//     assert_eq!(client.get_farm().num_users, 1);

//     client.unstake(&100_0000000, &fk1);
//     assert_eq!(client.get_farm().num_users, 0);
// }

// #[test]
// fn test_reward_schedule_with_decreasing_rate() {
//     let s = TestFarmsSetup::new();
//     let config = s.default_non_delegated_config();
//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     client.initialize_reward(&s.reward_token, &RewardType::Proportional);
//     client.add_rewards(&100_000_0000000, &s.admin, &s.reward_token);

//     let now = s.current_ts();
//     let schedule = RewardScheduleCurve {
//         points: vec![
//             &s.e,
//             RewardCurvePoint { ts_start: now, reward_per_time_unit: 2_0000000 },
//             RewardCurvePoint { ts_start: now + 100, reward_per_time_unit: 1_0000000 },
//         ],
//     };
//     client.update_reward_schedule(&s.reward_token, &schedule);

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);

//     s.pass_time(200);

//     let harvested = client.harvest(&s.reward_token, &fk);
//     assert_eq!(harvested, 300_0000000);
// }

// #[test]
// fn test_reward_limited_by_available_balance() {
//     let s = TestFarmsSetup::new();
//     let config = s.default_non_delegated_config();
//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     client.initialize_reward(&s.reward_token, &RewardType::Proportional);

//     client.add_rewards(&50_0000000, &s.admin, &s.reward_token);

//     let schedule = RewardScheduleCurve {
//         points: vec![
//             &s.e,
//             RewardCurvePoint { ts_start: s.current_ts(), reward_per_time_unit: 1_0000000 },
//         ],
//     };
//     client.update_reward_schedule(&s.reward_token, &schedule);

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);

//     s.pass_time(100);

//     let harvested = client.harvest(&s.reward_token, &fk);
//     assert_eq!(harvested, 50_0000000);
// }

// #[test]
// fn test_initialize_reward_max_limit() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());

//     for _ in 0..10 {
//         let admin = Address::generate(&s.e);
//         let token = s.e.register_stellar_asset_contract_v2(admin).address();
//         client.initialize_reward(&token, &RewardType::Proportional);
//     }

//     let admin = Address::generate(&s.e);
//     let token = s.e.register_stellar_asset_contract_v2(admin).address();
//     let result = client.try_initialize_reward(&token, &RewardType::Proportional);
//     assert!(result.is_err());
// }

// #[test]
// fn test_duplicate_reward_token_fails() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());

//     client.initialize_reward(&s.reward_token, &RewardType::Proportional);

//     let result = client.try_initialize_reward(&s.reward_token, &RewardType::Proportional);
//     assert!(result.is_err());
// }

// #[test]
// fn test_staker_joining_late_gets_only_future_rewards() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let fk0 = s.fk(&s.users[0]);
//     client.stake(&fk0, &100_0000000);

//     s.pass_time(100);

//     let fk1 = s.fk(&s.users[1]);
//     client.stake(&fk1, &100_0000000);

//     s.pass_time(100);

//     let h0 = client.harvest(&s.reward_token, &fk0);
//     let h1 = client.harvest(&s.reward_token, &fk1);

//     assert_eq!(h0, 150_0000000);
//     assert_eq!(h1, 50_0000000);
// }

// #[test]
// fn test_locking_with_expiry_mode() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     let lock_start = s.current_ts();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.locking_mode = LockingMode::WithExpiry;
//         nd.locking_ts = lock_start;
//         nd.locking_duration = 500;
//         nd.early_withdrawal_penalty_bps = 1000; // 10%
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);

//     client.stake(&fk, &1000_0000000);

//     s.pass_time(250);
//     client.unstake(&1000_0000000, &fk);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.pending_withdrawal_stake, 950_0000000);
// }

// #[test]
// fn test_withdraw_unused_exceeds_available_fails() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let result = client.try_withdraw_unused(&999_999_0000000, &s.admin, &s.reward_token);
//     assert!(result.is_err());
// }

// #[test]
// fn test_withdraw_slashed_exceeds_available_fails() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let result = client.try_withdraw_slashed(&100_0000000, &s.admin);
//     assert!(result.is_err());
// }

// fn auth_addrs(e: &Env) -> std::vec::Vec<Address> {
//     e.auths().into_iter().map(|(addr, _)| addr).collect()
// }

// #[test]
// fn test_stake_requires_owner_auth() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(user), "stake must require farming_key.owner auth");
// }

// #[test]
// fn test_unstake_requires_owner_auth() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     client.unstake(&100_0000000, &fk);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(user), "unstake must require farming_key.owner auth");
// }

// #[test]
// fn test_harvest_requires_owner_auth() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     client.harvest(&s.reward_token, &fk);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(user), "harvest must require farming_key.owner auth");
// }

// #[test]
// fn test_harvest_all_requires_owner_auth() {
//     let s = TestFarmsSetup::new();
//     let client = s.create_farm_with_rewards(s.default_non_delegated_config());

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     client.harvest_all(&fk);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(user), "harvest_all must require farming_key.owner auth");
// }

// #[test]
// fn test_withdraw_unstaked_requires_owner_auth() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     client.unstake(&100_0000000, &fk);
//     client.withdraw_unstaked(&fk);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(user), "withdraw_unstaked must require farming_key.owner auth");
// }

// #[test]
// fn test_refresh_farming_position_requires_owner_auth() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.deposit_warmup_period = 100;
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(200);

//     client.refresh_farming_position(&fk);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(user), "refresh_farming_position must require farming_key.owner auth");
// }

// #[test]
// fn test_set_stake_delegated_requires_delegate_auth() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let config = s.default_delegated_config(&delegate);

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.set_stake_delegated(&delegate, &fk, &1000_0000000);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(&delegate), "set_stake_delegated must require delegate_authority auth");
//     assert!(!addrs.contains(&s.users[0]), "set_stake_delegated must NOT require user auth");
// }

// #[test]
// fn test_freeze_farm_requires_farm_admin_auth() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     client.freeze_farm();

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(&s.admin), "freeze_farm must require farm admin auth");
// }

// #[test]
// fn test_withdraw_slashed_requires_farm_admin_auth() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     if let Delegation::NonDelegated(ref mut nd) = config.delegation {
//         nd.locking_mode = LockingMode::Continuous;
//         nd.locking_duration = 1000;
//         nd.early_withdrawal_penalty_bps = 1000;
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &1000_0000000);
//     client.unstake(&1000_0000000, &fk);

//     let recipient = Address::generate(&s.e);
//     client.withdraw_slashed(&100_0000000, &recipient);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(&s.admin), "withdraw_slashed must require farm admin auth");
// }

// #[test]
// fn test_withdraw_treasury_fees_uses_farm_admin_not_global_admin() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.treasury_fee_bps = 1000;

//     let client = s.create_farm_with_rewards(config);

//     let new_farm_admin = Address::generate(&s.e);
//     client.propose_admin(&new_farm_admin);
//     client.accept_admin();

//     let fk = s.fk(&s.users[0]);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);
//     client.harvest(&s.reward_token, &fk);

//     let recipient = Address::generate(&s.e);
//     client.withdraw_treasury_fees(&10_0000000, &recipient, &s.reward_token);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(&new_farm_admin), "withdraw_treasury_fees must require farm admin auth");
//     assert!(!addrs.contains(&s.admin), "withdraw_treasury_fees must NOT use global admin");
// }

// #[test]
// fn test_add_rewards_requires_funder_auth() {
//     let s = TestFarmsSetup::new();
//     let client = s.deploy_farm(s.default_non_delegated_config());
//     client.unfreeze_farm();

//     client.initialize_reward(&s.reward_token, &RewardType::Proportional);

//     client.add_rewards(&1000_0000000, &s.admin, &s.reward_token);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(&s.admin), "add_rewards must require funder auth");
// }

// #[test]
// fn test_permissionless_harvest_enabled() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.is_harvest_permissionless = true;

//     let client = s.create_farm_with_rewards(config);

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     let balance_before = s.reward_token_client.balance(user);
//     let harvested = client.harvest(&s.reward_token, &fk);
//     let balance_after = s.reward_token_client.balance(user);

//     assert_eq!(harvested, 100_0000000);
//     assert_eq!(balance_after - balance_before, 100_0000000);

//     let addrs = auth_addrs(&s.e);
//     assert!(!addrs.contains(user), "permissionless harvest must NOT require owner auth");
// }

// #[test]
// fn test_permissionless_harvest_disabled_requires_auth() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.is_harvest_permissionless = false;

//     let client = s.create_farm_with_rewards(config);

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     client.harvest(&s.reward_token, &fk);

//     let addrs = auth_addrs(&s.e);
//     assert!(addrs.contains(user), "non-permissionless harvest must require owner auth");
// }

// #[test]
// fn test_permissionless_harvest_all_enabled() {
//     let s = TestFarmsSetup::new();
//     let mut config = s.default_non_delegated_config();
//     config.is_harvest_permissionless = true;

//     let client = s.create_farm_with_rewards(config);

//     let user = &s.users[0];
//     let fk = s.fk(user);
//     client.stake(&fk, &100_0000000);
//     s.pass_time(100);

//     let balance_before = s.reward_token_client.balance(user);
//     client.harvest_all(&fk);
//     let balance_after = s.reward_token_client.balance(user);

//     assert_eq!(balance_after - balance_before, 100_0000000);

//     let addrs = auth_addrs(&s.e);
//     assert!(!addrs.contains(user), "permissionless harvest_all must NOT require owner auth");
// }

// #[test]
// fn test_second_delegate_authority_can_set_stake() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let second_delegate = Address::generate(&s.e);
//     let mut config = s.default_delegated_config(&delegate);
//     if let Delegation::Delegated(ref mut d) = config.delegation {
//         d.second_delegate_authority = Some(second_delegate.clone());
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let fk = s.fk(&s.users[0]);

//     client.set_stake_delegated(&delegate, &fk, &100_0000000);
//     assert_eq!(client.get_farming_position(&fk).active_stake, 100_0000000);

//     client.set_stake_delegated(&second_delegate, &fk, &200_0000000);
//     assert_eq!(client.get_farming_position(&fk).active_stake, 200_0000000);
// }

// #[test]
// fn test_unauthorized_caller_rejected_for_set_stake_delegated() {
//     let s = TestFarmsSetup::new();
//     let delegate = Address::generate(&s.e);
//     let second_delegate = Address::generate(&s.e);
//     let mut config = s.default_delegated_config(&delegate);
//     if let Delegation::Delegated(ref mut d) = config.delegation {
//         d.second_delegate_authority = Some(second_delegate.clone());
//     }

//     let client = s.deploy_farm(config);
//     client.unfreeze_farm();

//     let unauthorized = Address::generate(&s.e);
//     let fk = s.fk(&s.users[0]);

//     let result = client.try_set_stake_delegated(&unauthorized, &fk, &100_0000000);
//     assert!(result.is_err());
// }

// mod mock_oracle {
//     use sep_40_oracle::{Asset, PriceData, PriceFeedTrait};
//     use soroban_sdk::{Env, Map, Vec, contract, contractimpl, contracttype};

//     #[contracttype]
//     pub enum DataKey {
//         Prices,
//         Decimals,
//     }

//     #[contract]
//     pub struct MockOracleContract;

//     #[contractimpl]
//     impl MockOracleContract {
//         pub fn __constructor(e: Env, decimals: u32) {
//             e.storage().instance().set(&DataKey::Decimals, &decimals);
//         }

//         pub fn set_price(e: Env, asset: Asset, price: i128, timestamp: u64) {
//             let mut prices: Map<Asset, PriceData> =
//                 e.storage().instance().get(&DataKey::Prices).unwrap_or_else(|| Map::new(&e));
//             prices.set(asset, PriceData { price, timestamp });
//             e.storage().instance().set(&DataKey::Prices, &prices);
//         }
//     }

//     #[contractimpl]
//     impl PriceFeedTrait for MockOracleContract {
//         fn base(_e: Env) -> Asset {
//             Asset::Other(soroban_sdk::Symbol::new(&_e, "USD"))
//         }

//         fn assets(e: Env) -> Vec<Asset> {
//             let prices: Map<Asset, PriceData> =
//                 e.storage().instance().get(&DataKey::Prices).unwrap_or_else(|| Map::new(&e));
//             prices.keys()
//         }

//         fn decimals(e: Env) -> u32 {
//             e.storage().instance().get(&DataKey::Decimals).unwrap()
//         }

//         fn lastprice(e: Env, asset: Asset) -> Option<PriceData> {
//             let prices: Map<Asset, PriceData> =
//                 e.storage().instance().get(&DataKey::Prices).unwrap_or_else(|| Map::new(&e));
//             prices.get(asset)
//         }

//         fn resolution(_e: Env) -> u32 {
//             300
//         }

//         fn price(_e: Env, _asset: Asset, _timestamp: u64) -> Option<PriceData> {
//             unimplemented!()
//         }

//         fn prices(_e: Env, _asset: Asset, _records: u32) -> Option<Vec<PriceData>> {
//             unimplemented!()
//         }
//     }
// }

// #[test]
// fn test_oracle_deposit_cap_under_limit() {
//     let s = TestFarmsSetup::new();

//     let oracle_address = s.e.register(mock_oracle::MockOracleContract, (14_u32,));
//     let oracle_client = mock_oracle::MockOracleContractClient::new(&s.e, &oracle_address);

//     let price = 2_00000000000000_i128; // $2.00 with 14 decimals
//     oracle_client.set_price(
//         &sep_40_oracle::Asset::Stellar(s.stake_token.clone()),
//         &price,
//         &s.current_ts(),
//     );

//     let mut config = s.default_non_delegated_config();
//     config.deposit_cap = 500_0000000; // $500 USD (token has 7 decimals, price/10^oracle_decimals normalizes)
//     config.oracle = OptionalOracle::Some(OracleConfig {
//         oracle_address: oracle_address.clone(),
//         oracle_max_age: 3600,
//     });

//     let client = s.create_farm_with_rewards(config);

//     let fk = s.fk(&s.users[0]);
//     // 200 tokens * $2 = $400 USD, under $500 cap
//     client.stake(&fk, &200_0000000);

//     let pos = client.get_farming_position(&fk);
//     assert_eq!(pos.active_stake, 200_0000000);
// }

// #[test]
// fn test_oracle_deposit_cap_exceeded() {
//     let s = TestFarmsSetup::new();

//     let oracle_address = s.e.register(mock_oracle::MockOracleContract, (14_u32,));
//     let oracle_client = mock_oracle::MockOracleContractClient::new(&s.e, &oracle_address);

//     let price = 2_00000000000000_i128; // $2.00 with 14 decimals
//     oracle_client.set_price(
//         &sep_40_oracle::Asset::Stellar(s.stake_token.clone()),
//         &price,
//         &s.current_ts(),
//     );

//     let mut config = s.default_non_delegated_config();
//     config.deposit_cap = 500_0000000; // $500 USD
//     config.oracle = OptionalOracle::Some(OracleConfig {
//         oracle_address: oracle_address.clone(),
//         oracle_max_age: 3600,
//     });

//     let client = s.create_farm_with_rewards(config);

//     let fk = s.fk(&s.users[0]);
//     // 300 tokens * $2 = $600 USD, exceeds $500 cap
//     let result = client.try_stake(&fk, &300_0000000);
//     assert!(result.is_err());
// }

// #[test]
// fn test_oracle_stale_price_rejected() {
//     let s = TestFarmsSetup::new();

//     let oracle_address = s.e.register(mock_oracle::MockOracleContract, (14_u32,));
//     let oracle_client = mock_oracle::MockOracleContractClient::new(&s.e, &oracle_address);

//     let stale_ts = s.current_ts().saturating_sub(7200);
//     let price = 2_00000000000000_i128;
//     oracle_client.set_price(
//         &sep_40_oracle::Asset::Stellar(s.stake_token.clone()),
//         &price,
//         &stale_ts,
//     );

//     let mut config = s.default_non_delegated_config();
//     config.deposit_cap = 500_0000000;
//     config.oracle = OptionalOracle::Some(OracleConfig {
//         oracle_address: oracle_address.clone(),
//         oracle_max_age: 3600,
//     });

//     let client = s.create_farm_with_rewards(config);

//     let fk = s.fk(&s.users[0]);
//     let result = client.try_stake(&fk, &100_0000000);
//     assert!(result.is_err());
// }
