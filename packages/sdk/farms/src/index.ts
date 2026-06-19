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





export interface RewardCurvePoint {
  reward_per_time_unit: i128;
  ts_start: u64;
}


export interface RewardScheduleCurve {
  /**
 * Points defining the curve (up to `[MAX_CURVE_POINTS]` and in ascending order)
 */
points: Array<RewardCurvePoint>;
}

export const FCError = {
  0: {message:"InternalError"},
  1: {message:"NegativeInputAmount"},
  2: {message:"OverOrUnderflow"},
  10: {message:"InvalidRewardScheduleCurve"},
  12: {message:"InvalidFarmConfigUpdate"},
  13: {message:"FarmDoesNotExist"},
  14: {message:"FarmingPositionDoesNotExist"},
  15: {message:"RewardDoesNotExistOnFarm"},
  16: {message:"TokenIsAlreadyAReward"},
  17: {message:"FarmIsFrozen"},
  18: {message:"DelegatedFarm"},
  19: {message:"NotDelegatedFarm"},
  20: {message:"MaxFarmNumRewardsReached"},
  21: {message:"RewardUserOnceIsDisabled"},
  22: {message:"InsufficientStake"},
  23: {message:"InsufficientPendingWithdrawal"},
  24: {message:"PendingWithdrawalExists"},
  25: {message:"DepositCapExceeded"},
  26: {message:"WarmupNotComplete"},
  27: {message:"CooldownNotComplete"},
  28: {message:"ClaimTooSoon"},
  29: {message:"InsufficientAvailableRewards"},
  30: {message:"NoRewardsToHarvest"},
  31: {message:"InsufficientCurrentSlashedAmount"},
  32: {message:"InvalidAmount"},
  33: {message:"InvalidConfig"},
  34: {message:"ProposedAdminDoesNotExist"},
  35: {message:"InsufficientTreasuryFees"},
  36: {message:"TransferAmountMismatch"},
  37: {message:"OraclePriceUnavailable"},
  38: {message:"OraclePriceStale"},
  39: {message:"UnauthorizedCaller"},
  40: {message:"NoPendingDeposit"},
  41: {message:"NotInitialized"},
  42: {message:"FarmAlreadyExists"}
}


export interface Farm {
  admin: string;
  config: FarmConfig;
  cumulative_slashed_amount: i128;
  current_slashed_amount: i128;
  id: Buffer;
  is_frozen: boolean;
  num_users: u64;
  proposed_admin: Option<string>;
  /**
 * Reward tokens in initialization order; `reward_index` points here.
 */
rewards: Array<string>;
  total_staked: i128;
}


/**
 * Full farm view returned by queries: the farm itself plus the state of
 * every initialized reward (ordered by `reward_index`).
 */
export interface FarmState {
  farm: Farm;
  rewards: Array<RewardInfo>;
}

export type Delegation = {tag: "Delegated", values: readonly [DelegatedFarmConfig]} | {tag: "NonDelegated", values: readonly [NonDelegatedFarmConfig]};


export interface FarmConfig {
  delegation: Delegation;
  deposit_cap: i128;
  is_harvest_permissionless: boolean;
  is_reward_once_enabled: boolean;
  min_harvest_delay: u64;
  min_stake_amount: i128;
  oracle: OptionalOracle;
  token: string;
  treasury_fee_bps: i128;
}


export interface RewardInfo {
  accum_rewards_per_share_sc: i128;
  accumulated_treasury_fees: i128;
  last_issuance_ts: u64;
  reward_schedule_curve: RewardScheduleCurve;
  reward_token: string;
  reward_type: RewardType;
  rewards_available: i128;
  rewards_issued_cumulative: i128;
  rewards_issued_unclaimed: i128;
}

export type RewardType = {tag: "Proportional", values: void} | {tag: "Constant", values: void};

export type LockingMode = {tag: "None", values: void} | {tag: "Continuous", values: void} | {tag: "WithExpiry", values: void};


/**
 * Contract-wide configuration: the global admin who can initialize farms
 * and upgrade the contract.
 */
export interface GlobalConfig {
  admin: string;
  /**
 * Internal counter used to derive farm IDs when no seed is provided.
 */
num_farms: u64;
  proposed_admin: Option<string>;
}


export interface OracleConfig {
  oracle_address: string;
  oracle_max_age: u64;
}


/**
 * Configuration for initializing a reward on a farm.
 */
export interface RewardConfig {
  reward_schedule_curve: RewardScheduleCurve;
  reward_token: string;
  reward_type: RewardType;
}


export interface DelegateeState {
  active_stake: i128;
  last_claim_ts: Map<string, u64>;
  last_stake_ts: u64;
  pending_deposit_stake: i128;
  pending_deposit_ts: u64;
  pending_withdrawal_stake: i128;
  pending_withdrawal_ts: u64;
  rewards_tallies: Map<string, i128>;
  rewards_unclaimed: Map<string, i128>;
}

export type OptionalOracle = {tag: "None", values: void} | {tag: "Some", values: readonly [OracleConfig]};


export interface DelegatedFarmConfig {
  /**
 * The platform (e.g. a lending market) authorized to push stake updates
 * via `set_stake_delegated`.
 */
delegate_authority: string;
}


export interface NonDelegatedFarmConfig {
  deposit_warmup_period: u64;
  early_withdrawal_penalty_bps: i128;
  locking_duration: u64;
  locking_mode: LockingMode;
  locking_ts: u64;
  withdrawal_cooldown_period: u64;
}






















export type DataKey = {tag: "GlobalConfig", values: void} | {tag: "Farms", values: void} | {tag: "Farm", values: readonly [Buffer]} | {tag: "RewardInfo", values: readonly [Buffer, string]} | {tag: "DelegateeState", values: readonly [Buffer, Delegatee]};


/**
 * Delegatee identifier for farm stakes.
 * 
 * Supports multiple stake identities per owner address:
 * - Simple: just owner address (for contracts where user has single position)
 * - With seed: owner address + seed (for contracts with multiple obligations per user)
 */
export interface Delegatee {
  /**
 * The owner's address
 */
owner: string;
  /**
 * Optional seed to distinguish multiple positions per owner
 */
seed: Option<Buffer>;
}

/**
 * Asset type
 */
export type Asset = {tag: "Stellar", values: readonly [string]} | {tag: "Other", values: readonly [string]};


/**
 * Price data for an asset at a specific timestamp
 */
export interface PriceData {
  price: i128;
  timestamp: u64;
}

export interface Client {
  /**
   * Construct and simulate a stake transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  stake: ({delegatee, farm_id, amount}: {delegatee: Delegatee, farm_id: Buffer, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a harvest transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  harvest: ({delegatee, farm_id, reward_index}: {delegatee: Delegatee, farm_id: Buffer, reward_index: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a unstake transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unstake: ({delegatee, farm_id, amount}: {delegatee: Delegatee, farm_id: Buffer, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_farm: ({farm_id}: {farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<FarmState>>>

  /**
   * Construct and simulate a add_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_rewards: ({funder, farm_id, reward_index, amount}: {funder: string, farm_id: Buffer, reward_index: u32, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a freeze_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  freeze_farm: ({farm_id}: {farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a reward_once transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  reward_once: ({delegatee, farm_id, reward_index, amount}: {delegatee: Delegatee, farm_id: Buffer, reward_index: u32, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_all_farms transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_farms: (options?: MethodOptions) => Promise<AssembledTransaction<Array<Buffer>>>

  /**
   * Construct and simulate a propose_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_admin: ({proposed_admin}: {proposed_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a unfreeze_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  unfreeze_farm: ({farm_id}: {farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_fees: ({farm_id, reward_index, amount}: {farm_id: Buffer, reward_index: u32, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_farm: ({seed, config}: {seed: Option<Buffer>, config: FarmConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Buffer>>>

  /**
   * Construct and simulate a withdraw_slashed transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_slashed: ({farm_id, amount}: {farm_id: Buffer, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_farm_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_farm_admin: ({farm_id}: {farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_global_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_global_config: (options?: MethodOptions) => Promise<AssembledTransaction<Result<GlobalConfig>>>

  /**
   * Construct and simulate a initialize_reward transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_reward: ({farm_id, reward_config}: {farm_id: Buffer, reward_config: RewardConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<u32>>>

  /**
   * Construct and simulate a withdraw_unstaked transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_unstaked: ({delegatee, farm_id}: {delegatee: Delegatee, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a propose_farm_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_farm_admin: ({farm_id, proposed_admin}: {farm_id: Buffer, proposed_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_farm_config transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_farm_config: ({farm_id, config_update}: {farm_id: Buffer, config_update: FarmConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_delegatee_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_delegatee_state: ({delegatee, farm_id}: {delegatee: Delegatee, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<DelegateeState>>>

  /**
   * Construct and simulate a get_pending_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pending_rewards: ({delegatee, farm_id}: {delegatee: Delegatee, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Array<readonly [string, i128]>>>>

  /**
   * Construct and simulate a set_reward_schedule transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_reward_schedule: ({farm_id, reward_index, curve}: {farm_id: Buffer, reward_index: u32, curve: RewardScheduleCurve}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_stake_delegated transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_stake_delegated: ({delegatee, farm_id, new_stake}: {delegatee: Delegatee, farm_id: Buffer, new_stake: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_delegatee_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_delegatee_state: ({delegatee, farm_id}: {delegatee: Delegatee, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw_unused_rewards transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_unused_rewards: ({farm_id, reward_index, amount, recipient}: {farm_id: Buffer, reward_index: u32, amount: i128, recipient: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin}: {admin: string},
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
    return ContractClient.deploy({admin}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAAEFJld2FyZEN1cnZlUG9pbnQAAAACAAAAAAAAABRyZXdhcmRfcGVyX3RpbWVfdW5pdAAAAAsAAAAAAAAACHRzX3N0YXJ0AAAABg==",
        "AAAAAQAAAAAAAAAAAAAAE1Jld2FyZFNjaGVkdWxlQ3VydmUAAAAAAQAAAE1Qb2ludHMgZGVmaW5pbmcgdGhlIGN1cnZlICh1cCB0byBgW01BWF9DVVJWRV9QT0lOVFNdYCBhbmQgaW4gYXNjZW5kaW5nIG9yZGVyKQAAAAAAAAZwb2ludHMAAAAAA+oAAAfQAAAAEFJld2FyZEN1cnZlUG9pbnQ=",
        "AAAABAAAAAAAAAAAAAAAB0ZDRXJyb3IAAAAAIwAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAE05lZ2F0aXZlSW5wdXRBbW91bnQAAAAAAQAAAAAAAAAPT3Zlck9yVW5kZXJmbG93AAAAAAIAAAAAAAAAGkludmFsaWRSZXdhcmRTY2hlZHVsZUN1cnZlAAAAAAAKAAAAAAAAABdJbnZhbGlkRmFybUNvbmZpZ1VwZGF0ZQAAAAAMAAAAAAAAABBGYXJtRG9lc05vdEV4aXN0AAAADQAAAAAAAAAbRmFybWluZ1Bvc2l0aW9uRG9lc05vdEV4aXN0AAAAAA4AAAAAAAAAGFJld2FyZERvZXNOb3RFeGlzdE9uRmFybQAAAA8AAAAAAAAAFVRva2VuSXNBbHJlYWR5QVJld2FyZAAAAAAAABAAAAAAAAAADEZhcm1Jc0Zyb3plbgAAABEAAAAAAAAADURlbGVnYXRlZEZhcm0AAAAAAAASAAAAAAAAABBOb3REZWxlZ2F0ZWRGYXJtAAAAEwAAAAAAAAAYTWF4RmFybU51bVJld2FyZHNSZWFjaGVkAAAAFAAAAAAAAAAYUmV3YXJkVXNlck9uY2VJc0Rpc2FibGVkAAAAFQAAAAAAAAARSW5zdWZmaWNpZW50U3Rha2UAAAAAAAAWAAAAAAAAAB1JbnN1ZmZpY2llbnRQZW5kaW5nV2l0aGRyYXdhbAAAAAAAABcAAAAAAAAAF1BlbmRpbmdXaXRoZHJhd2FsRXhpc3RzAAAAABgAAAAAAAAAEkRlcG9zaXRDYXBFeGNlZWRlZAAAAAAAGQAAAAAAAAARV2FybXVwTm90Q29tcGxldGUAAAAAAAAaAAAAAAAAABNDb29sZG93bk5vdENvbXBsZXRlAAAAABsAAAAAAAAADENsYWltVG9vU29vbgAAABwAAAAAAAAAHEluc3VmZmljaWVudEF2YWlsYWJsZVJld2FyZHMAAAAdAAAAAAAAABJOb1Jld2FyZHNUb0hhcnZlc3QAAAAAAB4AAAAAAAAAIEluc3VmZmljaWVudEN1cnJlbnRTbGFzaGVkQW1vdW50AAAAHwAAAAAAAAANSW52YWxpZEFtb3VudAAAAAAAACAAAAAAAAAADUludmFsaWRDb25maWcAAAAAAAAhAAAAAAAAABlQcm9wb3NlZEFkbWluRG9lc05vdEV4aXN0AAAAAAAAIgAAAAAAAAAYSW5zdWZmaWNpZW50VHJlYXN1cnlGZWVzAAAAIwAAAAAAAAAWVHJhbnNmZXJBbW91bnRNaXNtYXRjaAAAAAAAJAAAAAAAAAAWT3JhY2xlUHJpY2VVbmF2YWlsYWJsZQAAAAAAJQAAAAAAAAAQT3JhY2xlUHJpY2VTdGFsZQAAACYAAAAAAAAAElVuYXV0aG9yaXplZENhbGxlcgAAAAAAJwAAAAAAAAAQTm9QZW5kaW5nRGVwb3NpdAAAACgAAAAAAAAADk5vdEluaXRpYWxpemVkAAAAAAApAAAAAAAAABFGYXJtQWxyZWFkeUV4aXN0cwAAAAAAACo=",
        "AAAAAQAAAAAAAAAAAAAABEZhcm0AAAAKAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABmNvbmZpZwAAAAAH0AAAAApGYXJtQ29uZmlnAAAAAAAAAAAAGWN1bXVsYXRpdmVfc2xhc2hlZF9hbW91bnQAAAAAAAALAAAAAAAAABZjdXJyZW50X3NsYXNoZWRfYW1vdW50AAAAAAALAAAAAAAAAAJpZAAAAAAD7gAAACAAAAAAAAAACWlzX2Zyb3plbgAAAAAAAAEAAAAAAAAACW51bV91c2VycwAAAAAAAAYAAAAAAAAADnByb3Bvc2VkX2FkbWluAAAAAAPoAAAAEwAAAEJSZXdhcmQgdG9rZW5zIGluIGluaXRpYWxpemF0aW9uIG9yZGVyOyBgcmV3YXJkX2luZGV4YCBwb2ludHMgaGVyZS4AAAAAAAdyZXdhcmRzAAAAA+oAAAATAAAAAAAAAAx0b3RhbF9zdGFrZWQAAAAL",
        "AAAAAQAAAHtGdWxsIGZhcm0gdmlldyByZXR1cm5lZCBieSBxdWVyaWVzOiB0aGUgZmFybSBpdHNlbGYgcGx1cyB0aGUgc3RhdGUgb2YKZXZlcnkgaW5pdGlhbGl6ZWQgcmV3YXJkIChvcmRlcmVkIGJ5IGByZXdhcmRfaW5kZXhgKS4AAAAAAAAAAAlGYXJtU3RhdGUAAAAAAAACAAAAAAAAAARmYXJtAAAH0AAAAARGYXJtAAAAAAAAAAdyZXdhcmRzAAAAA+oAAAfQAAAAClJld2FyZEluZm8AAA==",
        "AAAAAgAAAAAAAAAAAAAACkRlbGVnYXRpb24AAAAAAAIAAAABAAAAAAAAAAlEZWxlZ2F0ZWQAAAAAAAABAAAH0AAAABNEZWxlZ2F0ZWRGYXJtQ29uZmlnAAAAAAEAAAAAAAAADE5vbkRlbGVnYXRlZAAAAAEAAAfQAAAAFk5vbkRlbGVnYXRlZEZhcm1Db25maWcAAA==",
        "AAAAAQAAAAAAAAAAAAAACkZhcm1Db25maWcAAAAAAAkAAAAAAAAACmRlbGVnYXRpb24AAAAAB9AAAAAKRGVsZWdhdGlvbgAAAAAAAAAAAAtkZXBvc2l0X2NhcAAAAAALAAAAAAAAABlpc19oYXJ2ZXN0X3Blcm1pc3Npb25sZXNzAAAAAAAAAQAAAAAAAAAWaXNfcmV3YXJkX29uY2VfZW5hYmxlZAAAAAAAAQAAAAAAAAARbWluX2hhcnZlc3RfZGVsYXkAAAAAAAAGAAAAAAAAABBtaW5fc3Rha2VfYW1vdW50AAAACwAAAAAAAAAGb3JhY2xlAAAAAAfQAAAADk9wdGlvbmFsT3JhY2xlAAAAAAAAAAAABXRva2VuAAAAAAAAEwAAAAAAAAAQdHJlYXN1cnlfZmVlX2JwcwAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAClJld2FyZEluZm8AAAAAAAkAAAAAAAAAGmFjY3VtX3Jld2FyZHNfcGVyX3NoYXJlX3NjAAAAAAALAAAAAAAAABlhY2N1bXVsYXRlZF90cmVhc3VyeV9mZWVzAAAAAAAACwAAAAAAAAAQbGFzdF9pc3N1YW5jZV90cwAAAAYAAAAAAAAAFXJld2FyZF9zY2hlZHVsZV9jdXJ2ZQAAAAAAB9AAAAATUmV3YXJkU2NoZWR1bGVDdXJ2ZQAAAAAAAAAADHJld2FyZF90b2tlbgAAABMAAAAAAAAAC3Jld2FyZF90eXBlAAAAB9AAAAAKUmV3YXJkVHlwZQAAAAAAAAAAABFyZXdhcmRzX2F2YWlsYWJsZQAAAAAAAAsAAAAAAAAAGXJld2FyZHNfaXNzdWVkX2N1bXVsYXRpdmUAAAAAAAALAAAAAAAAABhyZXdhcmRzX2lzc3VlZF91bmNsYWltZWQAAAAL",
        "AAAAAgAAAAAAAAAAAAAAClJld2FyZFR5cGUAAAAAAAIAAAAAAAAAAAAAAAxQcm9wb3J0aW9uYWwAAAAAAAAAAAAAAAhDb25zdGFudA==",
        "AAAAAgAAAAAAAAAAAAAAC0xvY2tpbmdNb2RlAAAAAAMAAAAAAAAAAAAAAAROb25lAAAAAAAAAAAAAAAKQ29udGludW91cwAAAAAAAAAAAAAAAAAKV2l0aEV4cGlyeQAA",
        "AAAAAQAAAGBDb250cmFjdC13aWRlIGNvbmZpZ3VyYXRpb246IHRoZSBnbG9iYWwgYWRtaW4gd2hvIGNhbiBpbml0aWFsaXplIGZhcm1zCmFuZCB1cGdyYWRlIHRoZSBjb250cmFjdC4AAAAAAAAADEdsb2JhbENvbmZpZwAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAEJJbnRlcm5hbCBjb3VudGVyIHVzZWQgdG8gZGVyaXZlIGZhcm0gSURzIHdoZW4gbm8gc2VlZCBpcyBwcm92aWRlZC4AAAAAAAludW1fZmFybXMAAAAAAAAGAAAAAAAAAA5wcm9wb3NlZF9hZG1pbgAAAAAD6AAAABM=",
        "AAAAAQAAAAAAAAAAAAAADE9yYWNsZUNvbmZpZwAAAAIAAAAAAAAADm9yYWNsZV9hZGRyZXNzAAAAAAATAAAAAAAAAA5vcmFjbGVfbWF4X2FnZQAAAAAABg==",
        "AAAAAQAAADJDb25maWd1cmF0aW9uIGZvciBpbml0aWFsaXppbmcgYSByZXdhcmQgb24gYSBmYXJtLgAAAAAAAAAAAAxSZXdhcmRDb25maWcAAAADAAAAAAAAABVyZXdhcmRfc2NoZWR1bGVfY3VydmUAAAAAAAfQAAAAE1Jld2FyZFNjaGVkdWxlQ3VydmUAAAAAAAAAAAxyZXdhcmRfdG9rZW4AAAATAAAAAAAAAAtyZXdhcmRfdHlwZQAAAAfQAAAAClJld2FyZFR5cGUAAA==",
        "AAAAAQAAAAAAAAAAAAAADkRlbGVnYXRlZVN0YXRlAAAAAAAJAAAAAAAAAAxhY3RpdmVfc3Rha2UAAAALAAAAAAAAAA1sYXN0X2NsYWltX3RzAAAAAAAD7AAAABMAAAAGAAAAAAAAAA1sYXN0X3N0YWtlX3RzAAAAAAAABgAAAAAAAAAVcGVuZGluZ19kZXBvc2l0X3N0YWtlAAAAAAAACwAAAAAAAAAScGVuZGluZ19kZXBvc2l0X3RzAAAAAAAGAAAAAAAAABhwZW5kaW5nX3dpdGhkcmF3YWxfc3Rha2UAAAALAAAAAAAAABVwZW5kaW5nX3dpdGhkcmF3YWxfdHMAAAAAAAAGAAAAAAAAAA9yZXdhcmRzX3RhbGxpZXMAAAAD7AAAABMAAAALAAAAAAAAABFyZXdhcmRzX3VuY2xhaW1lZAAAAAAAA+wAAAATAAAACw==",
        "AAAAAgAAAAAAAAAAAAAADk9wdGlvbmFsT3JhY2xlAAAAAAACAAAAAAAAAAAAAAAETm9uZQAAAAEAAAAAAAAABFNvbWUAAAABAAAH0AAAAAxPcmFjbGVDb25maWc=",
        "AAAAAQAAAAAAAAAAAAAAE0RlbGVnYXRlZEZhcm1Db25maWcAAAAAAQAAAGBUaGUgcGxhdGZvcm0gKGUuZy4gYSBsZW5kaW5nIG1hcmtldCkgYXV0aG9yaXplZCB0byBwdXNoIHN0YWtlIHVwZGF0ZXMKdmlhIGBzZXRfc3Rha2VfZGVsZWdhdGVkYC4AAAASZGVsZWdhdGVfYXV0aG9yaXR5AAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAAFk5vbkRlbGVnYXRlZEZhcm1Db25maWcAAAAAAAYAAAAAAAAAFWRlcG9zaXRfd2FybXVwX3BlcmlvZAAAAAAAAAYAAAAAAAAAHGVhcmx5X3dpdGhkcmF3YWxfcGVuYWx0eV9icHMAAAALAAAAAAAAABBsb2NraW5nX2R1cmF0aW9uAAAABgAAAAAAAAAMbG9ja2luZ19tb2RlAAAH0AAAAAtMb2NraW5nTW9kZQAAAAAAAAAACmxvY2tpbmdfdHMAAAAAAAYAAAAAAAAAGndpdGhkcmF3YWxfY29vbGRvd25fcGVyaW9kAAAAAAAG",
        "AAAABQAAAAAAAAAAAAAABVN0YWtlAAAAAAAAAQAAAAVzdGFrZQAAAAAAAAMAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAABAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAB0hhcnZlc3QAAAAAAQAAAAdoYXJ2ZXN0AAAAAAQAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAABAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAEAAAAAAAAADHJld2FyZF90b2tlbgAAABMAAAABAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAB1Vuc3Rha2UAAAAAAQAAAAd1bnN0YWtlAAAAAAMAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAABAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAACkFkZFJld2FyZHMAAAAAAAEAAAALYWRkX3Jld2FyZHMAAAAABAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAAAAAAABmZ1bmRlcgAAAAAAEwAAAAEAAAAAAAAADHJld2FyZF90b2tlbgAAABMAAAABAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAACkZyZWV6ZUZhcm0AAAAAAAEAAAALZnJlZXplX2Zhcm0AAAAAAQAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAClJld2FyZE9uY2UAAAAAAAEAAAALcmV3YXJkX29uY2UAAAAABAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAAAAAAACWRlbGVnYXRlZQAAAAAAB9AAAAAJRGVsZWdhdGVlAAAAAAAAAQAAAAAAAAAMcmV3YXJkX3Rva2VuAAAAEwAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAC0FjY2VwdEFkbWluAAAAAAEAAAAMYWNjZXB0X2FkbWluAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADFByb3Bvc2VBZG1pbgAAAAEAAAANcHJvcG9zZV9hZG1pbgAAAAAAAAEAAAAAAAAADnByb3Bvc2VkX2FkbWluAAAAAAATAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAADFVuZnJlZXplRmFybQAAAAEAAAANdW5mcmVlemVfZmFybQAAAAAAAAEAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADkluaXRpYWxpemVGYXJtAAAAAAABAAAAD2luaXRpYWxpemVfZmFybQAAAAADAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAAAAAAAFdG9rZW4AAAAAAAATAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADldpdGhkcmF3VW51c2VkAAAAAAABAAAAD3dpdGhkcmF3X3VudXNlZAAAAAAEAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAAAAAAAAJcmVjaXBpZW50AAAAAAAAEwAAAAEAAAAAAAAADHJld2FyZF90b2tlbgAAABMAAAABAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAD0FjY2VwdEZhcm1BZG1pbgAAAAABAAAAEWFjY2VwdF9mYXJtX2FkbWluAAAAAAAAAQAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAAC",
        "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",
        "AAAAAAAAAAAAAAAFc3Rha2UAAAAAAAADAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAAAAAAAAAAAHaGFydmVzdAAAAAADAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAAAAAAMcmV3YXJkX2luZGV4AAAABAAAAAEAAAPpAAAACwAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAAAAAAAAAAAHdW5zdGFrZQAAAAADAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAALAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAIZ2V0X2Zhcm0AAAABAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAA+kAAAfQAAAACUZhcm1TdGF0ZQAAAAAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAAAAAAAAAAALYWRkX3Jld2FyZHMAAAAABAAAAAAAAAAGZnVuZGVyAAAAAAATAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAAAAAAxyZXdhcmRfaW5kZXgAAAAEAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAALZnJlZXplX2Zhcm0AAAAAAQAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAALcmV3YXJkX29uY2UAAAAABAAAAAAAAAAJZGVsZWdhdGVlAAAAAAAH0AAAAAlEZWxlZ2F0ZWUAAAAAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAAAAAAADHJld2FyZF9pbmRleAAAAAQAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAAMYWNjZXB0X2FkbWluAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAEAAAAAAAAABWFkbWluAAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAANZ2V0X2FsbF9mYXJtcwAAAAAAAAAAAAABAAAD6gAAA+4AAAAg",
        "AAAAAAAAAAAAAAANcHJvcG9zZV9hZG1pbgAAAAAAAAEAAAAAAAAADnByb3Bvc2VkX2FkbWluAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAAAAAAAAAAANdW5mcmVlemVfZmFybQAAAAAAAAEAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAANd2l0aGRyYXdfZmVlcwAAAAAAAAMAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAAAAAAADHJld2FyZF9pbmRleAAAAAQAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAAPaW5pdGlhbGl6ZV9mYXJtAAAAAAIAAAAAAAAABHNlZWQAAAPoAAAD7gAAACAAAAAAAAAABmNvbmZpZwAAAAAH0AAAAApGYXJtQ29uZmlnAAAAAAABAAAD6QAAA+4AAAAgAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAQd2l0aGRyYXdfc2xhc2hlZAAAAAIAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAARYWNjZXB0X2Zhcm1fYWRtaW4AAAAAAAABAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAAAAAAAAAAARZ2V0X2dsb2JhbF9jb25maWcAAAAAAAAAAAAAAQAAA+kAAAfQAAAADEdsb2JhbENvbmZpZwAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAAAAAAAAAAARaW5pdGlhbGl6ZV9yZXdhcmQAAAAAAAACAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAAAAAA1yZXdhcmRfY29uZmlnAAAAAAAH0AAAAAxSZXdhcmRDb25maWcAAAABAAAD6QAAAAQAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAARd2l0aGRyYXdfdW5zdGFrZWQAAAAAAAACAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAPpAAAACwAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAAAAAAAAAAAScHJvcG9zZV9mYXJtX2FkbWluAAAAAAACAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAAAAAA5wcm9wb3NlZF9hZG1pbgAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAASdXBkYXRlX2Zhcm1fY29uZmlnAAAAAAACAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAAAAAA1jb25maWdfdXBkYXRlAAAAAAAH0AAAAApGYXJtQ29uZmlnAAAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAATZ2V0X2RlbGVnYXRlZV9zdGF0ZQAAAAACAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAPpAAAH0AAAAA5EZWxlZ2F0ZWVTdGF0ZQAAAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAATZ2V0X3BlbmRpbmdfcmV3YXJkcwAAAAACAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAEAAAPpAAAD6gAAA+0AAAACAAAAEwAAAAsAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAATc2V0X3Jld2FyZF9zY2hlZHVsZQAAAAADAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAAAAAAxyZXdhcmRfaW5kZXgAAAAEAAAAAAAAAAVjdXJ2ZQAAAAAAB9AAAAATUmV3YXJkU2NoZWR1bGVDdXJ2ZQAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAATc2V0X3N0YWtlX2RlbGVnYXRlZAAAAAADAAAAAAAAAAlkZWxlZ2F0ZWUAAAAAAAfQAAAACURlbGVnYXRlZQAAAAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAAAAAAJbmV3X3N0YWtlAAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0ZDRXJyb3IA",
        "AAAAAAAAAAAAAAAXcmVmcmVzaF9kZWxlZ2F0ZWVfc3RhdGUAAAAAAgAAAAAAAAAJZGVsZWdhdGVlAAAAAAAH0AAAAAlEZWxlZ2F0ZWUAAAAAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdGQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAXd2l0aGRyYXdfdW51c2VkX3Jld2FyZHMAAAAABAAAAAAAAAAHZmFybV9pZAAAAAPuAAAAIAAAAAAAAAAMcmV3YXJkX2luZGV4AAAABAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAlyZWNpcGllbnQAAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHRkNFcnJvcgA=",
        "AAAAAQAAAP1EZWxlZ2F0ZWUgaWRlbnRpZmllciBmb3IgZmFybSBzdGFrZXMuCgpTdXBwb3J0cyBtdWx0aXBsZSBzdGFrZSBpZGVudGl0aWVzIHBlciBvd25lciBhZGRyZXNzOgotIFNpbXBsZToganVzdCBvd25lciBhZGRyZXNzIChmb3IgY29udHJhY3RzIHdoZXJlIHVzZXIgaGFzIHNpbmdsZSBwb3NpdGlvbikKLSBXaXRoIHNlZWQ6IG93bmVyIGFkZHJlc3MgKyBzZWVkIChmb3IgY29udHJhY3RzIHdpdGggbXVsdGlwbGUgb2JsaWdhdGlvbnMgcGVyIHVzZXIpAAAAAAAAAAAAAAlEZWxlZ2F0ZWUAAAAAAAACAAAAE1RoZSBvd25lcidzIGFkZHJlc3MAAAAABW93bmVyAAAAAAAAEwAAADlPcHRpb25hbCBzZWVkIHRvIGRpc3Rpbmd1aXNoIG11bHRpcGxlIHBvc2l0aW9ucyBwZXIgb3duZXIAAAAAAAAEc2VlZAAAA+gAAAPuAAAAIA==",
        "AAAAAgAAAApBc3NldCB0eXBlAAAAAAAAAAAABUFzc2V0AAAAAAAAAgAAAAEAAAAAAAAAB1N0ZWxsYXIAAAAAAQAAABMAAAABAAAAAAAAAAVPdGhlcgAAAAAAAAEAAAAR",
        "AAAAAQAAAC9QcmljZSBkYXRhIGZvciBhbiBhc3NldCBhdCBhIHNwZWNpZmljIHRpbWVzdGFtcAAAAAAAAAAACVByaWNlRGF0YQAAAAAAAAIAAAAAAAAABXByaWNlAAAAAAAACwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==" ]),
      options
    )
  }
  public readonly fromJSON = {
    stake: this.txFromJSON<Result<void>>,
        harvest: this.txFromJSON<Result<i128>>,
        unstake: this.txFromJSON<Result<i128>>,
        upgrade: this.txFromJSON<Result<void>>,
        get_farm: this.txFromJSON<Result<FarmState>>,
        add_rewards: this.txFromJSON<Result<void>>,
        freeze_farm: this.txFromJSON<Result<void>>,
        reward_once: this.txFromJSON<Result<void>>,
        accept_admin: this.txFromJSON<Result<void>>,
        get_all_farms: this.txFromJSON<Array<Buffer>>,
        propose_admin: this.txFromJSON<Result<void>>,
        unfreeze_farm: this.txFromJSON<Result<void>>,
        withdraw_fees: this.txFromJSON<Result<void>>,
        initialize_farm: this.txFromJSON<Result<Buffer>>,
        withdraw_slashed: this.txFromJSON<Result<void>>,
        accept_farm_admin: this.txFromJSON<Result<void>>,
        get_global_config: this.txFromJSON<Result<GlobalConfig>>,
        initialize_reward: this.txFromJSON<Result<u32>>,
        withdraw_unstaked: this.txFromJSON<Result<i128>>,
        propose_farm_admin: this.txFromJSON<Result<void>>,
        update_farm_config: this.txFromJSON<Result<void>>,
        get_delegatee_state: this.txFromJSON<Result<DelegateeState>>,
        get_pending_rewards: this.txFromJSON<Result<Array<readonly [string, i128]>>>,
        set_reward_schedule: this.txFromJSON<Result<void>>,
        set_stake_delegated: this.txFromJSON<Result<void>>,
        refresh_delegatee_state: this.txFromJSON<Result<void>>,
        withdraw_unused_rewards: this.txFromJSON<Result<void>>
  }
}