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
borrows: Map<string, BorrowObligation>;
  /**
 * Deposited collateral for the obligation, unique by deposit pool address
 */
deposits: Map<string, DepositObligation>;
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
 * in 1/`SCALED_ONE` units. Must be positive
 */
base_rate_per_second: i128;
  /**
 * The maximum percentage of an asset's value that can be held in an individual obligation in basis points
 * with respect to a total obligation's collateral value. LTV greater than that makes borrow position eligible to liquidation
 */
close_ltv_bps: i128;
  /**
 * Maximum percentage of a borrower's debt that can be liquidated
 */
liquidation_close_factor_bps: i128;
  /**
 * Additional discount given to liquidators when purchasing collateral
 */
liquidation_incentive_bps: i128;
  /**
 * The maximum percentage of an asset's value that can be borrowed in basis points(e.g, 7000 = 70%, etc)
 * with respect to a total obligation's collateral value
 */
open_ltv_bps: i128;
  /**
 * Positive Optimal Utilization Ratio
 */
optimal_utilization_ratio_bps: i128;
  /**
 * Percentage of interest payments allocated to protocol reserves
 */
reserve_ratio_bps: i128;
  /**
 * Interest rate slope before reaching optimal utilization ratio
 * Controls how aggressively rates increase with utilization below the optimal point
 */
slope1: i128;
  /**
 * Interest rate slope after exceeding optimal utilization ratio
 * Controls how aggressively rates increase with utilization above the optimal point
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

export type DataKey = {tag: "GlobalState", values: void} | {tag: "Pool", values: readonly [string]} | {tag: "Obligation", values: readonly [string]} | {tag: "Accrual", values: void} | {tag: "AllPools", values: void};

export const SoroswapLibraryError = {
  /**
   * SoroswapLibrary: insufficient amount
   */
  301: {message:"InsufficientAmount"},
  /**
   * SoroswapLibrary: insufficient liquidity
   */
  302: {message:"InsufficientLiquidity"},
  /**
   * SoroswapLibrary: insufficient input amount
   */
  303: {message:"InsufficientInputAmount"},
  /**
   * SoroswapLibrary: insufficient output amount
   */
  304: {message:"InsufficientOutputAmount"},
  /**
   * SoroswapLibrary: invalid path
   */
  305: {message:"InvalidPath"},
  /**
   * SoroswapLibrary: token_a and token_b have identical addresses
   */
  306: {message:"SortIdenticalTokens"}
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
  }) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_health_factor transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Gets computed user's current health factor in basis points (e.g, 9_200 = 0.92, 10_000 = 1, 10_500 = 1,05, etc)
   * 
   * ### Arguments
   * * `user` - user which health factor is computed
   */
  get_health_factor: ({user}: {user: string}, options?: {
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
   * Construct and simulate a get_asset_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns asset's decimals. Since Soroban smart contracts can operate only with SAC tokens, this value is currently always 7
   */
  get_asset_decimals: (options?: {
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
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_oracle_price_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns oracle price's decimals
   */
  get_oracle_price_decimals: (options?: {
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
  }) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_pool_asset_oracle_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns pool asset's oracle price
   * 
   * ### Arguments
   * * `pool_address` - address of asset which price is returned
   */
  get_pool_asset_oracle_price: ({pool_address}: {pool_address: string}, options?: {
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
   * * `leverage_multiplier` - leverage multiplier as a decimal (e.g., 7.0 for x7, 2.5 for x2.5, etc)
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
   * Returns the user's obligation which includes data about all of their deposits and borrows
   * 
   * ### Arguments
   * * `user` - user which obligation is returned
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
   * Returns the specific loan pool
   * 
   * ### Arguments
   * * `pool_address` - pool which data is returned
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
  }) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_apy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns APY calculated for the current utilization ratio of a pool in basis points (e.g., 2912 = 29.12%, etc)
   * 
   * ### Arguments
   * * `pool_address` - address of a pool for which APY is returned
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

  /**
   * Construct and simulate a get_optimal_apy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns APY calculated for the optimal utilization ratio of a pool in basis points (e.g., 4000 = 40.00%, etc)
   * 
   * ### Arguments
   * * `pool_address` - address of a pool for which optimal APY is returned
   */
  get_optimal_apy: ({_pool_address}: {_pool_address: string}, options?: {
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

  /**
   * Construct and simulate a sort_tokens transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Sorts two token addresses in a consistent order.
   * 
   * # Arguments
   * 
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<(Address, Address), SoroswapLibraryError>` where `Ok` contains a tuple with the sorted token addresses, and `Err` indicates an error such as identical tokens.
   */
  sort_tokens: ({token_a, token_b}: {token_a: string, token_b: string}, options?: {
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
  }) => Promise<AssembledTransaction<Result<readonly [string, string]>>>

  /**
   * Construct and simulate a pair_for transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Calculates the deterministic address for a pair without making any external calls.
   * check <https://github.com/paltalabs/deterministic-address-soroban>
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<Address, SoroswapLibraryError>` where `Ok` contains the deterministic address for the pair, and `Err` indicates an error such as identical tokens or an issue with sorting.
   */
  pair_for: ({factory, token_a, token_b}: {factory: string, token_a: string, token_b: string}, options?: {
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
  }) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a get_reserves_with_factory transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Fetches and sorts the reserves for a pair of tokens using the factory address.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<(i128, i128), SoroswapLibraryError>` where `Ok` contains a tuple of sorted reserves, and `Err` indicates an error such as identical tokens or an issue with sorting.
   */
  get_reserves_with_factory: ({factory, token_a, token_b}: {factory: string, token_a: string, token_b: string}, options?: {
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
  }) => Promise<AssembledTransaction<Result<readonly [i128, i128]>>>

  /**
   * Construct and simulate a get_reserves_with_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Fetches and sorts the reserves for a pair of tokens using the pair address.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `pair` - The pair address.
   * * `token_a` - The address of the first token.
   * * `token_b` - The address of the second token.
   * 
   * # Returns
   * 
   * Returns `Result<(i128, i128), SoroswapLibraryError>` where `Ok` contains a tuple of sorted reserves, and `Err` indicates an error such as identical tokens or an issue with sorting.
   */
  get_reserves_with_pair: ({pair, token_a, token_b}: {pair: string, token_a: string, token_b: string}, options?: {
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
  }) => Promise<AssembledTransaction<Result<readonly [i128, i128]>>>

  /**
   * Construct and simulate a quote transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given some amount of an asset and pair reserves, returns an equivalent amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_a` - The amount of the first asset.
   * * `reserve_a` - Reserves of the first asset in the pair.
   * * `reserve_b` - Reserves of the second asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the calculated equivalent amount, and `Err` indicates an error such as insufficient amount or liquidity
   */
  quote: ({amount_a, reserve_a, reserve_b}: {amount_a: i128, reserve_a: i128, reserve_b: i128}, options?: {
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
   * Construct and simulate a get_amount_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given an input amount of an asset and pair reserves, returns the maximum output amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_in` - The input amount of the asset.
   * * `reserve_in` - Reserves of the input asset in the pair.
   * * `reserve_out` - Reserves of the output asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the calculated maximum output amount, and `Err` indicates an error such as insufficient input amount or liquidity.
   */
  get_amount_out: ({amount_in, reserve_in, reserve_out}: {amount_in: i128, reserve_in: i128, reserve_out: i128}, options?: {
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
   * Construct and simulate a get_amount_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Given an output amount of an asset and pair reserves, returns a required input amount of the other asset.
   * 
   * # Arguments
   * 
   * * `amount_out` - The output amount of the asset.
   * * `reserve_in` - Reserves of the input asset in the pair.
   * * `reserve_out` - Reserves of the output asset in the pair.
   * 
   * # Returns
   * 
   * Returns `Result<i128, SoroswapLibraryError>` where `Ok` contains the required input amount, and `Err` indicates an error such as insufficient output amount or liquidity.
   */
  get_amount_in: ({amount_out, reserve_in, reserve_out}: {amount_out: i128, reserve_in: i128, reserve_out: i128}, options?: {
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
   * Construct and simulate a get_amounts_out transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Performs chained get_amount_out calculations on any number of pairs.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `amount_in` - The input amount.
   * * `path` - Vector of token addresses representing the path.
   * 
   * # Returns
   * 
   * Returns `Result<Vec<i128>, SoroswapLibraryError>` where `Ok` contains a vector of calculated amounts, and `Err` indicates an error such as an invalid path.
   */
  get_amounts_out: ({factory, amount_in, path}: {factory: string, amount_in: i128, path: Array<string>}, options?: {
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
  }) => Promise<AssembledTransaction<Result<Array<i128>>>>

  /**
   * Construct and simulate a get_amounts_in transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Performs chained get_amount_in calculations on any number of pairs.
   * 
   * # Arguments
   * 
   * * `e` - The environment.
   * * `factory` - The factory address.
   * * `amount_out` - The output amount.
   * * `path` - Vector of token addresses representing the path.
   * 
   * # Returns
   * 
   * Returns `Result<Vec<i128>, SoroswapLibraryError>` where `Ok` contains a vector of calculated amounts, and `Err` indicates an error such as an invalid path.
   */
  get_amounts_in: ({factory, amount_out, path}: {factory: string, amount_out: i128, path: Array<string>}, options?: {
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
  }) => Promise<AssembledTransaction<Result<Array<i128>>>>

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
        "AAAAAAAAAZZJbml0aWFsaXplcyBhIGxvYW4gcG9vbCBmb3IgYSBzcGVjaWZpYyBhc3NldAoKIyMjIEFyZ3VtZW50cwoqIGB0b2tlbl9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBjb3JyZXNwb25kaW5nIFNvcm9iYW4gQXNzZXQgQ29udHJhY3QKKiBgdG9rZW5fc3ltYm9sYCAtIHN5bWJvbCB3aGljaCByZXByZXNlbnRzIGEgcG9vbCdzIHRva2VuCiogYHNhbHRgIC0gb3B0aW9uYWwgc2FsdCBkYXRhLCB3aGljaCB3aGVuIHByb3ZpZGVkIGlzIHVzZWQgYWxvbmcgd2l0aCBgdG9rZW5fYWRkcmVzc2AgdG8gZGVyaXZlIGEgZGV0ZXJtaW5pc3RpYyBwb29sIGFkZHJlc3MKKiBgcG9vbF9jb25maWdgIC0gb3B0aW9uYWwgYFBvb2xDb25maWdgIGRhdGEuIElmIG5vdCBwcm92aWRlZCAtIGEgZGVmYXVsdCBwb29sIGNvbmZpZyBpcyB1c2VkAAAAAAAPaW5pdGlhbGl6ZV9wb29sAAAAAAQAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAAAAAAARzYWx0AAAD6AAAA+4AAAAgAAAAAAAAAAtwb29sX2NvbmZpZwAAAAPoAAAH0AAAAApQb29sQ29uZmlnAAAAAAABAAAD6QAAABMAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAK1HZXRzIGNvbXB1dGVkIHVzZXIncyBjdXJyZW50IGhlYWx0aCBmYWN0b3IgaW4gYmFzaXMgcG9pbnRzIChlLmcsIDlfMjAwID0gMC45MiwgMTBfMDAwID0gMSwgMTBfNTAwID0gMSwwNSwgZXRjKQoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggaGVhbHRoIGZhY3RvciBpcyBjb21wdXRlZAAAAAAAABFnZXRfaGVhbHRoX2ZhY3RvcgAAAAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAA+kAAAALAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAHpSZXR1cm5zIGFzc2V0J3MgZGVjaW1hbHMuIFNpbmNlIFNvcm9iYW4gc21hcnQgY29udHJhY3RzIGNhbiBvcGVyYXRlIG9ubHkgd2l0aCBTQUMgdG9rZW5zLCB0aGlzIHZhbHVlIGlzIGN1cnJlbnRseSBhbHdheXMgNwAAAAAAEmdldF9hc3NldF9kZWNpbWFscwAAAAAAAAAAAAEAAAAE",
        "AAAAAAAAAB9SZXR1cm5zIG9yYWNsZSBwcmljZSdzIGRlY2ltYWxzAAAAABlnZXRfb3JhY2xlX3ByaWNlX2RlY2ltYWxzAAAAAAAAAAAAAAEAAAAE",
        "AAAAAAAAAGxSZXR1cm5zIHBvb2wgYXNzZXQncyBvcmFjbGUgcHJpY2UKCiMjIyBBcmd1bWVudHMKKiBgcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYXNzZXQgd2hpY2ggcHJpY2UgaXMgcmV0dXJuZWQAAAAbZ2V0X3Bvb2xfYXNzZXRfb3JhY2xlX3ByaWNlAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAAAsAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAANhEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCBkZXBvc2l0cyBhIHRva2VuCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgZGVwb3NpdCBoYXBwZW5zCiogYGFtb3VudGAgLSBhbW91bnQgb2YgdG9rZW5zIHdoaWNoIGFyZSBnb2luZyB0byBiZSBkZXBvc2l0ZWQAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAARFTd2FwIHRva2VucyB2aWEgYSBzd2FwIHByb3ZpZGVyIGNvbnRyYWN0CgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCBkZXBvc2l0cyBhIHRva2VuCiogYHRva2VuX2luYCAtIGFkZHJlc3Mgb2YgYSB0b2tlbiB0aGF0IHdvdWxkIGJlIHRha2VuIGZyb20gdGhlIHVzZXIKKiBgdG9rZW5fb3V0YCAtIGFkZHJlc3Mgb2YgYSB0b2tlbiB0aGF0IHdvdWxkIGJlIGdpdmVuIHRvIHRoZSB1c2VyCiogYGFtb3VudGAgLSBleGFjdCBhbW91bnQgb2YgdGhlIGB0b2tlbl9pbmAAAAAAAAAEc3dhcAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAh0b2tlbl9pbgAAABMAAAAAAAAACXRva2VuX291dAAAAAAAABMAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAANZCb3Jyb3dzIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGJvcnJvd3MgYSB0b2tlbgoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgYm9ycm93IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGdvaW5nIHRvIGJlIGJvcnJvd2VkAAAAAAAGYm9ycm93AAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAWlBZGRzIHRva2VucyBpbnRvIHRoZSBsb2FuIHBvb2wgYXMgY29sbGF0ZXJhbCBvbmx5LgpUaGlzIGltcGxpZXMgdGhhdCB0aGV5IGFyZSBhbHdheXMgYXZhaWxhYmxlIGZvciBhIGhlYWx0aHkgd2l0aGRyYXdhbCBmb3IgdGhlCmNvc3Qgb2Ygbm90IGFjY3J1aW5nIGFuIGludGVyZXN0IHJhdGUKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHRoYXQgYWRkcyBjb2xsYXRlcmFsCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgY29sbGF0ZXJhbCBpcyBiZWluZyBhZGRlZAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHRva2VucyB3aGljaCBhcmUgYmVpbmcgYWRkZWQgYXMgYSBjb2xsYXRlcmFsAAAAAAAADmFkZF9jb2xsYXRlcmFsAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAALtSZXBheXMgYm9ycm93ZWQgdG9rZW5zCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCByZXBheXMgYm9ycm93ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSBib3Jyb3cgaGFwcGVuZWQKKiBgYW1vdW50YCAtIGFtb3VudCBvZiByZXBhaWQgdG9rZW5zAAAAAAVyZXBheQAAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAYlMaXF1aWRhdGVzIGJvcnJvd2VyJ3MgcG9zaXRpb24gaWYgcG9zaXRpb24ncyBoZWFsdGggZmFjdG9yIGNyaXRlcmlvbiBpc24ndCBtZXQKCiMjIyBBcmd1bWVudHMKKiBgbGlxdWlkYXRvcmAgLSBhZ2VudCB3aGljaCBsaXF1aWRhdGVzIHRoZSBib3Jyb3dlcidzIHBvc2l0aW9uCiogYGJvcnJvd19wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgYm9ycm93ZWQgdG9rZW5zIGFyZSByZXBhaWQgYnkgdGhlIGxpcXVpZGF0b3IKKiBgY29sbGF0ZXJhbF9wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgdG9rZW5zIGFyZSBzb2xkIHRvIHRoZSBsaXF1aWRhdG9yIHdpdGggYSBkaXNjb3VudAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHJlcGFpZCB0b2tlbnMAAAAAAAAJbGlxdWlkYXRlAAAAAAAABQAAAAAAAAAKbGlxdWlkYXRvcgAAAAAAEwAAAAAAAAAIYm9ycm93ZXIAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAAF2NvbGxhdGVyYWxfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAOhSZW1vdmVzIGNvbGxhdGVyYWwgdG9rZW5zIGZyb20gdGhlIGxvYW4gcG9vbCB0byB0aGUgdXNlcgoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggd2l0aGRyYXdzIGNvbGxhdGVyYWwgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB3aXRoZHJhd24gdG9rZW5zAAAAEXJlbW92ZV9jb2xsYXRlcmFsAAAAAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAOhXaXRoZHJhd3MgZGVwb3NpdGVkIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wgdG8gdGhlIHVzZXIKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIHdpdGhkcmF3cyBkZXBvc2l0ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB3aXRoZHJhd24gdG9rZW5zAAAACHdpdGhkcmF3AAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAPlDcmVhdGVzIGEgZmxhc2ggbG9hbgoKIyMjIEFyZ3VtZW50cwoqIGBjb250cmFjdGAgLSBjb250cmFjdCdzIGFkZHJlc3Mgd2hpY2ggbGV2ZXJhZ2VzIHRoZSBmbGFzaCBsb2FuZWQgYW1vdW50IGFuZCBhZGhlcmVzIHRvIGBlcmMzMTU2YCBzdGFuZGFyZAoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgZmxhc2ggbG9hbiBoYXBwZW5zCiogYGFtb3VudGAgLSBhbW91bnQgb2YgbGVudCB0b2tlbnMAAAAAAAAKZmxhc2hfbG9hbgAAAAAAAwAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAnpEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sIHdpdGggbGV2ZXJhZ2UuIExldmVyYWdlIGlzIGFjaGlldmVkIGJ5IHV0aWxpemluZyBmbGFzaCBsb2FuIGFuZCB0b2tlbiBzd2FwCgojIFdBUk5JTkcKVGhpcyBpbmNyZWFzZXMgdGhlIHBlcmNlaXZlZCBgc3VwcGx5IEFQUmAgb25seQp3aGVuIGAoYm9ycm93ZWQgdG9rZW4gYm9ycm93IEFQUiA8IHN1cHBseSB0b2tlbiBzdXBwbHkgQVBSKWAgaG9sZHMgdHJ1ZQoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgdGhhdCBkZXBvc2l0cyB0b2tlbnMgd2l0aCBsZXZlcmFnZQoqIGBkZXBvc2l0X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIHRvIHdoaWNoIHRoZSBkZXBvc2l0IGhhcHBlbnMKKiBgYm9ycm93X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIGZyb20gd2hpY2ggdGhlIGJvcnJvdyBoYXBwZW5zCiogYGFtb3VudGAgLSBvcmlnaW5hbCBib3Jyb3cgYW1vdW50IGJlZm9yZSB0aGUgbGV2ZXJhZ2UKKiBgbGV2ZXJhZ2VfbXVsdGlwbGllcmAgLSBsZXZlcmFnZSBtdWx0aXBsaWVyIGFzIGEgZGVjaW1hbCAoZS5nLiwgNy4wIGZvciB4NywgMi41IGZvciB4Mi41LCBldGMpAAAAAAAVZGVwb3NpdF93aXRoX2xldmVyYWdlAAAAAAAABQAAAAAAAAAEdXNlcgAAABMAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAE2xldmVyYWdlX211bHRpcGxpZXIAAAAABAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAW5EZWxldmVyYWdlcyBhbmQgd2l0aGRyYXdzIHRva2VucyBmcm9tIHRoZSBsZXZlcmFnZWQgZGVwb3NpdCBwb3NpdGlvbgoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgdGhhdCBkZWxldmVyYWdlcyBhbmQgd2l0aGRyYXdzIGZyb20gdGhlIHBvc2l0aW9uCiogYGRlcG9zaXRfcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIGZyb20gdGhlIHBhaXIgdG8gd2hpY2ggdGhlIGRlcG9zaXQgaGFwcGVuZWQKKiBgYm9ycm93X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIGZyb20gd2hpY2ggdGhlIGJvcnJvdyBoYXBwZW5lZAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHdpdGhkcmF3biB0b2tlbnMAAAAAABdkZWxldmVyYWdlX2FuZF93aXRoZHJhdwAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAJVSZXR1cm5zIHRoZSB1c2VyJ3Mgb2JsaWdhdGlvbiB3aGljaCBpbmNsdWRlcyBkYXRhIGFib3V0IGFsbCBvZiB0aGVpciBkZXBvc2l0cyBhbmQgYm9ycm93cwoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggb2JsaWdhdGlvbiBpcyByZXR1cm5lZAAAAAAAABNnZXRfdXNlcl9vYmxpZ2F0aW9uAAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAA+kAAAfQAAAACk9ibGlnYXRpb24AAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAFxSZXR1cm5zIHRoZSBzcGVjaWZpYyBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgcG9vbF9hZGRyZXNzYCAtIHBvb2wgd2hpY2ggZGF0YSBpcyByZXR1cm5lZAAAAAhnZXRfcG9vbAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAAEUG9vbAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAADRSZXR1cm5zIGEgbGlzdCBvZiBhbGwgcG9vbCBhZGRyZXNzZXMgaW4gdGhlIHByb3RvY29sAAAADWdldF9hbGxfcG9vbHMAAAAAAAAAAAAAAQAAA+oAAAAT",
        "AAAAAAAAALtSZXR1cm5zIEFQWSBjYWxjdWxhdGVkIGZvciB0aGUgY3VycmVudCB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wgaW4gYmFzaXMgcG9pbnRzIChlLmcuLCAyOTEyID0gMjkuMTIlLCBldGMpCgojIyMgQXJndW1lbnRzCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmb3Igd2hpY2ggQVBZIGlzIHJldHVybmVkAAAAAAdnZXRfYXB5AAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAANQ29tcG91bmRSYXRlcwAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAMNSZXR1cm5zIEFQWSBjYWxjdWxhdGVkIGZvciB0aGUgb3B0aW1hbCB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wgaW4gYmFzaXMgcG9pbnRzIChlLmcuLCA0MDAwID0gNDAuMDAlLCBldGMpCgojIyMgQXJndW1lbnRzCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmb3Igd2hpY2ggb3B0aW1hbCBBUFkgaXMgcmV0dXJuZWQAAAAAD2dldF9vcHRpbWFsX2FweQAAAAABAAAAAAAAAA1fcG9vbF9hZGRyZXNzAAAAAAAAEwAAAAEAAAPpAAAH0AAAAA1Db21wb3VuZFJhdGVzAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAABAAAAAAAAAAAAAAAFExlbmRpbmdDb250cmFjdEVycm9yAAAAHQAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAEVBvb2xBbHJlYWR5RXhpc3RzAAAAAAAAAQAAAAAAAAAQUG9vbERvZXNOb3RFeGlzdAAAAAIAAAAAAAAAFUludmFsaWRMb2FuUG9vbENvbmZpZwAAAAAAAAMAAAAAAAAAEk5vdEVub3VnaFBvb2xGdW5kcwAAAAAABAAAAAAAAAAWT2JsaWdhdGlvbkRvZXNOb3RFeGlzdAAAAAAABQAAAAAAAAATRGVwb3NpdERvZXNOb3RFeGlzdAAAAAAGAAAAAAAAABJOb25Qb3NpdGl2ZURlcG9zaXQAAAAAAAcAAAAAAAAAE05vblBvc2l0aXZlV2l0aGRyYXcAAAAACAAAAAAAAAATV2l0aGRyYXdPdmVyQmFsYW5jZQAAAAAJAAAAAAAAABBOb25Qb3NpdGl2ZVJlcGF5AAAACgAAAAAAAAAPT3Zlck9yVW5kZXJmbG93AAAAAAsAAAAAAAAAG09yYWNsZURvZXNOb3RLbm93QXNzZXRQcmljZQAAAAAMAAAAAAAAABJCb3Jyb3dEb2VzTm90RXhpc3QAAAAAAA0AAAAAAAAAKEhlYWx0aEZhY3RvcklzTG93ZXJUaGFuUmVxdWlyZWRUaHJlc2hvbGQAAAAOAAAAAAAAABtJbnZhbGlkTGlxdWlkYXRpb25UaHJlc2hvbGQAAAAADwAAAAAAAAAbTGlxdWlkYXRlZFBvc2l0aW9uSXNIZWFsdGh5AAAAABAAAAAAAAAAHUxpcXVpZGF0aW9uRXhjZWVkc0Nsb3NlRmFjdG9yAAAAAAAAEQAAAAAAAAAWTm9uUG9zaXRpdmVMaXF1aWRhdGlvbgAAAAAAEgAAAAAAAAARTm9uUG9zaXRpdmVCb3Jyb3cAAAAAAAATAAAAAAAAABpDb2xsYXRlcmFsUG9vbERvZXNOb3RFeGlzdAAAAAAAFAAAAAAAAAAUTm9uUG9zaXRpdmVGbGFzaExvYW4AAAAVAAAAAAAAABBJbnZhbGlkVGltZXN0YW1wAAAAFwAAAAAAAAAPU2VsZkxpcXVpZGF0aW9uAAAAABgAAAAAAAAAF0RlcG9zaXRQb29sRG9lc05vdEV4aXN0AAAAABsAAAAAAAAAFkJvcnJvd1Bvb2xEb2VzTm90RXhpc3QAAAAAABwAAAAAAAAAGUludmFsaWRMZXZlcmFnZU11bHRpcGxpZXIAAAAAAAAdAAAAAAAAABNJbnZhbGlkU3dhcFNsaXBwYWdlAAAAAB4AAAAAAAAAF0RlcGVuZGVuY3lDb250cmFjdEVycm9yAAAAAB8=",
        "AAAAAQAAAKtJbnRlcmVzdCByYXRlIG11bHRpcGxpZXJzIHByZXNlbnRlZCBhcyAoMSArIHh4eCkgd2hlcmUgYHh4eGAgaXMgYSBjb21wb3VuZCBpbnRlcmVzdCByYXRlLgpUaGUgcmVhbCBtdWx0aXBsaWVyKGUuZy4gMS4zMiwgMi41MywgZXRjKSBpcyBzY2FsZWQgdXAgd2l0aCBbYFNDQUxFRF9PTkVgXSB2YWx1ZS4AAAAAAAAAABdDb21wb3VuZFJhdGVNdWx0aXBsaWVycwAAAAACAAAAAAAAAAZib3Jyb3cAAAAAAAsAAAAAAAAABnN1cHBseQAAAAAACw==",
        "AAAAAQAAADNDb21wb3VuZCBpbnRlcmVzdCByYXRlcyByZXByZXNlbnRlZCBpbiBiYXNpcyBwb2ludHMAAAAAAAAAAA1Db21wb3VuZFJhdGVzAAAAAAAAAgAAAAAAAAAKYm9ycm93X2JwcwAAAAAABAAAAAAAAAAKc3VwcGx5X2JwcwAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAMAAABEQm9ycm93ZWQgbGlxdWlkaXR5IGZvciB0aGUgb2JsaWdhdGlvbiwgdW5pcXVlIGJ5IGJvcnJvdyBwb29sIGFkZHJlc3MAAAAHYm9ycm93cwAAAAPsAAAAEwAAB9AAAAAQQm9ycm93T2JsaWdhdGlvbgAAAEdEZXBvc2l0ZWQgY29sbGF0ZXJhbCBmb3IgdGhlIG9ibGlnYXRpb24sIHVuaXF1ZSBieSBkZXBvc2l0IHBvb2wgYWRkcmVzcwAAAAAIZGVwb3NpdHMAAAPsAAAAEwAAB9AAAAARRGVwb3NpdE9ibGlnYXRpb24AAAAAAAAVVGhlIG9ibGlnYXRpb24ncyB1c2VyAAAAAAAABHVzZXIAAAAT",
        "AAAAAQAAAAAAAAAAAAAAEEJvcnJvd09ibGlnYXRpb24AAAADAAAAKFRoZSBpbml0aWFsIGFtb3VudCBvZiB0aGUgYm9ycm93ZWQgdG9rZW4AAAAIYm9ycm93ZWQAAAALAAAAuVRoZSBudW1lcmljYWwgdmFsdWUgdGhhdCBpcyB1c2VkIHRvIGRldGVybWluZSB0aGUgc2NhbGluZyBmYWN0b3IgcmVxdWlyZWQgZm9yIHVwZGF0aW5nIHRoZSBwb3NpdGlvbiBhbW91bnQKd2l0aCBpbnRlcmVzdCwgaS5lLiBuZXdfYm9ycm93ZWQgPSAoY3VycmVudF9hY2NydWFsIFwgbGFzdF9hY2NydWFsKSAqIGJvcnJvd2VkAAAAAAAADGxhc3RfYWNjcnVhbAAAAAsAAAAdVGhlIGFtb3VudCBvZiB1bnBhaWQgaW50ZXJlc3QAAAAAAAAPdW5wYWlkX2ludGVyZXN0AAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAEURlcG9zaXRPYmxpZ2F0aW9uAAAAAAAAAgAAAAAAAAAKY29sbGF0ZXJhbAAAAAAACwAAAAAAAAAGc2hhcmVzAAAAAAAL",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAAKAAAALFRoZSBjdXJyZW50bHkgYXZhaWxhYmxlIGZvciBib3Jyb3dpbmcgdG9rZW5zAAAACWF2YWlsYWJsZQAAAAAAAAsAAAAjQ29uZmlndXJhdGlvbiBzZXR0aW5ncyBmb3IgdGhlIHBvb2wAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAC5VGhlIG51bWVyaWNhbCB2YWx1ZSB0aGF0IGlzIHVzZWQgdG8gZGV0ZXJtaW5lIHRoZSBzY2FsaW5nIGZhY3RvciByZXF1aXJlZCBmb3IgdXBkYXRpbmcgdGhlIGJvcnJvd2VkIGFtb3VudAp3aXRoIGludGVyZXN0LCBpLmUuIG5ld19ib3Jyb3dlZCA9IChjdXJyZW50X2FjY3J1YWwgXCBsYXN0X2FjY3J1YWwpICogYm9ycm93ZWQAAAAAAAAMbGFzdF9hY2NydWFsAAAACwAAADBUaGUgdGltZXN0YW1wIG9mIHRoZSBsYXN0IGFjY3J1YWwgcmUtY2FsY3VsYXRpb24AAAAWbGFzdF9hY2NydWFsX3RpbWVzdGFtcAAAAAAABgAAABxUaGUgYWRkcmVzcyBvZiB0aGUgbG9hbiBwb29sAAAADHBvb2xfYWRkcmVzcwAAABMAAAAxVGhlIGFkZHJlc3Mgb2YgdGhlIHRva2VuIGFzc29jaWF0ZWQgd2l0aCB0aGUgcG9vbAAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAFpUaGUgdGlja2VyIHN5bWJvbCBvZiB0aGUgYXNzb2NpYXRlZCB0b2tlbiwgd2hpY2ggaXMgdXNlZCB0byBpZGVudGlmeSB0aGUgdG9rZW4gaW4gdGhlIHBvb2wAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAVFRoZSB0b3RhbCBhbW91bnQgb2YgYm9ycm93ZWQgYXNzZXRzLiBUaGlzIHZhbHVlIGluY3JlYXNlcyB3aXRoIGludGVyZXN0IHJhdGUgYWNjcnVhbAAAAA50b3RhbF9ib3Jyb3dlZAAAAAAACwAAAEpUaGUgdG90YWwgYW1vdW50IG9mIGRlcG9zaXRlZCBjb2xsYXRlcmFsIGFzc2V0cyB0aGF0IGRvbid0IGFjY3J1ZSBpbnRlcmVzdAAAAAAAEHRvdGFsX2NvbGxhdGVyYWwAAAALAAAAOVRoZSB0b3RhbCBhbW91bnQgb2YgZGVwb3NpdGVkIGFzc2V0cyB0aGF0IGFjY3J1ZSBpbnRlcmVzdAAAAAAAAAx0b3RhbF9zaGFyZXMAAAAL",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAkAAAB0QmFzZSBpbnRlcmVzdCByYXRlIGFwcGxpZWQgcmVnYXJkbGVzcyBvZiB1dGlsaXphdGlvbiwgZXhwcmVzc2VkIHBlciBzZWNvbmQKaW4gMS9gU0NBTEVEX09ORWAgdW5pdHMuIE11c3QgYmUgcG9zaXRpdmUAAAAUYmFzZV9yYXRlX3Blcl9zZWNvbmQAAAALAAAA4lRoZSBtYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYW4gYXNzZXQncyB2YWx1ZSB0aGF0IGNhbiBiZSBoZWxkIGluIGFuIGluZGl2aWR1YWwgb2JsaWdhdGlvbiBpbiBiYXNpcyBwb2ludHMKd2l0aCByZXNwZWN0IHRvIGEgdG90YWwgb2JsaWdhdGlvbidzIGNvbGxhdGVyYWwgdmFsdWUuIExUViBncmVhdGVyIHRoYW4gdGhhdCBtYWtlcyBib3Jyb3cgcG9zaXRpb24gZWxpZ2libGUgdG8gbGlxdWlkYXRpb24AAAAAAA1jbG9zZV9sdHZfYnBzAAAAAAAACwAAAD5NYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYSBib3Jyb3dlcidzIGRlYnQgdGhhdCBjYW4gYmUgbGlxdWlkYXRlZAAAAAAAHGxpcXVpZGF0aW9uX2Nsb3NlX2ZhY3Rvcl9icHMAAAALAAAAQ0FkZGl0aW9uYWwgZGlzY291bnQgZ2l2ZW4gdG8gbGlxdWlkYXRvcnMgd2hlbiBwdXJjaGFzaW5nIGNvbGxhdGVyYWwAAAAAGWxpcXVpZGF0aW9uX2luY2VudGl2ZV9icHMAAAAAAAALAAAAm1RoZSBtYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYW4gYXNzZXQncyB2YWx1ZSB0aGF0IGNhbiBiZSBib3Jyb3dlZCBpbiBiYXNpcyBwb2ludHMoZS5nLCA3MDAwID0gNzAlLCBldGMpCndpdGggcmVzcGVjdCB0byBhIHRvdGFsIG9ibGlnYXRpb24ncyBjb2xsYXRlcmFsIHZhbHVlAAAAAAxvcGVuX2x0dl9icHMAAAALAAAAIlBvc2l0aXZlIE9wdGltYWwgVXRpbGl6YXRpb24gUmF0aW8AAAAAAB1vcHRpbWFsX3V0aWxpemF0aW9uX3JhdGlvX2JwcwAAAAAAAAsAAAA+UGVyY2VudGFnZSBvZiBpbnRlcmVzdCBwYXltZW50cyBhbGxvY2F0ZWQgdG8gcHJvdG9jb2wgcmVzZXJ2ZXMAAAAAABFyZXNlcnZlX3JhdGlvX2JwcwAAAAAAAAsAAACPSW50ZXJlc3QgcmF0ZSBzbG9wZSBiZWZvcmUgcmVhY2hpbmcgb3B0aW1hbCB1dGlsaXphdGlvbiByYXRpbwpDb250cm9scyBob3cgYWdncmVzc2l2ZWx5IHJhdGVzIGluY3JlYXNlIHdpdGggdXRpbGl6YXRpb24gYmVsb3cgdGhlIG9wdGltYWwgcG9pbnQAAAAABnNsb3BlMQAAAAAACwAAAI9JbnRlcmVzdCByYXRlIHNsb3BlIGFmdGVyIGV4Y2VlZGluZyBvcHRpbWFsIHV0aWxpemF0aW9uIHJhdGlvCkNvbnRyb2xzIGhvdyBhZ2dyZXNzaXZlbHkgcmF0ZXMgaW5jcmVhc2Ugd2l0aCB1dGlsaXphdGlvbiBhYm92ZSB0aGUgb3B0aW1hbCBwb2ludAAAAAAGc2xvcGUyAAAAAAAL",
        "AAAAAQAAAAAAAAAAAAAAB0FjY3J1YWwAAAAAAwAAAAAAAAAOYm9ycm93X2FjY3J1YWwAAAAAAAsAAAAAAAAAD2RlcG9zaXRfYWNjcnVhbAAAAAALAAAAAAAAAAl0aW1lc3RhbXAAAAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbGlxdWlkYXRpb25fdGhyZXNob2xkX2JwcwAAAAAAAAsAAAAAAAAABnN0YXR1cwAAAAAAAQ==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAABQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAEAAAAAAAAABFBvb2wAAAABAAAAEwAAAAEAAAAAAAAACk9ibGlnYXRpb24AAAAAAAEAAAATAAAAAAAAAAAAAAAHQWNjcnVhbAAAAAAAAAAAAAAAAAhBbGxQb29scw==",
        "AAAABAAAAAAAAAAAAAAAFFNvcm9zd2FwTGlicmFyeUVycm9yAAAABgAAACRTb3Jvc3dhcExpYnJhcnk6IGluc3VmZmljaWVudCBhbW91bnQAAAASSW5zdWZmaWNpZW50QW1vdW50AAAAAAEtAAAAJ1Nvcm9zd2FwTGlicmFyeTogaW5zdWZmaWNpZW50IGxpcXVpZGl0eQAAAAAVSW5zdWZmaWNpZW50TGlxdWlkaXR5AAAAAAABLgAAACpTb3Jvc3dhcExpYnJhcnk6IGluc3VmZmljaWVudCBpbnB1dCBhbW91bnQAAAAAABdJbnN1ZmZpY2llbnRJbnB1dEFtb3VudAAAAAEvAAAAK1Nvcm9zd2FwTGlicmFyeTogaW5zdWZmaWNpZW50IG91dHB1dCBhbW91bnQAAAAAGEluc3VmZmljaWVudE91dHB1dEFtb3VudAAAATAAAAAdU29yb3N3YXBMaWJyYXJ5OiBpbnZhbGlkIHBhdGgAAAAAAAALSW52YWxpZFBhdGgAAAABMQAAAD1Tb3Jvc3dhcExpYnJhcnk6IHRva2VuX2EgYW5kIHRva2VuX2IgaGF2ZSBpZGVudGljYWwgYWRkcmVzc2VzAAAAAAAAE1NvcnRJZGVudGljYWxUb2tlbnMAAAABMg==",
        "AAAAAAAAAVZTb3J0cyB0d28gdG9rZW4gYWRkcmVzc2VzIGluIGEgY29uc2lzdGVudCBvcmRlci4KCiMgQXJndW1lbnRzCgoqIGB0b2tlbl9hYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBmaXJzdCB0b2tlbi4KKiBgdG9rZW5fYmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgc2Vjb25kIHRva2VuLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8KEFkZHJlc3MsIEFkZHJlc3MpLCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgYSB0dXBsZSB3aXRoIHRoZSBzb3J0ZWQgdG9rZW4gYWRkcmVzc2VzLCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgaWRlbnRpY2FsIHRva2Vucy4AAAAAAAtzb3J0X3Rva2VucwAAAAACAAAAAAAAAAd0b2tlbl9hAAAAABMAAAAAAAAAB3Rva2VuX2IAAAAAEwAAAAEAAAPpAAAD7QAAAAIAAAATAAAAEwAAB9AAAAAUU29yb3N3YXBMaWJyYXJ5RXJyb3I=",
        "AAAAAAAAAgRDYWxjdWxhdGVzIHRoZSBkZXRlcm1pbmlzdGljIGFkZHJlc3MgZm9yIGEgcGFpciB3aXRob3V0IG1ha2luZyBhbnkgZXh0ZXJuYWwgY2FsbHMuCmNoZWNrIDxodHRwczovL2dpdGh1Yi5jb20vcGFsdGFsYWJzL2RldGVybWluaXN0aWMtYWRkcmVzcy1zb3JvYmFuPgoKIyBBcmd1bWVudHMKCiogYGVgIC0gVGhlIGVudmlyb25tZW50LgoqIGBmYWN0b3J5YCAtIFRoZSBmYWN0b3J5IGFkZHJlc3MuCiogYHRva2VuX2FgIC0gVGhlIGFkZHJlc3Mgb2YgdGhlIGZpcnN0IHRva2VuLgoqIGB0b2tlbl9iYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBzZWNvbmQgdG9rZW4uCgojIFJldHVybnMKClJldHVybnMgYFJlc3VsdDxBZGRyZXNzLCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgdGhlIGRldGVybWluaXN0aWMgYWRkcmVzcyBmb3IgdGhlIHBhaXIsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBpZGVudGljYWwgdG9rZW5zIG9yIGFuIGlzc3VlIHdpdGggc29ydGluZy4AAAAIcGFpcl9mb3IAAAADAAAAAAAAAAdmYWN0b3J5AAAAABMAAAAAAAAAB3Rva2VuX2EAAAAAEwAAAAAAAAAHdG9rZW5fYgAAAAATAAAAAQAAA+kAAAATAAAH0AAAABRTb3Jvc3dhcExpYnJhcnlFcnJvcg==",
        "AAAAAAAAAbZGZXRjaGVzIGFuZCBzb3J0cyB0aGUgcmVzZXJ2ZXMgZm9yIGEgcGFpciBvZiB0b2tlbnMgdXNpbmcgdGhlIGZhY3RvcnkgYWRkcmVzcy4KCiMgQXJndW1lbnRzCgoqIGBlYCAtIFRoZSBlbnZpcm9ubWVudC4KKiBgZmFjdG9yeWAgLSBUaGUgZmFjdG9yeSBhZGRyZXNzLgoqIGB0b2tlbl9hYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBmaXJzdCB0b2tlbi4KKiBgdG9rZW5fYmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgc2Vjb25kIHRva2VuLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8KGkxMjgsIGkxMjgpLCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgYSB0dXBsZSBvZiBzb3J0ZWQgcmVzZXJ2ZXMsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBpZGVudGljYWwgdG9rZW5zIG9yIGFuIGlzc3VlIHdpdGggc29ydGluZy4AAAAAABlnZXRfcmVzZXJ2ZXNfd2l0aF9mYWN0b3J5AAAAAAAAAwAAAAAAAAAHZmFjdG9yeQAAAAATAAAAAAAAAAd0b2tlbl9hAAAAABMAAAAAAAAAB3Rva2VuX2IAAAAAEwAAAAEAAAPpAAAD7QAAAAIAAAALAAAACwAAB9AAAAAUU29yb3N3YXBMaWJyYXJ5RXJyb3I=",
        "AAAAAAAAAa1GZXRjaGVzIGFuZCBzb3J0cyB0aGUgcmVzZXJ2ZXMgZm9yIGEgcGFpciBvZiB0b2tlbnMgdXNpbmcgdGhlIHBhaXIgYWRkcmVzcy4KCiMgQXJndW1lbnRzCgoqIGBlYCAtIFRoZSBlbnZpcm9ubWVudC4KKiBgcGFpcmAgLSBUaGUgcGFpciBhZGRyZXNzLgoqIGB0b2tlbl9hYCAtIFRoZSBhZGRyZXNzIG9mIHRoZSBmaXJzdCB0b2tlbi4KKiBgdG9rZW5fYmAgLSBUaGUgYWRkcmVzcyBvZiB0aGUgc2Vjb25kIHRva2VuLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8KGkxMjgsIGkxMjgpLCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgYSB0dXBsZSBvZiBzb3J0ZWQgcmVzZXJ2ZXMsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBpZGVudGljYWwgdG9rZW5zIG9yIGFuIGlzc3VlIHdpdGggc29ydGluZy4AAAAAAAAWZ2V0X3Jlc2VydmVzX3dpdGhfcGFpcgAAAAAAAwAAAAAAAAAEcGFpcgAAABMAAAAAAAAAB3Rva2VuX2EAAAAAEwAAAAAAAAAHdG9rZW5fYgAAAAATAAAAAQAAA+kAAAPtAAAAAgAAAAsAAAALAAAH0AAAABRTb3Jvc3dhcExpYnJhcnlFcnJvcg==",
        "AAAAAAAAAcVHaXZlbiBzb21lIGFtb3VudCBvZiBhbiBhc3NldCBhbmQgcGFpciByZXNlcnZlcywgcmV0dXJucyBhbiBlcXVpdmFsZW50IGFtb3VudCBvZiB0aGUgb3RoZXIgYXNzZXQuCgojIEFyZ3VtZW50cwoKKiBgYW1vdW50X2FgIC0gVGhlIGFtb3VudCBvZiB0aGUgZmlyc3QgYXNzZXQuCiogYHJlc2VydmVfYWAgLSBSZXNlcnZlcyBvZiB0aGUgZmlyc3QgYXNzZXQgaW4gdGhlIHBhaXIuCiogYHJlc2VydmVfYmAgLSBSZXNlcnZlcyBvZiB0aGUgc2Vjb25kIGFzc2V0IGluIHRoZSBwYWlyLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8aTEyOCwgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIHRoZSBjYWxjdWxhdGVkIGVxdWl2YWxlbnQgYW1vdW50LCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgaW5zdWZmaWNpZW50IGFtb3VudCBvciBsaXF1aWRpdHkAAAAAAAAFcXVvdGUAAAAAAAADAAAAAAAAAAhhbW91bnRfYQAAAAsAAAAAAAAACXJlc2VydmVfYQAAAAAAAAsAAAAAAAAACXJlc2VydmVfYgAAAAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAFFNvcm9zd2FwTGlicmFyeUVycm9y",
        "AAAAAAAAAd1HaXZlbiBhbiBpbnB1dCBhbW91bnQgb2YgYW4gYXNzZXQgYW5kIHBhaXIgcmVzZXJ2ZXMsIHJldHVybnMgdGhlIG1heGltdW0gb3V0cHV0IGFtb3VudCBvZiB0aGUgb3RoZXIgYXNzZXQuCgojIEFyZ3VtZW50cwoKKiBgYW1vdW50X2luYCAtIFRoZSBpbnB1dCBhbW91bnQgb2YgdGhlIGFzc2V0LgoqIGByZXNlcnZlX2luYCAtIFJlc2VydmVzIG9mIHRoZSBpbnB1dCBhc3NldCBpbiB0aGUgcGFpci4KKiBgcmVzZXJ2ZV9vdXRgIC0gUmVzZXJ2ZXMgb2YgdGhlIG91dHB1dCBhc3NldCBpbiB0aGUgcGFpci4KCiMgUmV0dXJucwoKUmV0dXJucyBgUmVzdWx0PGkxMjgsIFNvcm9zd2FwTGlicmFyeUVycm9yPmAgd2hlcmUgYE9rYCBjb250YWlucyB0aGUgY2FsY3VsYXRlZCBtYXhpbXVtIG91dHB1dCBhbW91bnQsIGFuZCBgRXJyYCBpbmRpY2F0ZXMgYW4gZXJyb3Igc3VjaCBhcyBpbnN1ZmZpY2llbnQgaW5wdXQgYW1vdW50IG9yIGxpcXVpZGl0eS4AAAAAAAAOZ2V0X2Ftb3VudF9vdXQAAAAAAAMAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAAAAAAACnJlc2VydmVfaW4AAAAAAAsAAAAAAAAAC3Jlc2VydmVfb3V0AAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAFFNvcm9zd2FwTGlicmFyeUVycm9y",
        "AAAAAAAAAdRHaXZlbiBhbiBvdXRwdXQgYW1vdW50IG9mIGFuIGFzc2V0IGFuZCBwYWlyIHJlc2VydmVzLCByZXR1cm5zIGEgcmVxdWlyZWQgaW5wdXQgYW1vdW50IG9mIHRoZSBvdGhlciBhc3NldC4KCiMgQXJndW1lbnRzCgoqIGBhbW91bnRfb3V0YCAtIFRoZSBvdXRwdXQgYW1vdW50IG9mIHRoZSBhc3NldC4KKiBgcmVzZXJ2ZV9pbmAgLSBSZXNlcnZlcyBvZiB0aGUgaW5wdXQgYXNzZXQgaW4gdGhlIHBhaXIuCiogYHJlc2VydmVfb3V0YCAtIFJlc2VydmVzIG9mIHRoZSBvdXRwdXQgYXNzZXQgaW4gdGhlIHBhaXIuCgojIFJldHVybnMKClJldHVybnMgYFJlc3VsdDxpMTI4LCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgdGhlIHJlcXVpcmVkIGlucHV0IGFtb3VudCwgYW5kIGBFcnJgIGluZGljYXRlcyBhbiBlcnJvciBzdWNoIGFzIGluc3VmZmljaWVudCBvdXRwdXQgYW1vdW50IG9yIGxpcXVpZGl0eS4AAAANZ2V0X2Ftb3VudF9pbgAAAAAAAAMAAAAAAAAACmFtb3VudF9vdXQAAAAAAAsAAAAAAAAACnJlc2VydmVfaW4AAAAAAAsAAAAAAAAAC3Jlc2VydmVfb3V0AAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAFFNvcm9zd2FwTGlicmFyeUVycm9y",
        "AAAAAAAAAZRQZXJmb3JtcyBjaGFpbmVkIGdldF9hbW91bnRfb3V0IGNhbGN1bGF0aW9ucyBvbiBhbnkgbnVtYmVyIG9mIHBhaXJzLgoKIyBBcmd1bWVudHMKCiogYGVgIC0gVGhlIGVudmlyb25tZW50LgoqIGBmYWN0b3J5YCAtIFRoZSBmYWN0b3J5IGFkZHJlc3MuCiogYGFtb3VudF9pbmAgLSBUaGUgaW5wdXQgYW1vdW50LgoqIGBwYXRoYCAtIFZlY3RvciBvZiB0b2tlbiBhZGRyZXNzZXMgcmVwcmVzZW50aW5nIHRoZSBwYXRoLgoKIyBSZXR1cm5zCgpSZXR1cm5zIGBSZXN1bHQ8VmVjPGkxMjg+LCBTb3Jvc3dhcExpYnJhcnlFcnJvcj5gIHdoZXJlIGBPa2AgY29udGFpbnMgYSB2ZWN0b3Igb2YgY2FsY3VsYXRlZCBhbW91bnRzLCBhbmQgYEVycmAgaW5kaWNhdGVzIGFuIGVycm9yIHN1Y2ggYXMgYW4gaW52YWxpZCBwYXRoLgAAAA9nZXRfYW1vdW50c19vdXQAAAAAAwAAAAAAAAAHZmFjdG9yeQAAAAATAAAAAAAAAAlhbW91bnRfaW4AAAAAAAALAAAAAAAAAARwYXRoAAAD6gAAABMAAAABAAAD6QAAA+oAAAALAAAH0AAAABRTb3Jvc3dhcExpYnJhcnlFcnJvcg==",
        "AAAAAAAAAZVQZXJmb3JtcyBjaGFpbmVkIGdldF9hbW91bnRfaW4gY2FsY3VsYXRpb25zIG9uIGFueSBudW1iZXIgb2YgcGFpcnMuCgojIEFyZ3VtZW50cwoKKiBgZWAgLSBUaGUgZW52aXJvbm1lbnQuCiogYGZhY3RvcnlgIC0gVGhlIGZhY3RvcnkgYWRkcmVzcy4KKiBgYW1vdW50X291dGAgLSBUaGUgb3V0cHV0IGFtb3VudC4KKiBgcGF0aGAgLSBWZWN0b3Igb2YgdG9rZW4gYWRkcmVzc2VzIHJlcHJlc2VudGluZyB0aGUgcGF0aC4KCiMgUmV0dXJucwoKUmV0dXJucyBgUmVzdWx0PFZlYzxpMTI4PiwgU29yb3N3YXBMaWJyYXJ5RXJyb3I+YCB3aGVyZSBgT2tgIGNvbnRhaW5zIGEgdmVjdG9yIG9mIGNhbGN1bGF0ZWQgYW1vdW50cywgYW5kIGBFcnJgIGluZGljYXRlcyBhbiBlcnJvciBzdWNoIGFzIGFuIGludmFsaWQgcGF0aC4AAAAAAAAOZ2V0X2Ftb3VudHNfaW4AAAAAAAMAAAAAAAAAB2ZhY3RvcnkAAAAAEwAAAAAAAAAKYW1vdW50X291dAAAAAAACwAAAAAAAAAEcGF0aAAAA+oAAAATAAAAAQAAA+kAAAPqAAAACwAAB9AAAAAUU29yb3N3YXBMaWJyYXJ5RXJyb3I=" ]),
      options
    )
  }
  public readonly fromJSON = {
    get_global_state: this.txFromJSON<GlobalState>,
        initialize_pool: this.txFromJSON<Result<string>>,
        get_health_factor: this.txFromJSON<Result<i128>>,
        get_asset_decimals: this.txFromJSON<u32>,
        get_oracle_price_decimals: this.txFromJSON<u32>,
        get_pool_asset_oracle_price: this.txFromJSON<Result<i128>>,
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
        get_all_pools: this.txFromJSON<Array<string>>,
        get_apy: this.txFromJSON<Result<CompoundRates>>,
        get_optimal_apy: this.txFromJSON<Result<CompoundRates>>,
        sort_tokens: this.txFromJSON<Result<readonly [string, string]>>,
        pair_for: this.txFromJSON<Result<string>>,
        get_reserves_with_factory: this.txFromJSON<Result<readonly [i128, i128]>>,
        get_reserves_with_pair: this.txFromJSON<Result<readonly [i128, i128]>>,
        quote: this.txFromJSON<Result<i128>>,
        get_amount_out: this.txFromJSON<Result<i128>>,
        get_amount_in: this.txFromJSON<Result<i128>>,
        get_amounts_out: this.txFromJSON<Result<Array<i128>>>,
        get_amounts_in: this.txFromJSON<Result<Array<i128>>>
  }
}