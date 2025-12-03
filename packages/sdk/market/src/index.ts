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
  AssembledTransactionOptions,
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
  1: {message:"NegativeInputAmount"},
  2: {message:"DependencyContractError"},
  3: {message:"MarketIsNotOwned"},
  4: {message:"BorrowForbiddenOnMarket"},
  5: {message:"DepositForbiddenOnMarket"},
  6: {message:"MarketIsFrozen"},
  7: {message:"InvalidMarketUpdate"},
  8: {message:"InvalidMarketStatusUpdate"},
  9: {message:"IncorrectRequestType"},
  10: {message:"OverOrUnderflow"},
  11: {message:"TooManyPositions"},
  12: {message:"MinCollateralValueIsNotMet"},
  100: {message:"PoolAlreadyExists"},
  101: {message:"PoolDoesNotExist"},
  102: {message:"InvalidLoanPoolConfig"},
  103: {message:"NotEnoughPoolFunds"},
  104: {message:"DepositPoolDoesNotExist"},
  105: {message:"BorrowPoolDoesNotExist"},
  106: {message:"CollateralPoolDoesNotExist"},
  107: {message:"PoolAlreadyContainsQueuedInConfigUpdate"},
  108: {message:"PoolDoesNotHaveQueuedInConfigUpdate"},
  109: {message:"PoolConfigUpdateIsNotYetApplicable"},
  110: {message:"BorrowForbiddenOnPool"},
  111: {message:"DepositForbiddenOnPool"},
  112: {message:"PoolIsFrozen"},
  113: {message:"InvalidBootstrapPeriod"},
  200: {message:"ObligationDoesNotExist"},
  201: {message:"DepositPositionDoesNotExist"},
  202: {message:"BorrowPositionDoesNotExist"},
  203: {message:"WithdrawScarcityOverLimit"},
  204: {message:"ScarcityCooldownPeriod"},
  205: {message:"BorrowPositionForAssetExists"},
  206: {message:"DepositPositionForAssetExists"},
  400: {message:"PoolSupplyLimitExceeded"},
  401: {message:"PoolUtilizationRatioCapExceeded"},
  500: {message:"OracleDoesNotKnowAssetPrice"},
  501: {message:"OracleStalePrice"},
  600: {message:"InvalidLiquidationInputs"},
  601: {message:"ObligationIsHealthy"},
  602: {message:"LiquidationExceedsCloseFactor"},
  603: {message:"BadDebtCoverageCriterionIsNotMet"},
  604: {message:"AssetCannotBeUsedAsCollateral"},
  605: {message:"LiquidationExcessiveDemandedCollateral"},
  700: {message:"InvalidLeverageMultiplier"},
  701: {message:"InvalidSwapSlippage"},
  702: {message:"MultiplyPairAlreadyExists"},
  703: {message:"MultiplyPairDoesNotExist"},
  704: {message:"LeveragePositionContainsBadDebt"},
  705: {message:"InconsistentDepositWithLeverage"}
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


/**
 * Represents the pool's plain data with additionally computed info. Intended to be used as a result of simulated read-only
 * invocations
 */
export interface PoolData {
  apy: AnnualPercentageYields;
  d_token_rate_ceil_bps: i128;
  j_token_rate_floor_bps: i128;
  oracle_asset_price: i128;
  pool: Pool;
  total_available_adjusted: i128;
  total_supply: i128;
}


/**
 * Represents the entire market's data(for every pool) with additionally computed info. Intended to be used as a result of simulated read-only
 * invocations
 */
export interface MarketData {
  asset_decimals: u32;
  global_state: GlobalState;
  multiply_pairs: Array<MultiplyPair>;
  oracle_price_decimals: u32;
  pools_data: Array<PoolData>;
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
borrows: Map<string, BorrowPosition>;
  /**
 * Deposited collateral for the obligation, unique by deposit pool address
 */
deposits: Map<string, DepositPosition>;
  /**
 * Count of non-empty positions
 */
positions_count: u32;
}


export interface BorrowPosition {
  /**
 * Amount of the total debt shares that the obligation contains
 */
d_tokens: i128;
  /**
 * Originally borrowed token amount. I.e., if the user borrows 100 tokens and 20 tokens
 * have been accrued with time as additional debt - this value remains 100. If, after that, the user repays the amount
 * that exceeds the debt accrual(like 30) - the value becomes 90. If the user instead borrows 10 tokens, the
 * value increases to 110. In any other case, it doesn't change. Its only purpose is to track the amount
 * of accrued unpaid interest
 */
originally_borrowed: i128;
}


export interface DepositPosition {
  /**
 * Accumulated value of collateral that doesn't accrue interest
 */
collateral: i128;
  /**
 * A share of total supplied tokens in the pool that obligation contains
 */
j_tokens: i128;
  /**
 * Timestamp of when the last scarcity withdraw took place
 */
last_scarcity_withdraw_ts: u64;
  /**
 * Originally deposited token amount. I.e., if the user deposits 100 tokens and 20 tokens
 * have been accrued with time - this value remains 100. If, after that, the user withdraws the amount
 * that exceeds the accrual(like 30) - the value becomes 90 (same goes for when `j_tokens` are seized
 * as collateral by a liquidator. If the user instead deposits 10 tokens, the
 * value increases to 110. In any other case, it doesn't change. Its only purpose is to track the amount
 * of received supply interest
 */
originally_deposited: i128;
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
 * Fee segregated to the protocol host(market deployer)
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
 * Amount of originally deposited tokens(minus all possible fees)
 */
deposited: i128;
  /**
 * Amount of `jTokens` to issue that represent the `deposited` amount in the pool
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
 * Amount of tokens to receive by the borrower(`borrower_new_debt` minus all fees)
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
 * Amount of tokens added as collateral(minus all possible fees)
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
 * Amount of the original deposit(in tokens) that is removed from the `DepositPosition`
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
 * Amount of `dTokens` to burn that represent the `real_repaid` amount in the pool
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


export interface LiquidationResult {
  /**
 * The amount of `dTokens` that are burned from the borrower's borrow position
 */
d_tokens_burned: i128;
  /**
 * The amount of debt tokens repaid by the liquidator
 */
debt_repaid: i128;
  /**
 * The amount of `jTokens` seized from the borrower's obligation and given away to the liquidator's obligation
 * in case the borrower's position doesn't contain enough plain collateral to cover the liquidation expenses
 */
j_tokens_seized: i128;
  /**
 * The amount of plain collateral seized from the borrower's obligation and transferred to the liquidator
 */
plain_collateral_seized: i128;
  /**
 * The amount of tokens representing the `j_tokens_seized` computed via ceiling
 */
tokens_from_j_tokens_seized: i128;
}


export interface Pool {
  /**
 * Amount of tokens that can be withdrawn by the host platform admin as a fee
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
 * Remaining supply bootstrap amounts that are distributed evenly among specified periods
 */
bootstrap_periods: Map<readonly [u64, u64], PoolBootstrapPeriod>;
  /**
 * Borrow annual percentage rate in basis points
 */
borrow_apr_bps: i128;
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
 * Supply annual percentage rate in basis points
 */
supply_apr_bps: i128;
  /**
 * The address of the token contract associated with the pool
 */
token_address: string;
  /**
 * The token symbol of the associated asset
 */
token_symbol: string;
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
  /**
 * Additional scalar (in basis points) used for the additional withdrawal fee when the utilization ratio
 * exceeds `utilization_ratio_limit_bps`
 */
withdraw_scarcity_fee_sc_bps: u32;
}


export interface PoolStatus {
  borrow_enabled: boolean;
  deposit_enabled: boolean;
}


export interface PoolConfig {
  accrual_model: AccrualModel;
  fee_config: PoolFeeConfig;
  health_config: PoolHealthConfig;
  interest_rate_model: InterestRateModel;
  status: PoolStatus;
}


export interface PoolHealthConfig {
  /**
 * The maximum percentage of an asset's value that can be held in an individual obligation in
 * basis points with respect to a total obligation's collateral value. LTV greater than
 * that makes borrow position eligible to liquidation
 */
close_ltv_bps: i128;
  /**
 * LTV calculated for unparameterized obligation positions(i.e., no openLTV/liability factors scaling) that marks
 * position as insolvent. Used as a means to avoid unprofitable health-improving liquidations
 */
insolvency_ltv_bps: i128;
  /**
 * The factor used to calculate the current borrow limit by multiplying the collateral value
 * by it before subtracting this value from the obligation's max borrow limit. Volatile
 * assets' pools are expected to have this value set way above 100%
 */
liability_factor_bps: i128;
  /**
 * Maximum percentage of a borrower's debt that can be liquidated at once
 */
liquidation_close_factor_bps: i128;
  /**
 * Maximum additional value in the received tokens that can be given to liquidators when purchasing collateral
 */
max_liquidation_incentive_bps: i128;
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
  /**
 * Cooldown period(in seconds) required between a pair of sequential withdrawals when the pool's utilization ratio exceeds
 * `utilization_ratio_limit_bps`
 */
withdraw_scarcity_cooldown_s: u64;
  /**
 * Basis points of the pool's total supply that can be withdrawn in a single operation when the pool's utilization ratio exceeds
 * `utilization_ratio_limit_bps`
 */
withdraw_scarcity_limit_bps: i128;
}


export interface PoolBootstrapPeriod {
  /**
 * Remaining bootstrap amount
 */
remaining_amount: i128;
  /**
 * Total provided bootstrap amount
 */
total_amount: i128;
}


/**
 * A request from the submission batch
 */
export interface Request {
  amount: i128;
  pool_address: string;
  request_type: u32;
}

export enum RequestType {
  Deposit = 0,
  Borrow = 1,
  Withdraw = 2,
  Repay = 3,
  AddCollateral = 4,
  RemoveCollateral = 5,
}


export interface GlobalState {
  admin: string;
  deployer: string;
  insolvency_ltv_bps: i128;
  is_owned: boolean;
  max_positions: u32;
  min_collateral_value: i128;
  name: string;
  oracle: string;
  status: u32;
  update_in_queue_period: Option<u64>;
}

export type MarketStatus = {tag: "Active", values: void} | {tag: "BorrowFrozen", values: void} | {tag: "DepositFrozen", values: void} | {tag: "Frozen", values: void};


export interface PoolUpdate {
  new_config: PoolConfig;
  queued_in_timestamp: u64;
}

export type DataKey = {tag: "Name", values: void} | {tag: "Admin", values: void} | {tag: "UpdateInQueuePeriod", values: void} | {tag: "IsOwned", values: void} | {tag: "DeployerHost", values: void} | {tag: "Oracle", values: void} | {tag: "MinCollateralValue", values: void} | {tag: "InsolvencyLtvBps", values: void} | {tag: "MaxPositions", values: void} | {tag: "GlobalState", values: void} | {tag: "Accrual", values: void} | {tag: "AllPools", values: void} | {tag: "AllObligations", values: void} | {tag: "AllMultiplyPairs", values: void} | {tag: "MarketStatus", values: void} | {tag: "ConfigUpdate", values: readonly [string]} | {tag: "Pool", values: readonly [string]} | {tag: "Obligation", values: readonly [ObligationKey]} | {tag: "MultiplyPair", values: readonly [readonly [string, string]]} | {tag: "EarnObligationSeed", values: void};


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
   * # Arguments
   * * `new_wasm_hash` - hash of the WASM binary uploaded to the network that's used as a new
   * version of the contract
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: AssembledTransactionOptions<null>) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a submit_requests_batch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  submit_requests_batch: ({user, requests}: {user: string, requests: Array<Request>}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_global_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_global_state: (options?: AssembledTransactionOptions<GlobalState>) => Promise<AssembledTransaction<GlobalState>>

  /**
   * Construct and simulate a update_market transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_market: ({new_max_positions, new_min_collateral_value}: {new_max_positions: u32, new_min_collateral_value: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_market_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_market_status: ({new_status}: {new_status: u32}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_pool: ({token_address, salt, pool_config}: {token_address: string, salt: Option<Buffer>, pool_config: Option<PoolConfig>}, options?: AssembledTransactionOptions<Result<string>>) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a initialize_multiply_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_multiply_pair: ({deposit_pool_address, borrow_pool_address}: {deposit_pool_address: string, borrow_pool_address: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a queue_in_pool_config_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_pool_config_update: ({pool_address, new_pool_config}: {pool_address: string, new_pool_config: PoolConfig}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel_pool_config_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_pool_config_update: ({pool_address}: {pool_address: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_pool_config_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_pool_config_update: ({pool_address}: {pool_address: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_pool_config_queued_in_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_config_queued_in_update: ({pool_address}: {pool_address: string}, options?: AssembledTransactionOptions<Result<PoolUpdate>>) => Promise<AssembledTransaction<Result<PoolUpdate>>>

  /**
   * Construct and simulate a bootstrap_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  bootstrap_pool: ({pool_address, sponsor, amount, start_period, end_period}: {pool_address: string, sponsor: string, amount: i128, start_period: u64, end_period: u64}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit_into_earn_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_into_earn_obligation: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a borrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  borrow: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap: ({user, token_in, token_out, amount_in}: {user: string, token_in: string, token_out: string, amount_in: i128}, options?: AssembledTransactionOptions<Result<i128>>) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a donate_to_reserve transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  donate_to_reserve: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_collateral: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_collateral: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a repay transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  repay: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a liquidate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  liquidate: ({liquidator, borrower, borrower_obligation_seed, borrow_pool_address, collateral_pool_address, repay_amount, min_demanded_collateral_amount}: {liquidator: string, borrower: string, borrower_obligation_seed: Option<Buffer>, borrow_pool_address: string, collateral_pool_address: string, repay_amount: i128, min_demanded_collateral_amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw_from_earn_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_from_earn_obligation: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a flash_loan transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  flash_loan: ({contract, caller, pool_address, amount}: {contract: string, caller: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit_with_leverage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_with_leverage: ({user, deposit_pool_address, borrow_pool_address, deposit_as_margin, amount, leverage_multiplier}: {user: string, deposit_pool_address: string, borrow_pool_address: string, deposit_as_margin: boolean, amount: i128, leverage_multiplier: u32}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw_from_leveraged transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_from_leveraged: ({user, deposit_pool_address, borrow_pool_address, amount}: {user: string, deposit_pool_address: string, borrow_pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a redeem_accumulated_market_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  redeem_accumulated_market_fees: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a redeem_accumulated_host_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  redeem_accumulated_host_fees: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cover_obligation_bad_debt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cover_obligation_bad_debt: ({bad_debt_obligation_user}: {bad_debt_obligation_user: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cover_multiply_pair_bad_debt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cover_multiply_pair_bad_debt: ({bad_debt_obligation_user, deposit_pool_address, borrow_pool_address}: {bad_debt_obligation_user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_asset_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_asset_decimals: (options?: AssembledTransactionOptions<u32>) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_oracle_price_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_oracle_price_decimals: (options?: AssembledTransactionOptions<u32>) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_pool_asset_oracle_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_asset_oracle_price: ({pool_address}: {pool_address: string}, options?: AssembledTransactionOptions<Result<i128>>) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a get_user_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_obligation: ({user}: {user: string}, options?: AssembledTransactionOptions<Result<Obligation>>) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a refresh_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_obligation: ({user}: {user: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_earn_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_earn_obligation: ({user}: {user: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_multiply_pair_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_multiply_pair_obligation: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_pool: ({pool_address}: {pool_address: string}, options?: AssembledTransactionOptions<Result<void>>) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_earn_user_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_earn_user_obligation: ({user}: {user: string}, options?: AssembledTransactionOptions<Result<Obligation>>) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a get_multiply_pair_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_multiply_pair_obligation: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: AssembledTransactionOptions<Result<Obligation>>) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a get_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool: ({pool_address}: {pool_address: string}, options?: AssembledTransactionOptions<Result<Pool>>) => Promise<AssembledTransaction<Result<Pool>>>

  /**
   * Construct and simulate a get_all_pools transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_pools: (options?: AssembledTransactionOptions<Array<string>>) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_market_data transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_market_data: (options?: AssembledTransactionOptions<Result<MarketData>>) => Promise<AssembledTransaction<Result<MarketData>>>

  /**
   * Construct and simulate a get_all_obligations transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_obligations: (options?: AssembledTransactionOptions<Array<ObligationKey>>) => Promise<AssembledTransaction<Array<ObligationKey>>>

  /**
   * Construct and simulate a get_multiply_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_multiply_pair: ({deposit_pool_address, borrow_pool_address}: {deposit_pool_address: string, borrow_pool_address: string}, options?: AssembledTransactionOptions<Result<MultiplyPair>>) => Promise<AssembledTransaction<Result<MultiplyPair>>>

  /**
   * Construct and simulate a get_all_multiply_pairs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_multiply_pairs: (options?: AssembledTransactionOptions<Array<MultiplyPair>>) => Promise<AssembledTransaction<Array<MultiplyPair>>>

  /**
   * Construct and simulate a get_pool_data transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_data: ({pool_address}: {pool_address: string}, options?: AssembledTransactionOptions<Result<PoolData>>) => Promise<AssembledTransaction<Result<PoolData>>>

  /**
   * Construct and simulate a reset_storage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   * Resets the contract's storage. Useful when the contract's invariants are broken and require
   * resetting on the testnet without re-deploying the contract
   */
  reset_storage: (options?: AssembledTransactionOptions<null>) => Promise<AssembledTransaction<null>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {name, admin, oracle, deployer, max_positions, min_collateral_value, insolvency_ltv_bps, update_in_queue_period}: {name: string, admin: string, oracle: string, deployer: string, max_positions: u32, min_collateral_value: i128, insolvency_ltv_bps: i128, update_in_queue_period: Option<u64>},
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
    return ContractClient.deploy({name, admin, oracle, deployer, max_positions, min_collateral_value, insolvency_ltv_bps, update_in_queue_period}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAAAAAAAAAAAAADEFjY3J1YWxNb2RlbAAAAAEAAAAAAAAAAAAAAApDb21wb3VuZGVkAAA=",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAgAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABm9yYWNsZQAAAAAAEwAAAAAAAAAIZGVwbG95ZXIAAAATAAAAAAAAAA1tYXhfcG9zaXRpb25zAAAAAAAABAAAAAAAAAAUbWluX2NvbGxhdGVyYWxfdmFsdWUAAAALAAAAAAAAABJpbnNvbHZlbmN5X2x0dl9icHMAAAAAAAsAAAAAAAAAFnVwZGF0ZV9pbl9xdWV1ZV9wZXJpb2QAAAAAA+gAAAAGAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAJtVcGdyYWRlcyB0aGUgbGVuZGluZyBjb250cmFjdAoKIyBBcmd1bWVudHMKKiBgbmV3X3dhc21faGFzaGAgLSBoYXNoIG9mIHRoZSBXQVNNIGJpbmFyeSB1cGxvYWRlZCB0byB0aGUgbmV0d29yayB0aGF0J3MgdXNlZCBhcyBhIG5ldwp2ZXJzaW9uIG9mIHRoZSBjb250cmFjdAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAA",
        "AAAAAAAAAAAAAAAVc3VibWl0X3JlcXVlc3RzX2JhdGNoAAAAAAAAAgAAAAAAAAAEdXNlcgAAABMAAAAAAAAACHJlcXVlc3RzAAAD6gAAB9AAAAAHUmVxdWVzdAAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAQZ2V0X2dsb2JhbF9zdGF0ZQAAAAAAAAABAAAH0AAAAAtHbG9iYWxTdGF0ZQA=",
        "AAAAAAAAAAAAAAANdXBkYXRlX21hcmtldAAAAAAAAAIAAAAAAAAAEW5ld19tYXhfcG9zaXRpb25zAAAAAAAABAAAAAAAAAAYbmV3X21pbl9jb2xsYXRlcmFsX3ZhbHVlAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAUdXBkYXRlX21hcmtldF9zdGF0dXMAAAABAAAAAAAAAApuZXdfc3RhdHVzAAAAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAPaW5pdGlhbGl6ZV9wb29sAAAAAAMAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAARzYWx0AAAD6AAAA+4AAAAgAAAAAAAAAAtwb29sX2NvbmZpZwAAAAPoAAAH0AAAAApQb29sQ29uZmlnAAAAAAABAAAD6QAAABMAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAYaW5pdGlhbGl6ZV9tdWx0aXBseV9wYWlyAAAAAgAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAbcXVldWVfaW5fcG9vbF9jb25maWdfdXBkYXRlAAAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAAD25ld19wb29sX2NvbmZpZwAAAAfQAAAAClBvb2xDb25maWcAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAZY2FuY2VsX3Bvb2xfY29uZmlnX3VwZGF0ZQAAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAYYXBwbHlfcG9vbF9jb25maWdfdXBkYXRlAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAgZ2V0X3Bvb2xfY29uZmlnX3F1ZXVlZF9pbl91cGRhdGUAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAAClBvb2xVcGRhdGUAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAOYm9vdHN0cmFwX3Bvb2wAAAAAAAUAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAAB3Nwb25zb3IAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAxzdGFydF9wZXJpb2QAAAAGAAAAAAAAAAplbmRfcGVyaW9kAAAAAAAGAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAcZGVwb3NpdF9pbnRvX2Vhcm5fb2JsaWdhdGlvbgAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAGYm9ycm93AAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAEc3dhcAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAh0b2tlbl9pbgAAABMAAAAAAAAACXRva2VuX291dAAAAAAAABMAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAARZG9uYXRlX3RvX3Jlc2VydmUAAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAOYWRkX2NvbGxhdGVyYWwAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAARcmVtb3ZlX2NvbGxhdGVyYWwAAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAFcmVwYXkAAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAJbGlxdWlkYXRlAAAAAAAABwAAAAAAAAAKbGlxdWlkYXRvcgAAAAAAEwAAAAAAAAAIYm9ycm93ZXIAAAATAAAAAAAAABhib3Jyb3dlcl9vYmxpZ2F0aW9uX3NlZWQAAAPoAAAD7gAAACAAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAXY29sbGF0ZXJhbF9wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAMcmVwYXlfYW1vdW50AAAACwAAAAAAAAAebWluX2RlbWFuZGVkX2NvbGxhdGVyYWxfYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAId2l0aGRyYXcAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAdd2l0aGRyYXdfZnJvbV9lYXJuX29ibGlnYXRpb24AAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAKZmxhc2hfbG9hbgAAAAAABAAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAVZGVwb3NpdF93aXRoX2xldmVyYWdlAAAAAAAABgAAAAAAAAAEdXNlcgAAABMAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAAAAABFkZXBvc2l0X2FzX21hcmdpbgAAAAAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAATbGV2ZXJhZ2VfbXVsdGlwbGllcgAAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAXd2l0aGRyYXdfZnJvbV9sZXZlcmFnZWQAAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAecmVkZWVtX2FjY3VtdWxhdGVkX21hcmtldF9mZWVzAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAccmVkZWVtX2FjY3VtdWxhdGVkX2hvc3RfZmVlcwAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAZY292ZXJfb2JsaWdhdGlvbl9iYWRfZGVidAAAAAAAAAEAAAAAAAAAGGJhZF9kZWJ0X29ibGlnYXRpb25fdXNlcgAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAcY292ZXJfbXVsdGlwbHlfcGFpcl9iYWRfZGVidAAAAAMAAAAAAAAAGGJhZF9kZWJ0X29ibGlnYXRpb25fdXNlcgAAABMAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAASZ2V0X2Fzc2V0X2RlY2ltYWxzAAAAAAAAAAAAAQAAAAQ=",
        "AAAAAAAAAAAAAAAZZ2V0X29yYWNsZV9wcmljZV9kZWNpbWFscwAAAAAAAAAAAAABAAAABA==",
        "AAAAAAAAAAAAAAAbZ2V0X3Bvb2xfYXNzZXRfb3JhY2xlX3ByaWNlAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAAAsAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAATZ2V0X3VzZXJfb2JsaWdhdGlvbgAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAScmVmcmVzaF9vYmxpZ2F0aW9uAAAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAXcmVmcmVzaF9lYXJuX29ibGlnYXRpb24AAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAgcmVmcmVzaF9tdWx0aXBseV9wYWlyX29ibGlnYXRpb24AAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAMcmVmcmVzaF9wb29sAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAYZ2V0X2Vhcm5fdXNlcl9vYmxpZ2F0aW9uAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAD6QAAB9AAAAAKT2JsaWdhdGlvbgAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAcZ2V0X211bHRpcGx5X3BhaXJfb2JsaWdhdGlvbgAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAPpAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAIZ2V0X3Bvb2wAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAABFBvb2wAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAANZ2V0X2FsbF9wb29scwAAAAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAAAAAAAAPZ2V0X21hcmtldF9kYXRhAAAAAAAAAAABAAAD6QAAB9AAAAAKTWFya2V0RGF0YQAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAATZ2V0X2FsbF9vYmxpZ2F0aW9ucwAAAAAAAAAAAQAAA+oAAAfQAAAADU9ibGlnYXRpb25LZXkAAAA=",
        "AAAAAAAAAAAAAAARZ2V0X211bHRpcGx5X3BhaXIAAAAAAAACAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAPpAAAH0AAAAAxNdWx0aXBseVBhaXIAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAWZ2V0X2FsbF9tdWx0aXBseV9wYWlycwAAAAAAAAAAAAEAAAPqAAAH0AAAAAxNdWx0aXBseVBhaXI=",
        "AAAAAAAAAAAAAAANZ2V0X3Bvb2xfZGF0YQAAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAAIUG9vbERhdGEAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAJZSZXNldHMgdGhlIGNvbnRyYWN0J3Mgc3RvcmFnZS4gVXNlZnVsIHdoZW4gdGhlIGNvbnRyYWN0J3MgaW52YXJpYW50cyBhcmUgYnJva2VuIGFuZCByZXF1aXJlCnJlc2V0dGluZyBvbiB0aGUgdGVzdG5ldCB3aXRob3V0IHJlLWRlcGxveWluZyB0aGUgY29udHJhY3QAAAAAAA1yZXNldF9zdG9yYWdlAAAAAAAAAAAAAAA=",
        "AAAABAAAAAAAAAAAAAAAB01DRXJyb3IAAAAAMgAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAE05lZ2F0aXZlSW5wdXRBbW91bnQAAAAAAQAAAAAAAAAXRGVwZW5kZW5jeUNvbnRyYWN0RXJyb3IAAAAAAgAAAAAAAAAQTWFya2V0SXNOb3RPd25lZAAAAAMAAAAAAAAAF0JvcnJvd0ZvcmJpZGRlbk9uTWFya2V0AAAAAAQAAAAAAAAAGERlcG9zaXRGb3JiaWRkZW5Pbk1hcmtldAAAAAUAAAAAAAAADk1hcmtldElzRnJvemVuAAAAAAAGAAAAAAAAABNJbnZhbGlkTWFya2V0VXBkYXRlAAAAAAcAAAAAAAAAGUludmFsaWRNYXJrZXRTdGF0dXNVcGRhdGUAAAAAAAAIAAAAAAAAABRJbmNvcnJlY3RSZXF1ZXN0VHlwZQAAAAkAAAAAAAAAD092ZXJPclVuZGVyZmxvdwAAAAAKAAAAAAAAABBUb29NYW55UG9zaXRpb25zAAAACwAAAAAAAAAaTWluQ29sbGF0ZXJhbFZhbHVlSXNOb3RNZXQAAAAAAAwAAAAAAAAAEVBvb2xBbHJlYWR5RXhpc3RzAAAAAAAAZAAAAAAAAAAQUG9vbERvZXNOb3RFeGlzdAAAAGUAAAAAAAAAFUludmFsaWRMb2FuUG9vbENvbmZpZwAAAAAAAGYAAAAAAAAAEk5vdEVub3VnaFBvb2xGdW5kcwAAAAAAZwAAAAAAAAAXRGVwb3NpdFBvb2xEb2VzTm90RXhpc3QAAAAAaAAAAAAAAAAWQm9ycm93UG9vbERvZXNOb3RFeGlzdAAAAAAAaQAAAAAAAAAaQ29sbGF0ZXJhbFBvb2xEb2VzTm90RXhpc3QAAAAAAGoAAAAAAAAAJ1Bvb2xBbHJlYWR5Q29udGFpbnNRdWV1ZWRJbkNvbmZpZ1VwZGF0ZQAAAABrAAAAAAAAACNQb29sRG9lc05vdEhhdmVRdWV1ZWRJbkNvbmZpZ1VwZGF0ZQAAAABsAAAAAAAAACJQb29sQ29uZmlnVXBkYXRlSXNOb3RZZXRBcHBsaWNhYmxlAAAAAABtAAAAAAAAABVCb3Jyb3dGb3JiaWRkZW5PblBvb2wAAAAAAABuAAAAAAAAABZEZXBvc2l0Rm9yYmlkZGVuT25Qb29sAAAAAABvAAAAAAAAAAxQb29sSXNGcm96ZW4AAABwAAAAAAAAABZJbnZhbGlkQm9vdHN0cmFwUGVyaW9kAAAAAABxAAAAAAAAABZPYmxpZ2F0aW9uRG9lc05vdEV4aXN0AAAAAADIAAAAAAAAABtEZXBvc2l0UG9zaXRpb25Eb2VzTm90RXhpc3QAAAAAyQAAAAAAAAAaQm9ycm93UG9zaXRpb25Eb2VzTm90RXhpc3QAAAAAAMoAAAAAAAAAGVdpdGhkcmF3U2NhcmNpdHlPdmVyTGltaXQAAAAAAADLAAAAAAAAABZTY2FyY2l0eUNvb2xkb3duUGVyaW9kAAAAAADMAAAAAAAAABxCb3Jyb3dQb3NpdGlvbkZvckFzc2V0RXhpc3RzAAAAzQAAAAAAAAAdRGVwb3NpdFBvc2l0aW9uRm9yQXNzZXRFeGlzdHMAAAAAAADOAAAAAAAAABdQb29sU3VwcGx5TGltaXRFeGNlZWRlZAAAAAGQAAAAAAAAAB9Qb29sVXRpbGl6YXRpb25SYXRpb0NhcEV4Y2VlZGVkAAAAAZEAAAAAAAAAG09yYWNsZURvZXNOb3RLbm93QXNzZXRQcmljZQAAAAH0AAAAAAAAABBPcmFjbGVTdGFsZVByaWNlAAAB9QAAAAAAAAAYSW52YWxpZExpcXVpZGF0aW9uSW5wdXRzAAACWAAAAAAAAAATT2JsaWdhdGlvbklzSGVhbHRoeQAAAAJZAAAAAAAAAB1MaXF1aWRhdGlvbkV4Y2VlZHNDbG9zZUZhY3RvcgAAAAAAAloAAAAAAAAAIEJhZERlYnRDb3ZlcmFnZUNyaXRlcmlvbklzTm90TWV0AAACWwAAAAAAAAAdQXNzZXRDYW5ub3RCZVVzZWRBc0NvbGxhdGVyYWwAAAAAAAJcAAAAAAAAACZMaXF1aWRhdGlvbkV4Y2Vzc2l2ZURlbWFuZGVkQ29sbGF0ZXJhbAAAAAACXQAAAAAAAAAZSW52YWxpZExldmVyYWdlTXVsdGlwbGllcgAAAAAAArwAAAAAAAAAE0ludmFsaWRTd2FwU2xpcHBhZ2UAAAACvQAAAAAAAAAZTXVsdGlwbHlQYWlyQWxyZWFkeUV4aXN0cwAAAAAAAr4AAAAAAAAAGE11bHRpcGx5UGFpckRvZXNOb3RFeGlzdAAAAr8AAAAAAAAAH0xldmVyYWdlUG9zaXRpb25Db250YWluc0JhZERlYnQAAAACwAAAAAAAAAAfSW5jb25zaXN0ZW50RGVwb3NpdFdpdGhMZXZlcmFnZQAAAALB",
        "AAAABQAAAAAAAAAAAAAAE0luaXRpYWxpemVQb29sRXZlbnQAAAAAAQAAABVpbml0aWFsaXplX3Bvb2xfZXZlbnQAAAAAAAADAAAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAAx0b2tlbl9zeW1ib2wAAAAQAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAG0luaXRpYWxpemVNdWx0aXBseVBhaXJFdmVudAAAAAABAAAAHmluaXRpYWxpemVfbXVsdGlwbHlfcGFpcl9ldmVudAAAAAAAAgAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAF1F1ZXVlSW5Qb29sQ29uZmlnVXBkYXRlAAAAAAEAAAAbcXVldWVfaW5fcG9vbF9jb25maWdfdXBkYXRlAAAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAAtwb29sX2NvbmZpZwAAAAfQAAAAClBvb2xDb25maWcAAAAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAFkNhbmNlbFBvb2xDb25maWdVcGRhdGUAAAAAAAEAAAAZY2FuY2VsX3Bvb2xfY29uZmlnX3VwZGF0ZQAAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAFUFwcGx5UG9vbENvbmZpZ1VwZGF0ZQAAAAAAAAEAAAAYYXBwbHlfcG9vbF9jb25maWdfdXBkYXRlAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAEkJvb3RzdHJhcFBvb2xFdmVudAAAAAAAAQAAABRib290c3RyYXBfcG9vbF9ldmVudAAAAAQAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAAdzcG9uc29yAAAAABMAAAABAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAAAAAAAAZwZXJpb2QAAAAAA+0AAAACAAAABgAAAAYAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADERlcG9zaXRFdmVudAAAAAEAAAANZGVwb3NpdF9ldmVudAAAAAAAAAMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAA5vYmxpZ2F0aW9uX2tleQAAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAAOZGVwb3NpdF9yZXN1bHQAAAAAB9AAAAANRGVwb3NpdFJlc3VsdAAAAAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAACVN3YXBFdmVudAAAAAAAAAEAAAAKc3dhcF9ldmVudAAAAAAABgAAAAAAAAAEdXNlcgAAABMAAAABAAAAAAAAAAh0b2tlbl9pbgAAABMAAAABAAAAAAAAAAl0b2tlbl9vdXQAAAAAAAATAAAAAQAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAAAAAAAAAAACmFtb3VudF9vdXQAAAAAAAsAAAAAAAAAAAAAAA9yZWNlaXZlZF9hbW91bnQAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAC0JvcnJvd0V2ZW50AAAAAAEAAAAMYm9ycm93X2V2ZW50AAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAA1ib3Jyb3dfcmVzdWx0AAAAAAAH0AAAAAxCb3Jyb3dSZXN1bHQAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAEkFkZENvbGxhdGVyYWxFdmVudAAAAAAAAQAAABRhZGRfY29sbGF0ZXJhbF9ldmVudAAAAAMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAA5vYmxpZ2F0aW9uX2tleQAAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAAVYWRkX2NvbGxhdGVyYWxfcmVzdWx0AAAAAAAH0AAAABNBZGRDb2xsYXRlcmFsUmVzdWx0AAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAClJlcGF5RXZlbnQAAAAAAAEAAAALcmVwYXlfZXZlbnQAAAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxyZXBheV9yZXN1bHQAAAfQAAAAC1JlcGF5UmVzdWx0AAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAADkxpcXVpZGF0ZUV2ZW50AAAAAAABAAAAD2xpcXVpZGF0ZV9ldmVudAAAAAAFAAAAAAAAAApsaXF1aWRhdG9yAAAAAAATAAAAAQAAAAAAAAAXYm9ycm93ZXJfb2JsaWdhdGlvbl9rZXkAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAQAAAAAAAAAXY29sbGF0ZXJhbF9wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAAAAAAAEmxpcXVpZGF0aW9uX3Jlc3VsdAAAAAAH0AAAABFMaXF1aWRhdGlvblJlc3VsdAAAAAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAFVJlbW92ZUNvbGxhdGVyYWxFdmVudAAAAAAAAAEAAAAXcmVtb3ZlX2NvbGxhdGVyYWxfZXZlbnQAAAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAABhyZW1vdmVfY29sbGF0ZXJhbF9yZXN1bHQAAAfQAAAAFlJlbW92ZUNvbGxhdGVyYWxSZXN1bHQAAAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAADVdpdGhkcmF3RXZlbnQAAAAAAAABAAAADndpdGhkcmF3X2V2ZW50AAAAAAADAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAOb2JsaWdhdGlvbl9rZXkAAAAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAEAAAAAAAAAD3dpdGhkcmF3X3Jlc3VsdAAAAAfQAAAADldpdGhkcmF3UmVzdWx0AAAAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADkZsYXNoTG9hbkV2ZW50AAAAAAABAAAAEGZsYXNoX2xvYW5fZXZlbnQAAAAEAAAAAAAAAAhjb250cmFjdAAAABMAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAAAAAAJZmVlc19wYWlkAAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAGERlcG9zaXRXaXRoTGV2ZXJhZ2VFdmVudAAAAAEAAAAbZGVwb3NpdF93aXRoX2xldmVyYWdlX2V2ZW50AAAAAAcAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAABAAAAAAAAAA9vcmlnaW5hbF9hbW91bnQAAAAACwAAAAAAAAAAAAAAE2xldmVyYWdlX211bHRpcGxpZXIAAAAABAAAAAAAAAAAAAAAFnRvdGFsX2RlcG9zaXRlZF9hbW91bnQAAAAAAAsAAAAAAAAAAAAAABV0b3RhbF9ib3Jyb3dlZF9hbW91bnQAAAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAGldpdGhkcmF3RnJvbUxldmVyYWdlZEV2ZW50AAAAAAABAAAAHXdpdGhkcmF3X2Zyb21fbGV2ZXJhZ2VkX2V2ZW50AAAAAAAABgAAAAAAAAAOb2JsaWdhdGlvbl9rZXkAAAAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAEAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAAAAAAAGndpdGhkcmF3bl90b193YWxsZXRfYW1vdW50AAAAAAALAAAAAAAAAAAAAAAWZGVwb3NpdF9yZWR1Y2VkX2Ftb3VudAAAAAAACwAAAAAAAAAAAAAAFWJvcnJvd19yZWR1Y2VkX2Ftb3VudAAAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAE0FjY3J1ZUludGVyZXN0RXZlbnQAAAAAAQAAABVhY2NydWVfaW50ZXJlc3RfZXZlbnQAAAAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAFExlZGdlclRpbWVzdGFtcEVycm9yAAAAAQAAABZsZWRnZXJfdGltZXN0YW1wX2Vycm9yAAAAAAACAAAAAAAAABFjdXJyZW50X3RpbWVzdGFtcAAAAAAAAAYAAAAAAAAAAAAAABBzdG9yZWRfdGltZXN0YW1wAAAABgAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAGExldmVyYWdlZFBvc2l0aW9uQmFkRGVidAAAAAEAAAAbbGV2ZXJhZ2VkX3Bvc2l0aW9uX2JhZF9kZWJ0AAAAAAYAAAAAAAAABHVzZXIAAAATAAAAAQAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAQAAAAAAAAAQZGVwb3NpdGVkX2Ftb3VudAAAAAsAAAAAAAAAAAAAAA9ib3Jyb3dlZF9hbW91bnQAAAAACwAAAAAAAAAAAAAAGGRlcG9zaXRlZF9hbW91bnRfc3dhcHBlZAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHUxldmVyYWdlRXhjZWVkc0JvcnJvd0NhcGFjaXR5AAAAAAAAAQAAACBsZXZlcmFnZV9leGNlZWRzX2JvcnJvd19jYXBhY2l0eQAAAAQAAAAAAAAABHVzZXIAAAATAAAAAQAAAAAAAAATZmxhc2hfYm9ycm93X2Ftb3VudAAAAAALAAAAAQAAAAAAAAASZmxhc2hfcmVwYXlfYW1vdW50AAAAAAALAAAAAAAAAAAAAAAZbWF4X2hlYWx0aHlfYm9ycm93X2Ftb3VudAAAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHFV0aWxpemF0aW9uUmF0aW9FeGNlZWRzTGltaXQAAAABAAAAH3V0aWxpemF0aW9uX3JhdGlvX2V4Y2VlZHNfbGltaXQAAAAAAgAAAAAAAAAVdXRpbGl6YXRpb25fcmF0aW9fYnBzAAAAAAAACwAAAAAAAAAAAAAAG3V0aWxpemF0aW9uX3JhdGlvX2xpbWl0X2JwcwAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAFlBvb2xJc01pc3NpbmdJblN0b3JhZ2UAAAAAAAEAAAAacG9vbF9pc19taXNzaW5nX2luX3N0b3JhZ2UAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHE9ibGlnYXRpb25Jc01pc3NpbmdJblN0b3JhZ2UAAAABAAAAIG9ibGlnYXRpb25faXNfbWlzc2luZ19pbl9zdG9yYWdlAAAAAQAAAAAAAAAOb2JsaWdhdGlvbl9rZXkAAAAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAHU9ibGlnYXRpb25BbW50QmVjb21lc05lZ2F0aXZlAAAAAAAAAQAAACBvYmxpZ2F0aW9uX2FtbnRfYmVjb21lc19uZWdhdGl2ZQAAAAIAAAAAAAAACm9sZF9hbW91bnQAAAAAAAsAAAAAAAAAAAAAAApuZXdfYW1vdW50AAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAGVBvb2xBbW91bnRCZWNvbWVzTmVnYXRpdmUAAAAAAAABAAAAHHBvb2xfYW1vdW50X2JlY29tZXNfbmVnYXRpdmUAAAACAAAAAAAAAApvbGRfYW1vdW50AAAAAAALAAAAAAAAAAAAAAAKbmV3X2Ftb3VudAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAG1Bvb2xJbmNvbnNpc3RlbnRUb3RhbFNoYXJlcwAAAAABAAAAHnBvb2xfaW5jb25zaXN0ZW50X3RvdGFsX3NoYXJlcwAAAAAAAgAAAAAAAAAMdG90YWxfc2hhcmVzAAAACwAAAAAAAAAAAAAAEWluZGl2aWR1YWxfc2hhcmVzAAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAG1Bvb2xJbmNvbnNpc3RlbnRUb3RhbFRva2VucwAAAAABAAAAHnBvb2xfaW5jb25zaXN0ZW50X3RvdGFsX3Rva2VucwAAAAAAAgAAAAAAAAAMdG90YWxfc2hhcmVzAAAACwAAAAAAAAAAAAAADHRvdGFsX3Rva2VucwAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHVBvb2xDb250YWluc0luY29uc2lzdGVudFN0YXRlAAAAAAAAAQAAACBwb29sX2NvbnRhaW5zX2luY29uc2lzdGVudF9zdGF0ZQAAAAEAAAAAAAAABHBvb2wAAAfQAAAABFBvb2wAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHU9ibGlnYXRpb25Jc1VuZXhwZWN0ZWRseUVtcHR5AAAAAAAAAQAAACBvYmxpZ2F0aW9uX2lzX3VuZXhwZWN0ZWRseV9lbXB0eQAAAAIAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAGkNvbXB1dGVkSW50ZXJlc3RJc05lZ2F0aXZlAAAAAAABAAAAHWNvbXB1dGVkX2ludGVyZXN0X2lzX25lZ2F0aXZlAAAAAAAABAAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAD3Bvc2l0aW9uX3NoYXJlcwAAAAALAAAAAAAAAAAAAAAXdG9rZW5zX2Zyb21fc2hhcmVzX2NlaWwAAAAACwAAAAAAAAAAAAAAEWNvbXB1dGVkX2ludGVyZXN0AAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAHVBvc2l0aW9uc0NvdW50QmVjb21lc05lZ2F0aXZlAAAAAAAAAQAAACBwb3NpdGlvbnNfY291bnRfYmVjb21lc19uZWdhdGl2ZQAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAApvYmxpZ2F0aW9uAAAAAAfQAAAACk9ibGlnYXRpb24AAAAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAHFJlY2VpdmVkVW5leHBlY3RlZFN3YXBBbW91bnQAAAABAAAAH3JlY2VpdmVkX3VuZXhwZWN0ZWRfc3dhcF9hbW91bnQAAAAABwAAAAAAAAAEdXNlcgAAABMAAAABAAAAAAAAAAh0b2tlbl9pbgAAABMAAAABAAAAAAAAAAl0b2tlbl9vdXQAAAAAAAATAAAAAQAAAAAAAAAJYW1vdW50X2luAAAAAAAACwAAAAAAAAAAAAAACmFtb3VudF9vdXQAAAAAAAsAAAAAAAAAAAAAABJleHBlY3RlZF9hbW91bnRfaW4AAAAAAAsAAAAAAAAAAAAAABNleHBlY3RlZF9hbW91bnRfb3V0AAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAACERiZ0V2ZW50AAAAAQAAAAlkYmdfZXZlbnQAAAAAAAABAAAAAAAAAAZzeW1ib2wAAAAAABEAAAABAAAAAg==",
        "AAAAAQAAADNDb21wb3VuZCBpbnRlcmVzdCByYXRlcyByZXByZXNlbnRlZCBpbiBiYXNpcyBwb2ludHMAAAAAAAAAABZBbm51YWxQZXJjZW50YWdlWWllbGRzAAAAAAACAAAAAAAAAApib3Jyb3dfYnBzAAAAAAAEAAAAAAAAAApzdXBwbHlfYnBzAAAAAAAE",
        "AAAAAgAAAAAAAAAAAAAAEUludGVyZXN0UmF0ZU1vZGVsAAAAAAAAAQAAAAEAAAAAAAAABktpbmtlZAAAAAAAAQAAB9AAAAAOS2lua2VkSVJDb25maWcAAA==",
        "AAAAAQAAAAAAAAAAAAAADktpbmtlZElSQ29uZmlnAAAAAAAGAAAARkJhc2UgQVBSIHRoYXQgaXMgYWNjcnVlZCByZWdhcmRsZXNzIG9mIHRoZSB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wAAAAAAAxiYXNlX2Fwcl9icHMAAAALAAAARUFQUiB0aGF0IGlzIGFjY3J1ZWQgd2hlbiB0aGUgdXRpbGl6YXRpb24gcmF0aW8gaXMgYXQgdGhlIGtpbmsgMSB2YWx1ZQAAAAAAAA1raW5rMV9hcHJfYnBzAAAAAAAACwAAABhLaW5rIDEgdXRpbGl6YXRpb24gcmF0aW8AAAAMa2luazFfdXJfYnBzAAAACwAAAEVBUFIgdGhhdCBpcyBhY2NydWVkIHdoZW4gdGhlIHV0aWxpemF0aW9uIHJhdGlvIGlzIGF0IHRoZSBraW5rIDIgdmFsdWUAAAAAAAANa2luazJfYXByX2JwcwAAAAAAAAsAAAAYS2luayAyIHV0aWxpemF0aW9uIHJhdGlvAAAADGtpbmsyX3VyX2JwcwAAAAsAAAA5QVBSIHRoYXQgaXMgYWNjcnVlZCB3aGVuIHRoZSB1dGlsaXphdGlvbiByYXRpbyBpcyBhdCAxMDAlAAAAAAAAC21heF9hcHJfYnBzAAAAAAs=",
        "AAAAAQAAAIRSZXByZXNlbnRzIHRoZSBwb29sJ3MgcGxhaW4gZGF0YSB3aXRoIGFkZGl0aW9uYWxseSBjb21wdXRlZCBpbmZvLiBJbnRlbmRlZCB0byBiZSB1c2VkIGFzIGEgcmVzdWx0IG9mIHNpbXVsYXRlZCByZWFkLW9ubHkKaW52b2NhdGlvbnMAAAAAAAAACFBvb2xEYXRhAAAABwAAAAAAAAADYXB5AAAAB9AAAAAWQW5udWFsUGVyY2VudGFnZVlpZWxkcwAAAAAAAAAAABVkX3Rva2VuX3JhdGVfY2VpbF9icHMAAAAAAAALAAAAAAAAABZqX3Rva2VuX3JhdGVfZmxvb3JfYnBzAAAAAAALAAAAAAAAABJvcmFjbGVfYXNzZXRfcHJpY2UAAAAAAAsAAAAAAAAABHBvb2wAAAfQAAAABFBvb2wAAAAAAAAAGHRvdGFsX2F2YWlsYWJsZV9hZGp1c3RlZAAAAAsAAAAAAAAADHRvdGFsX3N1cHBseQAAAAs=",
        "AAAAAQAAAJdSZXByZXNlbnRzIHRoZSBlbnRpcmUgbWFya2V0J3MgZGF0YShmb3IgZXZlcnkgcG9vbCkgd2l0aCBhZGRpdGlvbmFsbHkgY29tcHV0ZWQgaW5mby4gSW50ZW5kZWQgdG8gYmUgdXNlZCBhcyBhIHJlc3VsdCBvZiBzaW11bGF0ZWQgcmVhZC1vbmx5Cmludm9jYXRpb25zAAAAAAAAAAAKTWFya2V0RGF0YQAAAAAABQAAAAAAAAAOYXNzZXRfZGVjaW1hbHMAAAAAAAQAAAAAAAAADGdsb2JhbF9zdGF0ZQAAB9AAAAALR2xvYmFsU3RhdGUAAAAAAAAAAA5tdWx0aXBseV9wYWlycwAAAAAD6gAAB9AAAAAMTXVsdGlwbHlQYWlyAAAAAAAAABVvcmFjbGVfcHJpY2VfZGVjaW1hbHMAAAAAAAAEAAAAAAAAAApwb29sc19kYXRhAAAAAAPqAAAH0AAAAAhQb29sRGF0YQ==",
        "AAAAAQAAAAAAAAAAAAAADE11bHRpcGx5UGFpcgAAAAQAAAAyQWRkcmVzcyBvZiBhIHBvb2wgaW4gYSBwYWlyIGZvciBhIGxldmVyYWdlZCBib3Jyb3cAAAAAAAtib3Jyb3dfcG9vbAAAAAATAAAAM0FkZHJlc3Mgb2YgYSBwb29sIGluIGEgcGFpciBmb3IgYSBsZXZlcmFnZWQgZGVwb3NpdAAAAAAMZGVwb3NpdF9wb29sAAAAEwAAAF5NYXhpbXVtIGxldmVyYWdlIG11bHRpcGxpZXIgYmFzZWQgb24gYm9ycm93IHBvb2wgb3BlbkxUViB2YWx1ZS4gU2NhbGVkIHdpdGgKW2BMRVZFUkFHRV9TQ0FMRWBdAAAAAAAXbWF4X2xldmVyYWdlX211bHRpcGxpZXIAAAAABAAAAHREZXRlcm1pbmlzdGljYWxseSBjb21wdXRlZCB1bmlxdWUgc2VlZCBwZXIgYSBwYWlyLCB1c2VkIHRvIGRpc3Rpbmd1aXNoIGEgdXNlcidzIG11bHRpcGx5CnBhaXIgb2JsaWdhdGlvbiBmcm9tIG90aGVycwAAAARzZWVkAAAD7gAAACA=",
        "AAAAAQAAAAAAAAAAAAAADU9ibGlnYXRpb25LZXkAAAAAAAACAAAAAAAAAARzZWVkAAAD6AAAA+4AAAAgAAAAAAAAAAR1c2VyAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAMAAABEQm9ycm93ZWQgbGlxdWlkaXR5IGZvciB0aGUgb2JsaWdhdGlvbiwgdW5pcXVlIGJ5IGJvcnJvdyBwb29sIGFkZHJlc3MAAAAHYm9ycm93cwAAAAPsAAAAEwAAB9AAAAAOQm9ycm93UG9zaXRpb24AAAAAAEdEZXBvc2l0ZWQgY29sbGF0ZXJhbCBmb3IgdGhlIG9ibGlnYXRpb24sIHVuaXF1ZSBieSBkZXBvc2l0IHBvb2wgYWRkcmVzcwAAAAAIZGVwb3NpdHMAAAPsAAAAEwAAB9AAAAAPRGVwb3NpdFBvc2l0aW9uAAAAABxDb3VudCBvZiBub24tZW1wdHkgcG9zaXRpb25zAAAAD3Bvc2l0aW9uc19jb3VudAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAADkJvcnJvd1Bvc2l0aW9uAAAAAAACAAAAPEFtb3VudCBvZiB0aGUgdG90YWwgZGVidCBzaGFyZXMgdGhhdCB0aGUgb2JsaWdhdGlvbiBjb250YWlucwAAAAhkX3Rva2VucwAAAAsAAAGzT3JpZ2luYWxseSBib3Jyb3dlZCB0b2tlbiBhbW91bnQuIEkuZS4sIGlmIHRoZSB1c2VyIGJvcnJvd3MgMTAwIHRva2VucyBhbmQgMjAgdG9rZW5zCmhhdmUgYmVlbiBhY2NydWVkIHdpdGggdGltZSBhcyBhZGRpdGlvbmFsIGRlYnQgLSB0aGlzIHZhbHVlIHJlbWFpbnMgMTAwLiBJZiwgYWZ0ZXIgdGhhdCwgdGhlIHVzZXIgcmVwYXlzIHRoZSBhbW91bnQKdGhhdCBleGNlZWRzIHRoZSBkZWJ0IGFjY3J1YWwobGlrZSAzMCkgLSB0aGUgdmFsdWUgYmVjb21lcyA5MC4gSWYgdGhlIHVzZXIgaW5zdGVhZCBib3Jyb3dzIDEwIHRva2VucywgdGhlCnZhbHVlIGluY3JlYXNlcyB0byAxMTAuIEluIGFueSBvdGhlciBjYXNlLCBpdCBkb2Vzbid0IGNoYW5nZS4gSXRzIG9ubHkgcHVycG9zZSBpcyB0byB0cmFjayB0aGUgYW1vdW50Cm9mIGFjY3J1ZWQgdW5wYWlkIGludGVyZXN0AAAAABNvcmlnaW5hbGx5X2JvcnJvd2VkAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAD0RlcG9zaXRQb3NpdGlvbgAAAAAEAAAAPEFjY3VtdWxhdGVkIHZhbHVlIG9mIGNvbGxhdGVyYWwgdGhhdCBkb2Vzbid0IGFjY3J1ZSBpbnRlcmVzdAAAAApjb2xsYXRlcmFsAAAAAAALAAAARUEgc2hhcmUgb2YgdG90YWwgc3VwcGxpZWQgdG9rZW5zIGluIHRoZSBwb29sIHRoYXQgb2JsaWdhdGlvbiBjb250YWlucwAAAAAAAAhqX3Rva2VucwAAAAsAAAA3VGltZXN0YW1wIG9mIHdoZW4gdGhlIGxhc3Qgc2NhcmNpdHkgd2l0aGRyYXcgdG9vayBwbGFjZQAAAAAZbGFzdF9zY2FyY2l0eV93aXRoZHJhd190cwAAAAAAAAYAAAHqT3JpZ2luYWxseSBkZXBvc2l0ZWQgdG9rZW4gYW1vdW50LiBJLmUuLCBpZiB0aGUgdXNlciBkZXBvc2l0cyAxMDAgdG9rZW5zIGFuZCAyMCB0b2tlbnMKaGF2ZSBiZWVuIGFjY3J1ZWQgd2l0aCB0aW1lIC0gdGhpcyB2YWx1ZSByZW1haW5zIDEwMC4gSWYsIGFmdGVyIHRoYXQsIHRoZSB1c2VyIHdpdGhkcmF3cyB0aGUgYW1vdW50CnRoYXQgZXhjZWVkcyB0aGUgYWNjcnVhbChsaWtlIDMwKSAtIHRoZSB2YWx1ZSBiZWNvbWVzIDkwIChzYW1lIGdvZXMgZm9yIHdoZW4gYGpfdG9rZW5zYCBhcmUgc2VpemVkCmFzIGNvbGxhdGVyYWwgYnkgYSBsaXF1aWRhdG9yLiBJZiB0aGUgdXNlciBpbnN0ZWFkIGRlcG9zaXRzIDEwIHRva2VucywgdGhlCnZhbHVlIGluY3JlYXNlcyB0byAxMTAuIEluIGFueSBvdGhlciBjYXNlLCBpdCBkb2Vzbid0IGNoYW5nZS4gSXRzIG9ubHkgcHVycG9zZSBpcyB0byB0cmFjayB0aGUgYW1vdW50Cm9mIHJlY2VpdmVkIHN1cHBseSBpbnRlcmVzdAAAAAAAFG9yaWdpbmFsbHlfZGVwb3NpdGVkAAAACw==",
        "AAAAAQAAAE9HZW5lcmFsbHkgcmVwcmVzZW50cyBjb21wdXRlZCBmZWVzIGlzc3VlZCBieSBhbnkgcG9zc2libGUgb3BlcmF0aW9uIG9uIGEgbWFya2V0AAAAAAAAAAAMQ29tcHV0ZWRGZWVzAAAAAwAAACJTdW0gb2YgYG1hcmtldF9mZWVgIGFuZCBgaG9zdF9mZWVgAAAAAAAHZmVlX3N1bQAAAAALAAAANEZlZSBzZWdyZWdhdGVkIHRvIHRoZSBwcm90b2NvbCBob3N0KG1hcmtldCBkZXBsb3llcikAAAAIaG9zdF9mZWUAAAALAAAAIkZlZSBzZWdyZWdhdGVkIHRvIHRoZSBtYXJrZXQgYWRtaW4AAAAAAAptYXJrZXRfZmVlAAAAAAAL",
        "AAAAAQAAACZbYE9ibGlnYXRpb246OmRlcG9zaXRgXSByZXN1bHRpbmcgZGF0YQAAAAAAAAAAAA1EZXBvc2l0UmVzdWx0AAAAAAAAAwAAAAAAAAANY29tcHV0ZWRfZmVlcwAAAAAAB9AAAAAMQ29tcHV0ZWRGZWVzAAAAPkFtb3VudCBvZiBvcmlnaW5hbGx5IGRlcG9zaXRlZCB0b2tlbnMobWludXMgYWxsIHBvc3NpYmxlIGZlZXMpAAAAAAAJZGVwb3NpdGVkAAAAAAAACwAAAE5BbW91bnQgb2YgYGpUb2tlbnNgIHRvIGlzc3VlIHRoYXQgcmVwcmVzZW50IHRoZSBgZGVwb3NpdGVkYCBhbW91bnQgaW4gdGhlIHBvb2wAAAAAABFqX3Rva2Vuc190b19pc3N1ZQAAAAAAAAs=",
        "AAAAAQAAACVbYE9ibGlnYXRpb246OmJvcnJvd2BdIHJlc3VsdGluZyBkYXRhAAAAAAAAAAAAAAxCb3Jyb3dSZXN1bHQAAAAEAAAAREFtb3VudCBvZiBkZWJ0KGluIHRva2VucykgdGhhdCBpcyBhZGRlZCB0byB0aGUgYm9ycm93ZXIncyBvYmxpZ2F0aW9uAAAAEWJvcnJvd2VyX25ld19kZWJ0AAAAAAAACwAAAE9BbW91bnQgb2YgdG9rZW5zIHRvIHJlY2VpdmUgYnkgdGhlIGJvcnJvd2VyKGBib3Jyb3dlcl9uZXdfZGVidGAgbWludXMgYWxsIGZlZXMpAAAAABNib3Jyb3dlcl90b19yZWNlaXZlAAAAAAsAAAAAAAAADWNvbXB1dGVkX2ZlZXMAAAAAAAfQAAAADENvbXB1dGVkRmVlcwAAAFZBbW91bnQgb2YgYGRUb2tlbnNgIHRvIGlzc3VlIHRoYXQgcmVwcmVzZW50IHRoZSBgYm9ycm93ZXJfbmV3X2RlYnRgIGFtb3VudCBpbiB0aGUgcG9vbAAAAAAAEWRfdG9rZW5zX3RvX2lzc3VlAAAAAAAACw==",
        "AAAAAQAAAC1bYE9ibGlnYXRpb246OmFkZF9jb2xsYXRlcmFsYF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAAAAAE0FkZENvbGxhdGVyYWxSZXN1bHQAAAAAAgAAAD1BbW91bnQgb2YgdG9rZW5zIGFkZGVkIGFzIGNvbGxhdGVyYWwobWludXMgYWxsIHBvc3NpYmxlIGZlZXMpAAAAAAAAEGFkZGVkX2NvbGxhdGVyYWwAAAALAAAAAAAAAA1jb21wdXRlZF9mZWVzAAAAAAAH0AAAAAxDb21wdXRlZEZlZXM=",
        "AAAAAQAAACdbYE9ibGlnYXRpb246OndpdGhkcmF3YF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAAA5XaXRoZHJhd1Jlc3VsdAAAAAAABAAAAAAAAAANY29tcHV0ZWRfZmVlcwAAAAAAB9AAAAAMQ29tcHV0ZWRGZWVzAAAAVEFtb3VudCBvZiB0aGUgb3JpZ2luYWwgZGVwb3NpdChpbiB0b2tlbnMpIHRoYXQgaXMgcmVtb3ZlZCBmcm9tIHRoZSBgRGVwb3NpdFBvc2l0aW9uYAAAABBkZXBvc2l0X2RlY3JlYXNlAAAACwAAAFxBbW91bnQgb2YgYGpUb2tlbnNgIHRvIGJ1cm4gdGhhdCByZXByZXNlbnQgdGhlIGBkZXBvc2l0X2RlY3JlYXNlZF9hbW91bnRgIGFtb3VudCBpbiB0aGUKcG9vbAAAABBqX3Rva2Vuc190b19idXJuAAAACwAAAFRBbW91bnQgb2YgdG9rZW5zIHRvIHJlY2VpdmUgYnkgdGhlIHdpdGhkcmF3ZXIoYGRlcG9zaXRfZGVjcmVhc2VkX2Ftb3VudGAgbWludXMgZmVlcykAAAAVd2l0aGRyYXdlcl90b19yZWNlaXZlAAAAAAAACw==",
        "AAAAAQAAACRbYE9ibGlnYXRpb246OnJlcGF5YF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAC1JlcGF5UmVzdWx0AAAAAAQAAAA1RXhjZXNzIGFtb3VudCBnaXZlbiBieSB0aGUgYm9ycm93ZXIgdGhhdCBpcyBzZW50IGJhY2sAAAAAAAATYW1vdW50X3RvX3NlbmRfYmFjawAAAAALAAAAAAAAAA1jb21wdXRlZF9mZWVzAAAAAAAH0AAAAAxDb21wdXRlZEZlZXMAAABPQW1vdW50IG9mIGBkVG9rZW5zYCB0byBidXJuIHRoYXQgcmVwcmVzZW50IHRoZSBgcmVhbF9yZXBhaWRgIGFtb3VudCBpbiB0aGUgcG9vbAAAAAAQZF90b2tlbnNfdG9fYnVybgAAAAsAAAAhQW1vdW50IG9mIHRoZSBkZWJ0IHRoYXQgaXMgcmVwYWlkAAAAAAAAC2RlYnRfcmVwYWlkAAAAAAs=",
        "AAAAAQAAADBbYE9ibGlnYXRpb246OnJlbW92ZV9jb2xsYXRlcmFsYF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAFlJlbW92ZUNvbGxhdGVyYWxSZXN1bHQAAAAAAAMAAAAjQW1vdW50IG9mIGNvbGxhdGVyYWwgdG9rZW5zIHJlbW92ZWQAAAAAE2NvbGxhdGVyYWxfZGVjcmVhc2UAAAAACwAAAFpBbW91bnQgb2YgY29sbGF0ZXJhbCB0b2tlbnMgcmVjZWl2ZWQgYnkgdGhlIGNvbGxhdGVyYWwgcmVtb3ZlcihhY2NvdW50aW5nIHN1YnRyYWN0ZWQgZmVlcykAAAAAAB1jb2xsYXRlcmFsX3JlbW92ZXJfdG9fcmVjZWl2ZQAAAAAAAAsAAAAAAAAADWNvbXB1dGVkX2ZlZXMAAAAAAAfQAAAADENvbXB1dGVkRmVlcw==",
        "AAAAAQAAAC1bYE9ibGlnYXRpb246OmNvdmVyX2JhZF9kZWJ0YF0gcmVzdWx0aW5nIGRhdGEAAAAAAAAAAAAAEkNvdmVyQmFkRGVidFJlc3VsdAAAAAAAAgAAAE1gKHBvb2wgYWRkcmVzcywgYm9ycm93ZXIgZFRva2VucylgIHBhaXJzIGZvciBlYWNoIGJhZCBkZWJ0IG9ibGlnYXRpb24gYm9ycm93cwAAAAAAABlib3Jyb3dzX3RvX2JlX2NvbXBlbnNhdGVkAAAAAAAD6gAAA+0AAAACAAAAEwAAAAsAAABmYChwb29sIGFkZHJlc3MsIGJvcnJvd2VyIGpUb2tlbnMsIGJvcnJvd2VyIGNvbGxhdGVyYWwpYCB0dXBsZXMgZm9yIGVhY2ggYmFkIGRlYnQgb2JsaWdhdGlvbgpjb2xsYXRlcmFsAAAAAAAVY29sbGF0ZXJhbHNfdG9fcmVtb3ZlAAAAAAAD6gAAA+0AAAADAAAAEwAAAAsAAAAL",
        "AAAAAQAAAAAAAAAAAAAAEUxpcXVpZGF0aW9uUmVzdWx0AAAAAAAABQAAAEtUaGUgYW1vdW50IG9mIGBkVG9rZW5zYCB0aGF0IGFyZSBidXJuZWQgZnJvbSB0aGUgYm9ycm93ZXIncyBib3Jyb3cgcG9zaXRpb24AAAAAD2RfdG9rZW5zX2J1cm5lZAAAAAALAAAAMlRoZSBhbW91bnQgb2YgZGVidCB0b2tlbnMgcmVwYWlkIGJ5IHRoZSBsaXF1aWRhdG9yAAAAAAALZGVidF9yZXBhaWQAAAAACwAAANVUaGUgYW1vdW50IG9mIGBqVG9rZW5zYCBzZWl6ZWQgZnJvbSB0aGUgYm9ycm93ZXIncyBvYmxpZ2F0aW9uIGFuZCBnaXZlbiBhd2F5IHRvIHRoZSBsaXF1aWRhdG9yJ3Mgb2JsaWdhdGlvbgppbiBjYXNlIHRoZSBib3Jyb3dlcidzIHBvc2l0aW9uIGRvZXNuJ3QgY29udGFpbiBlbm91Z2ggcGxhaW4gY29sbGF0ZXJhbCB0byBjb3ZlciB0aGUgbGlxdWlkYXRpb24gZXhwZW5zZXMAAAAAAAAPal90b2tlbnNfc2VpemVkAAAAAAsAAABmVGhlIGFtb3VudCBvZiBwbGFpbiBjb2xsYXRlcmFsIHNlaXplZCBmcm9tIHRoZSBib3Jyb3dlcidzIG9ibGlnYXRpb24gYW5kIHRyYW5zZmVycmVkIHRvIHRoZSBsaXF1aWRhdG9yAAAAAAAXcGxhaW5fY29sbGF0ZXJhbF9zZWl6ZWQAAAAACwAAAExUaGUgYW1vdW50IG9mIHRva2VucyByZXByZXNlbnRpbmcgdGhlIGBqX3Rva2Vuc19zZWl6ZWRgIGNvbXB1dGVkIHZpYSBjZWlsaW5nAAAAG3Rva2Vuc19mcm9tX2pfdG9rZW5zX3NlaXplZAAAAAAL",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAARAAAASkFtb3VudCBvZiB0b2tlbnMgdGhhdCBjYW4gYmUgd2l0aGRyYXduIGJ5IHRoZSBob3N0IHBsYXRmb3JtIGFkbWluIGFzIGEgZmVlAAAAAAAVYWNjdW11bGF0ZWRfaG9zdF9mZWVzAAAAAAAACwAAAEVBbW91bnQgb2YgdG9rZW5zIHRoYXQgY2FuIGJlIHdpdGhkcmF3biBieSB0aGUgbWFya2V0J3MgYWRtaW4gYXMgYSBmZWUAAAAAAAAXYWNjdW11bGF0ZWRfbWFya2V0X2ZlZXMAAAAACwAAAFdBbW91bnQgb2YgdG9rZW5zIGluIHRoZSBpbnN1cmFuY2UgcmVzZXJ2ZSB0aGF0IGNhbiBiZSB1c2VkIHRvIGNvdmVyIGEgYmFkIGRlYnQgc2NlbmFyaW8AAAAAGGFjY3VtdWxhdGVkX3Jlc2VydmVfZmVlcwAAAAsAAABWUmVtYWluaW5nIHN1cHBseSBib290c3RyYXAgYW1vdW50cyB0aGF0IGFyZSBkaXN0cmlidXRlZCBldmVubHkgYW1vbmcgc3BlY2lmaWVkIHBlcmlvZHMAAAAAABFib290c3RyYXBfcGVyaW9kcwAAAAAAA+wAAAPtAAAAAgAAAAYAAAAGAAAH0AAAABNQb29sQm9vdHN0cmFwUGVyaW9kAAAAAC1Cb3Jyb3cgYW5udWFsIHBlcmNlbnRhZ2UgcmF0ZSBpbiBiYXNpcyBwb2ludHMAAAAAAAAOYm9ycm93X2Fwcl9icHMAAAAAAAsAAAAjQ29uZmlndXJhdGlvbiBzZXR0aW5ncyBmb3IgdGhlIHBvb2wAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAAwVGhlIHRpbWVzdGFtcCBvZiB0aGUgbGFzdCBhY2NydWFsIHJlLWNhbGN1bGF0aW9uAAAAFmxhc3RfYWNjcnVhbF90aW1lc3RhbXAAAAAAAAYAAADsVGhlIHJlc3VsdCBvZiBgVG9rZW5DbGllbnQ6Om5hbWUoJnNlbGYpYCBpbnZvY2F0aW9uOiBgbmF0aXZlYCBzdHJpbmcgZm9yIFhMTSBTQUMgYW5kIHRoZQpTQUMncyBuYXRpdmUgYXNzZXQgY29kZSBhbmQgYXNzZXQgaXNzdWVyIGNvbmNhdGVuYXRlZCB3aXRoIGA6YCBmb3Igb3RoZXIgU0FDcyhlLmcsCiJBUVVBOkdBSFBZV0xLNllSTjdDVllaT080SDNWRFJaN1BWRjVVSkdMWkNTUEFFSUtKRTJYU1dGNUxBR0VSIikAAAAEbmFtZQAAABAAAAAcVGhlIGFkZHJlc3Mgb2YgdGhlIGxvYW4gcG9vbAAAAAxwb29sX2FkZHJlc3MAAAATAAAALVN1cHBseSBhbm51YWwgcGVyY2VudGFnZSByYXRlIGluIGJhc2lzIHBvaW50cwAAAAAAAA5zdXBwbHlfYXByX2JwcwAAAAAACwAAADpUaGUgYWRkcmVzcyBvZiB0aGUgdG9rZW4gY29udHJhY3QgYXNzb2NpYXRlZCB3aXRoIHRoZSBwb29sAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAoVGhlIHRva2VuIHN5bWJvbCBvZiB0aGUgYXNzb2NpYXRlZCBhc3NldAAAAAx0b2tlbl9zeW1ib2wAAAAQAAAAPFRoZSB0b3RhbCBhbW91bnQgb2YgY3VycmVudGx5IGF2YWlsYWJsZSB0b2tlbnMgZm9yIGJvcnJvd2luZwAAAA90b3RhbF9hdmFpbGFibGUAAAAACwAAAFRUaGUgdG90YWwgYW1vdW50IG9mIGJvcnJvd2VkIGFzc2V0cy4gVGhpcyB2YWx1ZSBpbmNyZWFzZXMgd2l0aCBpbnRlcmVzdCByYXRlIGFjY3J1YWwAAAAOdG90YWxfYm9ycm93ZWQAAAAAAAsAAABKVGhlIHRvdGFsIGFtb3VudCBvZiBkZXBvc2l0ZWQgY29sbGF0ZXJhbCBhc3NldHMgdGhhdCBkb24ndCBhY2NydWUgaW50ZXJlc3QAAAAAABB0b3RhbF9jb2xsYXRlcmFsAAAACwAAAFtUaGUgdG90YWwgYGRUb2tlbnNgIGFtb3VudC4gUmVwcmVzZW50cyB0aGUgc3VtIG9mIGFsbCBkZWJ0IHNoYXJlcyBkaXN0cmlidXRlZCBhbW9uZyBkZWJ0b3JzAAAAAA50b3RhbF9kX3Rva2VucwAAAAAACwAAAHVUaGUgdG90YWwgYGpUb2tlbnNgIGFtb3VudC4gUmVwcmVzZW50cyB0aGUgc3VtIG9mIGFsbCB5aWVsZGluZyBpbnRlcmVzdCBjb2xsYXRlcmFsIHNoYXJlcwpkaXN0cmlidXRlZCBhbW9uZyBjcmVkaXRvcnMAAAAAAAAOdG90YWxfal90b2tlbnMAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAADVBvb2xGZWVDb25maWcAAAAAAAAKAAAAAAAAABZhZGRfY29sbGF0ZXJhbF9mZWVfYnBzAAAAAAAEAAAAAAAAAA5ib3Jyb3dfZmVlX2JwcwAAAAAABAAAAAAAAAAPZGVwb3NpdF9mZWVfYnBzAAAAAAQAAAAAAAAAEmZsYXNoX2xvYW5fZmVlX2JwcwAAAAAABAAAAAAAAAAMaG9zdF9mZWVfYnBzAAAABAAAAAAAAAAZcmVtb3ZlX2NvbGxhdGVyYWxfZmVlX2JwcwAAAAAAAAQAAAAAAAAADXJlcGF5X2ZlZV9icHMAAAAAAAAEAAAAAAAAAA10YWtlX3JhdGVfYnBzAAAAAAAABAAAAAAAAAAQd2l0aGRyYXdfZmVlX2JwcwAAAAQAAACLQWRkaXRpb25hbCBzY2FsYXIgKGluIGJhc2lzIHBvaW50cykgdXNlZCBmb3IgdGhlIGFkZGl0aW9uYWwgd2l0aGRyYXdhbCBmZWUgd2hlbiB0aGUgdXRpbGl6YXRpb24gcmF0aW8KZXhjZWVkcyBgdXRpbGl6YXRpb25fcmF0aW9fbGltaXRfYnBzYAAAAAAcd2l0aGRyYXdfc2NhcmNpdHlfZmVlX3NjX2JwcwAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xTdGF0dXMAAAAAAAIAAAAAAAAADmJvcnJvd19lbmFibGVkAAAAAAABAAAAAAAAAA9kZXBvc2l0X2VuYWJsZWQAAAAAAQ==",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAUAAAAAAAAADWFjY3J1YWxfbW9kZWwAAAAAAAfQAAAADEFjY3J1YWxNb2RlbAAAAAAAAAAKZmVlX2NvbmZpZwAAAAAH0AAAAA1Qb29sRmVlQ29uZmlnAAAAAAAAAAAAAA1oZWFsdGhfY29uZmlnAAAAAAAH0AAAABBQb29sSGVhbHRoQ29uZmlnAAAAAAAAABNpbnRlcmVzdF9yYXRlX21vZGVsAAAAB9AAAAARSW50ZXJlc3RSYXRlTW9kZWwAAAAAAAAAAAAABnN0YXR1cwAAAAAH0AAAAApQb29sU3RhdHVzAAA=",
        "AAAAAQAAAAAAAAAAAAAAEFBvb2xIZWFsdGhDb25maWcAAAAKAAAA4lRoZSBtYXhpbXVtIHBlcmNlbnRhZ2Ugb2YgYW4gYXNzZXQncyB2YWx1ZSB0aGF0IGNhbiBiZSBoZWxkIGluIGFuIGluZGl2aWR1YWwgb2JsaWdhdGlvbiBpbgpiYXNpcyBwb2ludHMgd2l0aCByZXNwZWN0IHRvIGEgdG90YWwgb2JsaWdhdGlvbidzIGNvbGxhdGVyYWwgdmFsdWUuIExUViBncmVhdGVyIHRoYW4KdGhhdCBtYWtlcyBib3Jyb3cgcG9zaXRpb24gZWxpZ2libGUgdG8gbGlxdWlkYXRpb24AAAAAAA1jbG9zZV9sdHZfYnBzAAAAAAAACwAAAMlMVFYgY2FsY3VsYXRlZCBmb3IgdW5wYXJhbWV0ZXJpemVkIG9ibGlnYXRpb24gcG9zaXRpb25zKGkuZS4sIG5vIG9wZW5MVFYvbGlhYmlsaXR5IGZhY3RvcnMgc2NhbGluZykgdGhhdCBtYXJrcwpwb3NpdGlvbiBhcyBpbnNvbHZlbnQuIFVzZWQgYXMgYSBtZWFucyB0byBhdm9pZCB1bnByb2ZpdGFibGUgaGVhbHRoLWltcHJvdmluZyBsaXF1aWRhdGlvbnMAAAAAAAASaW5zb2x2ZW5jeV9sdHZfYnBzAAAAAAALAAAA71RoZSBmYWN0b3IgdXNlZCB0byBjYWxjdWxhdGUgdGhlIGN1cnJlbnQgYm9ycm93IGxpbWl0IGJ5IG11bHRpcGx5aW5nIHRoZSBjb2xsYXRlcmFsIHZhbHVlCmJ5IGl0IGJlZm9yZSBzdWJ0cmFjdGluZyB0aGlzIHZhbHVlIGZyb20gdGhlIG9ibGlnYXRpb24ncyBtYXggYm9ycm93IGxpbWl0LiBWb2xhdGlsZQphc3NldHMnIHBvb2xzIGFyZSBleHBlY3RlZCB0byBoYXZlIHRoaXMgdmFsdWUgc2V0IHdheSBhYm92ZSAxMDAlAAAAABRsaWFiaWxpdHlfZmFjdG9yX2JwcwAAAAsAAABGTWF4aW11bSBwZXJjZW50YWdlIG9mIGEgYm9ycm93ZXIncyBkZWJ0IHRoYXQgY2FuIGJlIGxpcXVpZGF0ZWQgYXQgb25jZQAAAAAAHGxpcXVpZGF0aW9uX2Nsb3NlX2ZhY3Rvcl9icHMAAAALAAAAa01heGltdW0gYWRkaXRpb25hbCB2YWx1ZSBpbiB0aGUgcmVjZWl2ZWQgdG9rZW5zIHRoYXQgY2FuIGJlIGdpdmVuIHRvIGxpcXVpZGF0b3JzIHdoZW4gcHVyY2hhc2luZyBjb2xsYXRlcmFsAAAAAB1tYXhfbGlxdWlkYXRpb25faW5jZW50aXZlX2JwcwAAAAAAAAsAAACbVGhlIG1heGltdW0gcGVyY2VudGFnZSBvZiBhbiBhc3NldCdzIHZhbHVlIHRoYXQgY2FuIGJlIGJvcnJvd2VkIGluIGJhc2lzIHBvaW50cyhlLmcsIDcwMDAgPQo3MCUsIGV0Yykgd2l0aCByZXNwZWN0IHRvIGEgdG90YWwgb2JsaWdhdGlvbidzIGNvbGxhdGVyYWwgdmFsdWUAAAAADG9wZW5fbHR2X2JwcwAAAAsAAACHVGhlIG1heGltdW0gYW1vdW50IG9mIHN1cHBsaWVkIHRva2VucyB0aGF0IGNhbiBiZSBzdXBwbGllZCBpbiB0aGUgcG9vbChpLmUuLCBgYXZhaWxhYmxlYCArCmB0b3RhbF9ib3Jyb3dlZGApIDAgZGVub3RlcyB1bmxpbWl0ZWQgc3VwcGx5AAAAAAxzdXBwbHlfbGltaXQAAAALAAAASVRoZSBtYXhpbXVtIHV0aWxpemF0aW9uIHJhdGlvIHRoYXQgaXMgYWxsb3dlZCB0byBiZSByZWFjaGVkIHZpYSBib3Jyb3dpbmcAAAAAAAAbdXRpbGl6YXRpb25fcmF0aW9fbGltaXRfYnBzAAAAAAsAAACVQ29vbGRvd24gcGVyaW9kKGluIHNlY29uZHMpIHJlcXVpcmVkIGJldHdlZW4gYSBwYWlyIG9mIHNlcXVlbnRpYWwgd2l0aGRyYXdhbHMgd2hlbiB0aGUgcG9vbCdzIHV0aWxpemF0aW9uIHJhdGlvIGV4Y2VlZHMKYHV0aWxpemF0aW9uX3JhdGlvX2xpbWl0X2Jwc2AAAAAAAAAcd2l0aGRyYXdfc2NhcmNpdHlfY29vbGRvd25fcwAAAAYAAACbQmFzaXMgcG9pbnRzIG9mIHRoZSBwb29sJ3MgdG90YWwgc3VwcGx5IHRoYXQgY2FuIGJlIHdpdGhkcmF3biBpbiBhIHNpbmdsZSBvcGVyYXRpb24gd2hlbiB0aGUgcG9vbCdzIHV0aWxpemF0aW9uIHJhdGlvIGV4Y2VlZHMKYHV0aWxpemF0aW9uX3JhdGlvX2xpbWl0X2Jwc2AAAAAAG3dpdGhkcmF3X3NjYXJjaXR5X2xpbWl0X2JwcwAAAAAL",
        "AAAAAQAAAAAAAAAAAAAAE1Bvb2xCb290c3RyYXBQZXJpb2QAAAAAAgAAABpSZW1haW5pbmcgYm9vdHN0cmFwIGFtb3VudAAAAAAAEHJlbWFpbmluZ19hbW91bnQAAAALAAAAH1RvdGFsIHByb3ZpZGVkIGJvb3RzdHJhcCBhbW91bnQAAAAADHRvdGFsX2Ftb3VudAAAAAs=",
        "AAAAAQAAACNBIHJlcXVlc3QgZnJvbSB0aGUgc3VibWlzc2lvbiBiYXRjaAAAAAAAAAAAB1JlcXVlc3QAAAAAAwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAxyZXF1ZXN0X3R5cGUAAAAE",
        "AAAAAwAAAAAAAAAAAAAAC1JlcXVlc3RUeXBlAAAAAAYAAAAAAAAAB0RlcG9zaXQAAAAAAAAAAAAAAAAGQm9ycm93AAAAAAABAAAAAAAAAAhXaXRoZHJhdwAAAAIAAAAAAAAABVJlcGF5AAAAAAAAAwAAAAAAAAANQWRkQ29sbGF0ZXJhbAAAAAAAAAQAAAAAAAAAEFJlbW92ZUNvbGxhdGVyYWwAAAAF",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAoAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIZGVwbG95ZXIAAAATAAAAAAAAABJpbnNvbHZlbmN5X2x0dl9icHMAAAAAAAsAAAAAAAAACGlzX293bmVkAAAAAQAAAAAAAAANbWF4X3Bvc2l0aW9ucwAAAAAAAAQAAAAAAAAAFG1pbl9jb2xsYXRlcmFsX3ZhbHVlAAAACwAAAAAAAAAEbmFtZQAAABAAAAAAAAAABm9yYWNsZQAAAAAAEwAAAAAAAAAGc3RhdHVzAAAAAAAEAAAAAAAAABZ1cGRhdGVfaW5fcXVldWVfcGVyaW9kAAAAAAPoAAAABg==",
        "AAAAAgAAAAAAAAAAAAAADE1hcmtldFN0YXR1cwAAAAQAAAAAAAAAGkFsbCBvcGVyYXRpb25zIGFyZSBhbGxvd2VkAAAAAAAGQWN0aXZlAAAAAAAAAAAAIEJvcnJvdyBvcGVyYXRpb25zIGFyZSBwcm9oaWJpdGVkAAAADEJvcnJvd0Zyb3plbgAAAAAAAABAQm9ycm93aW5nIGFuZCBkZXBvc2l0aW5nIG9wZXJhdGlvbnMgb24gdGhlIG1hcmtldCBhcmUgcHJvaGliaXRlZAAAAA1EZXBvc2l0RnJvemVuAAAAAAAAAAAAACtBbGwgb3BlcmF0aW9ucyBvbiB0aGUgbWFya2V0IGFyZSBwcm9oaWJpdGVkAAAAAAZGcm96ZW4AAA==",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xVcGRhdGUAAAAAAAIAAAAAAAAACm5ld19jb25maWcAAAAAB9AAAAAKUG9vbENvbmZpZwAAAAAAAAAAABNxdWV1ZWRfaW5fdGltZXN0YW1wAAAAAAY=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAFAAAAAAAAAAAAAAABE5hbWUAAAAAAAAAAAAAAAVBZG1pbgAAAAAAAAAAAAAAAAAAE1VwZGF0ZUluUXVldWVQZXJpb2QAAAAAAAAAAAAAAAAHSXNPd25lZAAAAAAAAAAAAAAAAAxEZXBsb3llckhvc3QAAAAAAAAAAAAAAAZPcmFjbGUAAAAAAAAAAAAAAAAAEk1pbkNvbGxhdGVyYWxWYWx1ZQAAAAAAAAAAAAAAAAAQSW5zb2x2ZW5jeUx0dkJwcwAAAAAAAAAAAAAADE1heFBvc2l0aW9ucwAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAAAAAAAAAAAB0FjY3J1YWwAAAAAAAAAAAAAAAAIQWxsUG9vbHMAAAAAAAAAAAAAAA5BbGxPYmxpZ2F0aW9ucwAAAAAAAAAAAAAAAAAQQWxsTXVsdGlwbHlQYWlycwAAAAAAAAAAAAAADE1hcmtldFN0YXR1cwAAAAEAAAAAAAAADENvbmZpZ1VwZGF0ZQAAAAEAAAATAAAAAQAAAAAAAAAEUG9vbAAAAAEAAAATAAAAAQAAAAAAAAAKT2JsaWdhdGlvbgAAAAAAAQAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAEAAAAAAAAADE11bHRpcGx5UGFpcgAAAAEAAAPtAAAAAgAAABMAAAATAAAAAAAAAAAAAAASRWFybk9ibGlnYXRpb25TZWVkAAA=",
        "AAAAAQAAAC9QcmljZSBkYXRhIGZvciBhbiBhc3NldCBhdCBhIHNwZWNpZmljIHRpbWVzdGFtcAAAAAAAAAAACVByaWNlRGF0YQAAAAAAAAIAAAAAAAAABXByaWNlAAAAAAAACwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==",
        "AAAAAgAAAApBc3NldCB0eXBlAAAAAAAAAAAABUFzc2V0AAAAAAAAAgAAAAEAAAAAAAAAB1N0ZWxsYXIAAAAAAQAAABMAAAABAAAAAAAAAAVPdGhlcgAAAAAAAAEAAAAR" ]),
      options
    )
  }
  public readonly fromJSON = {
    upgrade: this.txFromJSON<null>,
        submit_requests_batch: this.txFromJSON<Result<void>>,
        get_global_state: this.txFromJSON<GlobalState>,
        update_market: this.txFromJSON<Result<void>>,
        update_market_status: this.txFromJSON<Result<void>>,
        initialize_pool: this.txFromJSON<Result<string>>,
        initialize_multiply_pair: this.txFromJSON<Result<void>>,
        queue_in_pool_config_update: this.txFromJSON<Result<void>>,
        cancel_pool_config_update: this.txFromJSON<Result<void>>,
        apply_pool_config_update: this.txFromJSON<Result<void>>,
        get_pool_config_queued_in_update: this.txFromJSON<Result<PoolUpdate>>,
        bootstrap_pool: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        deposit_into_earn_obligation: this.txFromJSON<Result<void>>,
        borrow: this.txFromJSON<Result<void>>,
        swap: this.txFromJSON<Result<i128>>,
        donate_to_reserve: this.txFromJSON<Result<void>>,
        add_collateral: this.txFromJSON<Result<void>>,
        remove_collateral: this.txFromJSON<Result<void>>,
        repay: this.txFromJSON<Result<void>>,
        liquidate: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        withdraw_from_earn_obligation: this.txFromJSON<Result<void>>,
        flash_loan: this.txFromJSON<Result<void>>,
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
        refresh_obligation: this.txFromJSON<Result<void>>,
        refresh_earn_obligation: this.txFromJSON<Result<void>>,
        refresh_multiply_pair_obligation: this.txFromJSON<Result<void>>,
        refresh_pool: this.txFromJSON<Result<void>>,
        get_earn_user_obligation: this.txFromJSON<Result<Obligation>>,
        get_multiply_pair_obligation: this.txFromJSON<Result<Obligation>>,
        get_pool: this.txFromJSON<Result<Pool>>,
        get_all_pools: this.txFromJSON<Array<string>>,
        get_market_data: this.txFromJSON<Result<MarketData>>,
        get_all_obligations: this.txFromJSON<Array<ObligationKey>>,
        get_multiply_pair: this.txFromJSON<Result<MultiplyPair>>,
        get_all_multiply_pairs: this.txFromJSON<Array<MultiplyPair>>,
        get_pool_data: this.txFromJSON<Result<PoolData>>,
        reset_storage: this.txFromJSON<null>
  }
}