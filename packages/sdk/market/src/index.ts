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




export type AccrualModel = {tag: "Compounded", values: void};

export const MCError = {
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
  30: {message:"NegativeInputAmount"},
  40: {message:"WithdrawOverBalance"},
  41: {message:"PoolSupplyLimitExceeded"},
  42: {message:"PoolUtilizationRatioCapExceeded"},
  43: {message:"CollateralRemovalOverbalance"},
  50: {message:"OracleDoesNotKnowAssetPrice"},
  51: {message:"OracleStalePrice"},
  60: {message:"HealthFactorIsLowerThanRequiredThreshold"},
  61: {message:"InvalidLiquidationThreshold"},
  62: {message:"LiquidatedPositionIsHealthy"},
  63: {message:"LiquidationExceedsCloseFactor"},
  64: {message:"SelfLiquidation"},
  65: {message:"LiquidationWithEqualCollateralAndDepositPools"},
  66: {message:"PositionDoesNotHaveBadDebt"},
  70: {message:"InvalidLeverageMultiplier"},
  71: {message:"InvalidSwapSlippage"},
  72: {message:"MultiplyPairAlreadyExists"},
  73: {message:"MultiplyPairDoesNotExist"}
}


/**
 * Linear annual interest rates represented in basis points
 */
export interface AnnualPercentageRates {
  borrow_bps: u64;
  supply_bps: u64;
}


/**
 * Compound interest rates represented in basis points
 */
export interface AnnualPercentageYields {
  borrow_bps: u32;
  supply_bps: u32;
}

export type InterestRateModel = {tag: "Kinked", values: readonly [KinkedIRConfig]};


export interface KinkedIRConfig {
  /**
 * Base APR that is accrued regardless of the utilization ratio of a pool
 */
base_apr_bps: i128;
  /**
 * APR that is accrued when the utilization ratio is at the kink 1 value
 */
kink1_apr_bps: i128;
  /**
 * Kink 1 utilization ratio
 */
kink1_ur_bps: i128;
  /**
 * APR that is accrued when the utilization ratio is at the kink 2 value
 */
kink2_apr_bps: i128;
  /**
 * Kink 2 utilization ratio
 */
kink2_ur_bps: i128;
  /**
 * APR that is accrued when the utilization ratio is at 100%
 */
max_apr_bps: i128;
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
  /**
 * Maximum leverage multiplier based on borrow pool openLTV value. Scaled with
 * [`LEVERAGE_SCALE`]
 */
max_leverage_multiplier: u32;
  /**
 * Deterministically computed unique seed per a pair, used to distinguish a user's multiply
 * pair obligation from others
 */
seed: Buffer;
}


export interface ObligationKey {
  seed: Option<Buffer>;
  user: string;
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
}


export interface BorrowObligation {
  /**
 * Accumulated value of initially borrowed tokens
 */
borrowed: i128;
  /**
 * Amount of the total debt shares that the obligation contains
 */
d_tokens: i128;
}


export interface DepositObligation {
  /**
 * Accumulated value of collateral that doesn't accrue interest
 */
collateral: i128;
  /**
 * Accumulated value of initially deposited tokens. E.g., if a user initially deposited 100
 * tokens, the time passed, which caused 2 tokens to be accrued, and the user deposited 20
 * more tokens - this value will be equal to 120
 */
deposited: i128;
  /**
 * A share of total supplied tokens in the pool that obligation contains
 */
j_tokens: i128;
}


/**
 * Generally represents computed fees issued by any possible operation on a market
 */
export interface ComputedFees {
  /**
 * Sum of `market_fee` and `host_fee`
 */
fee_sum: i128;
  /**
 * Fee segregated to the protocol host
 */
host_fee: i128;
  /**
 * Fee segregated to the market admin
 */
market_fee: i128;
}


/**
 * [`Obligation::deposit`] resulting data
 */
export interface DepositResult {
  computed_fees: ComputedFees;
  /**
 * Amount of originally deposited tokens(minus all fees)
 */
deposited: i128;
  /**
 * Amount of `jTokens` to issue that represent the `originally_deposited` amount in the pool
 */
j_tokens_to_issue: i128;
}


/**
 * [`Obligation::borrow`] resulting data
 */
export interface BorrowResult {
  /**
 * Amount of debt(in tokens) that is added to the borrower's obligation
 */
borrower_new_debt: i128;
  /**
 * Amount of tokens to receive by the borrower(`borrower_new_debt` minus fees)
 */
borrower_to_receive: i128;
  computed_fees: ComputedFees;
  /**
 * Amount of `dTokens` to issue that represent the `borrower_new_debt` amount in the pool
 */
d_tokens_to_issue: i128;
}


/**
 * [`Obligation::add_collateral`] resulting data
 */
export interface AddCollateralResult {
  /**
 * Amount of tokens added as collateral(with subtracted fees)
 */
added_collateral: i128;
  computed_fees: ComputedFees;
}


/**
 * [`Obligation::withdraw`] resulting data
 */
export interface WithdrawResult {
  computed_fees: ComputedFees;
  /**
 * Amount of the original deposit(in tokens) that is removed from the `DepositObligation`
 */
deposit_decrease: i128;
  /**
 * Amount of `jTokens` to burn that represent the `deposit_decreased_amount` amount in the
 * pool
 */
j_tokens_to_burn: i128;
  /**
 * Amount of tokens to receive by the withdrawer(`deposit_decreased_amount` minus fees)
 */
withdrawer_to_receive: i128;
}


/**
 * [`Obligation::repay`] resulting data
 */
export interface RepayResult {
  /**
 * Excess amount given by the borrower that is sent back
 */
amount_to_send_back: i128;
  computed_fees: ComputedFees;
  /**
 * Amount of `dTokens` to issue that represent the `real_repaid` amount in the pool
 */
d_tokens_to_burn: i128;
  /**
 * Amount of the debt that is repaid
 */
debt_repaid: i128;
}


/**
 * [`Obligation::remove_collateral`] resulting data
 */
export interface RemoveCollateralResult {
  /**
 * Amount of collateral tokens removed
 */
collateral_decrease: i128;
  /**
 * Amount of collateral tokens received by the collateral remover(accounting subtracted fees)
 */
collateral_remover_to_receive: i128;
  computed_fees: ComputedFees;
}


/**
 * [`Obligation::cover_bad_debt`] resulting data
 */
export interface CoverBadDebtResult {
  /**
 * `(pool address, borrower dTokens)` pairs for each bad debt obligation borrows
 */
borrows_to_be_compensated: Array<readonly [string, i128]>;
  /**
 * `(pool address, borrower jTokens, borrower collateral)` tuples for each bad debt obligation
 * collateral
 */
collaterals_to_remove: Array<readonly [string, i128, i128]>;
}


export interface Pool {
  /**
 * Amount of tokens that can be withdraw by the host platform admin as a fee
 */
accumulated_host_fees: i128;
  /**
 * Amount of tokens that can be withdrawn by the market's admin as a fee
 */
accumulated_market_fees: i128;
  /**
 * Amount of tokens in the insurance reserve that can be used to cover a bad debt scenario
 */
accumulated_reserve_fees: i128;
  /**
 * Configuration settings for the pool
 */
config: PoolConfig;
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
 * The address of the token contract associated with the pool
 */
token_address: string;
  /**
 * The ticker symbol of the associated token
 */
token_ticker: string;
  /**
 * The total amount of currently available tokens for borrowing
 */
total_available: i128;
  /**
 * The total amount of borrowed assets. This value increases with interest rate accrual
 */
total_borrowed: i128;
  /**
 * The total amount of deposited collateral assets that don't accrue interest
 */
total_collateral: i128;
  /**
 * The total `dTokens` amount. Represents the sum of all debt shares distributed among debtors
 */
total_d_tokens: i128;
  /**
 * The total `jTokens` amount. Represents the sum of all yielding interest collateral shares
 * distributed among creditors
 */
total_j_tokens: i128;
}


export interface PoolFeeConfig {
  add_collateral_fee_bps: u32;
  borrow_fee_bps: u32;
  deposit_fee_bps: u32;
  flash_loan_fee_bps: u32;
  host_fee_bps: u32;
  remove_collateral_fee_bps: u32;
  repay_fee_bps: u32;
  take_rate_bps: u32;
  withdraw_fee_bps: u32;
}


export interface PoolConfig {
  accrual_model: AccrualModel;
  fee_config: PoolFeeConfig;
  health_config: PoolHealthConfig;
  interest_rate_model: InterestRateModel;
}


export interface PoolHealthConfig {
  /**
 * The maximum percentage of an asset's value that can be held in an individual obligation in
 * basis points with respect to a total obligation's collateral value. LTV greater than
 * that makes borrow position eligible to liquidation
 */
close_ltv_bps: i128;
  /**
 * The factor used to calculate the current borrow limit by multiplying the collateral value
 * by it before subtracting this value from the obligation's max borrow limit. Volatile
 * assets' pools are expected to have this value set way above 100%
 */
liability_factor_bps: i128;
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
 * The maximum amount of supplied tokens that can be supplied in the pool(i.e., `available` +
 * `total_borrowed`) 0 denotes unlimited supply
 */
supply_limit: i128;
  /**
 * The maximum utilization ratio that is allowed to be reached via borrowing
 */
utilization_ratio_limit_bps: i128;
}


export interface GlobalState {
  admin: string;
  deployer: string;
  name: string;
  status: boolean;
}

export type DataKey = {tag: "GlobalState", values: void} | {tag: "Pool", values: readonly [string]} | {tag: "Obligation", values: readonly [ObligationKey]} | {tag: "MultiplyPair", values: readonly [readonly [string, string]]} | {tag: "Accrual", values: void} | {tag: "AllPools", values: void} | {tag: "AllObligations", values: void} | {tag: "AllMultiplyPairs", values: void} | {tag: "OracleAddress", values: void};


/**
 * Price data for an asset at a specific timestamp
 */
export interface PriceData {
  price: i128;
  timestamp: u64;
}

/**
 * Asset type
 */
export type Asset = {tag: "Stellar", values: readonly [string]} | {tag: "Other", values: readonly [string]};

export interface Client {
  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Upgrades the lending contract
   * 
   * ### Arguments
   * * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
   * version of the contract
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
   * Construct and simulate a get_oracle_address transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Gets the contract's oracle address
   */
  get_oracle_address: (options?: {
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
  }) => Promise<AssembledTransaction<string>>

  /**
   * Construct and simulate a initialize_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Initializes a loan pool for a specific asset
   * 
   * ### Arguments
   * * `token_address` - address of a corresponding Soroban Asset Contract
   * * `token_ticker` - symbol which represents a pool's token ticker
   * * `salt` - optional salt data, which, when provided, is used along with `token_address` to
   * derive a deterministic pool address
   * * `pool_config` - optional `PoolConfig` data. If not provided, a default pool config is used
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
   * * `deposit_pool_address` - address of a pool in a pair for a leveraged deposit
   * * `borrow_pool_address` - address of a pool in a pair for a leveraged borrow
   */
  initialize_multiply_pair: ({deposit_pool_address, borrow_pool_address}: {deposit_pool_address: string, borrow_pool_address: string}, options?: {
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
   * * `borrower` - the borrower whose position is being liquidated
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
   * Construct and simulate a clean_multiply_pairs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  clean_multiply_pairs: (options?: {
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
   * Construct and simulate a check_multiply_pair_exists transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  check_multiply_pair_exists: ({deposit_pool_address, borrow_pool_address}: {deposit_pool_address: string, borrow_pool_address: string}, options?: {
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
  }) => Promise<AssembledTransaction<boolean>>

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
   * * `deposit_as_margin` - flag that determines which asset(deposited or borrowed) will be used
   * as the provided by the user initial margin amount
   * * `amount` - original borrow amount before the leverage
   * * `leverage_multiplier` - leverage multiplier, where the last two digits represent decimal
   * places (e.g., 700 for x7.00, 255 for x2.55, etc.)
   */
  deposit_with_leverage: ({user, deposit_pool_address, borrow_pool_address, deposit_as_margin, amount, leverage_multiplier}: {user: string, deposit_pool_address: string, borrow_pool_address: string, deposit_as_margin: boolean, amount: i128, leverage_multiplier: u32}, options?: {
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
   * Construct and simulate a redeem_accumulated_market_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Redeems accumulated market fees
   * 
   * ### Arguments
   * * `user` - user that tries to redeem market fees
   * * `pool_address` - address of a pool whose fees are redeemed
   * * `amount` - desired amount of fees to redeem as tokens
   */
  redeem_accumulated_market_fees: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
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
   * Construct and simulate a redeem_accumulated_host_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Redeems accumulated host fees
   * 
   * ### Arguments
   * * `user` - user that tries to redeem host fees
   * * `pool_address` - address of a pool whose fees are redeemed
   * * `amount` - desired amount of fees to redeem as tokens
   */
  redeem_accumulated_host_fees: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: {
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
   * Construct and simulate a cover_obligation_bad_debt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Covers fully or partially bad debt if it exists under a user obligation. Socializes all
   * remaining bad debt in case the market reserves doesn't contain enough funds to cover it
   * completely
   * 
   * ### Arguments
   * * `bad_debt_obligation_user` - user that has a bad debt
   */
  cover_obligation_bad_debt: ({bad_debt_obligation_user}: {bad_debt_obligation_user: string}, options?: {
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
   * Construct and simulate a cover_multiply_pair_bad_debt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Covers fully or partially bad debt if it exists under a multiply pair user obligation.
   * Socializes all remaining bad debt in case the reserve doesn't contain enough funds to
   * cover it completely
   * 
   * ### Arguments
   * * `bad_debt_obligation_user` - user that has a bad debt
   * * `deposit_pool_address` - address of a pool from the pair to which the deposit happens
   * * `borrow_pool_address` - address of a pool from the pair from which the borrow happens
   */
  cover_multiply_pair_bad_debt: ({bad_debt_obligation_user, deposit_pool_address, borrow_pool_address}: {bad_debt_obligation_user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: {
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
   * Construct and simulate a get_multiply_pair_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the user's obligation for a specific multiply pair
   * 
   * ### Arguments
   * * `user` - user whose obligation is returned
   * * `deposit_pool_address` - address of a deposit pool from the pair
   * * `borrow_pool_address` - address of a borrow pool from the pair
   */
  get_multiply_pair_obligation: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: {
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
   * Construct and simulate a get_multiply_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns the specific multiply pair
   * 
   * ### Arguments
   * * `deposit_pool_address` - deposit pool of a pair that is returned
   * * `borrow_pool_address` - borrow pool of a pair that is returned
   */
  get_multiply_pair: ({deposit_pool_address, borrow_pool_address}: {deposit_pool_address: string, borrow_pool_address: string}, options?: {
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
  }) => Promise<AssembledTransaction<Result<MultiplyPair>>>

  /**
   * Construct and simulate a get_all_multiply_pairs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns a list of all multiply pairs registered for the market
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
   * Construct and simulate a get_apr transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Returns APR calculated for the current utilization ratio of a pool in basis points (e.g.,
   * 2912 = 29.12%, etc)
   * 
   * ### Arguments
   * * `pool_address` - address of a pool for which APR is returned
   */
  get_apr: ({pool_address}: {pool_address: string}, options?: {
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
  }) => Promise<AssembledTransaction<Result<AnnualPercentageRates>>>

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
  }) => Promise<AssembledTransaction<Result<AnnualPercentageYields>>>

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
        {name, admin, oracle, deployer}: {name: string, admin: string, oracle: string, deployer: string},
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
    return ContractClient.deploy({name, admin, oracle, deployer}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAAAAAAAAAAAAADEFjY3J1YWxNb2RlbAAAAAEAAAAAAAAAAAAAAApDb21wb3VuZGVkAAA=",
        "AAAAAAAAALlDb25zdHJ1Y3RzIHRoZSBtYXJrZXQgY29udHJhY3QKCiMjIyBBcmd1bWVudHMKKiBgYWRtaW5gIC0gbWFya2V0J3MgYWRtaW5pc3RyYXRvcgoqIGBuYW1lYCAtIG1hcmtldCdzIG5hbWUobm90IG5lY2Vzc2FyaWx5IHVuaXF1ZSkKKiBgb3JhY2xlYCAtIFNFUC00MCBjb21wbGlhbnQgb3JhY2xlJ3MgY29udHJhY3QgYWRkcmVzcwAAAAAAAA1fX2NvbnN0cnVjdG9yAAAAAAAABAAAAAAAAAAEbmFtZQAAABAAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAGb3JhY2xlAAAAAAATAAAAAAAAAAhkZXBsb3llcgAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAJ1VcGdyYWRlcyB0aGUgbGVuZGluZyBjb250cmFjdAoKIyMjIEFyZ3VtZW50cwoqIGBuZXdfd2FzbV9oYXNoYCAtIGhhc2ggb2YgdGhlIFdBU00gYmluYXJ5IHVwbG9hZGVkIHRvIHRoZSBuZXR3b3JrIHRoYXQncyB1c2VkIGFzIGEgbmV3CnZlcnNpb24gb2YgdGhlIGNvbnRyYWN0AAAAAAAAB3VwZ3JhZGUAAAAAAQAAAAAAAAANbmV3X3dhc21faGFzaAAAAAAAA+4AAAAgAAAAAA==",
        "AAAAAAAAACBHZXRzIHRoZSBjb250cmFjdCdzIGdsb2JhbCBzdGF0ZQAAABBnZXRfZ2xvYmFsX3N0YXRlAAAAAAAAAAEAAAfQAAAAC0dsb2JhbFN0YXRlAA==",
        "AAAAAAAAACJHZXRzIHRoZSBjb250cmFjdCdzIG9yYWNsZSBhZGRyZXNzAAAAAAASZ2V0X29yYWNsZV9hZGRyZXNzAAAAAAAAAAAAAQAAABM=",
        "AAAAAAAAAZ5Jbml0aWFsaXplcyBhIGxvYW4gcG9vbCBmb3IgYSBzcGVjaWZpYyBhc3NldAoKIyMjIEFyZ3VtZW50cwoqIGB0b2tlbl9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBjb3JyZXNwb25kaW5nIFNvcm9iYW4gQXNzZXQgQ29udHJhY3QKKiBgdG9rZW5fdGlja2VyYCAtIHN5bWJvbCB3aGljaCByZXByZXNlbnRzIGEgcG9vbCdzIHRva2VuIHRpY2tlcgoqIGBzYWx0YCAtIG9wdGlvbmFsIHNhbHQgZGF0YSwgd2hpY2gsIHdoZW4gcHJvdmlkZWQsIGlzIHVzZWQgYWxvbmcgd2l0aCBgdG9rZW5fYWRkcmVzc2AgdG8KZGVyaXZlIGEgZGV0ZXJtaW5pc3RpYyBwb29sIGFkZHJlc3MKKiBgcG9vbF9jb25maWdgIC0gb3B0aW9uYWwgYFBvb2xDb25maWdgIGRhdGEuIElmIG5vdCBwcm92aWRlZCwgYSBkZWZhdWx0IHBvb2wgY29uZmlnIGlzIHVzZWQAAAAAAA9pbml0aWFsaXplX3Bvb2wAAAAABAAAAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAAAAAADHRva2VuX3RpY2tlcgAAABEAAAAAAAAABHNhbHQAAAPoAAAD7gAAACAAAAAAAAAAC3Bvb2xfY29uZmlnAAAAA+gAAAfQAAAAClBvb2xDb25maWcAAAAAAAEAAAPpAAAAEwAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAMZJbml0aWFsaXplcyBhIG11bHRpcGx5IHBhaXIKCiMjIyBBcmd1bWVudHMKKiBgZGVwb3NpdF9wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgaW4gYSBwYWlyIGZvciBhIGxldmVyYWdlZCBkZXBvc2l0CiogYGJvcnJvd19wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgaW4gYSBwYWlyIGZvciBhIGxldmVyYWdlZCBib3Jyb3cAAAAAABhpbml0aWFsaXplX211bHRpcGx5X3BhaXIAAAACAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAANdEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB0aGF0IGRlcG9zaXRzIGEgdG9rZW4KKiBgcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIHRvIHdoaWNoIHRoZSBkZXBvc2l0IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGdvaW5nIHRvIGJlIGRlcG9zaXRlZAAAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAANZCb3Jyb3dzIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGJvcnJvd3MgYSB0b2tlbgoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgYm9ycm93IGhhcHBlbnMKKiBgYW1vdW50YCAtIGFtb3VudCBvZiB0b2tlbnMgd2hpY2ggYXJlIGdvaW5nIHRvIGJlIGJvcnJvd2VkAAAAAAAGYm9ycm93AAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAVhTd2FwIHRva2VucyB2aWEgYSBzd2FwIHByb3ZpZGVyIGNvbnRyYWN0LiBUaGlzIGd1YXJhbnRlZXMgYSBzd2FwCmFuZCBpcyBhZ25vc3RpYyB0byB0aGUgcG9zc2libGUgcHJpY2Ugc2xpcHBhZ2UKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIGRlcG9zaXRzIGEgdG9rZW4KKiBgdG9rZW5faW5gIC0gYWRkcmVzcyBvZiBhIHRva2VuIHRoYXQgd291bGQgYmUgdGFrZW4gZnJvbSB0aGUgdXNlcgoqIGB0b2tlbl9vdXRgIC0gYWRkcmVzcyBvZiBhIHRva2VuIHRoYXQgd291bGQgYmUgZ2l2ZW4gdG8gdGhlIHVzZXIKKiBgYW1vdW50YCAtIGV4YWN0IGFtb3VudCBvZiB0aGUgYHRva2VuX2luYAAAAARzd2FwAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAACHRva2VuX2luAAAAEwAAAAAAAAAJdG9rZW5fb3V0AAAAAAAAEwAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAEAAAPpAAAACwAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAWlBZGRzIHRva2VucyBpbnRvIHRoZSBsb2FuIHBvb2wgYXMgY29sbGF0ZXJhbCBvbmx5LgpUaGlzIGltcGxpZXMgdGhhdCB0aGV5IGFyZSBhbHdheXMgYXZhaWxhYmxlIGZvciBhIGhlYWx0aHkgd2l0aGRyYXdhbCBmb3IgdGhlCmNvc3Qgb2Ygbm90IGFjY3J1aW5nIGFuIGludGVyZXN0IHJhdGUKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHRoYXQgYWRkcyBjb2xsYXRlcmFsCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB0byB3aGljaCB0aGUgY29sbGF0ZXJhbCBpcyBiZWluZyBhZGRlZAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHRva2VucyB3aGljaCBhcmUgYmVpbmcgYWRkZWQgYXMgYSBjb2xsYXRlcmFsAAAAAAAADmFkZF9jb2xsYXRlcmFsAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAbFSZW1vdmVzIGNvbGxhdGVyYWwgdG9rZW5zIGZyb20gdGhlIGxvYW4gcG9vbCB0byB0aGUgdXNlcgoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggd2l0aGRyYXdzIGNvbGxhdGVyYWwgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGRlc2lyZWQgYW1vdW50IG9mIGNvbGxhdGVyYWwgdG9rZW5zIHRvIHJlbW92ZS4KVGhlIGFjdHVhbCBhbW91bnQgcmVtb3ZlZCBpcyBjYXBwZWQgdG8gbWFpbnRhaW4gdGhlIHBvc2l0aW9uJ3MgTFRWIGF0IGl0cyBPcGVuIExUViBvbiB0aGUKcG9vbC4gUGFzc2luZyBbYHU2NDo6TUFYYF0gKG9yIFtgaTEyODo6TUFYYF0pIGVmZmVjdGl2ZWx5IHJlbW92ZXMgYWxsIGF2YWlsYWJsZQpjb2xsYXRlcmFsAAAAAAAAEXJlbW92ZV9jb2xsYXRlcmFsAAAAAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAWVSZXBheXMgYm9ycm93ZWQgdG9rZW5zCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aGljaCByZXBheXMgYm9ycm93ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSBib3Jyb3cgaGFwcGVuZWQKKiBgYW1vdW50YCAtIHByb3ZpZGVkIGFtb3VudCBvZiB0b2tlbnMgdG8gcmVwYXkuIElmIHRoaXMgYW1vdW50IGV4Y2VlZHMgdGhlIHRvdGFsIGRlYnQsIG9ubHkKdGhlIG91dHN0YW5kaW5nIGRlYnQgd2lsbCBiZSByZXBhaWQuClBhc3NpbmcgW2B1NjQ6Ok1BWGBdIChvciBbYGkxMjg6Ok1BWGBdKSBjYW4gYmUgdXNlZCB0byByZXBheSB0aGUgZW50aXJlIGRlYnQAAAAAAAAFcmVwYXkAAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAchMaXF1aWRhdGVzIGJvcnJvd2VyJ3MgcG9zaXRpb24gaWYgcG9zaXRpb24ncyBoZWFsdGggZmFjdG9yIGNyaXRlcmlvbiBpc24ndCBtZXQKCiMjIyBBcmd1bWVudHMKKiBgbGlxdWlkYXRvcmAgLSBhZ2VudCB3aGljaCBsaXF1aWRhdGVzIHRoZSBib3Jyb3dlcidzIHBvc2l0aW9uCiogYGJvcnJvd2VyYCAtIHRoZSBib3Jyb3dlciB3aG9zZSBwb3NpdGlvbiBpcyBiZWluZyBsaXF1aWRhdGVkCiogYGJvcnJvd19wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgYm9ycm93ZWQgdG9rZW5zIGFyZSByZXBhaWQgYnkgdGhlCmxpcXVpZGF0b3IKKiBgY29sbGF0ZXJhbF9wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgdG9rZW5zIGFyZSBzb2xkIHRvIHRoZSBsaXF1aWRhdG9yIHdpdGgKYSBkaXNjb3VudAoqIGBhbW91bnRgIC0gYW1vdW50IG9mIHJlcGFpZCB0b2tlbnMAAAAJbGlxdWlkYXRlAAAAAAAABQAAAAAAAAAKbGlxdWlkYXRvcgAAAAAAEwAAAAAAAAAIYm9ycm93ZXIAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAAF2NvbGxhdGVyYWxfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAbFXaXRoZHJhd3MgZGVwb3NpdGVkIHRva2VucyBmcm9tIHRoZSBsb2FuIHBvb2wgdG8gdGhlIHVzZXIKCiMjIyBBcmd1bWVudHMKKiBgdXNlcmAgLSB1c2VyIHdoaWNoIHdpdGhkcmF3cyBkZXBvc2l0ZWQgdG9rZW5zCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHdoaWNoIHRoZSB3aXRoZHJhd2FsIGhhcHBlbnMKKiBgYW1vdW50YCAtIGRlc2lyZWQgYW1vdW50IG9mIHRva2VucyB0byB3aXRoZHJhdy4KVGhlIGFjdHVhbCBhbW91bnQgd2l0aGRyYXduIGlzIGNhcHBlZCB0byBtYWludGFpbiB0aGUgcG9zaXRpb24ncyBMVFYgYXQgaXRzIE9wZW4gTFRWIG9uIHRoZQpwb29sLiBQYXNzaW5nIFtgdTY0OjpNQVhgXSAob3IgW2BpMTI4OjpNQVhgXSkgY2FuIGJlIHVzZWQgdG8gd2l0aGRyYXcgYWxsIHRva2VucwphdmFpbGFibGUgZm9yIGl0AAAAAAAACHdpdGhkcmF3AAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAPlDcmVhdGVzIGEgZmxhc2ggbG9hbgoKIyMjIEFyZ3VtZW50cwoqIGBjb250cmFjdGAgLSBjb250cmFjdCdzIGFkZHJlc3Mgd2hpY2ggbGV2ZXJhZ2VzIHRoZSBmbGFzaCBsb2FuZWQgYW1vdW50IGFuZCBhZGhlcmVzIHRvCmBlcmMzMTU2YCBzdGFuZGFyZAoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB3aGljaCB0aGUgZmxhc2ggbG9hbiBoYXBwZW5zCiogYGFtb3VudGAgLSBhbW91bnQgb2YgbGVudCB0b2tlbnMAAAAAAAAKZmxhc2hfbG9hbgAAAAAAAwAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAUY2xlYW5fbXVsdGlwbHlfcGFpcnMAAAAAAAAAAA==",
        "AAAAAAAAAAAAAAAaY2hlY2tfbXVsdGlwbHlfcGFpcl9leGlzdHMAAAAAAAIAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAQAAAAE=",
        "AAAAAAAAAzVEZXBvc2l0cyB0b2tlbnMgaW50byB0aGUgbG9hbiBwb29sIHdpdGggbGV2ZXJhZ2UuIExldmVyYWdlIGlzIGFjaGlldmVkIGJ5IHV0aWxpemluZyBmbGFzaApsb2FuIGFuZCB0b2tlbiBzd2FwCgojIFdBUk5JTkcKVGhpcyBpbmNyZWFzZXMgdGhlIHBlcmNlaXZlZCBgc3VwcGx5IEFQUmAgb25seQp3aGVuIGAoYm9ycm93ZWQgdG9rZW4gYm9ycm93IEFQUiA8IHN1cHBseSB0b2tlbiBzdXBwbHkgQVBSKWAgaG9sZHMgdHJ1ZQoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgdGhhdCBkZXBvc2l0cyB0b2tlbnMgd2l0aCBsZXZlcmFnZQoqIGBkZXBvc2l0X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIHRvIHdoaWNoIHRoZSBkZXBvc2l0IGhhcHBlbnMKKiBgYm9ycm93X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIGZyb20gd2hpY2ggdGhlIGJvcnJvdyBoYXBwZW5zCiogYGRlcG9zaXRfYXNfbWFyZ2luYCAtIGZsYWcgdGhhdCBkZXRlcm1pbmVzIHdoaWNoIGFzc2V0KGRlcG9zaXRlZCBvciBib3Jyb3dlZCkgd2lsbCBiZSB1c2VkCmFzIHRoZSBwcm92aWRlZCBieSB0aGUgdXNlciBpbml0aWFsIG1hcmdpbiBhbW91bnQKKiBgYW1vdW50YCAtIG9yaWdpbmFsIGJvcnJvdyBhbW91bnQgYmVmb3JlIHRoZSBsZXZlcmFnZQoqIGBsZXZlcmFnZV9tdWx0aXBsaWVyYCAtIGxldmVyYWdlIG11bHRpcGxpZXIsIHdoZXJlIHRoZSBsYXN0IHR3byBkaWdpdHMgcmVwcmVzZW50IGRlY2ltYWwKcGxhY2VzIChlLmcuLCA3MDAgZm9yIHg3LjAwLCAyNTUgZm9yIHgyLjU1LCBldGMuKQAAAAAAABVkZXBvc2l0X3dpdGhfbGV2ZXJhZ2UAAAAAAAAGAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAAEWRlcG9zaXRfYXNfbWFyZ2luAAAAAAAAAQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAABNsZXZlcmFnZV9tdWx0aXBsaWVyAAAAAAQAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAopXaXRoZHJhd3MgdG9rZW5zIGZyb20gdGhlIGxldmVyYWdlZCBkZXBvc2l0IHBvc2l0aW9uIHdpdGhvdXQgYWZmZWN0aW5nIHRoZSBsZXZlcmFnZQptdWx0aXBsaWVyCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB0aGF0IGRlbGV2ZXJhZ2VzIGFuZCB3aXRoZHJhd3MgZnJvbSB0aGUgcG9zaXRpb24KKiBgZGVwb3NpdF9wb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgZnJvbSB0aGUgcGFpciB0byB3aGljaCB0aGUgZGVwb3NpdCBoYXBwZW5lZAoqIGBib3Jyb3dfcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBwb29sIGZyb20gdGhlIHBhaXIgZnJvbSB3aGljaCB0aGUgYm9ycm93IGhhcHBlbmVkCiogYGFtb3VudGAgLSBkZXNpcmVkIGFtb3VudCBvZiBkZXBvc2l0ZWQgdG9rZW5zIHRvIHdpdGhkcmF3LgpUaGUgYWN0dWFsIGFtb3VudCB3aXRoZHJhd24gaXMgY2FwcGVkIGJ5IHRoZSB2YWx1ZSBkaWZmZXJlbmNlIGJldHdlZW4gZGVwb3NpdGVkIGFuZCBib3Jyb3dlZAp0b2tlbnMgaW4gdGhlIGxldmVyYWdlZCBwb3NpdGlvbiAobWludXMgb3BlcmF0aW9uYWwgZmVlcykuIFBhc3NpbmcgW2B1NjQ6Ok1BWGBdIChvcgpbYGkxMjg6Ok1BWGBdKSBjYW4gYmUgdXNlZCB0byB3aXRoZHJhdyBhbGwgYXZhaWxhYmxlIHRva2VucwAAAAAAF3dpdGhkcmF3X2Zyb21fbGV2ZXJhZ2VkAAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAANRSZWRlZW1zIGFjY3VtdWxhdGVkIG1hcmtldCBmZWVzCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB0aGF0IHRyaWVzIHRvIHJlZGVlbSBtYXJrZXQgZmVlcwoqIGBwb29sX2FkZHJlc3NgIC0gYWRkcmVzcyBvZiBhIHBvb2wgd2hvc2UgZmVlcyBhcmUgcmVkZWVtZWQKKiBgYW1vdW50YCAtIGRlc2lyZWQgYW1vdW50IG9mIGZlZXMgdG8gcmVkZWVtIGFzIHRva2VucwAAAB5yZWRlZW1fYWNjdW11bGF0ZWRfbWFya2V0X2ZlZXMAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAANBSZWRlZW1zIGFjY3VtdWxhdGVkIGhvc3QgZmVlcwoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgdGhhdCB0cmllcyB0byByZWRlZW0gaG9zdCBmZWVzCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCB3aG9zZSBmZWVzIGFyZSByZWRlZW1lZAoqIGBhbW91bnRgIC0gZGVzaXJlZCBhbW91bnQgb2YgZmVlcyB0byByZWRlZW0gYXMgdG9rZW5zAAAAHHJlZGVlbV9hY2N1bXVsYXRlZF9ob3N0X2ZlZXMAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAQFDb3ZlcnMgZnVsbHkgb3IgcGFydGlhbGx5IGJhZCBkZWJ0IGlmIGl0IGV4aXN0cyB1bmRlciBhIHVzZXIgb2JsaWdhdGlvbi4gU29jaWFsaXplcyBhbGwKcmVtYWluaW5nIGJhZCBkZWJ0IGluIGNhc2UgdGhlIG1hcmtldCByZXNlcnZlcyBkb2Vzbid0IGNvbnRhaW4gZW5vdWdoIGZ1bmRzIHRvIGNvdmVyIGl0CmNvbXBsZXRlbHkKCiMjIyBBcmd1bWVudHMKKiBgYmFkX2RlYnRfb2JsaWdhdGlvbl91c2VyYCAtIHVzZXIgdGhhdCBoYXMgYSBiYWQgZGVidAAAAAAAABljb3Zlcl9vYmxpZ2F0aW9uX2JhZF9kZWJ0AAAAAAAAAQAAAAAAAAAYYmFkX2RlYnRfb2JsaWdhdGlvbl91c2VyAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAbdDb3ZlcnMgZnVsbHkgb3IgcGFydGlhbGx5IGJhZCBkZWJ0IGlmIGl0IGV4aXN0cyB1bmRlciBhIG11bHRpcGx5IHBhaXIgdXNlciBvYmxpZ2F0aW9uLgpTb2NpYWxpemVzIGFsbCByZW1haW5pbmcgYmFkIGRlYnQgaW4gY2FzZSB0aGUgcmVzZXJ2ZSBkb2Vzbid0IGNvbnRhaW4gZW5vdWdoIGZ1bmRzIHRvCmNvdmVyIGl0IGNvbXBsZXRlbHkKCiMjIyBBcmd1bWVudHMKKiBgYmFkX2RlYnRfb2JsaWdhdGlvbl91c2VyYCAtIHVzZXIgdGhhdCBoYXMgYSBiYWQgZGVidAoqIGBkZXBvc2l0X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIHRvIHdoaWNoIHRoZSBkZXBvc2l0IGhhcHBlbnMKKiBgYm9ycm93X3Bvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmcm9tIHRoZSBwYWlyIGZyb20gd2hpY2ggdGhlIGJvcnJvdyBoYXBwZW5zAAAAABxjb3Zlcl9tdWx0aXBseV9wYWlyX2JhZF9kZWJ0AAAAAwAAAAAAAAAYYmFkX2RlYnRfb2JsaWdhdGlvbl91c2VyAAAAEwAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAABhSZXR1cm5zIGFzc2V0J3MgZGVjaW1hbHMAAAASZ2V0X2Fzc2V0X2RlY2ltYWxzAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAB9SZXR1cm5zIG9yYWNsZSBwcmljZSdzIGRlY2ltYWxzAAAAABlnZXRfb3JhY2xlX3ByaWNlX2RlY2ltYWxzAAAAAAAAAAAAAAEAAAAE",
        "AAAAAAAAAGxSZXR1cm5zIHBvb2wgYXNzZXQncyBvcmFjbGUgcHJpY2UKCiMjIyBBcmd1bWVudHMKKiBgcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYXNzZXQgd2hpY2ggcHJpY2UgaXMgcmV0dXJuZWQAAAAbZ2V0X3Bvb2xfYXNzZXRfb3JhY2xlX3ByaWNlAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAAAsAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAJVSZXR1cm5zIHRoZSB1c2VyJ3Mgb2JsaWdhdGlvbiB3aGljaCBpbmNsdWRlcyBkYXRhIGFib3V0IGFsbCBvZiB0aGVpciBkZXBvc2l0cyBhbmQgYm9ycm93cwoKIyMjIEFyZ3VtZW50cwoqIGB1c2VyYCAtIHVzZXIgd2hpY2ggb2JsaWdhdGlvbiBpcyByZXR1cm5lZAAAAAAAABNnZXRfdXNlcl9vYmxpZ2F0aW9uAAAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAA+kAAAfQAAAACk9ibGlnYXRpb24AAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAPpSZXR1cm5zIHRoZSB1c2VyJ3Mgb2JsaWdhdGlvbiBmb3IgYSBzcGVjaWZpYyBtdWx0aXBseSBwYWlyCgojIyMgQXJndW1lbnRzCiogYHVzZXJgIC0gdXNlciB3aG9zZSBvYmxpZ2F0aW9uIGlzIHJldHVybmVkCiogYGRlcG9zaXRfcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBkZXBvc2l0IHBvb2wgZnJvbSB0aGUgcGFpcgoqIGBib3Jyb3dfcG9vbF9hZGRyZXNzYCAtIGFkZHJlc3Mgb2YgYSBib3Jyb3cgcG9vbCBmcm9tIHRoZSBwYWlyAAAAAAAcZ2V0X211bHRpcGx5X3BhaXJfb2JsaWdhdGlvbgAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAPpAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAFxSZXR1cm5zIHRoZSBzcGVjaWZpYyBsb2FuIHBvb2wKCiMjIyBBcmd1bWVudHMKKiBgcG9vbF9hZGRyZXNzYCAtIHBvb2wgd2hpY2ggZGF0YSBpcyByZXR1cm5lZAAAAAhnZXRfcG9vbAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAAEUG9vbAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAADRSZXR1cm5zIGEgbGlzdCBvZiBhbGwgcG9vbCBhZGRyZXNzZXMgaW4gdGhlIHByb3RvY29sAAAADWdldF9hbGxfcG9vbHMAAAAAAAAAAAAAAQAAA+oAAAAT",
        "AAAAAAAAADZSZXR1cm5zIGEgbGlzdCBvZiBhbGwgdXNlciBvYmxpZ2F0aW9ucyBpbiB0aGUgcHJvdG9jb2wAAAAAABNnZXRfYWxsX29ibGlnYXRpb25zAAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAALVSZXR1cm5zIHRoZSBzcGVjaWZpYyBtdWx0aXBseSBwYWlyCgojIyMgQXJndW1lbnRzCiogYGRlcG9zaXRfcG9vbF9hZGRyZXNzYCAtIGRlcG9zaXQgcG9vbCBvZiBhIHBhaXIgdGhhdCBpcyByZXR1cm5lZAoqIGBib3Jyb3dfcG9vbF9hZGRyZXNzYCAtIGJvcnJvdyBwb29sIG9mIGEgcGFpciB0aGF0IGlzIHJldHVybmVkAAAAAAAAEWdldF9tdWx0aXBseV9wYWlyAAAAAAAAAgAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAABAAAD6QAAB9AAAAAMTXVsdGlwbHlQYWlyAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAD5SZXR1cm5zIGEgbGlzdCBvZiBhbGwgbXVsdGlwbHkgcGFpcnMgcmVnaXN0ZXJlZCBmb3IgdGhlIG1hcmtldAAAAAAAFmdldF9hbGxfbXVsdGlwbHlfcGFpcnMAAAAAAAAAAAABAAAD6gAAB9AAAAAMTXVsdGlwbHlQYWly",
        "AAAAAAAAALtSZXR1cm5zIEFQUiBjYWxjdWxhdGVkIGZvciB0aGUgY3VycmVudCB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wgaW4gYmFzaXMgcG9pbnRzIChlLmcuLAoyOTEyID0gMjkuMTIlLCBldGMpCgojIyMgQXJndW1lbnRzCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmb3Igd2hpY2ggQVBSIGlzIHJldHVybmVkAAAAAAdnZXRfYXByAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAAVQW5udWFsUGVyY2VudGFnZVJhdGVzAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAALtSZXR1cm5zIEFQWSBjYWxjdWxhdGVkIGZvciB0aGUgY3VycmVudCB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wgaW4gYmFzaXMgcG9pbnRzIChlLmcuLAoyOTEyID0gMjkuMTIlLCBldGMpCgojIyMgQXJndW1lbnRzCiogYHBvb2xfYWRkcmVzc2AgLSBhZGRyZXNzIG9mIGEgcG9vbCBmb3Igd2hpY2ggQVBZIGlzIHJldHVybmVkAAAAAAdnZXRfYXB5AAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAAWQW5udWFsUGVyY2VudGFnZVlpZWxkcwAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAJZSZXNldHMgdGhlIGNvbnRyYWN0J3Mgc3RvcmFnZS4gVXNlZnVsIHdoZW4gdGhlIGNvbnRyYWN0J3MgaW52YXJpYW50cyBhcmUgYnJva2VuIGFuZCByZXF1aXJlCnJlc2V0dGluZyBvbiB0aGUgdGVzdG5ldCB3aXRob3V0IHJlLWRlcGxveWluZyB0aGUgY29udHJhY3QAAAAAAA1yZXNldF9zdG9yYWdlAAAAAAAAAAAAAAA=",
        "AAAABAAAAAAAAAAAAAAAB01DRXJyb3IAAAAAIAAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAD092ZXJPclVuZGVyZmxvdwAAAAABAAAAAAAAABBJbnZhbGlkVGltZXN0YW1wAAAAAgAAAAAAAAAXRGVwZW5kZW5jeUNvbnRyYWN0RXJyb3IAAAAAAwAAAAAAAAARUG9vbEFscmVhZHlFeGlzdHMAAAAAAAAKAAAAAAAAABBQb29sRG9lc05vdEV4aXN0AAAACwAAAAAAAAAVSW52YWxpZExvYW5Qb29sQ29uZmlnAAAAAAAADAAAAAAAAAASTm90RW5vdWdoUG9vbEZ1bmRzAAAAAAANAAAAAAAAABdEZXBvc2l0UG9vbERvZXNOb3RFeGlzdAAAAAAOAAAAAAAAABZCb3Jyb3dQb29sRG9lc05vdEV4aXN0AAAAAAAPAAAAAAAAABpDb2xsYXRlcmFsUG9vbERvZXNOb3RFeGlzdAAAAAAAEAAAAAAAAAAWT2JsaWdhdGlvbkRvZXNOb3RFeGlzdAAAAAAAFAAAAAAAAAATRGVwb3NpdERvZXNOb3RFeGlzdAAAAAAVAAAAAAAAABJCb3Jyb3dEb2VzTm90RXhpc3QAAAAAABYAAAAAAAAADk5lZ2F0aXZlQW1vdW50AAAAAAAeAAAAAAAAABNXaXRoZHJhd092ZXJCYWxhbmNlAAAAACgAAAAAAAAAF1Bvb2xTdXBwbHlMaW1pdEV4Y2VlZGVkAAAAACkAAAAAAAAAH1Bvb2xVdGlsaXphdGlvblJhdGlvQ2FwRXhjZWVkZWQAAAAAKgAAAAAAAAAcQ29sbGF0ZXJhbFJlbW92YWxPdmVyYmFsYW5jZQAAACsAAAAAAAAAG09yYWNsZURvZXNOb3RLbm93QXNzZXRQcmljZQAAAAAyAAAAAAAAABBPcmFjbGVTdGFsZVByaWNlAAAAMwAAAAAAAAAoSGVhbHRoRmFjdG9ySXNMb3dlclRoYW5SZXF1aXJlZFRocmVzaG9sZAAAADwAAAAAAAAAG0ludmFsaWRMaXF1aWRhdGlvblRocmVzaG9sZAAAAAA9AAAAAAAAABtMaXF1aWRhdGVkUG9zaXRpb25Jc0hlYWx0aHkAAAAAPgAAAAAAAAAdTGlxdWlkYXRpb25FeGNlZWRzQ2xvc2VGYWN0b3IAAAAAAAA/AAAAAAAAAA9TZWxmTGlxdWlkYXRpb24AAAAAQAAAAAAAAAAtTGlxdWlkYXRpb25XaXRoRXF1YWxDb2xsYXRlcmFsQW5kRGVwb3NpdFBvb2xzAAAAAAAAQQAAAAAAAAAaUG9zaXRpb25Eb2VzTm90SGF2ZUJhZERlYnQAAAAAAEIAAAAAAAAAGUludmFsaWRMZXZlcmFnZU11bHRpcGxpZXIAAAAAAABGAAAAAAAAABNJbnZhbGlkU3dhcFNsaXBwYWdlAAAAAEcAAAAAAAAAGU11bHRpcGx5UGFpckFscmVhZHlFeGlzdHMAAAAAAABIAAAAAAAAABhNdWx0aXBseVBhaXJEb2VzTm90RXhpc3QAAABJ",
        "AAAAAQAAADhMaW5lYXIgYW5udWFsIGludGVyZXN0IHJhdGVzIHJlcHJlc2VudGVkIGluIGJhc2lzIHBvaW50cwAAAAAAAAAVQW5udWFsUGVyY2VudGFnZVJhdGVzAAAAAAAAAgAAAAAAAAAKYm9ycm93X2JwcwAAAAAABgAAAAAAAAAKc3VwcGx5X2JwcwAAAAAABg==",
        "AAAAAQAAADNDb21wb3VuZCBpbnRlcmVzdCByYXRlcyByZXByZXNlbnRlZCBpbiBiYXNpcyBwb2ludHMAAAAAAAAAABZBbm51YWxQZXJjZW50YWdlWWllbGRzAAAAAAACAAAAAAAAAApib3Jyb3dfYnBzAAAAAAAEAAAAAAAAAApzdXBwbHlfYnBzAAAAAAAE",
        "AAAAAgAAAAAAAAAAAAAAEUludGVyZXN0UmF0ZU1vZGVsAAAAAAAAAQAAAAEAAAAAAAAABktpbmtlZAAAAAAAAQAAB9AAAAAOS2lua2VkSVJDb25maWcAAA==",
        "AAAAAQAAAAAAAAAAAAAADktpbmtlZElSQ29uZmlnAAAAAAAGAAAARkJhc2UgQVBSIHRoYXQgaXMgYWNjcnVlZCByZWdhcmRsZXNzIG9mIHRoZSB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wAAAAAAAxiYXNlX2Fwcl9icHMAAAALAAAARUFQUiB0aGF0IGlzIGFjY3J1ZWQgd2hlbiB0aGUgdXRpbGl6YXRpb24gcmF0aW8gaXMgYXQgdGhlIGtpbmsgMSB2YWx1ZQAAAAAAAA1raW5rMV9hcHJfYnBzAAAAAAAACwAAABhLaW5rIDEgdXRpbGl6YXRpb24gcmF0aW8AAAAMa2luazFfdXJfYnBzAAAACwAAAEVBUFIgdGhhdCBpcyBhY2NydWVkIHdoZW4gdGhlIHV0aWxpemF0aW9uIHJhdGlvIGlzIGF0IHRoZSBraW5rIDIgdmFsdWUAAAAAAAANa2luazJfYXByX2JwcwAAAAAAAAsAAAAYS2luayAyIHV0aWxpemF0aW9uIHJhdGlvAAAADGtpbmsyX3VyX2JwcwAAAAsAAAA5QVBSIHRoYXQgaXMgYWNjcnVlZCB3aGVuIHRoZSB1dGlsaXphdGlvbiByYXRpbyBpcyBhdCAxMDAlAAAAAAAAC21heF9hcHJfYnBzAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAADE11bHRpcGx5UGFpcgAAAAQAAAAyQWRkcmVzcyBvZiBhIHBvb2wgaW4gYSBwYWlyIGZvciBhIGxldmVyYWdlZCBib3Jyb3cAAAAAAAtib3Jyb3dfcG9vbAAAAAATAAAAM0FkZHJlc3Mgb2YgYSBwb29sIGluIGEgcGFpciBmb3IgYSBsZXZlcmFnZWQgZGVwb3NpdAAAAAAMZGVwb3NpdF9wb29sAAAAEwAAAF5NYXhpbXVtIGxldmVyYWdlIG11bHRpcGxpZXIgYmFzZWQgb24gYm9ycm93IHBvb2wgb3BlbkxUViB2YWx1ZS4gU2NhbGVkIHdpdGgKW2BMRVZFUkFHRV9TQ0FMRWBdAAAAAAAXbWF4X2xldmVyYWdlX211bHRpcGxpZXIAAAAABAAAAHREZXRlcm1pbmlzdGljYWxseSBjb21wdXRlZCB1bmlxdWUgc2VlZCBwZXIgYSBwYWlyLCB1c2VkIHRvIGRpc3Rpbmd1aXNoIGEgdXNlcidzIG11bHRpcGx5CnBhaXIgb2JsaWdhdGlvbiBmcm9tIG90aGVycwAAAARzZWVkAAAD7gAAACA=",
        "AAAAAQAAAAAAAAAAAAAADU9ibGlnYXRpb25LZXkAAAAAAAACAAAAAAAAAARzZWVkAAAD6AAAA+4AAAAgAAAAAAAAAAR1c2VyAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAIAAABEQm9ycm93ZWQgbGlxdWlkaXR5IGZvciB0aGUgb2JsaWdhdGlvbiwgdW5pcXVlIGJ5IGJvcnJvdyBwb29sIGFkZHJlc3MAAAAHYm9ycm93cwAAAAPsAAAAEwAAB9AAAAAQQm9ycm93T2JsaWdhdGlvbgAAAEdEZXBvc2l0ZWQgY29sbGF0ZXJhbCBmb3IgdGhlIG9ibGlnYXRpb24sIHVuaXF1ZSBieSBkZXBvc2l0IHBvb2wgYWRkcmVzcwAAAAAIZGVwb3NpdHMAAAPsAAAAEwAAB9AAAAARRGVwb3NpdE9ibGlnYXRpb24AAAA=",
        "AAAAAQAAAAAAAAAAAAAAEEJvcnJvd09ibGlnYXRpb24AAAACAAAALkFjY3VtdWxhdGVkIHZhbHVlIG9mIGluaXRpYWxseSBib3Jyb3dlZCB0b2tlbnMAAAAAAAhib3Jyb3dlZAAAAAsAAAA8QW1vdW50IG9mIHRoZSB0b3RhbCBkZWJ0IHNoYXJlcyB0aGF0IHRoZSBvYmxpZ2F0aW9uIGNvbnRhaW5zAAAACGRfdG9rZW5zAAAACw==",
        "AAAAAQAAAAAAAAAAAAAAEURlcG9zaXRPYmxpZ2F0aW9uAAAAAAAAAwAAADxBY2N1bXVsYXRlZCB2YWx1ZSBvZiBjb2xsYXRlcmFsIHRoYXQgZG9lc24ndCBhY2NydWUgaW50ZXJlc3QAAAAKY29sbGF0ZXJhbAAAAAAACwAAAN5BY2N1bXVsYXRlZCB2YWx1ZSBvZiBpbml0aWFsbHkgZGVwb3NpdGVkIHRva2Vucy4gRS5nLiwgaWYgYSB1c2VyIGluaXRpYWxseSBkZXBvc2l0ZWQgMTAwCnRva2VucywgdGhlIHRpbWUgcGFzc2VkLCB3aGljaCBjYXVzZWQgMiB0b2tlbnMgdG8gYmUgYWNjcnVlZCwgYW5kIHRoZSB1c2VyIGRlcG9zaXRlZCAyMAptb3JlIHRva2VucyAtIHRoaXMgdmFsdWUgd2lsbCBiZSBlcXVhbCB0byAxMjAAAAAAAAlkZXBvc2l0ZWQAAAAAAAALAAAARUEgc2hhcmUgb2YgdG90YWwgc3VwcGxpZWQgdG9rZW5zIGluIHRoZSBwb29sIHRoYXQgb2JsaWdhdGlvbiBjb250YWlucwAAAAAAAAhqX3Rva2VucwAAAAs=",
        "AAAAAQAAAE9HZW5lcmFsbHkgcmVwcmVzZW50cyBjb21wdXRlZCBmZWVzIGlzc3VlZCBieSBhbnkgcG9zc2libGUgb3BlcmF0aW9uIG9uIGEgbWFya2V0AAAAAAAAAAAMQ29tcHV0ZWRGZWVzAAAAAwAAACJTdW0gb2YgYG1hcmtldF9mZWVgIGFuZCBgaG9zdF9mZWVgAAAAAAAHZmVlX3N1bQAAAAALAAAAI0ZlZSBzZWdyZWdhdGVkIHRvIHRoZSBwcm90b2NvbCBob3N0AAAAAAhob3N0X2ZlZQAAAAsAAAAiRmVlIHNlZ3JlZ2F0ZWQgdG8gdGhlIG1hcmtldCBhZG1pbgAAAAAACm1hcmtldF9mZWUAAAAAAAs=",
        "AAAAAQAAACZbYE9ibGlnYXRpb246OmRlcG9zaXRgXSByZXN1bHRpbmcgZGF0YQAAAAAAAAAAAA1EZXBvc2l0UmVzdWx0AAAAAAAAAwAAAAAAAAANY29tcHV0ZWRfZmVlcwAAAAAAB9AAAAAMQ29tcHV0ZWRGZWVzAAAANUFtb3VudCBvZiBvcmlnaW5hbGx5IGRlcG9zaXRlZCB0b2tlbnMobWludXMgYWxsIGZlZXMpAAAAAAAACWRlcG9zaXRlZAAAAAAAAAsAAABZQW1vdW50IG9mIGBqVG9rZW5zYCB0byBpc3N1ZSB0aGF0IHJlcHJlc2VudCB0aGUgYG9yaWdpbmFsbHlfZGVwb3NpdGVkYCBhbW91bnQgaW4gdGhlIHBvb2wAAAAAAAARal90b2tlbnNfdG9faXNzdWUAAAAAAAAL",
        "AAAAAQAAACVbYE9ibGlnYXRpb246OmJvcnJvd2BdIHJlc3VsdGluZyBkYXRhAAAAAAAAAAAAAAxCb3Jyb3dSZXN1bHQAAAAEAAAAREFtb3VudCBvZiBkZWJ0KGluIHRva2VucykgdGhhdCBpcyBhZGRlZCB0byB0aGUgYm9ycm93ZXIncyBvYmxpZ2F0aW9uAAAAEWJvcnJvd2VyX25ld19kZWJ0AAAAAAAACwAAAEtBbW91bnQgb2YgdG9rZW5zIHRvIHJlY2VpdmUgYnkgdGhlIGJvcnJvd2VyKGBib3Jyb3dlcl9uZXdfZGVidGAgbWludXMgZmVlcykAAAAAE2JvcnJvd2VyX3RvX3JlY2VpdmUAAAAACwAAAAAAAAANY29tcHV0ZWRfZmVlcwAAAAAAB9AAAAAMQ29tcHV0ZWRGZWVzAAAAVkFtb3VudCBvZiBgZFRva2Vuc2AgdG8gaXNzdWUgdGhhdCByZXByZXNlbnQgdGhlIGBib3Jyb3dlcl9uZXdfZGVidGAgYW1vdW50IGluIHRoZSBwb29sAAAAAAARZF90b2tlbnNfdG9faXNzdWUAAAAAAAAL",
        "AAAAAQAAAC1bYE9ibGlnYXRpb246OmFkZF9jb2xsYXRlcmFsYF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAAAAAE0FkZENvbGxhdGVyYWxSZXN1bHQAAAAAAgAAADpBbW91bnQgb2YgdG9rZW5zIGFkZGVkIGFzIGNvbGxhdGVyYWwod2l0aCBzdWJ0cmFjdGVkIGZlZXMpAAAAAAAQYWRkZWRfY29sbGF0ZXJhbAAAAAsAAAAAAAAADWNvbXB1dGVkX2ZlZXMAAAAAAAfQAAAADENvbXB1dGVkRmVlcw==",
        "AAAAAQAAACdbYE9ibGlnYXRpb246OndpdGhkcmF3YF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAAA5XaXRoZHJhd1Jlc3VsdAAAAAAABAAAAAAAAAANY29tcHV0ZWRfZmVlcwAAAAAAB9AAAAAMQ29tcHV0ZWRGZWVzAAAAVkFtb3VudCBvZiB0aGUgb3JpZ2luYWwgZGVwb3NpdChpbiB0b2tlbnMpIHRoYXQgaXMgcmVtb3ZlZCBmcm9tIHRoZSBgRGVwb3NpdE9ibGlnYXRpb25gAAAAAAAQZGVwb3NpdF9kZWNyZWFzZQAAAAsAAABcQW1vdW50IG9mIGBqVG9rZW5zYCB0byBidXJuIHRoYXQgcmVwcmVzZW50IHRoZSBgZGVwb3NpdF9kZWNyZWFzZWRfYW1vdW50YCBhbW91bnQgaW4gdGhlCnBvb2wAAAAQal90b2tlbnNfdG9fYnVybgAAAAsAAABUQW1vdW50IG9mIHRva2VucyB0byByZWNlaXZlIGJ5IHRoZSB3aXRoZHJhd2VyKGBkZXBvc2l0X2RlY3JlYXNlZF9hbW91bnRgIG1pbnVzIGZlZXMpAAAAFXdpdGhkcmF3ZXJfdG9fcmVjZWl2ZQAAAAAAAAs=",
        "AAAAAQAAACRbYE9ibGlnYXRpb246OnJlcGF5YF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAC1JlcGF5UmVzdWx0AAAAAAQAAAA1RXhjZXNzIGFtb3VudCBnaXZlbiBieSB0aGUgYm9ycm93ZXIgdGhhdCBpcyBzZW50IGJhY2sAAAAAAAATYW1vdW50X3RvX3NlbmRfYmFjawAAAAALAAAAAAAAAA1jb21wdXRlZF9mZWVzAAAAAAAH0AAAAAxDb21wdXRlZEZlZXMAAABQQW1vdW50IG9mIGBkVG9rZW5zYCB0byBpc3N1ZSB0aGF0IHJlcHJlc2VudCB0aGUgYHJlYWxfcmVwYWlkYCBhbW91bnQgaW4gdGhlIHBvb2wAAAAQZF90b2tlbnNfdG9fYnVybgAAAAsAAAAhQW1vdW50IG9mIHRoZSBkZWJ0IHRoYXQgaXMgcmVwYWlkAAAAAAAAC2RlYnRfcmVwYWlkAAAAAAs=",
        "AAAAAQAAADBbYE9ibGlnYXRpb246OnJlbW92ZV9jb2xsYXRlcmFsYF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAFlJlbW92ZUNvbGxhdGVyYWxSZXN1bHQAAAAAAAMAAAAjQW1vdW50IG9mIGNvbGxhdGVyYWwgdG9rZW5zIHJlbW92ZWQAAAAAE2NvbGxhdGVyYWxfZGVjcmVhc2UAAAAACwAAAFpBbW91bnQgb2YgY29sbGF0ZXJhbCB0b2tlbnMgcmVjZWl2ZWQgYnkgdGhlIGNvbGxhdGVyYWwgcmVtb3ZlcihhY2NvdW50aW5nIHN1YnRyYWN0ZWQgZmVlcykAAAAAAB1jb2xsYXRlcmFsX3JlbW92ZXJfdG9fcmVjZWl2ZQAAAAAAAAsAAAAAAAAADWNvbXB1dGVkX2ZlZXMAAAAAAAfQAAAADENvbXB1dGVkRmVlcw==",
        "AAAAAQAAAC1bYE9ibGlnYXRpb246OmNvdmVyX2JhZF9kZWJ0YF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAAAAAEkNvdmVyQmFkRGVidFJlc3VsdAAAAAAAAgAAAE1gKHBvb2wgYWRkcmVzcywgYm9ycm93ZXIgZFRva2VucylgIHBhaXJzIGZvciBlYWNoIGJhZCBkZWJ0IG9ibGlnYXRpb24gYm9ycm93cwAAAAAAABlib3Jyb3dzX3RvX2JlX2NvbXBlbnNhdGVkAAAAAAAD6gAAA+0AAAACAAAAEwAAAAsAAABmYChwb29sIGFkZHJlc3MsIGJvcnJvd2VyIGpUb2tlbnMsIGJvcnJvd2VyIGNvbGxhdGVyYWwpYCB0dXBsZXMgZm9yIGVhY2ggYmFkIGRlYnQgb2JsaWdhdGlvbgpjb2xsYXRlcmFsAAAAAAAVY29sbGF0ZXJhbHNfdG9fcmVtb3ZlAAAAAAAD6gAAA+0AAAADAAAAEwAAAAsAAAAL",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAAOAAAASUFtb3VudCBvZiB0b2tlbnMgdGhhdCBjYW4gYmUgd2l0aGRyYXcgYnkgdGhlIGhvc3QgcGxhdGZvcm0gYWRtaW4gYXMgYSBmZWUAAAAAAAAVYWNjdW11bGF0ZWRfaG9zdF9mZWVzAAAAAAAACwAAAEVBbW91bnQgb2YgdG9rZW5zIHRoYXQgY2FuIGJlIHdpdGhkcmF3biBieSB0aGUgbWFya2V0J3MgYWRtaW4gYXMgYSBmZWUAAAAAAAAXYWNjdW11bGF0ZWRfbWFya2V0X2ZlZXMAAAAACwAAAFdBbW91bnQgb2YgdG9rZW5zIGluIHRoZSBpbnN1cmFuY2UgcmVzZXJ2ZSB0aGF0IGNhbiBiZSB1c2VkIHRvIGNvdmVyIGEgYmFkIGRlYnQgc2NlbmFyaW8AAAAAGGFjY3VtdWxhdGVkX3Jlc2VydmVfZmVlcwAAAAsAAAAjQ29uZmlndXJhdGlvbiBzZXR0aW5ncyBmb3IgdGhlIHBvb2wAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAAwVGhlIHRpbWVzdGFtcCBvZiB0aGUgbGFzdCBhY2NydWFsIHJlLWNhbGN1bGF0aW9uAAAAFmxhc3RfYWNjcnVhbF90aW1lc3RhbXAAAAAAAAYAAADsVGhlIHJlc3VsdCBvZiBgVG9rZW5DbGllbnQ6Om5hbWUoJnNlbGYpYCBpbnZvY2F0aW9uOiBgbmF0aXZlYCBzdHJpbmcgZm9yIFhMTSBTQUMgYW5kIHRoZQpTQUMncyBuYXRpdmUgYXNzZXQgY29kZSBhbmQgYXNzZXQgaXNzdWVyIGNvbmNhdGVuYXRlZCB3aXRoIGA6YCBmb3Igb3RoZXIgU0FDcyhlLmcsCiJBUVVBOkdBSFBZV0xLNllSTjdDVllaT080SDNWRFJaN1BWRjVVSkdMWkNTUEFFSUtKRTJYU1dGNUxBR0VSIikAAAAEbmFtZQAAABAAAAAcVGhlIGFkZHJlc3Mgb2YgdGhlIGxvYW4gcG9vbAAAAAxwb29sX2FkZHJlc3MAAAATAAAAOlRoZSBhZGRyZXNzIG9mIHRoZSB0b2tlbiBjb250cmFjdCBhc3NvY2lhdGVkIHdpdGggdGhlIHBvb2wAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAClUaGUgdGlja2VyIHN5bWJvbCBvZiB0aGUgYXNzb2NpYXRlZCB0b2tlbgAAAAAAAAx0b2tlbl90aWNrZXIAAAARAAAAPFRoZSB0b3RhbCBhbW91bnQgb2YgY3VycmVudGx5IGF2YWlsYWJsZSB0b2tlbnMgZm9yIGJvcnJvd2luZwAAAA90b3RhbF9hdmFpbGFibGUAAAAACwAAAFRUaGUgdG90YWwgYW1vdW50IG9mIGJvcnJvd2VkIGFzc2V0cy4gVGhpcyB2YWx1ZSBpbmNyZWFzZXMgd2l0aCBpbnRlcmVzdCByYXRlIGFjY3J1YWwAAAAOdG90YWxfYm9ycm93ZWQAAAAAAAsAAABKVGhlIHRvdGFsIGFtb3VudCBvZiBkZXBvc2l0ZWQgY29sbGF0ZXJhbCBhc3NldHMgdGhhdCBkb24ndCBhY2NydWUgaW50ZXJlc3QAAAAAABB0b3RhbF9jb2xsYXRlcmFsAAAACwAAAFtUaGUgdG90YWwgYGRUb2tlbnNgIGFtb3VudC4gUmVwcmVzZW50cyB0aGUgc3VtIG9mIGFsbCBkZWJ0IHNoYXJlcyBkaXN0cmlidXRlZCBhbW9uZyBkZWJ0b3JzAAAAAA50b3RhbF9kX3Rva2VucwAAAAAACwAAAHVUaGUgdG90YWwgYGpUb2tlbnNgIGFtb3VudC4gUmVwcmVzZW50cyB0aGUgc3VtIG9mIGFsbCB5aWVsZGluZyBpbnRlcmVzdCBjb2xsYXRlcmFsIHNoYXJlcwpkaXN0cmlidXRlZCBhbW9uZyBjcmVkaXRvcnMAAAAAAAAOdG90YWxfal90b2tlbnMAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAADVBvb2xGZWVDb25maWcAAAAAAAAJAAAAAAAAABZhZGRfY29sbGF0ZXJhbF9mZWVfYnBzAAAAAAAEAAAAAAAAAA5ib3Jyb3dfZmVlX2JwcwAAAAAABAAAAAAAAAAPZGVwb3NpdF9mZWVfYnBzAAAAAAQAAAAAAAAAEmZsYXNoX2xvYW5fZmVlX2JwcwAAAAAABAAAAAAAAAAMaG9zdF9mZWVfYnBzAAAABAAAAAAAAAAZcmVtb3ZlX2NvbGxhdGVyYWxfZmVlX2JwcwAAAAAAAAQAAAAAAAAADXJlcGF5X2ZlZV9icHMAAAAAAAAEAAAAAAAAAA10YWtlX3JhdGVfYnBzAAAAAAAABAAAAAAAAAAQd2l0aGRyYXdfZmVlX2JwcwAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAQAAAAAAAAADWFjY3J1YWxfbW9kZWwAAAAAAAfQAAAADEFjY3J1YWxNb2RlbAAAAAAAAAAKZmVlX2NvbmZpZwAAAAAH0AAAAA1Qb29sRmVlQ29uZmlnAAAAAAAAAAAAAA1oZWFsdGhfY29uZmlnAAAAAAAH0AAAABBQb29sSGVhbHRoQ29uZmlnAAAAAAAAABNpbnRlcmVzdF9yYXRlX21vZGVsAAAAB9AAAAARSW50ZXJlc3RSYXRlTW9kZWwAAAA=",
        "AAAAAQAAAAAAAAAAAAAAEFBvb2xIZWFsdGhDb25maWcAAAAHAAAA4lRoZSBtYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYW4gYXNzZXQncyB2YWx1ZSB0aGF0IGNhbiBiZSBoZWxkIGluIGFuIGluZGl2aWR1YWwgb2JsaWdhdGlvbiBpbgpiYXNpcyBwb2ludHMgd2l0aCByZXNwZWN0IHRvIGEgdG90YWwgb2JsaWdhdGlvbidzIGNvbGxhdGVyYWwgdmFsdWUuIExUViBncmVhdGVyIHRoYW4KdGhhdCBtYWtlcyBib3Jyb3cgcG9zaXRpb24gZWxpZ2libGUgdG8gbGlxdWlkYXRpb24AAAAAAA1jbG9zZV9sdHZfYnBzAAAAAAAACwAAAO9UaGUgZmFjdG9yIHVzZWQgdG8gY2FsY3VsYXRlIHRoZSBjdXJyZW50IGJvcnJvdyBsaW1pdCBieSBtdWx0aXBseWluZyB0aGUgY29sbGF0ZXJhbCB2YWx1ZQpieSBpdCBiZWZvcmUgc3VidHJhY3RpbmcgdGhpcyB2YWx1ZSBmcm9tIHRoZSBvYmxpZ2F0aW9uJ3MgbWF4IGJvcnJvdyBsaW1pdC4gVm9sYXRpbGUKYXNzZXRzJyBwb29scyBhcmUgZXhwZWN0ZWQgdG8gaGF2ZSB0aGlzIHZhbHVlIHNldCB3YXkgYWJvdmUgMTAwJQAAAAAUbGlhYmlsaXR5X2ZhY3Rvcl9icHMAAAALAAAAPk1heGltdW0gcGVyY2VudGFnZSBvZiBhIGJvcnJvd2VyJ3MgZGVidCB0aGF0IGNhbiBiZSBsaXF1aWRhdGVkAAAAAAAcbGlxdWlkYXRpb25fY2xvc2VfZmFjdG9yX2JwcwAAAAsAAABDQWRkaXRpb25hbCBkaXNjb3VudCBnaXZlbiB0byBsaXF1aWRhdG9ycyB3aGVuIHB1cmNoYXNpbmcgY29sbGF0ZXJhbAAAAAAZbGlxdWlkYXRpb25faW5jZW50aXZlX2JwcwAAAAAAAAsAAACbVGhlIG1heGltdW0gcGVyY2VudGFnZSBvZiBhbiBhc3NldCdzIHZhbHVlIHRoYXQgY2FuIGJlIGJvcnJvd2VkIGluIGJhc2lzIHBvaW50cyhlLmcsIDcwMDAgPQo3MCUsIGV0Yykgd2l0aCByZXNwZWN0IHRvIGEgdG90YWwgb2JsaWdhdGlvbidzIGNvbGxhdGVyYWwgdmFsdWUAAAAADG9wZW5fbHR2X2JwcwAAAAsAAACHVGhlIG1heGltdW0gYW1vdW50IG9mIHN1cHBsaWVkIHRva2VucyB0aGF0IGNhbiBiZSBzdXBwbGllZCBpbiB0aGUgcG9vbChpLmUuLCBgYXZhaWxhYmxlYCArCmB0b3RhbF9ib3Jyb3dlZGApIDAgZGVub3RlcyB1bmxpbWl0ZWQgc3VwcGx5AAAAAAxzdXBwbHlfbGltaXQAAAALAAAASVRoZSBtYXhpbXVtIHV0aWxpemF0aW9uIHJhdGlvIHRoYXQgaXMgYWxsb3dlZCB0byBiZSByZWFjaGVkIHZpYSBib3Jyb3dpbmcAAAAAAAAbdXRpbGl6YXRpb25fcmF0aW9fbGltaXRfYnBzAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAQAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIZGVwbG95ZXIAAAATAAAAAAAAAARuYW1lAAAAEAAAAAAAAAAGc3RhdHVzAAAAAAAB",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAACQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAEAAAAAAAAABFBvb2wAAAABAAAAEwAAAAEAAAAAAAAACk9ibGlnYXRpb24AAAAAAAEAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxNdWx0aXBseVBhaXIAAAABAAAD7QAAAAIAAAATAAAAEwAAAAAAAAAAAAAAB0FjY3J1YWwAAAAAAAAAAAAAAAAIQWxsUG9vbHMAAAAAAAAAAAAAAA5BbGxPYmxpZ2F0aW9ucwAAAAAAAAAAAAAAAAAQQWxsTXVsdGlwbHlQYWlycwAAAAAAAAAAAAAADU9yYWNsZUFkZHJlc3MAAAA=",
        "AAAAAQAAAC9QcmljZSBkYXRhIGZvciBhbiBhc3NldCBhdCBhIHNwZWNpZmljIHRpbWVzdGFtcAAAAAAAAAAACVByaWNlRGF0YQAAAAAAAAIAAAAAAAAABXByaWNlAAAAAAAACwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAgAAAApBc3NldCB0eXBlAAAAAAAAAAAABUFzc2V0AAAAAAAAAgAAAAEAAAAAAAAAB1N0ZWxsYXIAAAAAAQAAABMAAAABAAAAAAAAAAVPdGhlcgAAAAAAAAEAAAAR" ]),
      options
    )
  }
  public readonly fromJSON = {
    upgrade: this.txFromJSON<null>,
        get_global_state: this.txFromJSON<GlobalState>,
        get_oracle_address: this.txFromJSON<string>,
        initialize_pool: this.txFromJSON<Result<string>>,
        initialize_multiply_pair: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        borrow: this.txFromJSON<Result<void>>,
        swap: this.txFromJSON<Result<i128>>,
        add_collateral: this.txFromJSON<Result<void>>,
        remove_collateral: this.txFromJSON<Result<void>>,
        repay: this.txFromJSON<Result<void>>,
        liquidate: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        flash_loan: this.txFromJSON<Result<void>>,
        clean_multiply_pairs: this.txFromJSON<null>,
        check_multiply_pair_exists: this.txFromJSON<boolean>,
        deposit_with_leverage: this.txFromJSON<Result<void>>,
        withdraw_from_leveraged: this.txFromJSON<Result<void>>,
        redeem_accumulated_market_fees: this.txFromJSON<Result<void>>,
        redeem_accumulated_host_fees: this.txFromJSON<Result<void>>,
        cover_obligation_bad_debt: this.txFromJSON<Result<void>>,
        cover_multiply_pair_bad_debt: this.txFromJSON<Result<void>>,
        get_asset_decimals: this.txFromJSON<u32>,
        get_oracle_price_decimals: this.txFromJSON<u32>,
        get_pool_asset_oracle_price: this.txFromJSON<Result<i128>>,
        get_user_obligation: this.txFromJSON<Result<Obligation>>,
        get_multiply_pair_obligation: this.txFromJSON<Result<Obligation>>,
        get_pool: this.txFromJSON<Result<Pool>>,
        get_all_pools: this.txFromJSON<Array<string>>,
        get_all_obligations: this.txFromJSON<Array<string>>,
        get_multiply_pair: this.txFromJSON<Result<MultiplyPair>>,
        get_all_multiply_pairs: this.txFromJSON<Array<MultiplyPair>>,
        get_apr: this.txFromJSON<Result<AnnualPercentageRates>>,
        get_apy: this.txFromJSON<Result<AnnualPercentageYields>>,
        reset_storage: this.txFromJSON<null>
  }
}