import { Buffer } from "buffer";
import { Address } from '@stellar/stellar-sdk';
import {
  AssembledTransaction,
  Client as ContractClient,
  ClientOptions as ContractClientOptions,
  MethodOptions,
  Result,
  Spec as ContractSpec,
} from '@stellar/stellar-sdk/contract';
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
  Typepoint,
  Duration,
} from '@stellar/stellar-sdk/contract';
export * from '@stellar/stellar-sdk'
export * as contract from '@stellar/stellar-sdk/contract'
export * as rpc from '@stellar/stellar-sdk/rpc'

if (typeof window !== 'undefined') {
  //@ts-ignore Buffer exists
  window.Buffer = window.Buffer || Buffer;
}




export const Errors = {
  1: {message:"PoolAlreadyExists"},

  2: {message:"PoolDoesNotExist"},

  3: {message:"InvalidLoanPoolConfig"},

  4: {message:"NotEnoughPoolFunds"},

  5: {message:"ObligationDoesNotExist"},

  6: {message:"DepositDoesNotExist"},

  7: {message:"NonPositiveDeposit"},

  8: {message:"NonPositiveWithdraw"},

  9: {message:"WithdrawOverBalance"},

  10: {message:"NonPositiveRepay"},

  11: {message:"OverOrUnderflow"},

  12: {message:"OracleDoesNotKnowAssetPrice"},

  13: {message:"BorrowDoesNotExist"},

  14: {message:"HealthFactorIsLowerThanRequiredThreshold"},

  15: {message:"InvalidLiquidationThreshold"},

  16: {message:"LiquidatedPositionIsHealthy"},

  17: {message:"LiquidationExceedsCloseFactor"},

  18: {message:"NonPositiveLiquidation"}
}

/**
 * Interest rate multipliers presented as (1 + xxx) where `xxx` is a compound interest rate.
 * The real multiplier(e.g. 1.32, 2.53, etc) is scaled up with [`SCALED_ONE`] value.
 * 
 * # Examples:
 * ```
 * use lending::interest_rate::{CompoundRates, CompoundRateMultipliers};
 * 
 * let multipliers = CompoundRateMultipliers {
 * borrow_multiplier: 1320700048000, // x 1.3207
 * deposit_multiplier: 1000000000000  // x 1.0
 * };
 * 
 * let compound_rates: CompoundRates = multipliers.try_into().unwrap();
 * 
 * assert_eq!(compound_rates.borrow_rate_bps, 32_07); // 32.07%
 * assert_eq!(compound_rates.deposit_rate_bps, 00_00); // 0%
 * 
 * ```
 */
export interface CompoundRateMultipliers {
  borrow_multiplier: i128;
  deposit_multiplier: i128;
}


/**
 * Compound interest rates represented in basis points
 */
export interface CompoundRates {
  borrow_rate_bps: u32;
  deposit_rate_bps: u32;
}


export interface GlobalState {
  admin: string;
  liquidation_threshold_bps: i128;
  status: boolean;
}

export type DataKey = {tag: "GlobalState", values: void} | {tag: "Pool", values: readonly [PoolAddress]} | {tag: "Obligation", values: readonly [UserAddress]} | {tag: "Accrual", values: void};


export interface Pool {
  accrual: Accrual;
  /**
 * The total amount of borrowed assets
 */
borrowed: i128;
  /**
 * The total amount of deposited collateral assets that don't accrue interest
 */
collateral: i128;
  config: PoolConfig;
  /**
 * The total amount of deposited assets that accrue interest
 */
deposited: i128;
  token_address: string;
  token_ticker: string;
}


export interface PoolConfig {
  /**
 * Positive Base Rate in 1/[`SCALED_ONE`] units
 */
base_rate_per_second: i128;
  /**
 * Non-negative Close Factor percentage (< 100)
 */
close_factor_bps: i128;
  /**
 * Non-negative Liquidation Spread percentage (< 100)
 */
liquidation_spread_bps: i128;
  /**
 * Positive Optimal Utilization Ratio
 */
optimal_utilization_ratio_bps: i128;
  /**
 * Non-negative Reserve Ratio percentage (< 100)
 */
reserve_ratio_bps: i128;
  slope1: i128;
  slope2: i128;
}


export interface Obligation {
  borrows: Map<PoolAddress, BorrowObligation>;
  deposits: Map<PoolAddress, DepositObligation>;
}


export interface BorrowObligation {
  borrowed: i128;
  /**
 * The numerical value that is used to determine the scaling factor required for updating the position amount
 * with interest, i.e. new_borrowed = (current_accrual \ last_accrual) * borrowed
 */
last_accrual: i128;
}


export interface DepositObligation {
  collateral: i128;
  deposited: i128;
  /**
 * The numerical value that is used to determine the scaling factor required for updating the position amount
 * with interest, i.e. new_deposited = (current_accrual \ last_accrual) * deposited
 */
last_accrual: i128;
}


export interface Accrual {
  borrow_accrual: i128;
  deposit_accrual: i128;
  timestamp: u64;
}


export interface Client {
  /**
   * Construct and simulate a get_global_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Gets the contract's global state
   */
  get_global_state: (options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<GlobalState>>

  /**
   * Construct and simulate a initialize_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes a loan pool for a specific asset
   * 
   * ### Arguments
   * * `token_address` - address of a corresponding Soroban Asset Contract
   * * `token_symbol` - symbol which represents a pool's token
   * * `salt` - optional salt data, which when provided is used along with `token_address` to derive a deterministic pool address
   * * `pool_config` - optional `PoolConfig` data. If not provided - a default pool config is used
   */
  initialize_pool: ({token_address, token_ticker, salt, pool_config}: {token_address: string, token_ticker: string, salt: Option<Buffer>, pool_config: Option<PoolConfig>}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<PoolAddress>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits tokens into the loan pool
   * 
   * ### Arguments
   * * `user` - user which deposits a token
   * * `pool_address` - address of a pool to which the deposit happens
   * * `amount` - amount of tokens which are going to be deposited
   */
  deposit: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a borrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Borrows tokens from the loan pool
   * 
   * ### Arguments
   * * `user` - user which borrows a token
   * * `pool_address` - address of a pool from which the borrow happens
   * * `amount` - amount of tokens which are going to be borrowed
   */
  borrow: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits tokens into the loan pool as collateral only.
   * This implies that they are always available for a healthy withdrawal for the
   * cost of not accruing an interest rate
   * 
   * ### Arguments
   * * `user` - user which deposits a token
   * * `pool_address` - address of a pool to which the collateral deposit happens
   * * `amount` - amount of tokens which are being deposited as a collateral
   */
  deposit_collateral: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a repay transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Repays borrowed tokens
   * 
   * ### Arguments
   * * `user` - user which repays borrowed tokens
   * * `pool_address` - address of a pool from which the borrow happened
   * * `amount` - amount of repaid tokens
   */
  repay: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a liquidate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Liquidates borrower's position if position's health factor criterion isn't met
   * 
   * ### Arguments
   * * `user` - user which liquidates the borrower's position
   * * `pool_address` - address of a pool whose tokens are repaid by the liquidator
   * * `amount` - amount of repaid tokens
   */
  liquidate: ({user, borrower, pool_address, amount}: {user: string, borrower: string, pool_address: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws collateral tokens from the loan pool to the user
   * 
   * ### Arguments
   * * `user` - user which withdraws collateral tokens
   * * `pool_address` - address of a pool from which the withdrawal happens
   * * `amount` - amount of withdrawn tokens
   */
  withdraw_collateral: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws deposited tokens from the loan pool to the user
   * 
   * ### Arguments
   * * `user` - user which withdraws deposited tokens
   * * `pool_address` - address of a pool from which the withdrawal happens
   * * `amount` - amount of withdrawn tokens
   */
  withdraw: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_user_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_obligation: ({user}: {user: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a get_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool: ({pool_address}: {pool_address: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Option<Pool>>>

  /**
   * Construct and simulate a get_apy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_apy: ({pool_address}: {pool_address: string}, options?: {
    /**
     * The fee to pay for the transaction. Default: BASE_FEE
     */
    fee?: number;

    /**
     * The maximum amount of time to wait for the transaction to complete. Default: DEFAULT_TIMEOUT
     */
    timeoutInSeconds?: number;

    /**
     * Whether to automatically simulate the transaction when constructing the AssembledTransaction. Default: true
     */
    simulate?: boolean;
  }) => Promise<AssembledTransaction<Result<CompoundRates>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {admin, liquidation_threshold_percent}: {admin: string, liquidation_threshold_percent: Option<i128>},
    /** Options for initalizing a Client as well as for calling a method, with extras specific to deploying. */
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
    return ContractClient.deploy({admin, liquidation_threshold_percent}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAK9Db25zdHJ1Y3RzIHRoZSBsZW5kaW5nIGNvbnRyYWN0CgojIyMgQXJndW1lbnRzCiogYGFkbWluYCAtIGNvbnRyYWN0J3MgYWRtaW5pc3RyYXRvcgoqIGBsaXF1aWRhdGlvbl90aHJlc2hvbGRfcGVyY2VudGAgLSB0aHJlc2hvbGQgcGVyY2VudGFnZSB1c2VkIGZvciBoZWFsdGggZmFjdG9yIGNhbGN1bGF0aW9uAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAB1saXF1aWRhdGlvbl90aHJlc2hvbGRfcGVyY2VudAAAAAAAA+gAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAACBHZXRzIHRoZSBjb250cmFjdCdzIGdsb2JhbCBzdGF0ZQAAABBnZXRfZ2xvYmFsX3N0YXRlAAAAAAAAAAEAAAfQAAAAC0dsb2JhbFN0YXRlAA==",
        "AAAAAAAAAZZJbml0aWFsaXplcyBhIGxvYW4gcG9vbCBmb3IgYSBzcGVjaWZpYyBhc3NldAoKIyMjIEFyZ3VtZW50cwoqIGB0b2tlbl9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBjb3JyZXNwb25kaW5nIFNvcm9iYW4gQXNzZXQgQ29udHJhY3QKKiBgdG9rZW5fc3ltYm9sYCAtIHN5bWJvbCB3aGljaCByZXByZXNlbnRzIGEgcG9vbCdzIHRva2VuCiogYHNhbHRgIC0gb3B0aW9uYWwgc2FsdCBkYXRhLCB3aGljaCB3aGVuIHByb3ZpZGVkIGlzIHVzZWQgYWxvbmcgd2l0aCBgdG9rZW5fYWRkcmVzc2AgdG8gZGVyaXZlIGEgZGV0ZXJtaW5pc3RpYyBwb29sIGFkZHJlc3MKKiBgcG9vbF9jb25maWdgIC0gb3B0aW9uYWwgYFBvb2xDb25maWdgIGRhdGEuIElmIG5vdCBwcm92aWRlZCAtIGEgZGVmYXVsdCBwb29sIGNvbmZpZyBpcyB1c2VkAAAAAAAPaW5pdGlhbGl6ZV9wb29sAAAAAAQAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAAAAAAARzYWx0AAAD6AAAA+4AAAAgAAAAAAAAAAtwb29sX2NvbmZpZwAAAAPoAAAH0AAAAApQb29sQ29uZmlnAAAAAAABAAAD6QAAB9AAAAALUG9vbEFkZHJlc3MAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAANhEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCBkZXBvc2l0cyBhIHRva2VuCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgZGVwb3NpdCBoYXBwZW5zCiogYGFtb3VudGAgLSBhbW91bnQgb2YgdG9rZW5zIHdoaWNoIGFyZSBnb2luZyB0byBiZSBkZXBvc2l0ZWQAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAANZCb3Jyb3dzIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGJvcnJvd3MgYSB0b2tlbgoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgYm9ycm93IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGdvaW5nIHRvIGJlIGJvcnJvd2VkAAAAAAAGYm9ycm93AAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAXREZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sIGFzIGNvbGxhdGVyYWwgb25seS4KVGhpcyBpbXBsaWVzIHRoYXQgdGhleSBhcmUgYWx3YXlzIGF2YWlsYWJsZSBmb3IgYSBoZWFsdGh5IHdpdGhkcmF3YWwgZm9yIHRoZQpjb3N0IG9mIG5vdCBhY2NydWluZyBhbiBpbnRlcmVzdCByYXRlCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCBkZXBvc2l0cyBhIHRva2VuCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgY29sbGF0ZXJhbCBkZXBvc2l0IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGJlaW5nIGRlcG9zaXRlZCBhcyBhIGNvbGxhdGVyYWwAAAASZGVwb3NpdF9jb2xsYXRlcmFsAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAALtSZXBheXMgYm9ycm93ZWQgdG9rZW5zCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCByZXBheXMgYm9ycm93ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSBib3Jyb3cgaGFwcGVuZWQKKiBgYW1vdW50YCAtIGFtb3VudCBvZiByZXBhaWQgdG9rZW5zAAAAAAVyZXBheQAAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAQpMaXF1aWRhdGVzIGJvcnJvd2VyJ3MgcG9zaXRpb24gaWYgcG9zaXRpb24ncyBoZWFsdGggZmFjdG9yIGNyaXRlcmlvbiBpc24ndCBtZXQKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGxpcXVpZGF0ZXMgdGhlIGJvcnJvd2VyJ3MgcG9zaXRpb24KKiBgcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIHdob3NlIHRva2VucyBhcmUgcmVwYWlkIGJ5IHRoZSBsaXF1aWRhdG9yCiogYGFtb3VudGAgLSBhbW91bnQgb2YgcmVwYWlkIHRva2VucwAAAAAACWxpcXVpZGF0ZQAAAAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAhib3Jyb3dlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAOpXaXRoZHJhd3MgY29sbGF0ZXJhbCB0b2tlbnMgZnJvbSB0aGUgbG9hbiBwb29sIHRvIHRoZSB1c2VyCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCB3aXRoZHJhd3MgY29sbGF0ZXJhbCB0b2tlbnMKKiBgcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIGZyb20gd2hpY2ggdGhlIHdpdGhkcmF3YWwgaGFwcGVucwoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHdpdGhkcmF3biB0b2tlbnMAAAAAABN3aXRoZHJhd19jb2xsYXRlcmFsAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAOhXaXRoZHJhd3MgZGVwb3NpdGVkIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wgdG8gdGhlIHVzZXIKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIHdpdGhkcmF3cyBkZXBvc2l0ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB3aXRoZHJhd24gdG9rZW5zAAAACHdpdGhkcmF3AAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAAAAAAATZ2V0X3VzZXJfb2JsaWdhdGlvbgAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAAAAAAAIZ2V0X3Bvb2wAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+gAAAfQAAAABFBvb2w=",
        "AAAAAAAAAAAAAAAHZ2V0X2FweQAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAADUNvbXBvdW5kUmF0ZXMAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAABAAAAAAAAAAAAAAAFExlbmRpbmdDb250cmFjdEVycm9yAAAAEgAAAAAAAAARUG9vbEFscmVhZHlFeGlzdHMAAAAAAAABAAAAAAAAABBQb29sRG9lc05vdEV4aXN0AAAAAgAAAAAAAAAVSW52YWxpZExvYW5Qb29sQ29uZmlnAAAAAAAAAwAAAAAAAAASTm90RW5vdWdoUG9vbEZ1bmRzAAAAAAAEAAAAAAAAABZPYmxpZ2F0aW9uRG9lc05vdEV4aXN0AAAAAAAFAAAAAAAAABNEZXBvc2l0RG9lc05vdEV4aXN0AAAAAAYAAAAAAAAAEk5vblBvc2l0aXZlRGVwb3NpdAAAAAAABwAAAAAAAAATTm9uUG9zaXRpdmVXaXRoZHJhdwAAAAAIAAAAAAAAABNXaXRoZHJhd092ZXJCYWxhbmNlAAAAAAkAAAAAAAAAEE5vblBvc2l0aXZlUmVwYXkAAAAKAAAAAAAAAA9PdmVyT3JVbmRlcmZsb3cAAAAACwAAAAAAAAAbT3JhY2xlRG9lc05vdEtub3dBc3NldFByaWNlAAAAAAwAAAAAAAAAEkJvcnJvd0RvZXNOb3RFeGlzdAAAAAAADQAAAAAAAAAoSGVhbHRoRmFjdG9ySXNMb3dlclRoYW5SZXF1aXJlZFRocmVzaG9sZAAAAA4AAAAAAAAAG0ludmFsaWRMaXF1aWRhdGlvblRocmVzaG9sZAAAAAAPAAAAAAAAABtMaXF1aWRhdGVkUG9zaXRpb25Jc0hlYWx0aHkAAAAAEAAAAAAAAAAdTGlxdWlkYXRpb25FeGNlZWRzQ2xvc2VGYWN0b3IAAAAAAAARAAAAAAAAABZOb25Qb3NpdGl2ZUxpcXVpZGF0aW9uAAAAAAAS",
        "AAAAAQAAAk9JbnRlcmVzdCByYXRlIG11bHRpcGxpZXJzIHByZXNlbnRlZCBhcyAoMSArIHh4eCkgd2hlcmUgYHh4eGAgaXMgYSBjb21wb3VuZCBpbnRlcmVzdCByYXRlLgpUaGUgcmVhbCBtdWx0aXBsaWVyKGUuZy4gMS4zMiwgMi41MywgZXRjKSBpcyBzY2FsZWQgdXAgd2l0aCBbYFNDQUxFRF9PTkVgXSB2YWx1ZS4KCiMgRXhhbXBsZXM6CmBgYAp1c2UgbGVuZGluZzo6aW50ZXJlc3RfcmF0ZTo6e0NvbXBvdW5kUmF0ZXMsIENvbXBvdW5kUmF0ZU11bHRpcGxpZXJzfTsKCmxldCBtdWx0aXBsaWVycyA9IENvbXBvdW5kUmF0ZU11bHRpcGxpZXJzIHsKYm9ycm93X211bHRpcGxpZXI6IDEzMjA3MDAwNDgwMDAsIC8vIHggMS4zMjA3CmRlcG9zaXRfbXVsdGlwbGllcjogMTAwMDAwMDAwMDAwMCAgLy8geCAxLjAKfTsKCmxldCBjb21wb3VuZF9yYXRlczogQ29tcG91bmRSYXRlcyA9IG11bHRpcGxpZXJzLnRyeV9pbnRvKCkudW53cmFwKCk7Cgphc3NlcnRfZXEhKGNvbXBvdW5kX3JhdGVzLmJvcnJvd19yYXRlX2JwcywgMzJfMDcpOyAvLyAzMi4wNyUKYXNzZXJ0X2VxIShjb21wb3VuZF9yYXRlcy5kZXBvc2l0X3JhdGVfYnBzLCAwMF8wMCk7IC8vIDAlCgpgYGAAAAAAAAAAABdDb21wb3VuZFJhdGVNdWx0aXBsaWVycwAAAAACAAAAAAAAABFib3Jyb3dfbXVsdGlwbGllcgAAAAAAAAsAAAAAAAAAEmRlcG9zaXRfbXVsdGlwbGllcgAAAAAACw==",
        "AAAAAQAAADNDb21wb3VuZCBpbnRlcmVzdCByYXRlcyByZXByZXNlbnRlZCBpbiBiYXNpcyBwb2ludHMAAAAAAAAAAA1Db21wb3VuZFJhdGVzAAAAAAAAAgAAAAAAAAAPYm9ycm93X3JhdGVfYnBzAAAAAAQAAAAAAAAAEGRlcG9zaXRfcmF0ZV9icHMAAAAE",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbGlxdWlkYXRpb25fdGhyZXNob2xkX2JwcwAAAAAAAAsAAAAAAAAABnN0YXR1cwAAAAAAAQ==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABAAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAEAAAAAAAAABFBvb2wAAAABAAAH0AAAAAtQb29sQWRkcmVzcwAAAAABAAAAAAAAAApPYmxpZ2F0aW9uAAAAAAABAAAH0AAAAAtVc2VyQWRkcmVzcwAAAAAAAAAAAAAAAAdBY2NydWFsAA==",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAAHAAAAAAAAAAdhY2NydWFsAAAAB9AAAAAHQWNjcnVhbAAAAAAjVGhlIHRvdGFsIGFtb3VudCBvZiBib3Jyb3dlZCBhc3NldHMAAAAACGJvcnJvd2VkAAAACwAAAEpUaGUgdG90YWwgYW1vdW50IG9mIGRlcG9zaXRlZCBjb2xsYXRlcmFsIGFzc2V0cyB0aGF0IGRvbid0IGFjY3J1ZSBpbnRlcmVzdAAAAAAACmNvbGxhdGVyYWwAAAAAAAsAAAAAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAA5VGhlIHRvdGFsIGFtb3VudCBvZiBkZXBvc2l0ZWQgYXNzZXRzIHRoYXQgYWNjcnVlIGludGVyZXN0AAAAAAAACWRlcG9zaXRlZAAAAAAAAAsAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAAx0b2tlbl90aWNrZXIAAAAR",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAcAAAAsUG9zaXRpdmUgQmFzZSBSYXRlIGluIDEvW2BTQ0FMRURfT05FYF0gdW5pdHMAAAAUYmFzZV9yYXRlX3Blcl9zZWNvbmQAAAALAAAALE5vbi1uZWdhdGl2ZSBDbG9zZSBGYWN0b3IgcGVyY2VudGFnZSAoPCAxMDApAAAAEGNsb3NlX2ZhY3Rvcl9icHMAAAALAAAAMk5vbi1uZWdhdGl2ZSBMaXF1aWRhdGlvbiBTcHJlYWQgcGVyY2VudGFnZSAoPCAxMDApAAAAAAAWbGlxdWlkYXRpb25fc3ByZWFkX2JwcwAAAAAACwAAACJQb3NpdGl2ZSBPcHRpbWFsIFV0aWxpemF0aW9uIFJhdGlvAAAAAAAdb3B0aW1hbF91dGlsaXphdGlvbl9yYXRpb19icHMAAAAAAAALAAAALU5vbi1uZWdhdGl2ZSBSZXNlcnZlIFJhdGlvIHBlcmNlbnRhZ2UgKDwgMTAwKQAAAAAAABFyZXNlcnZlX3JhdGlvX2JwcwAAAAAAAAsAAAAAAAAABnNsb3BlMQAAAAAACwAAAAAAAAAGc2xvcGUyAAAAAAAL",
        "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAIAAAAAAAAAB2JvcnJvd3MAAAAD7AAAB9AAAAALUG9vbEFkZHJlc3MAAAAH0AAAABBCb3Jyb3dPYmxpZ2F0aW9uAAAAAAAAAAhkZXBvc2l0cwAAA+wAAAfQAAAAC1Bvb2xBZGRyZXNzAAAAB9AAAAARRGVwb3NpdE9ibGlnYXRpb24AAAA=",
        "AAAAAQAAAAAAAAAAAAAAEEJvcnJvd09ibGlnYXRpb24AAAACAAAAAAAAAAhib3Jyb3dlZAAAAAsAAAC5VGhlIG51bWVyaWNhbCB2YWx1ZSB0aGF0IGlzIHVzZWQgdG8gZGV0ZXJtaW5lIHRoZSBzY2FsaW5nIGZhY3RvciByZXF1aXJlZCBmb3IgdXBkYXRpbmcgdGhlIHBvc2l0aW9uIGFtb3VudAp3aXRoIGludGVyZXN0LCBpLmUuIG5ld19ib3Jyb3dlZCA9IChjdXJyZW50X2FjY3J1YWwgXCBsYXN0X2FjY3J1YWwpICogYm9ycm93ZWQAAAAAAAAMbGFzdF9hY2NydWFsAAAACw==",
        "AAAAAQAAAAAAAAAAAAAAEURlcG9zaXRPYmxpZ2F0aW9uAAAAAAAAAwAAAAAAAAAKY29sbGF0ZXJhbAAAAAAACwAAAAAAAAAJZGVwb3NpdGVkAAAAAAAACwAAALtUaGUgbnVtZXJpY2FsIHZhbHVlIHRoYXQgaXMgdXNlZCB0byBkZXRlcm1pbmUgdGhlIHNjYWxpbmcgZmFjdG9yIHJlcXVpcmVkIGZvciB1cGRhdGluZyB0aGUgcG9zaXRpb24gYW1vdW50CndpdGggaW50ZXJlc3QsIGkuZS4gbmV3X2RlcG9zaXRlZCA9IChjdXJyZW50X2FjY3J1YWwgXCBsYXN0X2FjY3J1YWwpICogZGVwb3NpdGVkAAAAAAxsYXN0X2FjY3J1YWwAAAAL",
        "AAAAAQAAAAAAAAAAAAAAB0FjY3J1YWwAAAAAAwAAAAAAAAAOYm9ycm93X2FjY3J1YWwAAAAAAAsAAAAAAAAAD2RlcG9zaXRfYWNjcnVhbAAAAAALAAAAAAAAAAl0aW1lc3RhbXAAAAAAAAAG" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_global_state: this.txFromJSON<GlobalState>,
        initialize_pool: this.txFromJSON<Result<PoolAddress>>,
        deposit: this.txFromJSON<Result<void>>,
        borrow: this.txFromJSON<Result<void>>,
        deposit_collateral: this.txFromJSON<Result<void>>,
        repay: this.txFromJSON<Result<void>>,
        liquidate: this.txFromJSON<Result<void>>,
        withdraw_collateral: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        get_user_obligation: this.txFromJSON<Result<Obligation>>,
        get_pool: this.txFromJSON<Option<Pool>>,
        get_apy: this.txFromJSON<Result<CompoundRates>>
  }
}