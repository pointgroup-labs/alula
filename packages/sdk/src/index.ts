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
  1: {message:"OverOrUnderflow"},
  2: {message:"InvalidTimestamp"},
  3: {message:"DependencyContractError"},
  10: {message:"PoolAlreadyExists"},
  11: {message:"PoolDoesNotExist"},
  12: {message:"InvalidLoanPoolConfig"},
  13: {message:"NotEnoughPoolFunds"},
  14: {message:"DepositPoolDoesNotExist"},
  15: {message:"BorrowPoolDoesNotExist"},
  16: {message:"CollateralPoolDoesNotExist"},
  20: {message:"ObligationDoesNotExist"},
  21: {message:"DepositDoesNotExist"},
  22: {message:"BorrowDoesNotExist"},
  30: {message:"NegativeDeposit"},
  31: {message:"NegativeWithdraw"},
  32: {message:"NegativeRepay"},
  33: {message:"NegativeLiquidation"},
  34: {message:"NegativeBorrow"},
  35: {message:"NegativeFlashLoan"},
  36: {message:"NegativeCollateralAddition"},
  37: {message:"NegativeCollateralRemoval"},
  40: {message:"WithdrawOverBalance"},
  41: {message:"SupplyLimitExceeded"},
  42: {message:"BorrowLimitExceeded"},
  43: {message:"CollateralRemovalOverbalance"},
  50: {message:"OracleDoesNotKnowAssetPrice"},
  51: {message:"OracleStalePrice"},
  60: {message:"HealthFactorIsLowerThanRequiredThreshold"},
  61: {message:"InvalidLiquidationThreshold"},
  62: {message:"LiquidatedPositionIsHealthy"},
  63: {message:"LiquidationExceedsCloseFactor"},
  64: {message:"SelfLiquidation"},
  65: {message:"LiquidationWithEqualCollateralAndDepositPools"},
  70: {message:"InvalidLeverageMultiplier"},
  71: {message:"InvalidSwapSlippage"},
  72: {message:"MultiplyPairAlreadyExists"}
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
 * The numerical value that is used to determine the scaling factor required for updating the
 * position amount with interest, i.e. new_borrowed = (current_accrual \ last_accrual) *
 * borrowed
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


export interface MultiplyPair {
  /**
 * Address of a pool in a pair for a leveraged borrow
 */
borrow_pool: string;
  /**
 * Address of a pool in a pair for a leveraged deposit
 */
deposit_pool: string;
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
 * The numerical value that is used to determine the scaling factor required for updating the
 * borrowed amount with interest, i.e. new_borrowed = (current_accrual \ last_accrual) *
 * borrowed
 */
last_accrual: i128;
  /**
 * The timestamp of the last accrual re-calculation
 */
last_accrual_timestamp: u64;
  /**
 * The result of `TokenClient::name(&self)` invocation: `native` string for XLM SAC and the
 * SAC's native asset code and asset issuer concatenated with `:` for other SACs(e.g,
 * "AQUA:GAHPYWLK6YRN7CVYZOO4H3VDRZ7PVF5UJGLZCSPAEIKJE2XSWF5LAGER")
 */
name: string;
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
 * The maximum percentage of an asset's value that can be held in an individual obligation in
 * basis points with respect to a total obligation's collateral value. LTV greater than
 * that makes borrow position eligible to liquidation
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
 * The maximum percentage of an asset's value that can be borrowed in basis points(e.g, 7000 =
 * 70%, etc) with respect to a total obligation's collateral value
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
  /**
 * The maximum amount of supplied tokens that can be supplied in the pool(i.e., `available` +
 * `total_borrowed`) 0 denotes unlimited supply
 */
supply_limit: i128;
  /**
 * The maximum utilization ratio that is allowed to be reached via borrowing
 */
utilization_ratio_limit_bps: i128;
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

export type DataKey = {tag: "GlobalState", values: void} | {tag: "Pool", values: readonly [PoolAddress]} | {tag: "Obligation", values: readonly [UserAddress]} | {tag: "MultiplyPair", values: readonly [MultiplyPair]} | {tag: "Accrual", values: void} | {tag: "AllPools", values: void} | {tag: "AllObligations", values: void} | {tag: "AllMultiplyPairs", values: void};

export interface Client {
  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Upgrades the lending contract
   * 
   * ### Arguments
   * * `new_wasm_hash` - hash of the WASM binary uploaded to the network that will be used as a
   * new version of the contract
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: {
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
  }) => Promise<AssembledTransaction<null>>

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
   * * `salt` - optional salt data, which when provided is used along with `token_address` to
   * derive a deterministic pool address
   * * `pool_config` - optional `PoolConfig` data. If not provided - a default pool config is
   * used
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
   * Construct and simulate a initialize_multiply_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes a multiply pair
   * 
   * ### Arguments
   * * `deposit_pool` - address of a pool in a pair for a leveraged deposit
   * * `borrow_pool` - address of a pool in a pair for a leveraged borrow
   */
  initialize_multiply_pair: ({deposit_pool, borrow_pool}: {deposit_pool: string, borrow_pool: string}, options?: {
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
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Deposits tokens into the loan pool
   * 
   * ### Arguments
   * * `user` - user that deposits a token
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
   * Swap tokens via a swap provider contract. This guarantees a swap
   * and is agnostic to the possible price slippage
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
   * Construct and simulate a remove_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Removes collateral tokens from the loan pool to the user
   * 
   * ### Arguments
   * * `user` - user which withdraws collateral tokens
   * * `pool_address` - address of a pool from which the withdrawal happens
   * * `amount` - desired amount of collateral tokens to remove.
   * The actual amount removed is capped to maintain the position's LTV at its Open LTV on the
   * pool. Passing [`u64::MAX`] (or [`i128::MAX`]) effectively removes all available
   * collateral
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
   * Construct and simulate a repay transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Repays borrowed tokens
   * 
   * ### Arguments
   * * `user` - user which repays borrowed tokens
   * * `pool_address` - address of a pool from which the borrow happened
   * * `amount` - provided amount of tokens to repay. If this amount exceeds the total debt, only
   * the outstanding debt will be repaid.
   * Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to repay the entire debt
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
   * * `borrow_pool_address` - address of a pool whose borrowed tokens are repaid by the
   * liquidator
   * * `collateral_pool_address` - address of a pool whose tokens are sold to the liquidator with
   * a discount
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
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws deposited tokens from the loan pool to the user
   * 
   * ### Arguments
   * * `user` - user which withdraws deposited tokens
   * * `pool_address` - address of a pool from which the withdrawal happens
   * * `amount` - desired amount of tokens to withdraw.
   * The actual amount withdrawn is capped to maintain the position's LTV at its Open LTV on the
   * pool. Passing [`u64::MAX`] (or [`i128::MAX`]) can be used to withdraw all tokens
   * available for it
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
   * * `contract` - contract's address which leverages the flash loaned amount and adheres to
   * `erc3156` standard
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
   * Deposits tokens into the loan pool with leverage. Leverage is achieved by utilizing flash
   * loan and token swap
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
   * * `leverage_multiplier` - leverage multiplier, where the last two digits represent
   * decimal places (e.g., 700 for x7.00, 255 for x2.55, etc.)
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
   * Construct and simulate a withdraw_from_leveraged transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Withdraws tokens from the leveraged deposit position without affecting the leverage
   * multiplier
   * 
   * ### Arguments
   * * `user` - user that deleverages and withdraws from the position
   * * `deposit_pool_address` - address of a pool from the pair to which the deposit happened
   * * `borrow_pool_address` - address of a pool from the pair from which the borrow happened
   * * `amount` - desired amount of deposited tokens to withdraw.
   * The actual amount withdrawn is capped by the value difference between deposited and borrowed
   * tokens in the leveraged position (minus operational fees). Passing [`u64::MAX`] (or
   * [`i128::MAX`]) can be used to withdraw all available tokens
   */
  withdraw_from_leveraged: ({user, deposit_pool_address, borrow_pool_address, amount}: {user: string, deposit_pool_address: string, borrow_pool_address: string, amount: i128}, options?: {
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
   * Construct and simulate a get_asset_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns asset's decimals
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
   * Construct and simulate a accrue_interest transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Accrues interest on a specific user's obligation and on its pools
   * 
   * ### Arguments
   * * `user` - user whose obligation interest is accrued
   */
  accrue_interest: ({user}: {user: string}, options?: {
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
   * Construct and simulate a get_all_obligations transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a list of all user obligations in the protocol
   */
  get_all_obligations: (options?: {
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
   * Construct and simulate a get_all_multiply_pairs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a list of all multiply pairs in the protocol
   */
  get_all_multiply_pairs: (options?: {
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
  }) => Promise<AssembledTransaction<Array<MultiplyPair>>>

  /**
   * Construct and simulate a get_apy transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns APY calculated for the current utilization ratio of a pool in basis points (e.g.,
   * 2912 = 29.12%, etc)
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
   * Returns APY calculated for the optimal utilization ratio of a pool in basis points (e.g.,
   * 4000 = 40.00%, etc)
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
   * Construct and simulate a reset_storage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Resets the contract's storage. Useful when the contract's invariants are broken and require
   * resetting on the testnet without re-deploying the contract
   */
  reset_storage: (options?: {
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
  }) => Promise<AssembledTransaction<null>>

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
        "AAAAAAAAAKNVcGdyYWRlcyB0aGUgbGVuZGluZyBjb250cmFjdAoKIyMjIEFyZ3VtZW50cwoqIGBuZXdfd2FzbV9oYXNoYCAtIGhhc2ggb2YgdGhlIFdBU00gYmluYXJ5IHVwbG9hZGVkIHRvIHRoZSBuZXR3b3JrIHRoYXQgd2lsbCBiZSB1c2VkIGFzIGEKbmV3IHZlcnNpb24gb2YgdGhlIGNvbnRyYWN0AAAAAAd1cGdyYWRlAAAAAAEAAAAAAAAADW5ld193YXNtX2hhc2gAAAAAAAPuAAAAIAAAAAA=",
        "AAAAAAAAACBHZXRzIHRoZSBjb250cmFjdCdzIGdsb2JhbCBzdGF0ZQAAABBnZXRfZ2xvYmFsX3N0YXRlAAAAAAAAAAEAAAfQAAAAC0dsb2JhbFN0YXRlAA==",
        "AAAAAAAAAZZJbml0aWFsaXplcyBhIGxvYW4gcG9vbCBmb3IgYSBzcGVjaWZpYyBhc3NldAoKIyMjIEFyZ3VtZW50cwoqIGB0b2tlbl9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBjb3JyZXNwb25kaW5nIFNvcm9iYW4gQXNzZXQgQ29udHJhY3QKKiBgdG9rZW5fc3ltYm9sYCAtIHN5bWJvbCB3aGljaCByZXByZXNlbnRzIGEgcG9vbCdzIHRva2VuCiogYHNhbHRgIC0gb3B0aW9uYWwgc2FsdCBkYXRhLCB3aGljaCB3aGVuIHByb3ZpZGVkIGlzIHVzZWQgYWxvbmcgd2l0aCBgdG9rZW5fYWRkcmVzc2AgdG8KZGVyaXZlIGEgZGV0ZXJtaW5pc3RpYyBwb29sIGFkZHJlc3MKKiBgcG9vbF9jb25maWdgIC0gb3B0aW9uYWwgYFBvb2xDb25maWdgIGRhdGEuIElmIG5vdCBwcm92aWRlZCAtIGEgZGVmYXVsdCBwb29sIGNvbmZpZyBpcwp1c2VkAAAAAAAPaW5pdGlhbGl6ZV9wb29sAAAAAAQAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAAAAAAARzYWx0AAAD6AAAA+4AAAAgAAAAAAAAAAtwb29sX2NvbmZpZwAAAAPoAAAH0AAAAApQb29sQ29uZmlnAAAAAAABAAAD6QAAABMAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAALZJbml0aWFsaXplcyBhIG11bHRpcGx5IHBhaXIKCiMjIyBBcmd1bWVudHMKKiBgZGVwb3NpdF9wb29sYCAtIGFkZHJlc3Mgb2YgYSBwb29sIGluIGEgcGFpciBmb3IgYSBsZXZlcmFnZWQgZGVwb3NpdAoqIGBib3Jyb3dfcG9vbGAgLSBhZGRyZXNzIG9mIGEgcG9vbCBpbiBhIHBhaXIgZm9yIGEgbGV2ZXJhZ2VkIGJvcnJvdwAAAAAAGGluaXRpYWxpemVfbXVsdGlwbHlfcGFpcgAAAAIAAAAAAAAADGRlcG9zaXRfcG9vbAAAABMAAAAAAAAAC2JvcnJvd19wb29sAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAANdEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB0aGF0IGRlcG9zaXRzIGEgdG9rZW4KKiBgcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIHRvIHdoaWNoIHRoZSBkZXBvc2l0IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGdvaW5nIHRvIGJlIGRlcG9zaXRlZAAAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAVhTd2FwIHRva2VucyB2aWEgYSBzd2FwIHByb3ZpZGVyIGNvbnRyYWN0LiBUaGlzIGd1YXJhbnRlZXMgYSBzd2FwCmFuZCBpcyBhZ25vc3RpYyB0byB0aGUgcG9zc2libGUgcHJpY2Ugc2xpcHBhZ2UKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGRlcG9zaXRzIGEgdG9rZW4KKiBgdG9rZW5faW5gIC0gYWRkcmVzcyBvZiBhIHRva2VuIHRoYXQgd291bGQgYmUgdGFrZW4gZnJvbSB0aGUgdXNlcgoqIGB0b2tlbl9vdXRgIC0gYWRkcmVzcyBvZiBhIHRva2VuIHRoYXQgd291bGQgYmUgZ2l2ZW4gdG8gdGhlIHVzZXIKKiBgYW1vdW50YCAtIGV4YWN0IGFtb3VudCBvZiB0aGUgYHRva2VuX2luYAAAAARzd2FwAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAACHRva2VuX2luAAAAEwAAAAAAAAAJdG9rZW5fb3V0AAAAAAAAEwAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAEAAAPpAAAACwAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAANZCb3Jyb3dzIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGJvcnJvd3MgYSB0b2tlbgoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgYm9ycm93IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGdvaW5nIHRvIGJlIGJvcnJvd2VkAAAAAAAGYm9ycm93AAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAWlBZGRzIHRva2VucyBpbnRvIHRoZSBsb2FuIHBvb2wgYXMgY29sbGF0ZXJhbCBvbmx5LgpUaGlzIGltcGxpZXMgdGhhdCB0aGV5IGFyZSBhbHdheXMgYXZhaWxhYmxlIGZvciBhIGhlYWx0aHkgd2l0aGRyYXdhbCBmb3IgdGhlCmNvc3Qgb2Ygbm90IGFjY3J1aW5nIGFuIGludGVyZXN0IHJhdGUKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHRoYXQgYWRkcyBjb2xsYXRlcmFsCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgY29sbGF0ZXJhbCBpcyBiZWluZyBhZGRlZAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHRva2VucyB3aGljaCBhcmUgYmVpbmcgYWRkZWQgYXMgYSBjb2xsYXRlcmFsAAAAAAAADmFkZF9jb2xsYXRlcmFsAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAbFSZW1vdmVzIGNvbGxhdGVyYWwgdG9rZW5zIGZyb20gdGhlIGxvYW4gcG9vbCB0byB0aGUgdXNlcgoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggd2l0aGRyYXdzIGNvbGxhdGVyYWwgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGRlc2lyZWQgYW1vdW50IG9mIGNvbGxhdGVyYWwgdG9rZW5zIHRvIHJlbW92ZS4KVGhlIGFjdHVhbCBhbW91bnQgcmVtb3ZlZCBpcyBjYXBwZWQgdG8gbWFpbnRhaW4gdGhlIHBvc2l0aW9uJ3MgTFRWIGF0IGl0cyBPcGVuIExUViBvbiB0aGUKcG9vbC4gUGFzc2luZyBbYHU2NDo6TUFYYF0gKG9yIFtgaTEyODo6TUFYYF0pIGVmZmVjdGl2ZWx5IHJlbW92ZXMgYWxsIGF2YWlsYWJsZQpjb2xsYXRlcmFsAAAAAAAAEXJlbW92ZV9jb2xsYXRlcmFsAAAAAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAWVSZXBheXMgYm9ycm93ZWQgdG9rZW5zCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCByZXBheXMgYm9ycm93ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSBib3Jyb3cgaGFwcGVuZWQKKiBgYW1vdW50YCAtIHByb3ZpZGVkIGFtb3VudCBvZiB0b2tlbnMgdG8gcmVwYXkuIElmIHRoaXMgYW1vdW50IGV4Y2VlZHMgdGhlIHRvdGFsIGRlYnQsIG9ubHkKdGhlIG91dHN0YW5kaW5nIGRlYnQgd2lsbCBiZSByZXBhaWQuClBhc3NpbmcgW2B1NjQ6Ok1BWGBdIChvciBbYGkxMjg6Ok1BWGBdKSBjYW4gYmUgdXNlZCB0byByZXBheSB0aGUgZW50aXJlIGRlYnQAAAAAAAAFcmVwYXkAAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAYlMaXF1aWRhdGVzIGJvcnJvd2VyJ3MgcG9zaXRpb24gaWYgcG9zaXRpb24ncyBoZWFsdGggZmFjdG9yIGNyaXRlcmlvbiBpc24ndCBtZXQKCiMjIyBBcmd1bWVudHMKKiBgbGlxdWlkYXRvcmAgLSBhZ2VudCB3aGljaCBsaXF1aWRhdGVzIHRoZSBib3Jyb3dlcidzIHBvc2l0aW9uCiogYGJvcnJvd19wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgYm9ycm93ZWQgdG9rZW5zIGFyZSByZXBhaWQgYnkgdGhlCmxpcXVpZGF0b3IKKiBgY29sbGF0ZXJhbF9wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgdG9rZW5zIGFyZSBzb2xkIHRvIHRoZSBsaXF1aWRhdG9yIHdpdGgKYSBkaXNjb3VudAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHJlcGFpZCB0b2tlbnMAAAAAAAAJbGlxdWlkYXRlAAAAAAAABQAAAAAAAAAKbGlxdWlkYXRvcgAAAAAAEwAAAAAAAAAIYm9ycm93ZXIAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAAF2NvbGxhdGVyYWxfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAbFXaXRoZHJhd3MgZGVwb3NpdGVkIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wgdG8gdGhlIHVzZXIKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIHdpdGhkcmF3cyBkZXBvc2l0ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGRlc2lyZWQgYW1vdW50IG9mIHRva2VucyB0byB3aXRoZHJhdy4KVGhlIGFjdHVhbCBhbW91bnQgd2l0aGRyYXduIGlzIGNhcHBlZCB0byBtYWludGFpbiB0aGUgcG9zaXRpb24ncyBMVFYgYXQgaXRzIE9wZW4gTFRWIG9uIHRoZQpwb29sLiBQYXNzaW5nIFtgdTY0OjpNQVhgXSAob3IgW2BpMTI4OjpNQVhgXSkgY2FuIGJlIHVzZWQgdG8gd2l0aGRyYXcgYWxsIHRva2VucwphdmFpbGFibGUgZm9yIGl0AAAAAAAACHdpdGhkcmF3AAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAPlDcmVhdGVzIGEgZmxhc2ggbG9hbgoKIyMjIEFyZ3VtZW50cwoqIGBjb250cmFjdGAgLSBjb250cmFjdCdzIGFkZHJlc3Mgd2hpY2ggbGV2ZXJhZ2VzIHRoZSBmbGFzaCBsb2FuZWQgYW1vdW50IGFuZCBhZGhlcmVzIHRvCmBlcmMzMTU2YCBzdGFuZGFyZAoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgZmxhc2ggbG9hbiBoYXBwZW5zCiogYGFtb3VudGAgLSBhbW91bnQgb2YgbGVudCB0b2tlbnMAAAAAAAAKZmxhc2hfbG9hbgAAAAAAAwAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAqZEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sIHdpdGggbGV2ZXJhZ2UuIExldmVyYWdlIGlzIGFjaGlldmVkIGJ5IHV0aWxpemluZyBmbGFzaApsb2FuIGFuZCB0b2tlbiBzd2FwCgojIFdBUk5JTkcKVGhpcyBpbmNyZWFzZXMgdGhlIHBlcmNlaXZlZCBgc3VwcGx5IEFQUmAgb25seQp3aGVuIGAoYm9ycm93ZWQgdG9rZW4gYm9ycm93IEFQUiA8IHN1cHBseSB0b2tlbiBzdXBwbHkgQVBSKWAgaG9sZHMgdHJ1ZQoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgdGhhdCBkZXBvc2l0cyB0b2tlbnMgd2l0aCBsZXZlcmFnZQoqIGBkZXBvc2l0X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIHRvIHdoaWNoIHRoZSBkZXBvc2l0IGhhcHBlbnMKKiBgYm9ycm93X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIGZyb20gd2hpY2ggdGhlIGJvcnJvdyBoYXBwZW5zCiogYGFtb3VudGAgLSBvcmlnaW5hbCBib3Jyb3cgYW1vdW50IGJlZm9yZSB0aGUgbGV2ZXJhZ2UKKiBgbGV2ZXJhZ2VfbXVsdGlwbGllcmAgLSBsZXZlcmFnZSBtdWx0aXBsaWVyLCB3aGVyZSB0aGUgbGFzdCB0d28gZGlnaXRzIHJlcHJlc2VudApkZWNpbWFsIHBsYWNlcyAoZS5nLiwgNzAwIGZvciB4Ny4wMCwgMjU1IGZvciB4Mi41NSwgZXRjLikAAAAAABVkZXBvc2l0X3dpdGhfbGV2ZXJhZ2UAAAAAAAAFAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAATbGV2ZXJhZ2VfbXVsdGlwbGllcgAAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAopXaXRoZHJhd3MgdG9rZW5zIGZyb20gdGhlIGxldmVyYWdlZCBkZXBvc2l0IHBvc2l0aW9uIHdpdGhvdXQgYWZmZWN0aW5nIHRoZSBsZXZlcmFnZQptdWx0aXBsaWVyCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB0aGF0IGRlbGV2ZXJhZ2VzIGFuZCB3aXRoZHJhd3MgZnJvbSB0aGUgcG9zaXRpb24KKiBgZGVwb3NpdF9wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB0aGUgcGFpciB0byB3aGljaCB0aGUgZGVwb3NpdCBoYXBwZW5lZAoqIGBib3Jyb3dfcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIGZyb20gdGhlIHBhaXIgZnJvbSB3aGljaCB0aGUgYm9ycm93IGhhcHBlbmVkCiogYGFtb3VudGAgLSBkZXNpcmVkIGFtb3VudCBvZiBkZXBvc2l0ZWQgdG9rZW5zIHRvIHdpdGhkcmF3LgpUaGUgYWN0dWFsIGFtb3VudCB3aXRoZHJhd24gaXMgY2FwcGVkIGJ5IHRoZSB2YWx1ZSBkaWZmZXJlbmNlIGJldHdlZW4gZGVwb3NpdGVkIGFuZCBib3Jyb3dlZAp0b2tlbnMgaW4gdGhlIGxldmVyYWdlZCBwb3NpdGlvbiAobWludXMgb3BlcmF0aW9uYWwgZmVlcykuIFBhc3NpbmcgW2B1NjQ6Ok1BWGBdIChvcgpbYGkxMjg6Ok1BWGBdKSBjYW4gYmUgdXNlZCB0byB3aXRoZHJhdyBhbGwgYXZhaWxhYmxlIHRva2VucwAAAAAAF3dpdGhkcmF3X2Zyb21fbGV2ZXJhZ2VkAAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAABhSZXR1cm5zIGFzc2V0J3MgZGVjaW1hbHMAAAASZ2V0X2Fzc2V0X2RlY2ltYWxzAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAB9SZXR1cm5zIG9yYWNsZSBwcmljZSdzIGRlY2ltYWxzAAAAABlnZXRfb3JhY2xlX3ByaWNlX2RlY2ltYWxzAAAAAAAAAAAAAAEAAAAE",
        "AAAAAAAAAGxSZXR1cm5zIHBvb2wgYXNzZXQncyBvcmFjbGUgcHJpY2UKCiMjIyBBcmd1bWVudHMKKiBgcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYXNzZXQgd2hpY2ggcHJpY2UgaXMgcmV0dXJuZWQAAAAbZ2V0X3Bvb2xfYXNzZXRfb3JhY2xlX3ByaWNlAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAAAsAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAJVSZXR1cm5zIHRoZSB1c2VyJ3Mgb2JsaWdhdGlvbiB3aGljaCBpbmNsdWRlcyBkYXRhIGFib3V0IGFsbCBvZiB0aGVpciBkZXBvc2l0cyBhbmQgYm9ycm93cwoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggb2JsaWdhdGlvbiBpcyByZXR1cm5lZAAAAAAAABNnZXRfdXNlcl9vYmxpZ2F0aW9uAAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAA+kAAAfQAAAACk9ibGlnYXRpb24AAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAIVBY2NydWVzIGludGVyZXN0IG9uIGEgc3BlY2lmaWMgdXNlcidzIG9ibGlnYXRpb24gYW5kIG9uIGl0cyBwb29scwoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hvc2Ugb2JsaWdhdGlvbiBpbnRlcmVzdCBpcyBhY2NydWVkAAAAAAAAD2FjY3J1ZV9pbnRlcmVzdAAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB0xDRXJyb3IA",
        "AAAAAAAAAFxSZXR1cm5zIHRoZSBzcGVjaWZpYyBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgcG9vbF9hZGRyZXNzYCAtIHBvb2wgd2hpY2ggZGF0YSBpcyByZXR1cm5lZAAAAAhnZXRfcG9vbAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAAEUG9vbAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAADRSZXR1cm5zIGEgbGlzdCBvZiBhbGwgcG9vbCBhZGRyZXNzZXMgaW4gdGhlIHByb3RvY29sAAAADWdldF9hbGxfcG9vbHMAAAAAAAAAAAAAAQAAA+oAAAAT",
        "AAAAAAAAADZSZXR1cm5zIGEgbGlzdCBvZiBhbGwgdXNlciBvYmxpZ2F0aW9ucyBpbiB0aGUgcHJvdG9jb2wAAAAAABNnZXRfYWxsX29ibGlnYXRpb25zAAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAADRSZXR1cm5zIGEgbGlzdCBvZiBhbGwgbXVsdGlwbHkgcGFpcnMgaW4gdGhlIHByb3RvY29sAAAAFmdldF9hbGxfbXVsdGlwbHlfcGFpcnMAAAAAAAAAAAABAAAD6gAAB9AAAAAMTXVsdGlwbHlQYWly",
        "AAAAAAAAALtSZXR1cm5zIEFQWSBjYWxjdWxhdGVkIGZvciB0aGUgY3VycmVudCB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wgaW4gYmFzaXMgcG9pbnRzIChlLmcuLAoyOTEyID0gMjkuMTIlLCBldGMpCgojIyMgQXJndW1lbnRzCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmb3Igd2hpY2ggQVBZIGlzIHJldHVybmVkAAAAAAdnZXRfYXB5AAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAANQ29tcG91bmRSYXRlcwAAAAAAB9AAAAAHTENFcnJvcgA=",
        "AAAAAAAAAMNSZXR1cm5zIEFQWSBjYWxjdWxhdGVkIGZvciB0aGUgb3B0aW1hbCB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wgaW4gYmFzaXMgcG9pbnRzIChlLmcuLAo0MDAwID0gNDAuMDAlLCBldGMpCgojIyMgQXJndW1lbnRzCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmb3Igd2hpY2ggb3B0aW1hbCBBUFkgaXMgcmV0dXJuZWQAAAAAD2dldF9vcHRpbWFsX2FweQAAAAABAAAAAAAAAA1fcG9vbF9hZGRyZXNzAAAAAAAAEwAAAAEAAAPpAAAH0AAAAA1Db21wb3VuZFJhdGVzAAAAAAAH0AAAAAdMQ0Vycm9yAA==",
        "AAAAAAAAAJZSZXNldHMgdGhlIGNvbnRyYWN0J3Mgc3RvcmFnZS4gVXNlZnVsIHdoZW4gdGhlIGNvbnRyYWN0J3MgaW52YXJpYW50cyBhcmUgYnJva2VuIGFuZCByZXF1aXJlCnJlc2V0dGluZyBvbiB0aGUgdGVzdG5ldCB3aXRob3V0IHJlLWRlcGxveWluZyB0aGUgY29udHJhY3QAAAAAAA1yZXNldF9zdG9yYWdlAAAAAAAAAAAAAAA=",
        "AAAABAAAAAAAAAAAAAAAFExlbmRpbmdDb250cmFjdEVycm9yAAAAJQAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAD092ZXJPclVuZGVyZmxvdwAAAAABAAAAAAAAABBJbnZhbGlkVGltZXN0YW1wAAAAAgAAAAAAAAAXRGVwZW5kZW5jeUNvbnRyYWN0RXJyb3IAAAAAAwAAAAAAAAARUG9vbEFscmVhZHlFeGlzdHMAAAAAAAAKAAAAAAAAABBQb29sRG9lc05vdEV4aXN0AAAACwAAAAAAAAAVSW52YWxpZExvYW5Qb29sQ29uZmlnAAAAAAAADAAAAAAAAAASTm90RW5vdWdoUG9vbEZ1bmRzAAAAAAANAAAAAAAAABdEZXBvc2l0UG9vbERvZXNOb3RFeGlzdAAAAAAOAAAAAAAAABZCb3Jyb3dQb29sRG9lc05vdEV4aXN0AAAAAAAPAAAAAAAAABpDb2xsYXRlcmFsUG9vbERvZXNOb3RFeGlzdAAAAAAAEAAAAAAAAAAWT2JsaWdhdGlvbkRvZXNOb3RFeGlzdAAAAAAAFAAAAAAAAAATRGVwb3NpdERvZXNOb3RFeGlzdAAAAAAVAAAAAAAAABJCb3Jyb3dEb2VzTm90RXhpc3QAAAAAABYAAAAAAAAAD05lZ2F0aXZlRGVwb3NpdAAAAAAeAAAAAAAAABBOZWdhdGl2ZVdpdGhkcmF3AAAAHwAAAAAAAAANTmVnYXRpdmVSZXBheQAAAAAAACAAAAAAAAAAE05lZ2F0aXZlTGlxdWlkYXRpb24AAAAAIQAAAAAAAAAOTmVnYXRpdmVCb3Jyb3cAAAAAACIAAAAAAAAAEU5lZ2F0aXZlRmxhc2hMb2FuAAAAAAAAIwAAAAAAAAAaTmVnYXRpdmVDb2xsYXRlcmFsQWRkaXRpb24AAAAAACQAAAAAAAAAGU5lZ2F0aXZlQ29sbGF0ZXJhbFJlbW92YWwAAAAAAAAlAAAAAAAAABNXaXRoZHJhd092ZXJCYWxhbmNlAAAAACgAAAAAAAAAE1N1cHBseUxpbWl0RXhjZWVkZWQAAAAAKQAAAAAAAAATQm9ycm93TGltaXRFeGNlZWRlZAAAAAAqAAAAAAAAABxDb2xsYXRlcmFsUmVtb3ZhbE92ZXJiYWxhbmNlAAAAKwAAAAAAAAAbT3JhY2xlRG9lc05vdEtub3dBc3NldFByaWNlAAAAADIAAAAAAAAAEE9yYWNsZVN0YWxlUHJpY2UAAAAzAAAAAAAAAChIZWFsdGhGYWN0b3JJc0xvd2VyVGhhblJlcXVpcmVkVGhyZXNob2xkAAAAPAAAAAAAAAAbSW52YWxpZExpcXVpZGF0aW9uVGhyZXNob2xkAAAAAD0AAAAAAAAAG0xpcXVpZGF0ZWRQb3NpdGlvbklzSGVhbHRoeQAAAAA+AAAAAAAAAB1MaXF1aWRhdGlvbkV4Y2VlZHNDbG9zZUZhY3RvcgAAAAAAAD8AAAAAAAAAD1NlbGZMaXF1aWRhdGlvbgAAAABAAAAAAAAAAC1MaXF1aWRhdGlvbldpdGhFcXVhbENvbGxhdGVyYWxBbmREZXBvc2l0UG9vbHMAAAAAAABBAAAAAAAAABlJbnZhbGlkTGV2ZXJhZ2VNdWx0aXBsaWVyAAAAAAAARgAAAAAAAAATSW52YWxpZFN3YXBTbGlwcGFnZQAAAABHAAAAAAAAABlNdWx0aXBseVBhaXJBbHJlYWR5RXhpc3RzAAAAAAAASA==",
        "AAAAAQAAAKtJbnRlcmVzdCByYXRlIG11bHRpcGxpZXJzIHByZXNlbnRlZCBhcyAoMSArIHh4eCkgd2hlcmUgYHh4eGAgaXMgYSBjb21wb3VuZCBpbnRlcmVzdCByYXRlLgpUaGUgcmVhbCBtdWx0aXBsaWVyKGUuZy4gMS4zMiwgMi41MywgZXRjKSBpcyBzY2FsZWQgdXAgd2l0aCBbYFNDQUxFRF9PTkVgXSB2YWx1ZS4AAAAAAAAAABdDb21wb3VuZFJhdGVNdWx0aXBsaWVycwAAAAACAAAAAAAAAAZib3Jyb3cAAAAAAAsAAAAAAAAABnN1cHBseQAAAAAACw==",
        "AAAAAQAAADNDb21wb3VuZCBpbnRlcmVzdCByYXRlcyByZXByZXNlbnRlZCBpbiBiYXNpcyBwb2ludHMAAAAAAAAAAA1Db21wb3VuZFJhdGVzAAAAAAAAAgAAAAAAAAAKYm9ycm93X2JwcwAAAAAABAAAAAAAAAAKc3VwcGx5X2JwcwAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAMAAABEQm9ycm93ZWQgbGlxdWlkaXR5IGZvciB0aGUgb2JsaWdhdGlvbiwgdW5pcXVlIGJ5IGJvcnJvdyBwb29sIGFkZHJlc3MAAAAHYm9ycm93cwAAAAPsAAAAEwAAB9AAAAAQQm9ycm93T2JsaWdhdGlvbgAAAEdEZXBvc2l0ZWQgY29sbGF0ZXJhbCBmb3IgdGhlIG9ibGlnYXRpb24sIHVuaXF1ZSBieSBkZXBvc2l0IHBvb2wgYWRkcmVzcwAAAAAIZGVwb3NpdHMAAAPsAAAAEwAAB9AAAAARRGVwb3NpdE9ibGlnYXRpb24AAAAAAAAVVGhlIG9ibGlnYXRpb24ncyB1c2VyAAAAAAAABHVzZXIAAAAT",
        "AAAAAQAAAAAAAAAAAAAAEEJvcnJvd09ibGlnYXRpb24AAAADAAAAKFRoZSBpbml0aWFsIGFtb3VudCBvZiB0aGUgYm9ycm93ZWQgdG9rZW4AAAAIYm9ycm93ZWQAAAALAAAAuVRoZSBudW1lcmljYWwgdmFsdWUgdGhhdCBpcyB1c2VkIHRvIGRldGVybWluZSB0aGUgc2NhbGluZyBmYWN0b3IgcmVxdWlyZWQgZm9yIHVwZGF0aW5nIHRoZQpwb3NpdGlvbiBhbW91bnQgd2l0aCBpbnRlcmVzdCwgaS5lLiBuZXdfYm9ycm93ZWQgPSAoY3VycmVudF9hY2NydWFsIFwgbGFzdF9hY2NydWFsKSAqCmJvcnJvd2VkAAAAAAAADGxhc3RfYWNjcnVhbAAAAAsAAAAdVGhlIGFtb3VudCBvZiB1bnBhaWQgaW50ZXJlc3QAAAAAAAAPdW5wYWlkX2ludGVyZXN0AAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAEURlcG9zaXRPYmxpZ2F0aW9uAAAAAAAAAgAAAAAAAAAKY29sbGF0ZXJhbAAAAAAACwAAAAAAAAAGc2hhcmVzAAAAAAAL",
        "AAAAAQAAAAAAAAAAAAAADE11bHRpcGx5UGFpcgAAAAIAAAAyQWRkcmVzcyBvZiBhIHBvb2wgaW4gYSBwYWlyIGZvciBhIGxldmVyYWdlZCBib3Jyb3cAAAAAAAtib3Jyb3dfcG9vbAAAAAATAAAAM0FkZHJlc3Mgb2YgYSBwb29sIGluIGEgcGFpciBmb3IgYSBsZXZlcmFnZWQgZGVwb3NpdAAAAAAMZGVwb3NpdF9wb29sAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAALAAAALFRoZSBjdXJyZW50bHkgYXZhaWxhYmxlIGZvciBib3Jyb3dpbmcgdG9rZW5zAAAACWF2YWlsYWJsZQAAAAAAAAsAAAAjQ29uZmlndXJhdGlvbiBzZXR0aW5ncyBmb3IgdGhlIHBvb2wAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAC5VGhlIG51bWVyaWNhbCB2YWx1ZSB0aGF0IGlzIHVzZWQgdG8gZGV0ZXJtaW5lIHRoZSBzY2FsaW5nIGZhY3RvciByZXF1aXJlZCBmb3IgdXBkYXRpbmcgdGhlCmJvcnJvd2VkIGFtb3VudCB3aXRoIGludGVyZXN0LCBpLmUuIG5ld19ib3Jyb3dlZCA9IChjdXJyZW50X2FjY3J1YWwgXCBsYXN0X2FjY3J1YWwpICoKYm9ycm93ZWQAAAAAAAAMbGFzdF9hY2NydWFsAAAACwAAADBUaGUgdGltZXN0YW1wIG9mIHRoZSBsYXN0IGFjY3J1YWwgcmUtY2FsY3VsYXRpb24AAAAWbGFzdF9hY2NydWFsX3RpbWVzdGFtcAAAAAAABgAAAOxUaGUgcmVzdWx0IG9mIGBUb2tlbkNsaWVudDo6bmFtZSgmc2VsZilgIGludm9jYXRpb246IGBuYXRpdmVgIHN0cmluZyBmb3IgWExNIFNBQyBhbmQgdGhlClNBQydzIG5hdGl2ZSBhc3NldCBjb2RlIGFuZCBhc3NldCBpc3N1ZXIgY29uY2F0ZW5hdGVkIHdpdGggYDpgIGZvciBvdGhlciBTQUNzKGUuZywKIkFRVUE6R0FIUFlXTEs2WVJON0NWWVpPTzRIM1ZEUlo3UFZGNVVKR0xaQ1NQQUVJS0pFMlhTV0Y1TEFHRVIiKQAAAARuYW1lAAAAEAAAABxUaGUgYWRkcmVzcyBvZiB0aGUgbG9hbiBwb29sAAAADHBvb2xfYWRkcmVzcwAAABMAAAAxVGhlIGFkZHJlc3Mgb2YgdGhlIHRva2VuIGFzc29jaWF0ZWQgd2l0aCB0aGUgcG9vbAAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAFpUaGUgdGlja2VyIHN5bWJvbCBvZiB0aGUgYXNzb2NpYXRlZCB0b2tlbiwgd2hpY2ggaXMgdXNlZCB0byBpZGVudGlmeSB0aGUgdG9rZW4gaW4gdGhlIHBvb2wAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAVFRoZSB0b3RhbCBhbW91bnQgb2YgYm9ycm93ZWQgYXNzZXRzLiBUaGlzIHZhbHVlIGluY3JlYXNlcyB3aXRoIGludGVyZXN0IHJhdGUgYWNjcnVhbAAAAA50b3RhbF9ib3Jyb3dlZAAAAAAACwAAAEpUaGUgdG90YWwgYW1vdW50IG9mIGRlcG9zaXRlZCBjb2xsYXRlcmFsIGFzc2V0cyB0aGF0IGRvbid0IGFjY3J1ZSBpbnRlcmVzdAAAAAAAEHRvdGFsX2NvbGxhdGVyYWwAAAALAAAAOVRoZSB0b3RhbCBhbW91bnQgb2YgZGVwb3NpdGVkIGFzc2V0cyB0aGF0IGFjY3J1ZSBpbnRlcmVzdAAAAAAAAAx0b3RhbF9zaGFyZXMAAAAL",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAsAAAB0QmFzZSBpbnRlcmVzdCByYXRlIGFwcGxpZWQgcmVnYXJkbGVzcyBvZiB1dGlsaXphdGlvbiwgZXhwcmVzc2VkIHBlciBzZWNvbmQKaW4gMS9gU0NBTEVEX09ORWAgdW5pdHMuIE11c3QgYmUgcG9zaXRpdmUAAAAUYmFzZV9yYXRlX3Blcl9zZWNvbmQAAAALAAAA4lRoZSBtYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYW4gYXNzZXQncyB2YWx1ZSB0aGF0IGNhbiBiZSBoZWxkIGluIGFuIGluZGl2aWR1YWwgb2JsaWdhdGlvbiBpbgpiYXNpcyBwb2ludHMgd2l0aCByZXNwZWN0IHRvIGEgdG90YWwgb2JsaWdhdGlvbidzIGNvbGxhdGVyYWwgdmFsdWUuIExUViBncmVhdGVyIHRoYW4KdGhhdCBtYWtlcyBib3Jyb3cgcG9zaXRpb24gZWxpZ2libGUgdG8gbGlxdWlkYXRpb24AAAAAAA1jbG9zZV9sdHZfYnBzAAAAAAAACwAAAD5NYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYSBib3Jyb3dlcidzIGRlYnQgdGhhdCBjYW4gYmUgbGlxdWlkYXRlZAAAAAAAHGxpcXVpZGF0aW9uX2Nsb3NlX2ZhY3Rvcl9icHMAAAALAAAAQ0FkZGl0aW9uYWwgZGlzY291bnQgZ2l2ZW4gdG8gbGlxdWlkYXRvcnMgd2hlbiBwdXJjaGFzaW5nIGNvbGxhdGVyYWwAAAAAGWxpcXVpZGF0aW9uX2luY2VudGl2ZV9icHMAAAAAAAALAAAAm1RoZSBtYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYW4gYXNzZXQncyB2YWx1ZSB0aGF0IGNhbiBiZSBib3Jyb3dlZCBpbiBiYXNpcyBwb2ludHMoZS5nLCA3MDAwID0KNzAlLCBldGMpIHdpdGggcmVzcGVjdCB0byBhIHRvdGFsIG9ibGlnYXRpb24ncyBjb2xsYXRlcmFsIHZhbHVlAAAAAAxvcGVuX2x0dl9icHMAAAALAAAAIlBvc2l0aXZlIE9wdGltYWwgVXRpbGl6YXRpb24gUmF0aW8AAAAAAB1vcHRpbWFsX3V0aWxpemF0aW9uX3JhdGlvX2JwcwAAAAAAAAsAAAA+UGVyY2VudGFnZSBvZiBpbnRlcmVzdCBwYXltZW50cyBhbGxvY2F0ZWQgdG8gcHJvdG9jb2wgcmVzZXJ2ZXMAAAAAABFyZXNlcnZlX3JhdGlvX2JwcwAAAAAAAAsAAACPSW50ZXJlc3QgcmF0ZSBzbG9wZSBiZWZvcmUgcmVhY2hpbmcgb3B0aW1hbCB1dGlsaXphdGlvbiByYXRpbwpDb250cm9scyBob3cgYWdncmVzc2l2ZWx5IHJhdGVzIGluY3JlYXNlIHdpdGggdXRpbGl6YXRpb24gYmVsb3cgdGhlIG9wdGltYWwgcG9pbnQAAAAABnNsb3BlMQAAAAAACwAAAI9JbnRlcmVzdCByYXRlIHNsb3BlIGFmdGVyIGV4Y2VlZGluZyBvcHRpbWFsIHV0aWxpemF0aW9uIHJhdGlvCkNvbnRyb2xzIGhvdyBhZ2dyZXNzaXZlbHkgcmF0ZXMgaW5jcmVhc2Ugd2l0aCB1dGlsaXphdGlvbiBhYm92ZSB0aGUgb3B0aW1hbCBwb2ludAAAAAAGc2xvcGUyAAAAAAALAAAAh1RoZSBtYXhpbXVtIGFtb3VudCBvZiBzdXBwbGllZCB0b2tlbnMgdGhhdCBjYW4gYmUgc3VwcGxpZWQgaW4gdGhlIHBvb2woaS5lLiwgYGF2YWlsYWJsZWAgKwpgdG90YWxfYm9ycm93ZWRgKSAwIGRlbm90ZXMgdW5saW1pdGVkIHN1cHBseQAAAAAMc3VwcGx5X2xpbWl0AAAACwAAAElUaGUgbWF4aW11bSB1dGlsaXphdGlvbiByYXRpbyB0aGF0IGlzIGFsbG93ZWQgdG8gYmUgcmVhY2hlZCB2aWEgYm9ycm93aW5nAAAAAAAAG3V0aWxpemF0aW9uX3JhdGlvX2xpbWl0X2JwcwAAAAAL",
        "AAAAAQAAAAAAAAAAAAAAB0FjY3J1YWwAAAAAAwAAAAAAAAAOYm9ycm93X2FjY3J1YWwAAAAAAAsAAAAAAAAAD2RlcG9zaXRfYWNjcnVhbAAAAAALAAAAAAAAAAl0aW1lc3RhbXAAAAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAMAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAZbGlxdWlkYXRpb25fdGhyZXNob2xkX2JwcwAAAAAAAAsAAAAAAAAABnN0YXR1cwAAAAAAAQ==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAACAAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAEAAAAAAAAABFBvb2wAAAABAAAH0AAAAAtQb29sQWRkcmVzcwAAAAABAAAAAAAAAApPYmxpZ2F0aW9uAAAAAAABAAAH0AAAAAtVc2VyQWRkcmVzcwAAAAABAAAAAAAAAAxNdWx0aXBseVBhaXIAAAABAAAH0AAAAAxNdWx0aXBseVBhaXIAAAAAAAAAAAAAAAdBY2NydWFsAAAAAAAAAAAAAAAACEFsbFBvb2xzAAAAAAAAAAAAAAAOQWxsT2JsaWdhdGlvbnMAAAAAAAAAAAAAAAAAEEFsbE11bHRpcGx5UGFpcnM=" ]),
      options
    )
  }
  public readonly fromJSON = {
    upgrade: this.txFromJSON<null>,
        get_global_state: this.txFromJSON<GlobalState>,
        initialize_pool: this.txFromJSON<Result<string>>,
        initialize_multiply_pair: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        swap: this.txFromJSON<Result<i128>>,
        borrow: this.txFromJSON<Result<void>>,
        add_collateral: this.txFromJSON<Result<void>>,
        remove_collateral: this.txFromJSON<Result<void>>,
        repay: this.txFromJSON<Result<void>>,
        liquidate: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        flash_loan: this.txFromJSON<Result<void>>,
        deposit_with_leverage: this.txFromJSON<Result<void>>,
        withdraw_from_leveraged: this.txFromJSON<Result<void>>,
        get_asset_decimals: this.txFromJSON<u32>,
        get_oracle_price_decimals: this.txFromJSON<u32>,
        get_pool_asset_oracle_price: this.txFromJSON<Result<i128>>,
        get_user_obligation: this.txFromJSON<Result<Obligation>>,
        accrue_interest: this.txFromJSON<Result<void>>,
        get_pool: this.txFromJSON<Result<Pool>>,
        get_all_pools: this.txFromJSON<Array<string>>,
        get_all_obligations: this.txFromJSON<Array<string>>,
        get_all_multiply_pairs: this.txFromJSON<Array<MultiplyPair>>,
        get_apy: this.txFromJSON<Result<CompoundRates>>,
        get_optimal_apy: this.txFromJSON<Result<CompoundRates>>,
        reset_storage: this.txFromJSON<null>
  }
}