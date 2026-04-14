import { Buffer } from "buffer";
import { Address } from "@stellar/stellar-sdk";
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from "@stellar/stellar-sdk/contract";
import type {
  u32,
  i32,
  u64,
  i64,
  u128,
  i128,
  u256,
  i256,
  Option,
  Timepoint,
  Duration,
} from "@stellar/stellar-sdk/contract";
export * from "@stellar/stellar-sdk";
export * as contract from "@stellar/stellar-sdk/contract";
export * as rpc from "@stellar/stellar-sdk/rpc";

if (typeof window !== "undefined") {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export const LiquidityPoolRouterError = {
  301: {message:"PoolNotFound"},
  302: {message:"BadFee"},
  303: {message:"StableswapHashMissing"},
  305: {message:"PoolsOverMax"},
  306: {message:"StableswapPoolsOverMax"},
  307: {message:"PathIsEmpty"},
  308: {message:"TokensAreNotForReward"},
  309: {message:"LiquidityNotFilled"},
  310: {message:"LiquidityAlreadyFilled"},
  311: {message:"VotingShareExceedsMax"},
  312: {message:"LiquidityCalculationError"},
  313: {message:"RewardsNotConfigured"},
  314: {message:"RewardsAlreadyConfigured"},
  315: {message:"DuplicatesNotAllowed"},
  316: {message:"InvalidPoolType"},
  317: {message:"RewardDurationTooShort"},
  318: {message:"RewardAmountTooLow"},
  319: {message:"GaugeRewardsDisabledForPool"},
  320: {message:"UnsupportedTokensNum"},
  321: {message:"PathMustEndWithRewardToken"},
  322: {message:"ConcentratedPoolsOverMax"},
  323: {message:"RewardTokenChangeWhileActive"},
  324: {message:"SwapChainAmountIsZero"},
  2002: {message:"TokensNotSorted"},
  2020: {message:"InMaxNotSatisfied"}
}

export const PoolError = {
  401: {message:"PoolAlreadyExists"},
  404: {message:"PoolNotFound"}
}


export interface LiquidityPoolData {
  address: string;
  pool_type: LiquidityPoolType;
}

export enum LiquidityPoolType {
  MissingPool = 0,
  ConstantProduct = 1,
  StableSwap = 2,
  Concentrated = 3,
}


export interface GlobalRewardsConfig {
  expired_at: u64;
  tps: u128;
}


export interface LiquidityPoolRewardInfo {
  processed: boolean;
  total_liquidity: u256;
  voting_share: u32;
}

export const AccessControlError = {
  101: {message:"RoleNotFound"},
  102: {message:"Unauthorized"},
  103: {message:"AdminAlreadySet"},
  104: {message:"BadRoleUsage"},
  2906: {message:"AnotherActionActive"},
  2907: {message:"NoActionActive"},
  2908: {message:"ActionNotReadyYet"}
}

export type WASMDataKey = {tag: "TokenHash", values: void} | {tag: "TokenFutureWASM", values: void} | {tag: "GaugeWASM", values: void} | {tag: "FutureGaugeWASM", values: void} | {tag: "ConstantPoolHash", values: void} | {tag: "StableSwapPoolHash", values: void};

export const RewardsError = {
  701: {message:"PastTimeNotAllowed"},
  702: {message:"SameRewardsConfig"}
}


export interface PoolRewardData {
  accumulated: u128;
  block: u64;
  claimed: u128;
  last_time: u64;
}


export interface UserRewardData {
  last_block: u64;
  pool_accumulated: u128;
  to_claim: u128;
}


export interface PoolRewardConfig {
  expired_at: u64;
  tps: u128;
}

export const Errors = {
  2906: {message:"AnotherActionActive"},
  2907: {message:"NoActionActive"},
  2908: {message:"ActionNotReadyYet"}
}

export const MathError = {
  510: {message:"NumberOverflow"}
}

export const StorageError = {
  201: {message:"AlreadyInitialized"},
  501: {message:"ValueNotInitialized"},
  502: {message:"ValueMissing"},
  503: {message:"ValueConversionError"}
}

export interface Client {
  /**
   * Construct and simulate a swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap: ({user, tokens, token_in, token_out, pool_index, in_amount, out_min}: {user: string, tokens: Array<string>, token_in: string, token_out: string, pool_index: Buffer, in_amount: u128, out_min: u128}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a claim transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim: ({user, tokens, pool_index}: {user: string, tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit: ({user, tokens, pool_index, desired_amounts, min_shares}: {user: string, tokens: Array<string>, pool_index: Buffer, desired_amounts: Array<u128>, min_shares: u128}, options?: MethodOptions) => Promise<AssembledTransaction<readonly [Array<u128>, u128]>>

  /**
   * Construct and simulate a version transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  version: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_info: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Map<string, any>>>

  /**
   * Construct and simulate a get_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a share_id transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  share_id: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw: ({user, tokens, pool_index, share_amount, min_amounts}: {user: string, tokens: Array<string>, pool_index: Buffer, share_amount: u128, min_amounts: Array<u128>}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u128>>>

  /**
   * Construct and simulate a get_plane transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_plane: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_pools transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pools: ({tokens}: {tokens: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Map<Buffer, string>>>

  /**
   * Construct and simulate a pool_type transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pool_type: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_tokens: ({index}: {index: u128}, options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a init_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  init_admin: ({account}: {account: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a remove_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_pool: ({user, tokens, pool_hash}: {user: string, tokens: Array<string>, pool_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_reserves transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_reserves: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Array<u128>>>

  /**
   * Construct and simulate a swap_chained transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap_chained: ({user, swaps_chain, token_in, in_amount, out_min}: {user: string, swaps_chain: Array<readonly [Array<string>, Buffer, string]>, token_in: string, in_amount: u128, out_min: u128}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a apply_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_upgrade: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Buffer>>

  /**
   * Construct and simulate a contract_name transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  contract_name: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a estimate_swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  estimate_swap: ({tokens, token_in, token_out, pool_index, in_amount}: {tokens: Array<string>, token_in: string, token_out: string, pool_index: Buffer, in_amount: u128}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_liquidity transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_liquidity: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u256>>

  /**
   * Construct and simulate a set_pool_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_pool_hash: ({admin, new_hash}: {admin: string, new_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a commit_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  commit_upgrade: ({admin, new_wasm_hash}: {admin: string, new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a fill_liquidity transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  fill_liquidity: ({admin, tokens}: {admin: string, tokens: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a revert_upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  revert_upgrade: ({admin}: {admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_token_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_token_hash: ({admin, new_hash}: {admin: string, new_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_user_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_reward: ({user, tokens, pool_index}: {user: string, tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a set_pools_plane transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_pools_plane: ({admin, plane}: {admin: string, plane: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_rewards_info transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_rewards_info: ({user, tokens, pool_index}: {user: string, tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Map<string, i128>>>

  /**
   * Construct and simulate a get_total_shares transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_shares: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a set_reward_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_reward_token: ({admin, reward_token}: {admin: string, reward_token: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_emergency_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_emergency_mode: (options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a get_future_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_future_address: ({role_name}: {role_name: string}, options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_rewards_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_rewards_config: (options?: MethodOptions) => Promise<AssembledTransaction<Map<string, i128>>>

  /**
   * Construct and simulate a init_standard_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  init_standard_pool: ({user, tokens, fee_fraction}: {user: string, tokens: Array<string>, fee_fraction: u32}, options?: MethodOptions) => Promise<AssembledTransaction<readonly [Buffer, string]>>

  /**
   * Construct and simulate a set_emergency_mode transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_emergency_mode: ({emergency_admin, value}: {emergency_admin: string, value: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a config_pool_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  config_pool_rewards: ({admin, tokens, pool_index}: {admin: string, tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_total_liquidity transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_liquidity: ({tokens}: {tokens: Array<string>}, options?: MethodOptions) => Promise<AssembledTransaction<u256>>

  /**
   * Construct and simulate a init_config_storage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  init_config_storage: ({admin, config_storage}: {admin: string, config_storage: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_privileged_addrs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_privileged_addrs: (options?: MethodOptions) => Promise<AssembledTransaction<Map<string, Array<string>>>>

  /**
   * Construct and simulate a init_stableswap_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  init_stableswap_pool: ({user, tokens, fee_fraction}: {user: string, tokens: Array<string>, fee_fraction: u32}, options?: MethodOptions) => Promise<AssembledTransaction<readonly [Buffer, string]>>

  /**
   * Construct and simulate a set_privileged_addrs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_privileged_addrs: ({admin, rewards_admin, operations_admin, pause_admin, emergency_pause_admins, system_fee_admin}: {admin: string, rewards_admin: string, operations_admin: string, pause_admin: string, emergency_pause_admins: Array<string>, system_fee_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a config_global_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  config_global_rewards: ({user, reward_tps, expired_at, tokens_votes}: {user: string, reward_tps: u128, expired_at: u64, tokens_votes: Array<readonly [Array<string>, u32]>}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_tokens_for_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_tokens_for_reward: (options?: MethodOptions) => Promise<AssembledTransaction<Map<Array<string>, readonly [u32, boolean, u256]>>>

  /**
   * Construct and simulate a get_tokens_sets_count transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_tokens_sets_count: (options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a init_concentrated_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  init_concentrated_pool: ({user, tokens, fee}: {user: string, tokens: Array<string>, fee: u32}, options?: MethodOptions) => Promise<AssembledTransaction<readonly [Buffer, string]>>

  /**
   * Construct and simulate a set_rewards_gauge_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_rewards_gauge_hash: ({admin, new_hash}: {admin: string, new_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a pool_gauge_switch_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pool_gauge_switch_token: ({admin, token, enabled}: {admin: string, token: string, enabled: boolean}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_reward_boost_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_reward_boost_config: ({admin, reward_boost_token, reward_boost_feed}: {admin: string, reward_boost_token: string, reward_boost_feed: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a apply_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_transfer_ownership: ({admin, role_name}: {admin: string, role_name: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_liquidity_calculator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_liquidity_calculator: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_total_claimed_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_claimed_reward: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a pool_gauge_token_enabled transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pool_gauge_token_enabled: ({token}: {token: string}, options?: MethodOptions) => Promise<AssembledTransaction<boolean>>

  /**
   * Construct and simulate a set_liquidity_calculator transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_liquidity_calculator: ({admin, calculator}: {admin: string, calculator: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_stableswap_pool_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_stableswap_pool_hash: ({admin, new_hash}: {admin: string, new_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a commit_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  commit_transfer_ownership: ({admin, role_name, new_address}: {admin: string, role_name: string, new_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_protocol_fee_fraction transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_protocol_fee_fraction: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a revert_transfer_ownership transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  revert_transfer_ownership: ({admin, role_name}: {admin: string, role_name: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a set_protocol_fee_fraction transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_protocol_fee_fraction: ({admin, new_fraction}: {admin: string, new_fraction: u32}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_pools_for_tokens_range transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pools_for_tokens_range: ({start, end}: {start: u128, end: u128}, options?: MethodOptions) => Promise<AssembledTransaction<Array<readonly [Array<string>, Map<Buffer, string>]>>>

  /**
   * Construct and simulate a pool_gauge_schedule_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pool_gauge_schedule_reward: ({distributor, pool_tokens, pool_hash, distribute_token, tps, start_at, duration, swaps_chain_proof}: {distributor: string, pool_tokens: Array<string>, pool_hash: Buffer, distribute_token: string, tps: u128, start_at: Option<u64>, duration: u64, swaps_chain_proof: Array<readonly [Array<string>, Buffer, string]>}, options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a set_concentrated_pool_hash transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_concentrated_pool_hash: ({admin, new_hash}: {admin: string, new_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a configure_init_pool_payment transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  configure_init_pool_payment: ({admin, token, standard_pool_amount, stable_pool_amount, concentrated_pool_amount, to}: {admin: string, token: string, standard_pool_amount: u128, stable_pool_amount: u128, concentrated_pool_amount: u128, to: string}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_init_pool_payment_token transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_init_pool_payment_token: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_total_configured_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_configured_reward: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a pool_gauge_get_min_duration transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pool_gauge_get_min_duration: (options?: MethodOptions) => Promise<AssembledTransaction<u64>>

  /**
   * Construct and simulate a swap_chained_strict_receive transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap_chained_strict_receive: ({user, swaps_chain, token_in, out_amount, max_in}: {user: string, swaps_chain: Array<readonly [Array<string>, Buffer, string]>, token_in: string, out_amount: u128, max_in: u128}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_conc_pool_payment_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_conc_pool_payment_amount: (options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_total_accumulated_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_accumulated_reward: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_total_outstanding_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_total_outstanding_reward: ({tokens, pool_index}: {tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a distribute_outstanding_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  distribute_outstanding_reward: ({user, from, tokens, pool_index}: {user: string, from: string, tokens: Array<string>, pool_index: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_init_pool_payment_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_init_pool_payment_address: (options?: MethodOptions) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a get_stable_pool_payment_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_stable_pool_payment_amount: (options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a pool_gauge_get_min_daily_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pool_gauge_get_min_daily_amount: (options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a get_standard_pool_payment_amount transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_standard_pool_payment_amount: (options?: MethodOptions) => Promise<AssembledTransaction<u128>>

  /**
   * Construct and simulate a pool_gauge_set_reward_thresholds transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  pool_gauge_set_reward_thresholds: ({admin, min_reward_equivalent_day, min_duration_seconds}: {admin: string, min_reward_equivalent_day: u128, min_duration_seconds: u64}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
    /** Options for initializing a Client as well as for calling a method, with extras specific to deploying. */
    options: MethodOptions &
      Omit<ContractClientOptions, "contractId"> & {
        /** The hash of the Wasm blob, which must already be installed on-chain. */
        wasmHash: Buffer | string;
        /** Salt used to generate the contract's ID. Passed through to {@link Operation.createCustomContract}. Default: random. */
        salt?: Buffer | Uint8Array;
        /** The format used to decode `wasmHash`, if it's provided as a string. */
        format?: "hex" | "base64";
      }
  ): Promise<AssembledTransaction<T>> {
    return ContractClient.deploy(null, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAABAAAAAAAAAAAAAAAGExpcXVpZGl0eVBvb2xSb3V0ZXJFcnJvcgAAABkAAAAAAAAADFBvb2xOb3RGb3VuZAAAAS0AAAAAAAAABkJhZEZlZQAAAAABLgAAAAAAAAAVU3RhYmxlc3dhcEhhc2hNaXNzaW5nAAAAAAABLwAAAAAAAAAMUG9vbHNPdmVyTWF4AAABMQAAAAAAAAAWU3RhYmxlc3dhcFBvb2xzT3Zlck1heAAAAAABMgAAAAAAAAALUGF0aElzRW1wdHkAAAABMwAAAAAAAAAVVG9rZW5zQXJlTm90Rm9yUmV3YXJkAAAAAAABNAAAAAAAAAASTGlxdWlkaXR5Tm90RmlsbGVkAAAAAAE1AAAAAAAAABZMaXF1aWRpdHlBbHJlYWR5RmlsbGVkAAAAAAE2AAAAAAAAABVWb3RpbmdTaGFyZUV4Y2VlZHNNYXgAAAAAAAE3AAAAAAAAABlMaXF1aWRpdHlDYWxjdWxhdGlvbkVycm9yAAAAAAABOAAAAAAAAAAUUmV3YXJkc05vdENvbmZpZ3VyZWQAAAE5AAAAAAAAABhSZXdhcmRzQWxyZWFkeUNvbmZpZ3VyZWQAAAE6AAAAAAAAABREdXBsaWNhdGVzTm90QWxsb3dlZAAAATsAAAAAAAAAD0ludmFsaWRQb29sVHlwZQAAAAE8AAAAAAAAABZSZXdhcmREdXJhdGlvblRvb1Nob3J0AAAAAAE9AAAAAAAAABJSZXdhcmRBbW91bnRUb29Mb3cAAAAAAT4AAAAAAAAAG0dhdWdlUmV3YXJkc0Rpc2FibGVkRm9yUG9vbAAAAAE/AAAAAAAAABRVbnN1cHBvcnRlZFRva2Vuc051bQAAAUAAAAAAAAAAGlBhdGhNdXN0RW5kV2l0aFJld2FyZFRva2VuAAAAAAFBAAAAAAAAABhDb25jZW50cmF0ZWRQb29sc092ZXJNYXgAAAFCAAAAAAAAABxSZXdhcmRUb2tlbkNoYW5nZVdoaWxlQWN0aXZlAAABQwAAAAAAAAAVU3dhcENoYWluQW1vdW50SXNaZXJvAAAAAAABRAAAAAAAAAAPVG9rZW5zTm90U29ydGVkAAAAB9IAAAAAAAAAEUluTWF4Tm90U2F0aXNmaWVkAAAAAAAH5A==",
        "AAAABAAAAAAAAAAAAAAACVBvb2xFcnJvcgAAAAAAAAIAAAAAAAAAEVBvb2xBbHJlYWR5RXhpc3RzAAAAAAABkQAAAAAAAAAMUG9vbE5vdEZvdW5kAAABlA==",
        "AAAAAQAAAAAAAAAAAAAAEUxpcXVpZGl0eVBvb2xEYXRhAAAAAAAAAgAAAAAAAAAHYWRkcmVzcwAAAAATAAAAAAAAAAlwb29sX3R5cGUAAAAAAAfQAAAAEUxpcXVpZGl0eVBvb2xUeXBlAAAA",
        "AAAAAwAAAAAAAAAAAAAAEUxpcXVpZGl0eVBvb2xUeXBlAAAAAAAABAAAAAAAAAALTWlzc2luZ1Bvb2wAAAAAAAAAAAAAAAAPQ29uc3RhbnRQcm9kdWN0AAAAAAEAAAAAAAAAClN0YWJsZVN3YXAAAAAAAAIAAAAAAAAADENvbmNlbnRyYXRlZAAAAAM=",
        "AAAAAQAAAAAAAAAAAAAAE0dsb2JhbFJld2FyZHNDb25maWcAAAAAAgAAAAAAAAAKZXhwaXJlZF9hdAAAAAAABgAAAAAAAAADdHBzAAAAAAo=",
        "AAAAAQAAAAAAAAAAAAAAF0xpcXVpZGl0eVBvb2xSZXdhcmRJbmZvAAAAAAMAAAAAAAAACXByb2Nlc3NlZAAAAAAAAAEAAAAAAAAAD3RvdGFsX2xpcXVpZGl0eQAAAAAMAAAAAAAAAAx2b3Rpbmdfc2hhcmUAAAAE",
        "AAAAAAAAAAAAAAAEc3dhcAAAAAcAAAAAAAAABHVzZXIAAAATAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAAAAAAh0b2tlbl9pbgAAABMAAAAAAAAACXRva2VuX291dAAAAAAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAAAAAAlpbl9hbW91bnQAAAAAAAAKAAAAAAAAAAdvdXRfbWluAAAAAAoAAAABAAAACg==",
        "AAAAAAAAAAAAAAAFY2xhaW0AAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAKcG9vbF9pbmRleAAAAAAD7gAAACAAAAABAAAACg==",
        "AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAAFAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAKcG9vbF9pbmRleAAAAAAD7gAAACAAAAAAAAAAD2Rlc2lyZWRfYW1vdW50cwAAAAPqAAAACgAAAAAAAAAKbWluX3NoYXJlcwAAAAAACgAAAAEAAAPtAAAAAgAAA+oAAAAKAAAACg==",
        "AAAAAAAAAAAAAAAHdmVyc2lvbgAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAIZ2V0X2luZm8AAAACAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAAAAAApwb29sX2luZGV4AAAAAAPuAAAAIAAAAAEAAAPsAAAAEQAAAAA=",
        "AAAAAAAAAAAAAAAIZ2V0X3Bvb2wAAAACAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAAAAAApwb29sX2luZGV4AAAAAAPuAAAAIAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAIc2hhcmVfaWQAAAACAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAAAAAApwb29sX2luZGV4AAAAAAPuAAAAIAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAId2l0aGRyYXcAAAAFAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAKcG9vbF9pbmRleAAAAAAD7gAAACAAAAAAAAAADHNoYXJlX2Ftb3VudAAAAAoAAAAAAAAAC21pbl9hbW91bnRzAAAAA+oAAAAKAAAAAQAAA+oAAAAK",
        "AAAAAAAAAAAAAAAJZ2V0X3BsYW5lAAAAAAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAJZ2V0X3Bvb2xzAAAAAAAAAQAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAEAAAPsAAAD7gAAACAAAAAT",
        "AAAAAAAAAAAAAAAJcG9vbF90eXBlAAAAAAAAAgAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAKcG9vbF9pbmRleAAAAAAD7gAAACAAAAABAAAAEQ==",
        "AAAAAAAAAAAAAAAKZ2V0X3Rva2VucwAAAAAAAQAAAAAAAAAFaW5kZXgAAAAAAAAKAAAAAQAAA+oAAAAT",
        "AAAAAAAAAAAAAAAKaW5pdF9hZG1pbgAAAAAAAQAAAAAAAAAHYWNjb3VudAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAALcmVtb3ZlX3Bvb2wAAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACXBvb2xfaGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAMZ2V0X3Jlc2VydmVzAAAAAgAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAKcG9vbF9pbmRleAAAAAAD7gAAACAAAAABAAAD6gAAAAo=",
        "AAAAAAAAAAAAAAAMc3dhcF9jaGFpbmVkAAAABQAAAAAAAAAEdXNlcgAAABMAAAAAAAAAC3N3YXBzX2NoYWluAAAAA+oAAAPtAAAAAwAAA+oAAAATAAAD7gAAACAAAAATAAAAAAAAAAh0b2tlbl9pbgAAABMAAAAAAAAACWluX2Ftb3VudAAAAAAAAAoAAAAAAAAAB291dF9taW4AAAAACgAAAAEAAAAK",
        "AAAAAAAAAAAAAAANYXBwbHlfdXBncmFkZQAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPuAAAAIA==",
        "AAAAAAAAAAAAAAANY29udHJhY3RfbmFtZQAAAAAAAAAAAAABAAAAEQ==",
        "AAAAAAAAAAAAAAANZXN0aW1hdGVfc3dhcAAAAAAAAAUAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACHRva2VuX2luAAAAEwAAAAAAAAAJdG9rZW5fb3V0AAAAAAAAEwAAAAAAAAAKcG9vbF9pbmRleAAAAAAD7gAAACAAAAAAAAAACWluX2Ftb3VudAAAAAAAAAoAAAABAAAACg==",
        "AAAAAAAAAAAAAAANZ2V0X2xpcXVpZGl0eQAAAAAAAAIAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAQAAAAw=",
        "AAAAAAAAAAAAAAANc2V0X3Bvb2xfaGFzaAAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIbmV3X2hhc2gAAAPuAAAAIAAAAAA=",
        "AAAAAAAAAAAAAAAOY29tbWl0X3VwZ3JhZGUAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAOZmlsbF9saXF1aWRpdHkAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAOcmV2ZXJ0X3VwZ3JhZGUAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAOc2V0X3Rva2VuX2hhc2gAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIbmV3X2hhc2gAAAPuAAAAIAAAAAA=",
        "AAAAAAAAAAAAAAAPZ2V0X3VzZXJfcmV3YXJkAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAAAAAApwb29sX2luZGV4AAAAAAPuAAAAIAAAAAEAAAAK",
        "AAAAAAAAAAAAAAAPc2V0X3Bvb2xzX3BsYW5lAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFcGxhbmUAAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAQZ2V0X3Jld2FyZHNfaW5mbwAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAAAAAApwb29sX2luZGV4AAAAAAPuAAAAIAAAAAEAAAPsAAAAEQAAAAs=",
        "AAAAAAAAAAAAAAAQZ2V0X3RvdGFsX3NoYXJlcwAAAAIAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAQc2V0X3Jld2FyZF90b2tlbgAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAMcmV3YXJkX3Rva2VuAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAASZ2V0X2VtZXJnZW5jeV9tb2RlAAAAAAAAAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAASZ2V0X2Z1dHVyZV9hZGRyZXNzAAAAAAABAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAQAAABM=",
        "AAAAAAAAAAAAAAASZ2V0X3Jld2FyZHNfY29uZmlnAAAAAAAAAAAAAQAAA+wAAAARAAAACw==",
        "AAAAAAAAAAAAAAASaW5pdF9zdGFuZGFyZF9wb29sAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAMZmVlX2ZyYWN0aW9uAAAABAAAAAEAAAPtAAAAAgAAA+4AAAAgAAAAEw==",
        "AAAAAAAAAAAAAAASc2V0X2VtZXJnZW5jeV9tb2RlAAAAAAACAAAAAAAAAA9lbWVyZ2VuY3lfYWRtaW4AAAAAEwAAAAAAAAAFdmFsdWUAAAAAAAABAAAAAA==",
        "AAAAAAAAAAAAAAATY29uZmlnX3Bvb2xfcmV3YXJkcwAAAAADAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAATZ2V0X3RvdGFsX2xpcXVpZGl0eQAAAAABAAAAAAAAAAZ0b2tlbnMAAAAAA+oAAAATAAAAAQAAAAw=",
        "AAAAAAAAAAAAAAATaW5pdF9jb25maWdfc3RvcmFnZQAAAAACAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADmNvbmZpZ19zdG9yYWdlAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAUZ2V0X3ByaXZpbGVnZWRfYWRkcnMAAAAAAAAAAQAAA+wAAAARAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAUaW5pdF9zdGFibGVzd2FwX3Bvb2wAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAMZmVlX2ZyYWN0aW9uAAAABAAAAAEAAAPtAAAAAgAAA+4AAAAgAAAAEw==",
        "AAAAAAAAAAAAAAAUc2V0X3ByaXZpbGVnZWRfYWRkcnMAAAAGAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAADXJld2FyZHNfYWRtaW4AAAAAAAATAAAAAAAAABBvcGVyYXRpb25zX2FkbWluAAAAEwAAAAAAAAALcGF1c2VfYWRtaW4AAAAAEwAAAAAAAAAWZW1lcmdlbmN5X3BhdXNlX2FkbWlucwAAAAAD6gAAABMAAAAAAAAAEHN5c3RlbV9mZWVfYWRtaW4AAAATAAAAAA==",
        "AAAAAAAAAAAAAAAVY29uZmlnX2dsb2JhbF9yZXdhcmRzAAAAAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAACnJld2FyZF90cHMAAAAAAAoAAAAAAAAACmV4cGlyZWRfYXQAAAAAAAYAAAAAAAAADHRva2Vuc192b3RlcwAAA+oAAAPtAAAAAgAAA+oAAAATAAAABAAAAAA=",
        "AAAAAAAAAAAAAAAVZ2V0X3Rva2Vuc19mb3JfcmV3YXJkAAAAAAAAAAAAAAEAAAPsAAAD6gAAABMAAAPtAAAAAwAAAAQAAAABAAAADA==",
        "AAAAAAAAAAAAAAAVZ2V0X3Rva2Vuc19zZXRzX2NvdW50AAAAAAAAAAAAAAEAAAAK",
        "AAAAAAAAAAAAAAAWaW5pdF9jb25jZW50cmF0ZWRfcG9vbAAAAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAAA2ZlZQAAAAAEAAAAAQAAA+0AAAACAAAD7gAAACAAAAAT",
        "AAAAAAAAAAAAAAAWc2V0X3Jld2FyZHNfZ2F1Z2VfaGFzaAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAhuZXdfaGFzaAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAXcG9vbF9nYXVnZV9zd2l0Y2hfdG9rZW4AAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAV0b2tlbgAAAAAAABMAAAAAAAAAB2VuYWJsZWQAAAAAAQAAAAA=",
        "AAAAAAAAAAAAAAAXc2V0X3Jld2FyZF9ib29zdF9jb25maWcAAAAAAwAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAABJyZXdhcmRfYm9vc3RfdG9rZW4AAAAAABMAAAAAAAAAEXJld2FyZF9ib29zdF9mZWVkAAAAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAYYXBwbHlfdHJhbnNmZXJfb3duZXJzaGlwAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAlyb2xlX25hbWUAAAAAAAARAAAAAA==",
        "AAAAAAAAAAAAAAAYZ2V0X2xpcXVpZGl0eV9jYWxjdWxhdG9yAAAAAAAAAAEAAAAT",
        "AAAAAAAAAAAAAAAYZ2V0X3RvdGFsX2NsYWltZWRfcmV3YXJkAAAAAgAAAAAAAAAGdG9rZW5zAAAAAAPqAAAAEwAAAAAAAAAKcG9vbF9pbmRleAAAAAAD7gAAACAAAAABAAAACg==",
        "AAAAAAAAAAAAAAAYcG9vbF9nYXVnZV90b2tlbl9lbmFibGVkAAAAAQAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAQAAAAE=",
        "AAAAAAAAAAAAAAAYc2V0X2xpcXVpZGl0eV9jYWxjdWxhdG9yAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAApjYWxjdWxhdG9yAAAAAAATAAAAAA==",
        "AAAAAAAAAAAAAAAYc2V0X3N0YWJsZXN3YXBfcG9vbF9oYXNoAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAhuZXdfaGFzaAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAAAAAAAAZY29tbWl0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAAAAAALbmV3X2FkZHJlc3MAAAAAEwAAAAA=",
        "AAAAAAAAAAAAAAAZZ2V0X3Byb3RvY29sX2ZlZV9mcmFjdGlvbgAAAAAAAAAAAAABAAAABA==",
        "AAAAAAAAAAAAAAAZcmV2ZXJ0X3RyYW5zZmVyX293bmVyc2hpcAAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAJcm9sZV9uYW1lAAAAAAAAEQAAAAA=",
        "AAAAAAAAAAAAAAAZc2V0X3Byb3RvY29sX2ZlZV9mcmFjdGlvbgAAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAMbmV3X2ZyYWN0aW9uAAAABAAAAAA=",
        "AAAAAAAAAAAAAAAaZ2V0X3Bvb2xzX2Zvcl90b2tlbnNfcmFuZ2UAAAAAAAIAAAAAAAAABXN0YXJ0AAAAAAAACgAAAAAAAAADZW5kAAAAAAoAAAABAAAD6gAAA+0AAAACAAAD6gAAABMAAAPsAAAD7gAAACAAAAAT",
        "AAAAAAAAAAAAAAAacG9vbF9nYXVnZV9zY2hlZHVsZV9yZXdhcmQAAAAAAAgAAAAAAAAAC2Rpc3RyaWJ1dG9yAAAAABMAAAAAAAAAC3Bvb2xfdG9rZW5zAAAAA+oAAAATAAAAAAAAAAlwb29sX2hhc2gAAAAAAAPuAAAAIAAAAAAAAAAQZGlzdHJpYnV0ZV90b2tlbgAAABMAAAAAAAAAA3RwcwAAAAAKAAAAAAAAAAhzdGFydF9hdAAAA+gAAAAGAAAAAAAAAAhkdXJhdGlvbgAAAAYAAAAAAAAAEXN3YXBzX2NoYWluX3Byb29mAAAAAAAD6gAAA+0AAAADAAAD6gAAABMAAAPuAAAAIAAAABMAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAac2V0X2NvbmNlbnRyYXRlZF9wb29sX2hhc2gAAAAAAAIAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIbmV3X2hhc2gAAAPuAAAAIAAAAAA=",
        "AAAAAAAAAAAAAAAbY29uZmlndXJlX2luaXRfcG9vbF9wYXltZW50AAAAAAYAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAAAAABRzdGFuZGFyZF9wb29sX2Ftb3VudAAAAAoAAAAAAAAAEnN0YWJsZV9wb29sX2Ftb3VudAAAAAAACgAAAAAAAAAYY29uY2VudHJhdGVkX3Bvb2xfYW1vdW50AAAACgAAAAAAAAACdG8AAAAAABMAAAAA",
        "AAAAAAAAAAAAAAAbZ2V0X2luaXRfcG9vbF9wYXltZW50X3Rva2VuAAAAAAAAAAABAAAAEw==",
        "AAAAAAAAAAAAAAAbZ2V0X3RvdGFsX2NvbmZpZ3VyZWRfcmV3YXJkAAAAAAIAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAbcG9vbF9nYXVnZV9nZXRfbWluX2R1cmF0aW9uAAAAAAAAAAABAAAABg==",
        "AAAAAAAAAAAAAAAbc3dhcF9jaGFpbmVkX3N0cmljdF9yZWNlaXZlAAAAAAUAAAAAAAAABHVzZXIAAAATAAAAAAAAAAtzd2Fwc19jaGFpbgAAAAPqAAAD7QAAAAMAAAPqAAAAEwAAA+4AAAAgAAAAEwAAAAAAAAAIdG9rZW5faW4AAAATAAAAAAAAAApvdXRfYW1vdW50AAAAAAAKAAAAAAAAAAZtYXhfaW4AAAAAAAoAAAABAAAACg==",
        "AAAAAAAAAAAAAAAcZ2V0X2NvbmNfcG9vbF9wYXltZW50X2Ftb3VudAAAAAAAAAABAAAACg==",
        "AAAAAAAAAAAAAAAcZ2V0X3RvdGFsX2FjY3VtdWxhdGVkX3Jld2FyZAAAAAIAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAcZ2V0X3RvdGFsX291dHN0YW5kaW5nX3Jld2FyZAAAAAIAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAdZGlzdHJpYnV0ZV9vdXRzdGFuZGluZ19yZXdhcmQAAAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAEZnJvbQAAABMAAAAAAAAABnRva2VucwAAAAAD6gAAABMAAAAAAAAACnBvb2xfaW5kZXgAAAAAA+4AAAAgAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAdZ2V0X2luaXRfcG9vbF9wYXltZW50X2FkZHJlc3MAAAAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAAAAAAAeZ2V0X3N0YWJsZV9wb29sX3BheW1lbnRfYW1vdW50AAAAAAAAAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAfcG9vbF9nYXVnZV9nZXRfbWluX2RhaWx5X2Ftb3VudAAAAAAAAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAgZ2V0X3N0YW5kYXJkX3Bvb2xfcGF5bWVudF9hbW91bnQAAAAAAAAAAQAAAAo=",
        "AAAAAAAAAAAAAAAgcG9vbF9nYXVnZV9zZXRfcmV3YXJkX3RocmVzaG9sZHMAAAADAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAAGW1pbl9yZXdhcmRfZXF1aXZhbGVudF9kYXkAAAAAAAAKAAAAAAAAABRtaW5fZHVyYXRpb25fc2Vjb25kcwAAAAYAAAAA",
        "AAAABAAAAAAAAAAAAAAAEkFjY2Vzc0NvbnRyb2xFcnJvcgAAAAAABwAAAAAAAAAMUm9sZU5vdEZvdW5kAAAAZQAAAAAAAAAMVW5hdXRob3JpemVkAAAAZgAAAAAAAAAPQWRtaW5BbHJlYWR5U2V0AAAAAGcAAAAAAAAADEJhZFJvbGVVc2FnZQAAAGgAAAAAAAAAE0Fub3RoZXJBY3Rpb25BY3RpdmUAAAALWgAAAAAAAAAOTm9BY3Rpb25BY3RpdmUAAAAAC1sAAAAAAAAAEUFjdGlvbk5vdFJlYWR5WWV0AAAAAAALXA==",
        "AAAAAgAAAAAAAAAAAAAAC1dBU01EYXRhS2V5AAAAAAYAAAAAAAAAAAAAAAlUb2tlbkhhc2gAAAAAAAAAAAAAAAAAAA9Ub2tlbkZ1dHVyZVdBU00AAAAAAAAAAAAAAAAJR2F1Z2VXQVNNAAAAAAAAAAAAAAAAAAAPRnV0dXJlR2F1Z2VXQVNNAAAAAAAAAAAAAAAAEENvbnN0YW50UG9vbEhhc2gAAAAAAAAAAAAAABJTdGFibGVTd2FwUG9vbEhhc2gAAA==",
        "AAAABAAAAAAAAAAAAAAADFJld2FyZHNFcnJvcgAAAAIAAAAAAAAAElBhc3RUaW1lTm90QWxsb3dlZAAAAAACvQAAAAAAAAARU2FtZVJld2FyZHNDb25maWcAAAAAAAK+",
        "AAAAAQAAAAAAAAAAAAAADlBvb2xSZXdhcmREYXRhAAAAAAAEAAAAAAAAAAthY2N1bXVsYXRlZAAAAAAKAAAAAAAAAAVibG9jawAAAAAAAAYAAAAAAAAAB2NsYWltZWQAAAAACgAAAAAAAAAJbGFzdF90aW1lAAAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADlVzZXJSZXdhcmREYXRhAAAAAAADAAAAAAAAAApsYXN0X2Jsb2NrAAAAAAAGAAAAAAAAABBwb29sX2FjY3VtdWxhdGVkAAAACgAAAAAAAAAIdG9fY2xhaW0AAAAK",
        "AAAAAQAAAAAAAAAAAAAAEFBvb2xSZXdhcmRDb25maWcAAAACAAAAAAAAAApleHBpcmVkX2F0AAAAAAAGAAAAAAAAAAN0cHMAAAAACg==",
        "AAAABAAAAAAAAAAAAAAABUVycm9yAAAAAAAAAwAAAAAAAAATQW5vdGhlckFjdGlvbkFjdGl2ZQAAAAtaAAAAAAAAAA5Ob0FjdGlvbkFjdGl2ZQAAAAALWwAAAAAAAAARQWN0aW9uTm90UmVhZHlZZXQAAAAAAAtc",
        "AAAABAAAAAAAAAAAAAAACU1hdGhFcnJvcgAAAAAAAAEAAAAAAAAADk51bWJlck92ZXJmbG93AAAAAAH+",
        "AAAABAAAAAAAAAAAAAAADFN0b3JhZ2VFcnJvcgAAAAQAAAAAAAAAEkFscmVhZHlJbml0aWFsaXplZAAAAAAAyQAAAAAAAAATVmFsdWVOb3RJbml0aWFsaXplZAAAAAH1AAAAAAAAAAxWYWx1ZU1pc3NpbmcAAAH2AAAAAAAAABRWYWx1ZUNvbnZlcnNpb25FcnJvcgAAAfc=" ]),
      options
    )
  }
  public readonly fromJSON = {
    swap: this.txFromJSON<u128>,
        claim: this.txFromJSON<u128>,
        deposit: this.txFromJSON<readonly [Array<u128>, u128]>,
        version: this.txFromJSON<u32>,
        get_info: this.txFromJSON<Map<string, any>>,
        get_pool: this.txFromJSON<string>,
        share_id: this.txFromJSON<string>,
        withdraw: this.txFromJSON<Array<u128>>,
        get_plane: this.txFromJSON<string>,
        get_pools: this.txFromJSON<Map<Buffer, string>>,
        pool_type: this.txFromJSON<string>,
        get_tokens: this.txFromJSON<Array<string>>,
        init_admin: this.txFromJSON<null>,
        remove_pool: this.txFromJSON<null>,
        get_reserves: this.txFromJSON<Array<u128>>,
        swap_chained: this.txFromJSON<u128>,
        apply_upgrade: this.txFromJSON<Buffer>,
        contract_name: this.txFromJSON<string>,
        estimate_swap: this.txFromJSON<u128>,
        get_liquidity: this.txFromJSON<u256>,
        set_pool_hash: this.txFromJSON<null>,
        commit_upgrade: this.txFromJSON<null>,
        fill_liquidity: this.txFromJSON<null>,
        revert_upgrade: this.txFromJSON<null>,
        set_token_hash: this.txFromJSON<null>,
        get_user_reward: this.txFromJSON<u128>,
        set_pools_plane: this.txFromJSON<null>,
        get_rewards_info: this.txFromJSON<Map<string, i128>>,
        get_total_shares: this.txFromJSON<u128>,
        set_reward_token: this.txFromJSON<null>,
        get_emergency_mode: this.txFromJSON<boolean>,
        get_future_address: this.txFromJSON<string>,
        get_rewards_config: this.txFromJSON<Map<string, i128>>,
        init_standard_pool: this.txFromJSON<readonly [Buffer, string]>,
        set_emergency_mode: this.txFromJSON<null>,
        config_pool_rewards: this.txFromJSON<u128>,
        get_total_liquidity: this.txFromJSON<u256>,
        init_config_storage: this.txFromJSON<null>,
        get_privileged_addrs: this.txFromJSON<Map<string, Array<string>>>,
        init_stableswap_pool: this.txFromJSON<readonly [Buffer, string]>,
        set_privileged_addrs: this.txFromJSON<null>,
        config_global_rewards: this.txFromJSON<null>,
        get_tokens_for_reward: this.txFromJSON<Map<Array<string>, readonly [u32, boolean, u256]>>,
        get_tokens_sets_count: this.txFromJSON<u128>,
        init_concentrated_pool: this.txFromJSON<readonly [Buffer, string]>,
        set_rewards_gauge_hash: this.txFromJSON<null>,
        pool_gauge_switch_token: this.txFromJSON<null>,
        set_reward_boost_config: this.txFromJSON<null>,
        apply_transfer_ownership: this.txFromJSON<null>,
        get_liquidity_calculator: this.txFromJSON<string>,
        get_total_claimed_reward: this.txFromJSON<u128>,
        pool_gauge_token_enabled: this.txFromJSON<boolean>,
        set_liquidity_calculator: this.txFromJSON<null>,
        set_stableswap_pool_hash: this.txFromJSON<null>,
        commit_transfer_ownership: this.txFromJSON<null>,
        get_protocol_fee_fraction: this.txFromJSON<u32>,
        revert_transfer_ownership: this.txFromJSON<null>,
        set_protocol_fee_fraction: this.txFromJSON<null>,
        get_pools_for_tokens_range: this.txFromJSON<Array<readonly [Array<string>, Map<Buffer, string>]>>,
        pool_gauge_schedule_reward: this.txFromJSON<string>,
        set_concentrated_pool_hash: this.txFromJSON<null>,
        configure_init_pool_payment: this.txFromJSON<null>,
        get_init_pool_payment_token: this.txFromJSON<string>,
        get_total_configured_reward: this.txFromJSON<u128>,
        pool_gauge_get_min_duration: this.txFromJSON<u64>,
        swap_chained_strict_receive: this.txFromJSON<u128>,
        get_conc_pool_payment_amount: this.txFromJSON<u128>,
        get_total_accumulated_reward: this.txFromJSON<u128>,
        get_total_outstanding_reward: this.txFromJSON<u128>,
        distribute_outstanding_reward: this.txFromJSON<u128>,
        get_init_pool_payment_address: this.txFromJSON<string>,
        get_stable_pool_payment_amount: this.txFromJSON<u128>,
        pool_gauge_get_min_daily_amount: this.txFromJSON<u128>,
        get_standard_pool_payment_amount: this.txFromJSON<u128>,
        pool_gauge_set_reward_thresholds: this.txFromJSON<null>
  }
}