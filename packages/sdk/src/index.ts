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




export const LendingContractError = {
  0: {message:"InternalError"},
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
  18: {message:"NonPositiveLiquidation"},
  19: {message:"NonPositiveBorrow"},
  20: {message:"CollateralPoolDoesNotExist"},
  21: {message:"NonPositiveFlashLoan"},
  23: {message:"InvalidTimestamp"},
  24: {message:"SelfLiquidation"},
  27: {message:"DepositPoolDoesNotExist"},
  28: {message:"BorrowPoolDoesNotExist"},
  29: {message:"InvalidLeverageMultiplier"},
  30: {message:"InvalidSwapSlippage"},
  31: {message:"DependencyContractError"}
}


/**
 * Interest rate multipliers presented as (1 + xxx) where `xxx` is a compound interest rate.
 * The real multiplier(e.g. 1.32, 2.53, etc) is scaled up with [`SCALED_ONE`] value.
 */
export interface CompoundRateMultipliers {
  borrow: i128;
  supply: i128;
}


/**
 * Compound interest rates represented in basis points
 */
export interface CompoundRates {
  borrow_bps: u32;
  supply_bps: u32;
}


export interface Obligation {
  /**
 * Borrowed liquidity for the obligation, unique by borrow pool address
 */
borrows: Map<PoolAddress, BorrowObligation>;
  /**
 * Deposited collateral for the obligation, unique by deposit pool address
 */
deposits: Map<PoolAddress, DepositObligation>;
  /**
 * The obligation's user
 */
user: string;
}


export interface BorrowObligation {
  /**
 * The initial amount of the borrowed token
 */
borrowed: i128;
  /**
 * The numerical value that is used to determine the scaling factor required for updating the position amount
 * with interest, i.e. new_borrowed = (current_accrual \ last_accrual) * borrowed
 */
last_accrual: i128;
  /**
 * The amount of unpaid interest
 */
unpaid_interest: i128;
}


export interface DepositObligation {
  collateral: i128;
  shares: i128;
}


export interface Pool {
  /**
 * The currently available for borrowing tokens
 */
available: i128;
  /**
 * Configuration settings for the pool
 */
config: PoolConfig;
  /**
 * The numerical value that is used to determine the scaling factor required for updating the borrowed amount
 * with interest, i.e. new_borrowed = (current_accrual \ last_accrual) * borrowed
 */
last_accrual: i128;
  /**
 * The timestamp of the last accrual re-calculation
 */
last_accrual_timestamp: u64;
  /**
 * The address of the loan pool
 */
pool_address: string;
  /**
 * The address of the token associated with the pool
 */
token_address: string;
  /**
 * The ticker symbol of the associated token, which is used to identify the token in the pool
 */
token_ticker: string;
  /**
 * The total amount of borrowed assets. This value increases with interest rate accrual
 */
total_borrowed: i128;
  /**
 * The total amount of deposited collateral assets that don't accrue interest
 */
total_collateral: i128;
  /**
 * The total amount of deposited assets that accrue interest
 */
total_shares: i128;
}


export interface PoolConfig {
  /**
 * Base interest rate applied regardless of utilization, expressed per second
 * in 1/`SCALED_ONE` units. Must be positive.
 */
base_rate_per_second: i128;
  /**
 * Maximum percentage of a borrower's debt that can be liquidated.
 */
liquidation_close_factor_bps: i128;
  /**
 * Additional discount given to liquidators when purchasing collateral.
 */
liquidation_incentive_bps: i128;
  /**
 * Positive Optimal Utilization Ratio
 */
optimal_utilization_ratio_bps: i128;
  /**
 * Percentage of interest payments allocated to protocol reserves.
 */
reserve_ratio_bps: i128;
  /**
 * Interest rate slope before reaching optimal utilization ratio.
 * Controls how aggressively rates increase with utilization below the optimal point.
 */
slope1: i128;
  /**
 * Interest rate slope after exceeding optimal utilization ratio.
 * Controls how aggressively rates increase with utilization above the optimal point.
 */
slope2: i128;
}


export interface Accrual {
  borrow_accrual: i128;
  deposit_accrual: i128;
  timestamp: u64;
}


export interface GlobalState {
  admin: string;
  liquidation_threshold_bps: i128;
  status: boolean;
}

export type DataKey = {tag: "GlobalState", values: void} | {tag: "Pool", values: readonly [PoolAddress]} | {tag: "Obligation", values: readonly [UserAddress]} | {tag: "Accrual", values: void} | {tag: "AllPools", values: void};

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
   * Construct and simulate a swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Swap tokens via a swap provider contract
   * 
   * ### Arguments
   * * `user` - user which deposits a token
   * * `token_in` - address of a token that would be taken from the user
   * * `token_out` - address of a token that would be given to the user
   * * `amount` - exact amount of the `token_in`
   */
  swap: ({user, token_in, token_out, amount_in}: {user: string, token_in: string, token_out: string, amount_in: i128}, options?: {
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
  }) => Promise<AssembledTransaction<Result<i128>>>

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
   * Construct and simulate a add_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Adds tokens into the loan pool as collateral only.
   * This implies that they are always available for a healthy withdrawal for the
   * cost of not accruing an interest rate
   * 
   * ### Arguments
   * * `user` - user that adds collateral
   * * `pool_address` - address of a pool to which the collateral is being added
   * * `amount` - amount of tokens which are being added as a collateral
   */
  add_collateral: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
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
   * * `liquidator` - agent which liquidates the borrower's position
   * * `borrow_pool_address` - address of a pool whose borrowed tokens are repaid by the liquidator
   * * `collateral_pool_address` - address of a pool whose tokens are sold to the liquidator with a discount
   * * `amount` - amount of repaid tokens
   */
  liquidate: ({liquidator, borrower, borrow_pool_address, collateral_pool_address, amount}: {liquidator: string, borrower: string, borrow_pool_address: string, collateral_pool_address: string, amount: i128}, options?: {
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
   * Construct and simulate a remove_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes collateral tokens from the loan pool to the user
   * 
   * ### Arguments
   * * `user` - user which withdraws collateral tokens
   * * `pool_address` - address of a pool from which the withdrawal happens
   * * `amount` - amount of withdrawn tokens
   */
  remove_collateral: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
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
   * Construct and simulate a flash_loan transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Creates a flash loan
   * 
   * ### Arguments
   * * `contract` - contract's address which leverages the flash loaned amount and adheres to `erc3156` standard
   * * `pool_address` - address of a pool from which the flash loan happens
   * * `amount` - amount of lent tokens
   */
  flash_loan: ({contract, pool_address, amount}: {contract: string, pool_address: string, amount: i128}, options?: {
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
   * Construct and simulate a deposit_with_leverage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits tokens into the loan pool with leverage. Leverage is achieved by utilizing flash loan and token swap
   * 
   * # WARNING
   * This increases the perceived `supply APR` only
   * when `(borrowed token borrow APR < supply token supply APR)` holds true
   * 
   * ### Arguments
   * * `user` - user that deposits tokens with leverage
   * * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
   * * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
   * * `amount` - original borrow amount before the leverage
   * * `leverage_multiplier` - leverage multiplier as a decimal (e.g., 7.0 for x7, 2.5 for x2.5
   */
  deposit_with_leverage: ({user, deposit_pool_address, borrow_pool_address, amount, leverage_multiplier}: {user: string, deposit_pool_address: string, borrow_pool_address: string, amount: i128, leverage_multiplier: u32}, options?: {
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
   * Construct and simulate a deleverage_and_withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deleverages and withdraws tokens from the leveraged deposit position
   * 
   * ### Arguments
   * * `user` - user that deleverages and withdraws from the position
   * * `deposit_pool_address` - address of a pool from the pair to which the deposit happened
   * * `borrow_pool_address` - address of a pool from the pair from which the borrow happened
   * * `amount` - amount of withdrawn tokens
   */
  deleverage_and_withdraw: ({user, deposit_pool_address, borrow_pool_address, amount}: {user: string, deposit_pool_address: string, borrow_pool_address: string, amount: i128}, options?: {
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
  }) => Promise<AssembledTransaction<Result<Pool>>>

  /**
   * Construct and simulate a get_all_pools transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a list of all pool addresses in the protocol
   */
  get_all_pools: (options?: {
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
  }) => Promise<AssembledTransaction<Array<PoolAddress>>>

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
    return ContractClient.deploy({admin, liquidation_threshold_percent}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAAAAAK9Db25zdHJ1Y3RzIHRoZSBsZW5kaW5nIGNvbnRyYWN0CgojIyMgQXJndW1lbnRzCiogYGFkbWluYCAtIGNvbnRyYWN0J3MgYWRtaW5pc3RyYXRvcgoqIGBsaXF1aWRhdGlvbl90aHJlc2hvbGRfcGVyY2VudGAgLSB0aHJlc2hvbGQgcGVyY2VudGFnZSB1c2VkIGZvciBoZWFsdGggZmFjdG9yIGNhbGN1bGF0aW9uAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAAAgAAAAAAAAAFYWRtaW4AAAAAAAATAAAAAAAAAB1saXF1aWRhdGlvbl90aHJlc2hvbGRfcGVyY2VudAAAAAAAA+gAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAACBHZXRzIHRoZSBjb250cmFjdCdzIGdsb2JhbCBzdGF0ZQAAABBnZXRfZ2xvYmFsX3N0YXRlAAAAAAAAAAEAAAfQAAAAC0dsb2JhbFN0YXRlAA==",
        "AAAAAAAAAZZJbml0aWFsaXplcyBhIGxvYW4gcG9vbCBmb3IgYSBzcGVjaWZpYyBhc3NldAoKIyMjIEFyZ3VtZW50cwoqIGB0b2tlbl9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBjb3JyZXNwb25kaW5nIFNvcm9iYW4gQXNzZXQgQ29udHJhY3QKKiBgdG9rZW5fc3ltYm9sYCAtIHN5bWJvbCB3aGljaCByZXByZXNlbnRzIGEgcG9vbCdzIHRva2VuCiogYHNhbHRgIC0gb3B0aW9uYWwgc2FsdCBkYXRhLCB3aGljaCB3aGVuIHByb3ZpZGVkIGlzIHVzZWQgYWxvbmcgd2l0aCBgdG9rZW5fYWRkcmVzc2AgdG8gZGVyaXZlIGEgZGV0ZXJtaW5pc3RpYyBwb29sIGFkZHJlc3MKKiBgcG9vbF9jb25maWdgIC0gb3B0aW9uYWwgYFBvb2xDb25maWdgIGRhdGEuIElmIG5vdCBwcm92aWRlZCAtIGEgZGVmYXVsdCBwb29sIGNvbmZpZyBpcyB1c2VkAAAAAAAPaW5pdGlhbGl6ZV9wb29sAAAAAAQAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAAAAAAARzYWx0AAAD6AAAA+4AAAAgAAAAAAAAAAtwb29sX2NvbmZpZwAAAAPoAAAH0AAAAApQb29sQ29uZmlnAAAAAAABAAAD6QAAB9AAAAALUG9vbEFkZHJlc3MAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAANhEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCBkZXBvc2l0cyBhIHRva2VuCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgZGVwb3NpdCBoYXBwZW5zCiogYGFtb3VudGAgLSBhbW91bnQgb2YgdG9rZW5zIHdoaWNoIGFyZSBnb2luZyB0byBiZSBkZXBvc2l0ZWQAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAARFTd2FwIHRva2VucyB2aWEgYSBzd2FwIHByb3ZpZGVyIGNvbnRyYWN0CgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCBkZXBvc2l0cyBhIHRva2VuCiogYHRva2VuX2luYCAtIGFkZHJlc3Mgb2YgYSB0b2tlbiB0aGF0IHdvdWxkIGJlIHRha2VuIGZyb20gdGhlIHVzZXIKKiBgdG9rZW5fb3V0YCAtIGFkZHJlc3Mgb2YgYSB0b2tlbiB0aGF0IHdvdWxkIGJlIGdpdmVuIHRvIHRoZSB1c2VyCiogYGFtb3VudGAgLSBleGFjdCBhbW91bnQgb2YgdGhlIGB0b2tlbl9pbmAAAAAAAAAEc3dhcAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAh0b2tlbl9pbgAAABMAAAAAAAAACXRva2VuX291dAAAAAAAABMAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAANZCb3Jyb3dzIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGJvcnJvd3MgYSB0b2tlbgoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgYm9ycm93IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGdvaW5nIHRvIGJlIGJvcnJvd2VkAAAAAAAGYm9ycm93AAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAWlBZGRzIHRva2VucyBpbnRvIHRoZSBsb2FuIHBvb2wgYXMgY29sbGF0ZXJhbCBvbmx5LgpUaGlzIGltcGxpZXMgdGhhdCB0aGV5IGFyZSBhbHdheXMgYXZhaWxhYmxlIGZvciBhIGhlYWx0aHkgd2l0aGRyYXdhbCBmb3IgdGhlCmNvc3Qgb2Ygbm90IGFjY3J1aW5nIGFuIGludGVyZXN0IHJhdGUKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHRoYXQgYWRkcyBjb2xsYXRlcmFsCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgY29sbGF0ZXJhbCBpcyBiZWluZyBhZGRlZAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHRva2VucyB3aGljaCBhcmUgYmVpbmcgYWRkZWQgYXMgYSBjb2xsYXRlcmFsAAAAAAAADmFkZF9jb2xsYXRlcmFsAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAALtSZXBheXMgYm9ycm93ZWQgdG9rZW5zCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCByZXBheXMgYm9ycm93ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSBib3Jyb3cgaGFwcGVuZWQKKiBgYW1vdW50YCAtIGFtb3VudCBvZiByZXBhaWQgdG9rZW5zAAAAAAVyZXBheQAAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAYlMaXF1aWRhdGVzIGJvcnJvd2VyJ3MgcG9zaXRpb24gaWYgcG9zaXRpb24ncyBoZWFsdGggZmFjdG9yIGNyaXRlcmlvbiBpc24ndCBtZXQKCiMjIyBBcmd1bWVudHMKKiBgbGlxdWlkYXRvcmAgLSBhZ2VudCB3aGljaCBsaXF1aWRhdGVzIHRoZSBib3Jyb3dlcidzIHBvc2l0aW9uCiogYGJvcnJvd19wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgYm9ycm93ZWQgdG9rZW5zIGFyZSByZXBhaWQgYnkgdGhlIGxpcXVpZGF0b3IKKiBgY29sbGF0ZXJhbF9wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgdG9rZW5zIGFyZSBzb2xkIHRvIHRoZSBsaXF1aWRhdG9yIHdpdGggYSBkaXNjb3VudAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHJlcGFpZCB0b2tlbnMAAAAAAAAJbGlxdWlkYXRlAAAAAAAABQAAAAAAAAAKbGlxdWlkYXRvcgAAAAAAEwAAAAAAAAAIYm9ycm93ZXIAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAAF2NvbGxhdGVyYWxfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAOhSZW1vdmVzIGNvbGxhdGVyYWwgdG9rZW5zIGZyb20gdGhlIGxvYW4gcG9vbCB0byB0aGUgdXNlcgoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggd2l0aGRyYXdzIGNvbGxhdGVyYWwgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB3aXRoZHJhd24gdG9rZW5zAAAAEXJlbW92ZV9jb2xsYXRlcmFsAAAAAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAOhXaXRoZHJhd3MgZGVwb3NpdGVkIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wgdG8gdGhlIHVzZXIKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIHdpdGhkcmF3cyBkZXBvc2l0ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB3aXRoZHJhd24gdG9rZW5zAAAACHdpdGhkcmF3AAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAPlDcmVhdGVzIGEgZmxhc2ggbG9hbgoKIyMjIEFyZ3VtZW50cwoqIGBjb250cmFjdGAgLSBjb250cmFjdCdzIGFkZHJlc3Mgd2hpY2ggbGV2ZXJhZ2VzIHRoZSBmbGFzaCBsb2FuZWQgYW1vdW50IGFuZCBhZGhlcmVzIHRvIGBlcmMzMTU2YCBzdGFuZGFyZAoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgZmxhc2ggbG9hbiBoYXBwZW5zCiogYGFtb3VudGAgLSBhbW91bnQgb2YgbGVudCB0b2tlbnMAAAAAAAAKZmxhc2hfbG9hbgAAAAAAAwAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAnREZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sIHdpdGggbGV2ZXJhZ2UuIExldmVyYWdlIGlzIGFjaGlldmVkIGJ5IHV0aWxpemluZyBmbGFzaCBsb2FuIGFuZCB0b2tlbiBzd2FwCgojIFdBUk5JTkcKVGhpcyBpbmNyZWFzZXMgdGhlIHBlcmNlaXZlZCBgc3VwcGx5IEFQUmAgb25seQp3aGVuIGAoYm9ycm93ZWQgdG9rZW4gYm9ycm93IEFQUiA8IHN1cHBseSB0b2tlbiBzdXBwbHkgQVBSKWAgaG9sZHMgdHJ1ZQoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgdGhhdCBkZXBvc2l0cyB0b2tlbnMgd2l0aCBsZXZlcmFnZQoqIGBkZXBvc2l0X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIHRvIHdoaWNoIHRoZSBkZXBvc2l0IGhhcHBlbnMKKiBgYm9ycm93X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIGZyb20gd2hpY2ggdGhlIGJvcnJvdyBoYXBwZW5zCiogYGFtb3VudGAgLSBvcmlnaW5hbCBib3Jyb3cgYW1vdW50IGJlZm9yZSB0aGUgbGV2ZXJhZ2UKKiBgbGV2ZXJhZ2VfbXVsdGlwbGllcmAgLSBsZXZlcmFnZSBtdWx0aXBsaWVyIGFzIGEgZGVjaW1hbCAoZS5nLiwgNy4wIGZvciB4NywgMi41IGZvciB4Mi41AAAAFWRlcG9zaXRfd2l0aF9sZXZlcmFnZQAAAAAAAAUAAAAAAAAABHVzZXIAAAATAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAABNsZXZlcmFnZV9tdWx0aXBsaWVyAAAAAAQAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAW5EZWxldmVyYWdlcyBhbmQgd2l0aGRyYXdzIHRva2VucyBmcm9tIHRoZSBsZXZlcmFnZWQgZGVwb3NpdCBwb3NpdGlvbgoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgdGhhdCBkZWxldmVyYWdlcyBhbmQgd2l0aGRyYXdzIGZyb20gdGhlIHBvc2l0aW9uCiogYGRlcG9zaXRfcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIGZyb20gdGhlIHBhaXIgdG8gd2hpY2ggdGhlIGRlcG9zaXQgaGFwcGVuZWQKKiBgYm9ycm93X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIGZyb20gd2hpY2ggdGhlIGJvcnJvdyBoYXBwZW5lZAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHdpdGhkcmF3biB0b2tlbnMAAAAAABdkZWxldmVyYWdlX2FuZF93aXRoZHJhdwAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAAAAAAATZ2V0X3VzZXJfb2JsaWdhdGlvbgAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAAAAAAAIZ2V0X3Bvb2wAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAABFBvb2wAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAADRSZXR1cm5zIGEgbGlzdCBvZiBhbGwgcG9vbCBhZGRyZXNzZXMgaW4gdGhlIHByb3RvY29sAAAADWdldF9hbGxfcG9vbHMAAAAAAAAAAAAAAQAAA+oAAAfQAAAAC1Bvb2xBZGRyZXNzAA==",
        "AAAAAAAAAAAAAAAHZ2V0X2FweQAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAADUNvbXBvdW5kUmF0ZXMAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAABAAAAAAAAAAAAAAAFExlbmRpbmdDb250cmFjdEVycm9yAAAAHQAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAEVBvb2xBbHJlYWR5RXhpc3RzAAAAAAAAAQAAAAAAAAAQUG9vbERvZXNOb3RFeGlzdAAAAAIAAAAAAAAAFUludmFsaWRMb2FuUG9vbENvbmZpZwAAAAAAAAMAAAAAAAAAEk5vdEVub3VnaFBvb2xGdW5kcwAAAAAABAAAAAAAAAAWT2JsaWdhdGlvbkRvZXNOb3RFeGlzdAAAAAAABQAAAAAAAAATRGVwb3NpdERvZXNOb3RFeGlzdAAAAAAGAAAAAAAAABJOb25Qb3NpdGl2ZURlcG9zaXQAAAAAAAcAAAAAAAAAE05vblBvc2l0aXZlV2l0aGRyYXcAAAAACAAAAAAAAAATV2l0aGRyYXdPdmVyQmFsYW5jZQAAAAAJAAAAAAAAABBOb25Qb3NpdGl2ZVJlcGF5AAAACgAAAAAAAAAPT3Zlck9yVW5kZXJmbG93AAAAAAsAAAAAAAAAG09yYWNsZURvZXNOb3RLbm93QXNzZXRQcmljZQAAAAAMAAAAAAAAABJCb3Jyb3dEb2VzTm90RXhpc3QAAAAAAA0AAAAAAAAAKEhlYWx0aEZhY3RvcklzTG93ZXJUaGFuUmVxdWlyZWRUaHJlc2hvbGQAAAAOAAAAAAAAABtJbnZhbGlkTGlxdWlkYXRpb25UaHJlc2hvbGQAAAAADwAAAAAAAAAbTGlxdWlkYXRlZFBvc2l0aW9uSXNIZWFsdGh5AAAAABAAAAAAAAAAHUxpcXVpZGF0aW9uRXhjZWVkc0Nsb3NlRmFjdG9yAAAAAAAAEQAAAAAAAAAWTm9uUG9zaXRpdmVMaXF1aWRhdGlvbgAAAAAAEgAAAAAAAAARTm9uUG9zaXRpdmVCb3Jyb3cAAAAAAAATAAAAAAAAABpDb2xsYXRlcmFsUG9vbERvZXNOb3RFeGlzdAAAAAAAFAAAAAAAAAAUTm9uUG9zaXRpdmVGbGFzaExvYW4AAAAVAAAAAAAAABBJbnZhbGlkVGltZXN0YW1wAAAAFwAAAAAAAAAPU2VsZkxpcXVpZGF0aW9uAAAAABgAAAAAAAAAF0RlcG9zaXRQb29sRG9lc05vdEV4aXN0AAAAABsAAAAAAAAAFkJvcnJvd1Bvb2xEb2VzTm90RXhpc3QAAAAAABwAAAAAAAAAGUludmFsaWRMZXZlcmFnZU11bHRpcGxpZXIAAAAAAAAdAAAAAAAAABNJbnZhbGlkU3dhcFNsaXBwYWdlAAAAAB4AAAAAAAAAF0RlcGVuZGVuY3lDb250cmFjdEVycm9yAAAAAB8=",
        "AAAAAQAAAKtJbnRlcmVzdCByYXRlIG11bHRpcGxpZXJzIHByZXNlbnRlZCBhcyAoMSArIHh4eCkgd2hlcmUgYHh4eGAgaXMgYSBjb21wb3VuZCBpbnRlcmVzdCByYXRlLgpUaGUgcmVhbCBtdWx0aXBsaWVyKGUuZy4gMS4zMiwgMi41MywgZXRjKSBpcyBzY2FsZWQgdXAgd2l0aCBbYFNDQUxFRF9PTkVgXSB2YWx1ZS4AAAAAAAAAABdDb21wb3VuZFJhdGVNdWx0aXBsaWVycwAAAAACAAAAAAAAAAZib3Jyb3cAAAAAAAsAAAAAAAAABnN1cHBseQAAAAAACw==",
        "AAAAAQAAADNDb21wb3VuZCBpbnRlcmVzdCByYXRlcyByZXByZXNlbnRlZCBpbiBiYXNpcyBwb2ludHMAAAAAAAAAAA1Db21wb3VuZFJhdGVzAAAAAAAAAgAAAAAAAAAKYm9ycm93X2JwcwAAAAAABAAAAAAAAAAKc3VwcGx5X2JwcwAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAMAAABEQm9ycm93ZWQgbGlxdWlkaXR5IGZvciB0aGUgb2JsaWdhdGlvbiwgdW5pcXVlIGJ5IGJvcnJvdyBwb29sIGFkZHJlc3MAAAAHYm9ycm93cwAAAAPsAAAH0AAAAAtQb29sQWRkcmVzcwAAAAfQAAAAEEJvcnJvd09ibGlnYXRpb24AAABHRGVwb3NpdGVkIGNvbGxhdGVyYWwgZm9yIHRoZSBvYmxpZ2F0aW9uLCB1bmlxdWUgYnkgZGVwb3NpdCBwb29sIGFkZHJlc3MAAAAACGRlcG9zaXRzAAAD7AAAB9AAAAALUG9vbEFkZHJlc3MAAAAH0AAAABFEZXBvc2l0T2JsaWdhdGlvbgAAAAAAABVUaGUgb2JsaWdhdGlvbidzIHVzZXIAAAAAAAAEdXNlcgAAABM=",
        "AAAAAQAAAAAAAAAAAAAAEEJvcnJvd09ibGlnYXRpb24AAAADAAAAKFRoZSBpbml0aWFsIGFtb3VudCBvZiB0aGUgYm9ycm93ZWQgdG9rZW4AAAAIYm9ycm93ZWQAAAALAAAAuVRoZSBudW1lcmljYWwgdmFsdWUgdGhhdCBpcyB1c2VkIHRvIGRldGVybWluZSB0aGUgc2NhbGluZyBmYWN0b3IgcmVxdWlyZWQgZm9yIHVwZGF0aW5nIHRoZSBwb3NpdGlvbiBhbW91bnQKd2l0aCBpbnRlcmVzdCwgaS5lLiBuZXdfYm9ycm93ZWQgPSAoY3VycmVudF9hY2NydWFsIFwgbGFzdF9hY2NydWFsKSAqIGJvcnJvd2VkAAAAAAAADGxhc3RfYWNjcnVhbAAAAAsAAAAdVGhlIGFtb3VudCBvZiB1bnBhaWQgaW50ZXJlc3QAAAAAAAAPdW5wYWlkX2ludGVyZXN0AAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAEURlcG9zaXRPYmxpZ2F0aW9uAAAAAAAAAgAAAAAAAAAKY29sbGF0ZXJhbAAAAAAACwAAAAAAAAAGc2hhcmVzAAAAAAAL",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAAKAAAALFRoZSBjdXJyZW50bHkgYXZhaWxhYmxlIGZvciBib3Jyb3dpbmcgdG9rZW5zAAAACWF2YWlsYWJsZQAAAAAAAAsAAAAjQ29uZmlndXJhdGlvbiBzZXR0aW5ncyBmb3IgdGhlIHBvb2wAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAC5VGhlIG51bWVyaWNhbCB2YWx1ZSB0aGF0IGlzIHVzZWQgdG8gZGV0ZXJtaW5lIHRoZSBzY2FsaW5nIGZhY3RvciByZXF1aXJlZCBmb3IgdXBkYXRpbmcgdGhlIGJvcnJvd2VkIGFtb3VudAp3aXRoIGludGVyZXN0LCBpLmUuIG5ld19ib3Jyb3dlZCA9IChjdXJyZW50X2FjY3J1YWwgXCBsYXN0X2FjY3J1YWwpICogYm9ycm93ZWQAAAAAAAAMbGFzdF9hY2NydWFsAAAACwAAADBUaGUgdGltZXN0YW1wIG9mIHRoZSBsYXN0IGFjY3J1YWwgcmUtY2FsY3VsYXRpb24AAAAWbGFzdF9hY2NydWFsX3RpbWVzdGFtcAAAAAAABgAAABxUaGUgYWRkcmVzcyBvZiB0aGUgbG9hbiBwb29sAAAADHBvb2xfYWRkcmVzcwAAABMAAAAxVGhlIGFkZHJlc3Mgb2YgdGhlIHRva2VuIGFzc29jaWF0ZWQgd2l0aCB0aGUgcG9vbAAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAFpUaGUgdGlja2VyIHN5bWJvbCBvZiB0aGUgYXNzb2NpYXRlZCB0b2tlbiwgd2hpY2ggaXMgdXNlZCB0byBpZGVudGlmeSB0aGUgdG9rZW4gaW4gdGhlIHBvb2wAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAVFRoZSB0b3RhbCBhbW91bnQgb2YgYm9ycm93ZWQgYXNzZXRzLiBUaGlzIHZhbHVlIGluY3JlYXNlcyB3aXRoIGludGVyZXN0IHJhdGUgYWNjcnVhbAAAAA50b3RhbF9ib3Jyb3dlZAAAAAAACwAAAEpUaGUgdG90YWwgYW1vdW50IG9mIGRlcG9zaXRlZCBjb2xsYXRlcmFsIGFzc2V0cyB0aGF0IGRvbid0IGFjY3J1ZSBpbnRlcmVzdAAAAAAAEHRvdGFsX2NvbGxhdGVyYWwAAAALAAAAOVRoZSB0b3RhbCBhbW91bnQgb2YgZGVwb3NpdGVkIGFzc2V0cyB0aGF0IGFjY3J1ZSBpbnRlcmVzdAAAAAAAAAx0b3RhbF9zaGFyZXMAAAAL",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAcAAAB1QmFzZSBpbnRlcmVzdCByYXRlIGFwcGxpZWQgcmVnYXJkbGVzcyBvZiB1dGlsaXphdGlvbiwgZXhwcmVzc2VkIHBlciBzZWNvbmQKaW4gMS9gU0NBTEVEX09ORWAgdW5pdHMuIE11c3QgYmUgcG9zaXRpdmUuAAAAAAAAFGJhc2VfcmF0ZV9wZXJfc2Vjb25kAAAACwAAAD9NYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYSBib3Jyb3dlcidzIGRlYnQgdGhhdCBjYW4gYmUgbGlxdWlkYXRlZC4AAAAAHGxpcXVpZGF0aW9uX2Nsb3NlX2ZhY3Rvcl9icHMAAAALAAAAREFkZGl0aW9uYWwgZGlzY291bnQgZ2l2ZW4gdG8gbGlxdWlkYXRvcnMgd2hlbiBwdXJjaGFzaW5nIGNvbGxhdGVyYWwuAAAAGWxpcXVpZGF0aW9uX2luY2VudGl2ZV9icHMAAAAAAAALAAAAIlBvc2l0aXZlIE9wdGltYWwgVXRpbGl6YXRpb24gUmF0aW8AAAAAAB1vcHRpbWFsX3V0aWxpemF0aW9uX3JhdGlvX2JwcwAAAAAAAAsAAAA/UGVyY2VudGFnZSBvZiBpbnRlcmVzdCBwYXltZW50cyBhbGxvY2F0ZWQgdG8gcHJvdG9jb2wgcmVzZXJ2ZXMuAAAAABFyZXNlcnZlX3JhdGlvX2JwcwAAAAAAAAsAAACRSW50ZXJlc3QgcmF0ZSBzbG9wZSBiZWZvcmUgcmVhY2hpbmcgb3B0aW1hbCB1dGlsaXphdGlvbiByYXRpby4KQ29udHJvbHMgaG93IGFnZ3Jlc3NpdmVseSByYXRlcyBpbmNyZWFzZSB3aXRoIHV0aWxpemF0aW9uIGJlbG93IHRoZSBvcHRpbWFsIHBvaW50LgAAAAAAAAZzbG9wZTEAAAAAAAsAAACRSW50ZXJlc3QgcmF0ZSBzbG9wZSBhZnRlciBleGNlZWRpbmcgb3B0aW1hbCB1dGlsaXphdGlvbiByYXRpby4KQ29udHJvbHMgaG93IGFnZ3Jlc3NpdmVseSByYXRlcyBpbmNyZWFzZSB3aXRoIHV0aWxpemF0aW9uIGFib3ZlIHRoZSBvcHRpbWFsIHBvaW50LgAAAAAAAAZzbG9wZTIAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAB0FjY3J1YWwAAAAAAwAAAAAAAAAOYm9ycm93X2FjY3J1YWwAAAAAAAsAAAAAAAAAD2RlcG9zaXRfYWNjcnVhbAAAAAALAAAAAAAAAAl0aW1lc3RhbXAAAAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbGlxdWlkYXRpb25fdGhyZXNob2xkX2JwcwAAAAAAAAsAAAAAAAAABnN0YXR1cwAAAAAAAQ==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAEAAAAAAAAABFBvb2wAAAABAAAH0AAAAAtQb29sQWRkcmVzcwAAAAABAAAAAAAAAApPYmxpZ2F0aW9uAAAAAAABAAAH0AAAAAtVc2VyQWRkcmVzcwAAAAAAAAAAAAAAAAdBY2NydWFsAAAAAAAAAAAAAAAACEFsbFBvb2xz" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_global_state: this.txFromJSON<GlobalState>,
        initialize_pool: this.txFromJSON<Result<PoolAddress>>,
        deposit: this.txFromJSON<Result<void>>,
        swap: this.txFromJSON<Result<i128>>,
        borrow: this.txFromJSON<Result<void>>,
        add_collateral: this.txFromJSON<Result<void>>,
        repay: this.txFromJSON<Result<void>>,
        liquidate: this.txFromJSON<Result<void>>,
        remove_collateral: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        flash_loan: this.txFromJSON<Result<void>>,
        deposit_with_leverage: this.txFromJSON<Result<void>>,
        deleverage_and_withdraw: this.txFromJSON<Result<void>>,
        get_user_obligation: this.txFromJSON<Result<Obligation>>,
        get_pool: this.txFromJSON<Result<Pool>>,
        get_all_pools: this.txFromJSON<Array<PoolAddress>>,
        get_apy: this.txFromJSON<Result<CompoundRates>>
  }
}