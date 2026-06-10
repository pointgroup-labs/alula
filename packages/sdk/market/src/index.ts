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





export interface Obligation {
  borrows: Map<string, BorrowPosition>;
  deposits: Map<string, DepositPosition>;
  insurance_fund_requests_ids: Map<readonly [string, u64], u64>;
  positions_count: u32;
}


export interface RepayResult {
  amount_to_send_back: i128;
  d_tokens_to_burn: i128;
  debt_repaid: i128;
  operation_fees: OperationFees;
}


export interface BorrowResult {
  borrower_new_debt: i128;
  borrower_to_receive: i128;
  d_tokens_to_issue: i128;
  operation_fees: OperationFees;
}


export interface DepositResult {
  deposited: i128;
  j_tokens_to_issue: i128;
  operation_fees: OperationFees;
}


export interface ObligationKey {
  seed: Option<Buffer>;
  user: string;
}


export interface OperationFees {
  fee_sum: i128;
  referrer_fee: i128;
}


export interface BorrowPosition {
  d_tokens: i128;
  originally_borrowed: i128;
}


export interface WithdrawResult {
  deposit_decrease: i128;
  j_tokens_to_burn: i128;
  operation_fees: OperationFees;
  withdrawer_to_receive: i128;
}


export interface DepositPosition {
  collateral: i128;
  j_tokens: i128;
  last_scarcity_withdraw_ts: u64;
  originally_deposited: i128;
}


export interface LiquidationResult {
  amount_to_send_back: i128;
  d_tokens_burned: i128;
  debt_repaid: i128;
  j_tokens_seized: i128;
  plain_collateral_seized: i128;
  tokens_from_j_tokens_seized: i128;
}


export interface AddCollateralResult {
  added_collateral: i128;
  operation_fees: OperationFees;
}


export interface RemoveCollateralResult {
  collateral_decrease: i128;
  collateral_remover_to_receive: i128;
  operation_fees: OperationFees;
}


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


export interface PoolData {
  apy: AnnualPercentageYields;
  d_token_rate_ceil_bps: i128;
  j_token_rate_floor_bps: i128;
  oracle_asset_price: i128;
  pool: Pool;
  total_available_adjusted: i128;
  total_supply: i128;
}


export interface MarketData {
  global_state: GlobalState;
  oracle_price_decimals: u32;
  pools_data: Array<PoolData>;
}


export interface Pool {
  bad_debt_lock_d: u64;
  bad_debt_request_count: u32;
  borrow_apr_bps: i128;
  config: PoolConfig;
  farm_debt: Option<Buffer>;
  farm_supply: Option<Buffer>;
  interest_rate_modifier_bps: i128;
  last_accrual_timestamp: u64;
  name: string;
  operation_fees_sum: i128;
  pool_address: string;
  supply_apr_bps: i128;
  take_rate_fees_sum: i128;
  token_address: string;
  token_decimals: u32;
  token_symbol: string;
  total_available: i128;
  total_borrowed: i128;
  total_collateral: i128;
  total_d_tokens: i128;
  total_j_tokens: i128;
}


export interface PoolConfig {
  accrual_model: AccrualModel;
  fee_config: PoolFeeConfig;
  health_config: PoolHealthConfig;
  interest_rate_model: InterestRateModel;
  ir_reactivity_constant: u32;
  status: PoolStatus;
  target_utilization_ratio_bps: i128;
}


export interface PoolStatus {
  flags: u32;
}


export interface PoolFeeConfig {
  add_collateral_fee_bps: u32;
  borrow_fee_bps: u32;
  deposit_fee_bps: u32;
  flash_loan_fee_bps: u32;
  operation_fee_beneficiaries: Option<Map<string, u32>>;
  referrers: Option<Map<string, u32>>;
  remove_collateral_fee_bps: u32;
  repay_fee_bps: u32;
  take_rate_beneficiaries: Option<Map<string, u32>>;
  take_rate_bps: u32;
  withdraw_fee_bps: u32;
  withdraw_max_scarcity_fee_bps: u32;
}


export interface PoolHealthConfig {
  close_ltv_bps: i128;
  liability_factor_bps: i128;
  liquidation_close_factor_bps: i128;
  max_liquidation_incentive_bps: i128;
  open_ltv_bps: i128;
  supply_limit: i128;
  utilization_ratio_limit_bps: i128;
  withdraw_scarcity_cooldown_s: u64;
  withdraw_scarcity_limit_bps: i128;
}

export const MCError = {
  0: {message:"InternalError"},
  1: {message:"InvalidInputAmount"},
  2: {message:"DependencyContractError"},
  3: {message:"MarketIsNotOwned"},
  4: {message:"BorrowForbiddenOnMarket"},
  5: {message:"DepositForbiddenOnMarket"},
  6: {message:"MarketIsFrozen"},
  7: {message:"InvalidMarketConfigOrUpdate"},
  8: {message:"IncorrectRequestType"},
  9: {message:"OverOrUnderflow"},
  10: {message:"TooManyPositions"},
  11: {message:"MinCollateralValueIsNotMet"},
  12: {message:"NonPositiveSharesAmount"},
  100: {message:"InvalidInitialization"},
  101: {message:"PoolDoesNotExist"},
  102: {message:"InvalidLoanPoolConfig"},
  103: {message:"NotEnoughPoolFunds"},
  104: {message:"DepositPoolDoesNotExist"},
  105: {message:"BorrowPoolDoesNotExist"},
  106: {message:"CollateralPoolDoesNotExist"},
  107: {message:"PoolAlreadyContainsQueuedPoolSet"},
  108: {message:"PoolDoesNotHaveQueuedPoolSet"},
  109: {message:"PoolSetIsNotYetApplicable"},
  110: {message:"OperationForbiddenOnPool"},
  111: {message:"MarketAlreadyContainsQueuedInConfigUpdate"},
  112: {message:"MarketDoesNotHaveQueuedInConfigUpdate"},
  113: {message:"MarketConfigUpdateIsNotYetApplicable"},
  114: {message:"PoolBadDebtLocked"},
  200: {message:"ObligationDoesNotExist"},
  201: {message:"DepositPositionDoesNotExist"},
  202: {message:"BorrowPositionDoesNotExist"},
  203: {message:"WithdrawScarcityOverLimit"},
  204: {message:"ScarcityCooldownPeriod"},
  205: {message:"BorrowPositionForAssetExists"},
  206: {message:"DepositPositionForAssetExists"},
  207: {message:"UnhealthyOperation"},
  400: {message:"PoolSupplyLimitExceeded"},
  401: {message:"PoolUtilizationRatioCapExceeded"},
  500: {message:"OracleDoesNotKnowAssetPrice"},
  501: {message:"OracleStalePrice"},
  502: {message:"NonPositiveOraclePrice"},
  600: {message:"InvalidLiquidationInputs"},
  601: {message:"ObligationIsHealthy"},
  602: {message:"ObligationContainsOpenCoverBadDebtRequests"},
  603: {message:"BadDebtCoverageCriterionIsNotMet"},
  604: {message:"AssetCannotBeUsedAsCollateral"},
  605: {message:"LiquidationExcessiveDemandedCollateral"},
  701: {message:"InvalidSwap"},
  702: {message:"FlashBorrowAlreadyRegistered"},
  703: {message:"SwapSlippageExceeded"}
}



























































export type AccrualModel = {tag: "Compounded", values: void};

export type Request = {tag: "Deposit", values: readonly [StandardRequest]} | {tag: "Borrow", values: readonly [StandardRequest]} | {tag: "Withdraw", values: readonly [StandardRequest]} | {tag: "Repay", values: readonly [StandardRequest]} | {tag: "AddCollateral", values: readonly [StandardRequest]} | {tag: "RemoveCollateral", values: readonly [StandardRequest]} | {tag: "FlashBorrow", values: readonly [StandardRequest]} | {tag: "SwapExactTokens", values: readonly [SwapExactTokensRequest]} | {tag: "SwapForExactTokens", values: readonly [SwapForExactTokensRequest]} | {tag: "Liquidate", values: readonly [LiquidateRequest]};


export interface StandardRequest {
  amount: i128;
  pool_address: string;
}


export interface LiquidateRequest {
  borrow_pool_address: string;
  borrower_obligation_key: ObligationKey;
  collateral_pool_address: string;
  min_demanded_collateral_amount: i128;
  repay_amount: i128;
}


export interface SwapExactTokensRequest {
  amount_in: i128;
  min_amount_out: i128;
  path: Array<string>;
  swap_provider: string;
}


export interface SwapForExactTokensRequest {
  amount_out: i128;
  max_amount_in: i128;
  path: Array<string>;
  swap_provider: string;
}

export type DataKey = {tag: "Name", values: void} | {tag: "Admin", values: void} | {tag: "Oracle", values: void} | {tag: "Accrual", values: void} | {tag: "IsOwned", values: void} | {tag: "AllPools", values: void} | {tag: "GlobalState", values: void} | {tag: "DeployerHost", values: void} | {tag: "MaxPositions", values: void} | {tag: "MarketStatus", values: void} | {tag: "FarmsContract", values: void} | {tag: "QueuedPoolSet", values: readonly [string]} | {tag: "Pool", values: readonly [string]} | {tag: "InsuranceFund", values: void} | {tag: "InsolvencyLtvBps", values: void} | {tag: "EarnObligationSeed", values: void} | {tag: "MinCollateralValueCents", values: void} | {tag: "UpdateInQueuePeriod", values: void} | {tag: "MarketConfigUpdate", values: void} | {tag: "Obligation", values: readonly [ObligationKey]} | {tag: "ProposedAdmin", values: void} | {tag: "BadDebtLockDuration", values: void};


export interface GlobalState {
  admin: string;
  bad_debt_lock_d: u64;
  deployer: string;
  insolvency_ltv_bps: i128;
  insurance_fund: string;
  is_owned: boolean;
  max_positions: u32;
  min_collateral_value_cents: i128;
  name: string;
  oracle: string;
  status: u32;
  update_in_queue_period: u64;
}

export type MarketStatus = {tag: "Active", values: void} | {tag: "BorrowFrozen", values: void} | {tag: "BorrowFrozenByAdmin", values: void} | {tag: "DepositFrozen", values: void} | {tag: "DepositFrozenByAdmin", values: void} | {tag: "Frozen", values: void} | {tag: "FrozenByAdmin", values: void};


export interface MarketUpdate {
  new_bad_debt_lock_d: u64;
  new_max_positions: u32;
  new_min_collateral_value_cents: i128;
  queued_in_timestamp: u64;
}


export interface QueuedPoolSet {
  new_config: PoolConfig;
  queued_in_timestamp: u64;
}


export interface MarketInitParams {
  bad_debt_lock_d: u64;
  insolvency_ltv_bps: i128;
  is_owned: boolean;
  max_positions: u32;
  min_collateral_value_cents: i128;
  update_in_queue_period: u64;
}


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

export type CoverageStatus = {tag: "Pending", values: void} | {tag: "Ready", values: readonly [i128]};

export type IssueRequestResult = {tag: "Recorded", values: readonly [u64]} | {tag: "Immediate", values: readonly [i128]};

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
   * Construct and simulate a repay transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  repay: ({user, pool_address, amount, referrer}: {user: ObligationKey, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a borrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  borrow: ({user, pool_address, amount, referrer}: {user: ObligationKey, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit: ({user, pool_address, amount, referrer}: {user: ObligationKey, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a upgrade transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a get_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Pool>>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw: ({user, pool_address, amount, referrer}: {user: ObligationKey, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a liquidate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  liquidate: ({liquidator, borrower, borrow_pool_address, collateral_pool_address, repay_amount, min_demanded_collateral_amount}: {liquidator: string, borrower: ObligationKey, borrow_pool_address: string, collateral_pool_address: string, repay_amount: i128, min_demanded_collateral_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a flash_loan transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  flash_loan: ({contract, caller, pool_address, amount}: {contract: string, caller: string, pool_address: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_pool: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_all_pools transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_pools: (options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_pool_data transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_data: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<PoolData>>>

  /**
   * Construct and simulate a add_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_collateral: ({user, pool_address, amount, referrer}: {user: ObligationKey, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_pool_set transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_pool_set: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel_pool_set transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_pool_set: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_market_data transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_market_data: (options?: MethodOptions) => Promise<AssembledTransaction<Result<MarketData>>>

  /**
   * Construct and simulate a clear_pool_farms transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  clear_pool_farms: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_global_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_global_state: (options?: MethodOptions) => Promise<AssembledTransaction<GlobalState>>

  /**
   * Construct and simulate a propose_new_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_new_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a queue_in_pool_set transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_pool_set: ({pool_address, pool_config}: {pool_address: string, pool_config: PoolConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_collateral: ({user, pool_address, amount, referrer}: {user: ObligationKey, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a simulate_withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  simulate_withdraw: ({user, pool_address, amount, referrer}: {user: ObligationKey, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<WithdrawResult>>>

  /**
   * Construct and simulate a get_farms_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_farms_contract: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a refresh_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_obligation: ({user}: {user: ObligationKey}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_farms_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_farms_contract: ({farms_contract}: {farms_contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_pool_debt_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_pool_debt_farm: ({pool_address, farm_id}: {pool_address: string, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_pool_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_pool_status: ({pool_address, new_status_flags}: {pool_address: string, new_status_flags: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_market_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_market_update: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_queued_pool_set transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_queued_pool_set: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<QueuedPoolSet>>>

  /**
   * Construct and simulate a get_user_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_obligation: ({user}: {user: ObligationKey}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a cancel_market_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_market_update: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a clear_farms_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  clear_farms_contract: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_pool_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  distribute_pool_fees: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a issue_cover_bad_debt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  issue_cover_bad_debt: ({user}: {user: ObligationKey}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_pool_supply_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_pool_supply_farm: ({pool_address, farm_id}: {pool_address: string, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_market_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_market_status: ({new_status}: {new_status: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_proposed_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_proposed_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a submit_requests_batch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  submit_requests_batch: ({user, requests, referrer}: {user: ObligationKey, requests: Array<Request>, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a queue_in_market_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_market_update: ({new_max_positions, new_min_collateral_value_cents, new_bad_debt_lock_d}: {new_max_positions: u32, new_min_collateral_value_cents: i128, new_bad_debt_lock_d: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_obligation_farms transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_obligation_farms: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_all_pools_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  distribute_all_pools_fees: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a fund_update_market_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  fund_update_market_status: ({new_status}: {new_status: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_oracle_price_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_oracle_price_decimals: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_market_queued_in_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_market_queued_in_update: (options?: MethodOptions) => Promise<AssembledTransaction<Result<MarketUpdate>>>

  /**
   * Construct and simulate a get_pool_asset_oracle_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_asset_oracle_price: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a claim_cover_bad_debt_results transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_cover_bad_debt_results: ({user}: {user: ObligationKey}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_operation_fees_beneficiaries transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_operation_fees_beneficiaries: ({pool_address, beneficiaries}: {pool_address: string, beneficiaries: Map<string, u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_take_rate_fees_beneficiaries transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_take_rate_fees_beneficiaries: ({pool_address, beneficiaries}: {pool_address: string, beneficiaries: Map<string, u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {name, admin, oracle, insurance_fund, deployer, params}: {name: string, admin: string, oracle: string, insurance_fund: string, deployer: string, params: MarketInitParams},
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
    return ContractClient.deploy({name, admin, oracle, insurance_fund, deployer, params}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAQAAAAAAAAAB2JvcnJvd3MAAAAD7AAAABMAAAfQAAAADkJvcnJvd1Bvc2l0aW9uAAAAAAAAAAAACGRlcG9zaXRzAAAD7AAAABMAAAfQAAAAD0RlcG9zaXRQb3NpdGlvbgAAAAAAAAAAG2luc3VyYW5jZV9mdW5kX3JlcXVlc3RzX2lkcwAAAAPsAAAD7QAAAAIAAAATAAAABgAAAAYAAAAAAAAAD3Bvc2l0aW9uc19jb3VudAAAAAAE",
        "AAAAAQAAAAAAAAAAAAAAC1JlcGF5UmVzdWx0AAAAAAQAAAAAAAAAE2Ftb3VudF90b19zZW5kX2JhY2sAAAAACwAAAAAAAAAQZF90b2tlbnNfdG9fYnVybgAAAAsAAAAAAAAAC2RlYnRfcmVwYWlkAAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAADEJvcnJvd1Jlc3VsdAAAAAQAAAAAAAAAEWJvcnJvd2VyX25ld19kZWJ0AAAAAAAACwAAAAAAAAATYm9ycm93ZXJfdG9fcmVjZWl2ZQAAAAALAAAAAAAAABFkX3Rva2Vuc190b19pc3N1ZQAAAAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAADURlcG9zaXRSZXN1bHQAAAAAAAADAAAAAAAAAAlkZXBvc2l0ZWQAAAAAAAALAAAAAAAAABFqX3Rva2Vuc190b19pc3N1ZQAAAAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAADU9ibGlnYXRpb25LZXkAAAAAAAACAAAAAAAAAARzZWVkAAAD6AAAA+4AAAAgAAAAAAAAAAR1c2VyAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAADU9wZXJhdGlvbkZlZXMAAAAAAAACAAAAAAAAAAdmZWVfc3VtAAAAAAsAAAAAAAAADHJlZmVycmVyX2ZlZQAAAAs=",
        "AAAAAQAAAAAAAAAAAAAADkJvcnJvd1Bvc2l0aW9uAAAAAAACAAAAAAAAAAhkX3Rva2VucwAAAAsAAAAAAAAAE29yaWdpbmFsbHlfYm9ycm93ZWQAAAAACw==",
        "AAAAAQAAAAAAAAAAAAAADldpdGhkcmF3UmVzdWx0AAAAAAAEAAAAAAAAABBkZXBvc2l0X2RlY3JlYXNlAAAACwAAAAAAAAAQal90b2tlbnNfdG9fYnVybgAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAAAAAAAAAAAFXdpdGhkcmF3ZXJfdG9fcmVjZWl2ZQAAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAD0RlcG9zaXRQb3NpdGlvbgAAAAAEAAAAAAAAAApjb2xsYXRlcmFsAAAAAAALAAAAAAAAAAhqX3Rva2VucwAAAAsAAAAAAAAAGWxhc3Rfc2NhcmNpdHlfd2l0aGRyYXdfdHMAAAAAAAAGAAAAAAAAABRvcmlnaW5hbGx5X2RlcG9zaXRlZAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAEUxpcXVpZGF0aW9uUmVzdWx0AAAAAAAABgAAAAAAAAATYW1vdW50X3RvX3NlbmRfYmFjawAAAAALAAAAAAAAAA9kX3Rva2Vuc19idXJuZWQAAAAACwAAAAAAAAALZGVidF9yZXBhaWQAAAAACwAAAAAAAAAPal90b2tlbnNfc2VpemVkAAAAAAsAAAAAAAAAF3BsYWluX2NvbGxhdGVyYWxfc2VpemVkAAAAAAsAAAAAAAAAG3Rva2Vuc19mcm9tX2pfdG9rZW5zX3NlaXplZAAAAAAL",
        "AAAAAQAAAAAAAAAAAAAAE0FkZENvbGxhdGVyYWxSZXN1bHQAAAAAAgAAAAAAAAAQYWRkZWRfY29sbGF0ZXJhbAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAAFlJlbW92ZUNvbGxhdGVyYWxSZXN1bHQAAAAAAAMAAAAAAAAAE2NvbGxhdGVyYWxfZGVjcmVhc2UAAAAACwAAAAAAAAAdY29sbGF0ZXJhbF9yZW1vdmVyX3RvX3JlY2VpdmUAAAAAAAALAAAAAAAAAA5vcGVyYXRpb25fZmVlcwAAAAAH0AAAAA1PcGVyYXRpb25GZWVzAAAA",
        "AAAAAQAAAAAAAAAAAAAAFkFubnVhbFBlcmNlbnRhZ2VZaWVsZHMAAAAAAAIAAAAAAAAACmJvcnJvd19icHMAAAAAAAQAAAAAAAAACnN1cHBseV9icHMAAAAAAAQ=",
        "AAAAAgAAAAAAAAAAAAAAEUludGVyZXN0UmF0ZU1vZGVsAAAAAAAAAQAAAAEAAAAAAAAABktpbmtlZAAAAAAAAQAAB9AAAAAOS2lua2VkSVJDb25maWcAAA==",
        "AAAAAQAAAAAAAAAAAAAADktpbmtlZElSQ29uZmlnAAAAAAAGAAAARkJhc2UgQVBSIHRoYXQgaXMgYWNjcnVlZCByZWdhcmRsZXNzIG9mIHRoZSB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wAAAAAAAxiYXNlX2Fwcl9icHMAAAALAAAARUFQUiB0aGF0IGlzIGFjY3J1ZWQgd2hlbiB0aGUgdXRpbGl6YXRpb24gcmF0aW8gaXMgYXQgdGhlIGtpbmsgMSB2YWx1ZQAAAAAAAA1raW5rMV9hcHJfYnBzAAAAAAAACwAAABhLaW5rIDEgdXRpbGl6YXRpb24gcmF0aW8AAAAMa2luazFfdXJfYnBzAAAACwAAAEVBUFIgdGhhdCBpcyBhY2NydWVkIHdoZW4gdGhlIHV0aWxpemF0aW9uIHJhdGlvIGlzIGF0IHRoZSBraW5rIDIgdmFsdWUAAAAAAAANa2luazJfYXByX2JwcwAAAAAAAAsAAAAYS2luayAyIHV0aWxpemF0aW9uIHJhdGlvAAAADGtpbmsyX3VyX2JwcwAAAAsAAAA5QVBSIHRoYXQgaXMgYWNjcnVlZCB3aGVuIHRoZSB1dGlsaXphdGlvbiByYXRpbyBpcyBhdCAxMDAlAAAAAAAAC21heF9hcHJfYnBzAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAACFBvb2xEYXRhAAAABwAAAAAAAAADYXB5AAAAB9AAAAAWQW5udWFsUGVyY2VudGFnZVlpZWxkcwAAAAAAAAAAABVkX3Rva2VuX3JhdGVfY2VpbF9icHMAAAAAAAALAAAAAAAAABZqX3Rva2VuX3JhdGVfZmxvb3JfYnBzAAAAAAALAAAAAAAAABJvcmFjbGVfYXNzZXRfcHJpY2UAAAAAAAsAAAAAAAAABHBvb2wAAAfQAAAABFBvb2wAAAAAAAAAGHRvdGFsX2F2YWlsYWJsZV9hZGp1c3RlZAAAAAsAAAAAAAAADHRvdGFsX3N1cHBseQAAAAs=",
        "AAAAAQAAAAAAAAAAAAAACk1hcmtldERhdGEAAAAAAAMAAAAAAAAADGdsb2JhbF9zdGF0ZQAAB9AAAAALR2xvYmFsU3RhdGUAAAAAAAAAABVvcmFjbGVfcHJpY2VfZGVjaW1hbHMAAAAAAAAEAAAAAAAAAApwb29sc19kYXRhAAAAAAPqAAAH0AAAAAhQb29sRGF0YQ==",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAAVAAAAAAAAAA9iYWRfZGVidF9sb2NrX2QAAAAABgAAAAAAAAAWYmFkX2RlYnRfcmVxdWVzdF9jb3VudAAAAAAABAAAAAAAAAAOYm9ycm93X2Fwcl9icHMAAAAAAAsAAAAAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAAAAAAACWZhcm1fZGVidAAAAAAAA+gAAAPuAAAAIAAAAAAAAAALZmFybV9zdXBwbHkAAAAD6AAAA+4AAAAgAAAAAAAAABppbnRlcmVzdF9yYXRlX21vZGlmaWVyX2JwcwAAAAAACwAAAAAAAAAWbGFzdF9hY2NydWFsX3RpbWVzdGFtcAAAAAAABgAAAAAAAAAEbmFtZQAAABAAAAAAAAAAEm9wZXJhdGlvbl9mZWVzX3N1bQAAAAAACwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAOc3VwcGx5X2Fwcl9icHMAAAAAAAsAAAAAAAAAEnRha2VfcmF0ZV9mZWVzX3N1bQAAAAAACwAAAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAAAAAADnRva2VuX2RlY2ltYWxzAAAAAAAEAAAAAAAAAAx0b2tlbl9zeW1ib2wAAAAQAAAAAAAAAA90b3RhbF9hdmFpbGFibGUAAAAACwAAAAAAAAAOdG90YWxfYm9ycm93ZWQAAAAAAAsAAAAAAAAAEHRvdGFsX2NvbGxhdGVyYWwAAAALAAAAAAAAAA50b3RhbF9kX3Rva2VucwAAAAAACwAAAAAAAAAOdG90YWxfal90b2tlbnMAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAcAAAAAAAAADWFjY3J1YWxfbW9kZWwAAAAAAAfQAAAADEFjY3J1YWxNb2RlbAAAAAAAAAAKZmVlX2NvbmZpZwAAAAAH0AAAAA1Qb29sRmVlQ29uZmlnAAAAAAAAAAAAAA1oZWFsdGhfY29uZmlnAAAAAAAH0AAAABBQb29sSGVhbHRoQ29uZmlnAAAAAAAAABNpbnRlcmVzdF9yYXRlX21vZGVsAAAAB9AAAAARSW50ZXJlc3RSYXRlTW9kZWwAAAAAAAAAAAAAFmlyX3JlYWN0aXZpdHlfY29uc3RhbnQAAAAAAAQAAAAAAAAABnN0YXR1cwAAAAAH0AAAAApQb29sU3RhdHVzAAAAAAAAAAAAHHRhcmdldF91dGlsaXphdGlvbl9yYXRpb19icHMAAAAL",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xTdGF0dXMAAAAAAAEAAAAAAAAABWZsYWdzAAAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAADVBvb2xGZWVDb25maWcAAAAAAAAMAAAAAAAAABZhZGRfY29sbGF0ZXJhbF9mZWVfYnBzAAAAAAAEAAAAAAAAAA5ib3Jyb3dfZmVlX2JwcwAAAAAABAAAAAAAAAAPZGVwb3NpdF9mZWVfYnBzAAAAAAQAAAAAAAAAEmZsYXNoX2xvYW5fZmVlX2JwcwAAAAAABAAAAAAAAAAbb3BlcmF0aW9uX2ZlZV9iZW5lZmljaWFyaWVzAAAAA+gAAAPsAAAAEwAAAAQAAAAAAAAACXJlZmVycmVycwAAAAAAA+gAAAPsAAAAEwAAAAQAAAAAAAAAGXJlbW92ZV9jb2xsYXRlcmFsX2ZlZV9icHMAAAAAAAAEAAAAAAAAAA1yZXBheV9mZWVfYnBzAAAAAAAABAAAAAAAAAAXdGFrZV9yYXRlX2JlbmVmaWNpYXJpZXMAAAAD6AAAA+wAAAATAAAABAAAAAAAAAANdGFrZV9yYXRlX2JwcwAAAAAAAAQAAAAAAAAAEHdpdGhkcmF3X2ZlZV9icHMAAAAEAAAAAAAAAB13aXRoZHJhd19tYXhfc2NhcmNpdHlfZmVlX2JwcwAAAAAAAAQ=",
        "AAAAAQAAAAAAAAAAAAAAEFBvb2xIZWFsdGhDb25maWcAAAAJAAAAAAAAAA1jbG9zZV9sdHZfYnBzAAAAAAAACwAAAAAAAAAUbGlhYmlsaXR5X2ZhY3Rvcl9icHMAAAALAAAAAAAAABxsaXF1aWRhdGlvbl9jbG9zZV9mYWN0b3JfYnBzAAAACwAAAAAAAAAdbWF4X2xpcXVpZGF0aW9uX2luY2VudGl2ZV9icHMAAAAAAAALAAAAAAAAAAxvcGVuX2x0dl9icHMAAAALAAAAAAAAAAxzdXBwbHlfbGltaXQAAAALAAAAAAAAABt1dGlsaXphdGlvbl9yYXRpb19saW1pdF9icHMAAAAACwAAAAAAAAAcd2l0aGRyYXdfc2NhcmNpdHlfY29vbGRvd25fcwAAAAYAAAAAAAAAG3dpdGhkcmF3X3NjYXJjaXR5X2xpbWl0X2JwcwAAAAAL",
        "AAAABAAAAAAAAAAAAAAAB01DRXJyb3IAAAAAMgAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAEkludmFsaWRJbnB1dEFtb3VudAAAAAAAAQAAAAAAAAAXRGVwZW5kZW5jeUNvbnRyYWN0RXJyb3IAAAAAAgAAAAAAAAAQTWFya2V0SXNOb3RPd25lZAAAAAMAAAAAAAAAF0JvcnJvd0ZvcmJpZGRlbk9uTWFya2V0AAAAAAQAAAAAAAAAGERlcG9zaXRGb3JiaWRkZW5Pbk1hcmtldAAAAAUAAAAAAAAADk1hcmtldElzRnJvemVuAAAAAAAGAAAAAAAAABtJbnZhbGlkTWFya2V0Q29uZmlnT3JVcGRhdGUAAAAABwAAAAAAAAAUSW5jb3JyZWN0UmVxdWVzdFR5cGUAAAAIAAAAAAAAAA9PdmVyT3JVbmRlcmZsb3cAAAAACQAAAAAAAAAQVG9vTWFueVBvc2l0aW9ucwAAAAoAAAAAAAAAGk1pbkNvbGxhdGVyYWxWYWx1ZUlzTm90TWV0AAAAAAALAAAAAAAAABdOb25Qb3NpdGl2ZVNoYXJlc0Ftb3VudAAAAAAMAAAAAAAAABVJbnZhbGlkSW5pdGlhbGl6YXRpb24AAAAAAABkAAAAAAAAABBQb29sRG9lc05vdEV4aXN0AAAAZQAAAAAAAAAVSW52YWxpZExvYW5Qb29sQ29uZmlnAAAAAAAAZgAAAAAAAAASTm90RW5vdWdoUG9vbEZ1bmRzAAAAAABnAAAAAAAAABdEZXBvc2l0UG9vbERvZXNOb3RFeGlzdAAAAABoAAAAAAAAABZCb3Jyb3dQb29sRG9lc05vdEV4aXN0AAAAAABpAAAAAAAAABpDb2xsYXRlcmFsUG9vbERvZXNOb3RFeGlzdAAAAAAAagAAAAAAAAAgUG9vbEFscmVhZHlDb250YWluc1F1ZXVlZFBvb2xTZXQAAABrAAAAAAAAABxQb29sRG9lc05vdEhhdmVRdWV1ZWRQb29sU2V0AAAAbAAAAAAAAAAZUG9vbFNldElzTm90WWV0QXBwbGljYWJsZQAAAAAAAG0AAAAAAAAAGE9wZXJhdGlvbkZvcmJpZGRlbk9uUG9vbAAAAG4AAAAAAAAAKU1hcmtldEFscmVhZHlDb250YWluc1F1ZXVlZEluQ29uZmlnVXBkYXRlAAAAAAAAbwAAAAAAAAAlTWFya2V0RG9lc05vdEhhdmVRdWV1ZWRJbkNvbmZpZ1VwZGF0ZQAAAAAAAHAAAAAAAAAAJE1hcmtldENvbmZpZ1VwZGF0ZUlzTm90WWV0QXBwbGljYWJsZQAAAHEAAAAAAAAAEVBvb2xCYWREZWJ0TG9ja2VkAAAAAAAAcgAAAAAAAAAWT2JsaWdhdGlvbkRvZXNOb3RFeGlzdAAAAAAAyAAAAAAAAAAbRGVwb3NpdFBvc2l0aW9uRG9lc05vdEV4aXN0AAAAAMkAAAAAAAAAGkJvcnJvd1Bvc2l0aW9uRG9lc05vdEV4aXN0AAAAAADKAAAAAAAAABlXaXRoZHJhd1NjYXJjaXR5T3ZlckxpbWl0AAAAAAAAywAAAAAAAAAWU2NhcmNpdHlDb29sZG93blBlcmlvZAAAAAAAzAAAAAAAAAAcQm9ycm93UG9zaXRpb25Gb3JBc3NldEV4aXN0cwAAAM0AAAAAAAAAHURlcG9zaXRQb3NpdGlvbkZvckFzc2V0RXhpc3RzAAAAAAAAzgAAAAAAAAASVW5oZWFsdGh5T3BlcmF0aW9uAAAAAADPAAAAAAAAABdQb29sU3VwcGx5TGltaXRFeGNlZWRlZAAAAAGQAAAAAAAAAB9Qb29sVXRpbGl6YXRpb25SYXRpb0NhcEV4Y2VlZGVkAAAAAZEAAAAAAAAAG09yYWNsZURvZXNOb3RLbm93QXNzZXRQcmljZQAAAAH0AAAAAAAAABBPcmFjbGVTdGFsZVByaWNlAAAB9QAAAAAAAAAWTm9uUG9zaXRpdmVPcmFjbGVQcmljZQAAAAAB9gAAAAAAAAAYSW52YWxpZExpcXVpZGF0aW9uSW5wdXRzAAACWAAAAAAAAAATT2JsaWdhdGlvbklzSGVhbHRoeQAAAAJZAAAAAAAAACpPYmxpZ2F0aW9uQ29udGFpbnNPcGVuQ292ZXJCYWREZWJ0UmVxdWVzdHMAAAAAAloAAAAAAAAAIEJhZERlYnRDb3ZlcmFnZUNyaXRlcmlvbklzTm90TWV0AAACWwAAAAAAAAAdQXNzZXRDYW5ub3RCZVVzZWRBc0NvbGxhdGVyYWwAAAAAAAJcAAAAAAAAACZMaXF1aWRhdGlvbkV4Y2Vzc2l2ZURlbWFuZGVkQ29sbGF0ZXJhbAAAAAACXQAAAAAAAAALSW52YWxpZFN3YXAAAAACvQAAAAAAAAAcRmxhc2hCb3Jyb3dBbHJlYWR5UmVnaXN0ZXJlZAAAAr4AAAAAAAAAFFN3YXBTbGlwcGFnZUV4Y2VlZGVkAAACvw==",
        "AAAABQAAAAAAAAAAAAAACVN3YXBFeGFjdAAAAAAAAAEAAAAKc3dhcF9leGFjdAAAAAAABgAAAAAAAAANc3dhcF9wcm92aWRlcgAAAAAAABMAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAEAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAAAAAAAAAAAAA5taW5fYW1vdW50X291dAAAAAAACwAAAAAAAAAAAAAAD3JlY2VpdmVkX2Ftb3VudAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAClJlcGF5RXZlbnQAAAAAAAEAAAALcmVwYXlfZXZlbnQAAAAABAAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAApvYmxpZ2F0aW9uAAAAAAfQAAAACk9ibGlnYXRpb24AAAAAAAAAAAAAAAAADHJlcGF5X3Jlc3VsdAAAB9AAAAALUmVwYXlSZXN1bHQAAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAC0JvcnJvd0V2ZW50AAAAAAEAAAAMYm9ycm93X2V2ZW50AAAABAAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAApvYmxpZ2F0aW9uAAAAAAfQAAAACk9ibGlnYXRpb24AAAAAAAAAAAAAAAAADWJvcnJvd19yZXN1bHQAAAAAAAfQAAAADEJvcnJvd1Jlc3VsdAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAC0ZsYXNoQm9ycm93AAAAAAEAAAAMZmxhc2hfYm9ycm93AAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAAAAAABHVzZXIAAAATAAAAAAAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAC1JlZnJlc2hQb29sAAAAAAEAAAAMcmVmcmVzaF9wb29sAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAADEFwcGx5UG9vbFNldAAAAAEAAAAOYXBwbHlfcG9vbF9zZXQAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADERlcG9zaXRFdmVudAAAAAEAAAANZGVwb3NpdF9ldmVudAAAAAAAAAQAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAA5vYmxpZ2F0aW9uX2tleQAAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAAKb2JsaWdhdGlvbgAAAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAAAAAAAAAAAAA5kZXBvc2l0X3Jlc3VsdAAAAAAH0AAAAA1EZXBvc2l0UmVzdWx0AAAAAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADFN3YXBGb3JFeGFjdAAAAAEAAAAOc3dhcF9mb3JfZXhhY3QAAAAAAAYAAAAAAAAADXN3YXBfcHJvdmlkZXIAAAAAAAATAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAAAAAAAARwYXRoAAAD6gAAABMAAAAAAAAAAAAAAA1tYXhfYW1vdW50X2luAAAAAAAACwAAAAAAAAAAAAAACmFtb3VudF9vdXQAAAAAAAsAAAAAAAAAAAAAAAtzZW50X2Ftb3VudAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADUNhbmNlbFBvb2xTZXQAAAAAAAABAAAAD2NhbmNlbF9wb29sX3NldAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAADUNsYWltTWlzbWF0Y2gAAAAAAAABAAAADmNsYWltX21pc21hdGNoAAAAAAAFAAAAAAAAAA5vYmxpZ2F0aW9uX2tleQAAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAACnJlcXVlc3RfaWQAAAAAAAYAAAAAAAAAAAAAAA9hcHByb3ZlZF9hbW91bnQAAAAACwAAAAAAAAAAAAAAD2FjdHVhbF9yZWNlaXZlZAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADVdpdGhkcmF3RXZlbnQAAAAAAAABAAAADndpdGhkcmF3X2V2ZW50AAAAAAAEAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAOb2JsaWdhdGlvbl9rZXkAAAAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAEAAAAAAAAACm9ibGlnYXRpb24AAAAAA+gAAAfQAAAACk9ibGlnYXRpb24AAAAAAAAAAAAAAAAAD3dpdGhkcmF3X3Jlc3VsdAAAAAfQAAAADldpdGhkcmF3UmVzdWx0AAAAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADkZsYXNoTG9hbkV2ZW50AAAAAAABAAAAEGZsYXNoX2xvYW5fZXZlbnQAAAAFAAAAAAAAAAhjb250cmFjdAAAABMAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAJaW5pdGlhdG9yAAAAAAAAEwAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAAAAAACWZlZXNfcGFpZAAAAAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAADkxpcXVpZGF0ZUV2ZW50AAAAAAABAAAAD2xpcXVpZGF0ZV9ldmVudAAAAAAHAAAAAAAAAApsaXF1aWRhdG9yAAAAAAATAAAAAQAAAAAAAAAXYm9ycm93ZXJfb2JsaWdhdGlvbl9rZXkAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAQAAAAAAAAAXY29sbGF0ZXJhbF9wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAAAAAAAE2JvcnJvd2VyX29ibGlnYXRpb24AAAAD6AAAB9AAAAAKT2JsaWdhdGlvbgAAAAAAAAAAAAAAAAAVbGlxdWlkYXRvcl9vYmxpZ2F0aW9uAAAAAAAD6AAAB9AAAAAKT2JsaWdhdGlvbgAAAAAAAAAAAAAAAAASbGlxdWlkYXRpb25fcmVzdWx0AAAAAAfQAAAAEUxpcXVpZGF0aW9uUmVzdWx0AAAAAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAADlF1ZXVlSW5Qb29sU2V0AAAAAAABAAAAEXF1ZXVlX2luX3Bvb2xfc2V0AAAAAAAAAgAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAC3Bvb2xfY29uZmlnAAAAB9AAAAAKUG9vbENvbmZpZwAAAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAD1Byb3Bvc2VOZXdBZG1pbgAAAAABAAAAEXByb3Bvc2VfbmV3X2FkbWluAAAAAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAEFBvb2xGYXJtU2V0RXZlbnQAAAABAAAAE3Bvb2xfZmFybV9zZXRfZXZlbnQAAAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAAAAAAAAAAAAAlmYXJtX2tpbmQAAAAAAAARAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAEUlzc3VlQ292ZXJCYWREZWJ0AAAAAAAAAQAAABRpc3N1ZV9jb3Zlcl9iYWRfZGVidAAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAEVBvb2xCYWREZWJ0TG9ja2VkAAAAAAAAAQAAABRwb29sX2JhZF9kZWJ0X2xvY2tlZAAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAAhkZWFkbGluZQAAAAYAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAEVJlZnJlc2hPYmxpZ2F0aW9uAAAAAAAAAQAAABJyZWZyZXNoX29ibGlnYXRpb24AAAAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAEkFkZENvbGxhdGVyYWxFdmVudAAAAAAAAQAAABRhZGRfY29sbGF0ZXJhbF9ldmVudAAAAAQAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAA5vYmxpZ2F0aW9uX2tleQAAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAAKb2JsaWdhdGlvbgAAAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAAAAAAAAAAAABVhZGRfY29sbGF0ZXJhbF9yZXN1bHQAAAAAAAfQAAAAE0FkZENvbGxhdGVyYWxSZXN1bHQAAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAEkRpc3RyaWJ1dGVQb29sRmVlcwAAAAAAAQAAABRkaXN0cmlidXRlX3Bvb2xfZmVlcwAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAEkR1cGxpY2F0ZVJlcXVlc3RJZAAAAAAAAQAAABRkdXBsaWNhdGVfcmVxdWVzdF9pZAAAAAMAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAKcmVxdWVzdF9pZAAAAAAABgAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAElVwZGF0ZU1hcmtldFN0YXR1cwAAAAAAAQAAABR1cGRhdGVfbWFya2V0X3N0YXR1cwAAAAEAAAAAAAAACm5ld19zdGF0dXMAAAAAB9AAAAAMTWFya2V0U3RhdHVzAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAE0FjY2VwdEFkbWluUHJvcG9zYWwAAAAAAQAAABVhY2NlcHRfYWRtaW5fcHJvcG9zYWwAAAAAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAE0luaXRpYWxpemVQb29sRXZlbnQAAAAAAQAAABVpbml0aWFsaXplX3Bvb2xfZXZlbnQAAAAAAAADAAAAAAAAAA10b2tlbl9hZGRyZXNzAAAAAAAAEwAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAAx0b2tlbl9zeW1ib2wAAAAQAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAE1Bvb2xCYWREZWJ0VW5sb2NrZWQAAAAAAQAAABZwb29sX2JhZF9kZWJ0X3VubG9ja2VkAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAFEluY29uc2lzdGVudFN3YXBTZW50AAAAAQAAABZpbmNvbnNpc3RlbnRfc3dhcF9zZW50AAAAAAAEAAAAAAAAAA1zd2FwX3Byb3ZpZGVyAAAAAAAAEwAAAAEAAAAAAAAABHBhdGgAAAPqAAAAEwAAAAAAAAAAAAAAC3NlbnRfYW1vdW50AAAAAAsAAAAAAAAAAAAAAA1tYXhfYW1vdW50X2luAAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAFExlZGdlclRpbWVzdGFtcEVycm9yAAAAAQAAABZsZWRnZXJfdGltZXN0YW1wX2Vycm9yAAAAAAACAAAAAAAAABFjdXJyZW50X3RpbWVzdGFtcAAAAAAAAAYAAAAAAAAAAAAAABBzdG9yZWRfdGltZXN0YW1wAAAABgAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAFUZhcm1zQ29udHJhY3RTZXRFdmVudAAAAAAAAAEAAAAYZmFybXNfY29udHJhY3Rfc2V0X2V2ZW50AAAAAQAAAAAAAAAOZmFybXNfY29udHJhY3QAAAAAABMAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAFVBvb2xGYXJtc0NsZWFyZWRFdmVudAAAAAAAAAEAAAAYcG9vbF9mYXJtc19jbGVhcmVkX2V2ZW50AAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAFVJlbW92ZUNvbGxhdGVyYWxFdmVudAAAAAAAAAEAAAAXcmVtb3ZlX2NvbGxhdGVyYWxfZXZlbnQAAAAABAAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAApvYmxpZ2F0aW9uAAAAAAPoAAAH0AAAAApPYmxpZ2F0aW9uAAAAAAAAAAAAAAAAABhyZW1vdmVfY29sbGF0ZXJhbF9yZXN1bHQAAAfQAAAAFlJlbW92ZUNvbGxhdGVyYWxSZXN1bHQAAAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAFkZ1bmRVcGRhdGVNYXJrZXRTdGF0dXMAAAAAAAEAAAAZZnVuZF91cGRhdGVfbWFya2V0X3N0YXR1cwAAAAAAAAEAAAAAAAAACm5ld19zdGF0dXMAAAAAB9AAAAAMTWFya2V0U3RhdHVzAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAFlBvb2xJc01pc3NpbmdJblN0b3JhZ2UAAAAAAAEAAAAacG9vbF9pc19taXNzaW5nX2luX3N0b3JhZ2UAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAF0FwcGx5TWFya2V0Q29uZmlnVXBkYXRlAAAAAAEAAAAaYXBwbHlfbWFya2V0X2NvbmZpZ191cGRhdGUAAAAAAAMAAAAAAAAAEW5ld19tYXhfcG9zaXRpb25zAAAAAAAABAAAAAAAAAAAAAAAHm5ld19taW5fY29sbGF0ZXJhbF92YWx1ZV9jZW50cwAAAAAACwAAAAAAAAAAAAAAE25ld19iYWRfZGVidF9sb2NrX2QAAAAABgAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAF0JhZERlYnRSZXF1ZXN0Q2FuY2VsbGVkAAAAAAEAAAAaYmFkX2RlYnRfcmVxdWVzdF9jYW5jZWxsZWQAAAAAAAMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAApyZXF1ZXN0X2lkAAAAAAAGAAAAAAAAAJhgdHJ1ZWAgaWYgdGhlIHJlcXVlc3Qgd2FzIG1pc3NpbmcgZnJvbSB0aGUgSW5zdXJhbmNlIEZ1bmQgKGUuZy4gYXJjaGl2ZWQpOwpgZmFsc2VgIGlmIGl0IHdhcyBzdGlsbCBQZW5kaW5nIHBhc3QgdGhlIGRlYWRsaW5lIGFuZCB3YXMgYWN0aXZlbHkgY2FuY2VsbGVkLgAAAAdtaXNzaW5nAAAAAAEAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAGENhbmNlbE1hcmtldENvbmZpZ1VwZGF0ZQAAAAEAAAAbY2FuY2VsX21hcmtldF9jb25maWdfdXBkYXRlAAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAGENsYWltQ292ZXJCYWREZWJ0UmVzdWx0cwAAAAEAAAAcY2xhaW1fY292ZXJfYmFkX2RlYnRfcmVzdWx0cwAAAAEAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAGEluY29uc2lzdGVudFN3YXBSZWNlaXZlZAAAAAEAAAAaaW5jb25zaXN0ZW50X3N3YXBfcmVjZWl2ZWQAAAAAAAQAAAAAAAAADXN3YXBfcHJvdmlkZXIAAAAAAAATAAAAAQAAAAAAAAAEcGF0aAAAA+oAAAATAAAAAAAAAAAAAAAPcmVjZWl2ZWRfYW1vdW50AAAAAAsAAAAAAAAAAAAAAA5taW5fYW1vdW50X291dAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAGE5vblBvc2l0aXZlRFRva2Vuc0JvcnJvdwAAAAEAAAAcbm9uX3Bvc2l0aXZlX2RfdG9rZW5zX2JvcnJvdwAAAAMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAABZtaW50ZWRfZF90b2tlbnNfYW1vdW50AAAAAAALAAAAAAAAAAAAAAAUcmVhbF9ib3Jyb3dlZF9hbW91bnQAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAGFNldFRha2VSYXRlQmVuZWZpY2lhcmllcwAAAAEAAAAbc2V0X3Rha2VfcmF0ZV9iZW5lZmljaWFyaWVzAAAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAA1iZW5lZmljaWFyaWVzAAAAAAAD7AAAABMAAAAEAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAGUZhcm1zQ29udHJhY3RDbGVhcmVkRXZlbnQAAAAAAAABAAAAHGZhcm1zX2NvbnRyYWN0X2NsZWFyZWRfZXZlbnQAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAGVBvb2xBbW91bnRCZWNvbWVzTmVnYXRpdmUAAAAAAAABAAAAHHBvb2xfYW1vdW50X2JlY29tZXNfbmVnYXRpdmUAAAACAAAAAAAAAApvbGRfYW1vdW50AAAAAAALAAAAAAAAAAAAAAAKbmV3X2Ftb3VudAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAGVF1ZXVlSW5NYXJrZXRDb25maWdVcGRhdGUAAAAAAAABAAAAHXF1ZXVlX2luX21hcmtldF9jb25maWdfdXBkYXRlAAAAAAAAAwAAAAAAAAARbmV3X21heF9wb3NpdGlvbnMAAAAAAAAEAAAAAAAAAAAAAAAebmV3X21pbl9jb2xsYXRlcmFsX3ZhbHVlX2NlbnRzAAAAAAALAAAAAAAAAAAAAAATbmV3X2JhZF9kZWJ0X2xvY2tfZAAAAAAGAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAGVJlZmVycmVyRmVlRXhjZWVkc09wc0ZlZXMAAAAAAAABAAAAHXJlZmVycmVyX2ZlZV9leGNlZWRzX29wc19mZWVzAAAAAAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAEm9wZXJhdGlvbl9mZWVzX3N1bQAAAAAACwAAAAAAAAAAAAAADHJlZmVycmVyX2ZlZQAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAGkNvbXB1dGVkSW50ZXJlc3RJc05lZ2F0aXZlAAAAAAABAAAAHWNvbXB1dGVkX2ludGVyZXN0X2lzX25lZ2F0aXZlAAAAAAAABAAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAD3Bvc2l0aW9uX3NoYXJlcwAAAAALAAAAAAAAAAAAAAAXdG9rZW5zX2Zyb21fc2hhcmVzX2NlaWwAAAAACwAAAAAAAAAAAAAAEWNvbXB1dGVkX2ludGVyZXN0AAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAGk5vblBvc2l0aXZlSlRva2Vuc1dpdGhkcmF3AAAAAAABAAAAHm5vbl9wb3NpdGl2ZV9qX3Rva2Vuc193aXRoZHJhdwAAAAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAFmJ1cm5lZF9qX3Rva2Vuc19hbW91bnQAAAAAAAsAAAAAAAAAAAAAABBkZXBvc2l0X2RlY3JlYXNlAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAG0luc3VyYW5jZUZ1bmRNaXNzaW5nUmVxdWVzdAAAAAABAAAAHmluc3VyYW5jZV9mdW5kX21pc3NpbmdfcmVxdWVzdAAAAAAAAwAAAAAAAAAOb2JsaWdhdGlvbl9rZXkAAAAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAApyZXF1ZXN0X2lkAAAAAAAGAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAG1Bvb2xJbmNvbnNpc3RlbnRUb3RhbFNoYXJlcwAAAAABAAAAHnBvb2xfaW5jb25zaXN0ZW50X3RvdGFsX3NoYXJlcwAAAAAAAgAAAAAAAAAMdG90YWxfc2hhcmVzAAAACwAAAAAAAAAAAAAAEWluZGl2aWR1YWxfc2hhcmVzAAAAAAAACwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAG1Bvb2xJbmNvbnNpc3RlbnRUb3RhbFRva2VucwAAAAABAAAAHnBvb2xfaW5jb25zaXN0ZW50X3RvdGFsX3Rva2VucwAAAAAAAgAAAAAAAAAMdG90YWxfc2hhcmVzAAAACwAAAAAAAAAAAAAADHRvdGFsX3Rva2VucwAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHFV0aWxpemF0aW9uUmF0aW9FeGNlZWRzTGltaXQAAAABAAAAH3V0aWxpemF0aW9uX3JhdGlvX2V4Y2VlZHNfbGltaXQAAAAAAgAAAAAAAAAVdXRpbGl6YXRpb25fcmF0aW9fYnBzAAAAAAAACwAAAAAAAAAAAAAAG3V0aWxpemF0aW9uX3JhdGlvX2xpbWl0X2JwcwAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAHUluY29uc2lzdGVudEltbWVkaWF0ZUNvdmVyYWdlAAAAAAAAAQAAAB9pbmNvbnNpc3RlbnRfaW1tZWRpYXRlX2NvdmVyYWdlAAAAAAQAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAMYmFsYW5jZV9kaWZmAAAACwAAAAEAAAAAAAAAC2RlYnRfYW1vdW50AAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHU9ibGlnYXRpb25BbW50QmVjb21lc05lZ2F0aXZlAAAAAAAAAQAAACBvYmxpZ2F0aW9uX2FtbnRfYmVjb21lc19uZWdhdGl2ZQAAAAIAAAAAAAAACm9sZF9hbW91bnQAAAAAAAsAAAAAAAAAAAAAAApuZXdfYW1vdW50AAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAHU9ibGlnYXRpb25GYXJtc1JlZnJlc2hlZEV2ZW50AAAAAAAAAQAAACBvYmxpZ2F0aW9uX2Zhcm1zX3JlZnJlc2hlZF9ldmVudAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAQAAAAAAAAAQbnVtX3N1cHBseV9mYXJtcwAAAAQAAAAAAAAAAAAAAA5udW1fZGVidF9mYXJtcwAAAAAABAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAHU9ibGlnYXRpb25Jc1VuZXhwZWN0ZWRseUVtcHR5AAAAAAAAAQAAACBvYmxpZ2F0aW9uX2lzX3VuZXhwZWN0ZWRseV9lbXB0eQAAAAIAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAHVBvb2xDb250YWluc0luY29uc2lzdGVudFN0YXRlAAAAAAAAAQAAACBwb29sX2NvbnRhaW5zX2luY29uc2lzdGVudF9zdGF0ZQAAAAEAAAAAAAAABHBvb2wAAAfQAAAABFBvb2wAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAHVBvc2l0aW9uc0NvdW50QmVjb21lc05lZ2F0aXZlAAAAAAAAAQAAACBwb3NpdGlvbnNfY291bnRfYmVjb21lc19uZWdhdGl2ZQAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAApvYmxpZ2F0aW9uAAAAAAfQAAAACk9ibGlnYXRpb24AAAAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAHVJlZmVycmVySXNVbmV4cGVjdGVkbHlNaXNzaW5nAAAAAAAAAQAAACByZWZlcnJlcl9pc191bmV4cGVjdGVkbHlfbWlzc2luZwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAHVNldE9wZXJhdGlvbkZlZXNCZW5lZmljaWFyaWVzAAAAAAAAAQAAACBzZXRfb3BlcmF0aW9uX2ZlZXNfYmVuZWZpY2lhcmllcwAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAA1iZW5lZmljaWFyaWVzAAAAAAAD7AAAABMAAAAEAAAAAAAAAAI=",
        "AAAAAgAAAAAAAAAAAAAADEFjY3J1YWxNb2RlbAAAAAEAAAAAAAAAAAAAAApDb21wb3VuZGVkAAA=",
        "AAAAAgAAAAAAAAAAAAAAB1JlcXVlc3QAAAAACgAAAAEAAAAAAAAAB0RlcG9zaXQAAAAAAQAAB9AAAAAPU3RhbmRhcmRSZXF1ZXN0AAAAAAEAAAAAAAAABkJvcnJvdwAAAAAAAQAAB9AAAAAPU3RhbmRhcmRSZXF1ZXN0AAAAAAEAAAAAAAAACFdpdGhkcmF3AAAAAQAAB9AAAAAPU3RhbmRhcmRSZXF1ZXN0AAAAAAEAAAAAAAAABVJlcGF5AAAAAAAAAQAAB9AAAAAPU3RhbmRhcmRSZXF1ZXN0AAAAAAEAAAAAAAAADUFkZENvbGxhdGVyYWwAAAAAAAABAAAH0AAAAA9TdGFuZGFyZFJlcXVlc3QAAAAAAQAAAAAAAAAQUmVtb3ZlQ29sbGF0ZXJhbAAAAAEAAAfQAAAAD1N0YW5kYXJkUmVxdWVzdAAAAAABAAAAAAAAAAtGbGFzaEJvcnJvdwAAAAABAAAH0AAAAA9TdGFuZGFyZFJlcXVlc3QAAAAAAQAAAAAAAAAPU3dhcEV4YWN0VG9rZW5zAAAAAAEAAAfQAAAAFlN3YXBFeGFjdFRva2Vuc1JlcXVlc3QAAAAAAAEAAAAAAAAAElN3YXBGb3JFeGFjdFRva2VucwAAAAAAAQAAB9AAAAAZU3dhcEZvckV4YWN0VG9rZW5zUmVxdWVzdAAAAAAAAAEAAAAAAAAACUxpcXVpZGF0ZQAAAAAAAAEAAAfQAAAAEExpcXVpZGF0ZVJlcXVlc3Q=",
        "AAAAAQAAAAAAAAAAAAAAD1N0YW5kYXJkUmVxdWVzdAAAAAACAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAADHBvb2xfYWRkcmVzcwAAABM=",
        "AAAAAQAAAAAAAAAAAAAAEExpcXVpZGF0ZVJlcXVlc3QAAAAFAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAAAAAAAF2JvcnJvd2VyX29ibGlnYXRpb25fa2V5AAAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAAAAAAXY29sbGF0ZXJhbF9wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAebWluX2RlbWFuZGVkX2NvbGxhdGVyYWxfYW1vdW50AAAAAAALAAAAAAAAAAxyZXBheV9hbW91bnQAAAAL",
        "AAAAAQAAAAAAAAAAAAAAFlN3YXBFeGFjdFRva2Vuc1JlcXVlc3QAAAAAAAQAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAAAAAAADm1pbl9hbW91bnRfb3V0AAAAAAALAAAAAAAAAARwYXRoAAAD6gAAABMAAAAAAAAADXN3YXBfcHJvdmlkZXIAAAAAAAAT",
        "AAAAAQAAAAAAAAAAAAAAGVN3YXBGb3JFeGFjdFRva2Vuc1JlcXVlc3QAAAAAAAAEAAAAAAAAAAphbW91bnRfb3V0AAAAAAALAAAAAAAAAA1tYXhfYW1vdW50X2luAAAAAAAACwAAAAAAAAAEcGF0aAAAA+oAAAATAAAAAAAAAA1zd2FwX3Byb3ZpZGVyAAAAAAAAEw==",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAFgAAAAAAAAAAAAAABE5hbWUAAAAAAAAAAAAAAAVBZG1pbgAAAAAAAAAAAAAAAAAABk9yYWNsZQAAAAAAAAAAAAAAAAAHQWNjcnVhbAAAAAAAAAAAAAAAAAdJc093bmVkAAAAAAAAAAAAAAAACEFsbFBvb2xzAAAAAAAAAAAAAAALR2xvYmFsU3RhdGUAAAAAAAAAAAAAAAAMRGVwbG95ZXJIb3N0AAAAAAAAAAAAAAAMTWF4UG9zaXRpb25zAAAAAAAAAAAAAAAMTWFya2V0U3RhdHVzAAAAAAAAAAAAAAANRmFybXNDb250cmFjdAAAAAAAAAEAAAAAAAAADVF1ZXVlZFBvb2xTZXQAAAAAAAABAAAAEwAAAAEAAAAAAAAABFBvb2wAAAABAAAAEwAAAAAAAAAAAAAADUluc3VyYW5jZUZ1bmQAAAAAAAAAAAAAAAAAABBJbnNvbHZlbmN5THR2QnBzAAAAAAAAAAAAAAASRWFybk9ibGlnYXRpb25TZWVkAAAAAAAAAAAAAAAAABdNaW5Db2xsYXRlcmFsVmFsdWVDZW50cwAAAAAAAAAAAAAAABNVcGRhdGVJblF1ZXVlUGVyaW9kAAAAAAAAAAAAAAAAEk1hcmtldENvbmZpZ1VwZGF0ZQAAAAAAAQAAAAAAAAAKT2JsaWdhdGlvbgAAAAAAAQAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAAAAAAAAAAADVByb3Bvc2VkQWRtaW4AAAAAAAAAAAAAAAAAABNCYWREZWJ0TG9ja0R1cmF0aW9uAA==",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAwAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAPYmFkX2RlYnRfbG9ja19kAAAAAAYAAAAAAAAACGRlcGxveWVyAAAAEwAAAAAAAAASaW5zb2x2ZW5jeV9sdHZfYnBzAAAAAAALAAAAAAAAAA5pbnN1cmFuY2VfZnVuZAAAAAAAEwAAAAAAAAAIaXNfb3duZWQAAAABAAAAAAAAAA1tYXhfcG9zaXRpb25zAAAAAAAABAAAAAAAAAAabWluX2NvbGxhdGVyYWxfdmFsdWVfY2VudHMAAAAAAAsAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAZvcmFjbGUAAAAAABMAAAAAAAAABnN0YXR1cwAAAAAABAAAAAAAAAAWdXBkYXRlX2luX3F1ZXVlX3BlcmlvZAAAAAAABg==",
        "AAAAAgAAAAAAAAAAAAAADE1hcmtldFN0YXR1cwAAAAcAAAAAAAAAAAAAAAZBY3RpdmUAAAAAAAAAAAAAAAAADEJvcnJvd0Zyb3plbgAAAAAAAAAAAAAAE0JvcnJvd0Zyb3plbkJ5QWRtaW4AAAAAAAAAAAAAAAANRGVwb3NpdEZyb3plbgAAAAAAAAAAAAAAAAAAFERlcG9zaXRGcm96ZW5CeUFkbWluAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAA1Gcm96ZW5CeUFkbWluAAAA",
        "AAAAAQAAAAAAAAAAAAAADE1hcmtldFVwZGF0ZQAAAAQAAAAAAAAAE25ld19iYWRfZGVidF9sb2NrX2QAAAAABgAAAAAAAAARbmV3X21heF9wb3NpdGlvbnMAAAAAAAAEAAAAAAAAAB5uZXdfbWluX2NvbGxhdGVyYWxfdmFsdWVfY2VudHMAAAAAAAsAAAAAAAAAE3F1ZXVlZF9pbl90aW1lc3RhbXAAAAAABg==",
        "AAAAAQAAAAAAAAAAAAAADVF1ZXVlZFBvb2xTZXQAAAAAAAACAAAAAAAAAApuZXdfY29uZmlnAAAAAAfQAAAAClBvb2xDb25maWcAAAAAAAAAAAATcXVldWVkX2luX3RpbWVzdGFtcAAAAAAG",
        "AAAAAQAAAAAAAAAAAAAAEE1hcmtldEluaXRQYXJhbXMAAAAGAAAAAAAAAA9iYWRfZGVidF9sb2NrX2QAAAAABgAAAAAAAAASaW5zb2x2ZW5jeV9sdHZfYnBzAAAAAAALAAAAAAAAAAhpc19vd25lZAAAAAEAAAAAAAAADW1heF9wb3NpdGlvbnMAAAAAAAAEAAAAAAAAABptaW5fY29sbGF0ZXJhbF92YWx1ZV9jZW50cwAAAAAACwAAAAAAAAAWdXBkYXRlX2luX3F1ZXVlX3BlcmlvZAAAAAAABg==",
        "AAAAAAAAAAAAAAAFcmVwYXkAAAAAAAAEAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAGYm9ycm93AAAAAAAEAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAAEAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAA",
        "AAAAAAAAAAAAAAAIZ2V0X3Bvb2wAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAABFBvb2wAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAId2l0aGRyYXcAAAAEAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAJbGlxdWlkYXRlAAAAAAAABgAAAAAAAAAKbGlxdWlkYXRvcgAAAAAAEwAAAAAAAAAIYm9ycm93ZXIAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAXY29sbGF0ZXJhbF9wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAMcmVwYXlfYW1vdW50AAAACwAAAAAAAAAebWluX2RlbWFuZGVkX2NvbGxhdGVyYWxfYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAKZmxhc2hfbG9hbgAAAAAABAAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAMcmVmcmVzaF9wb29sAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAYAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABm9yYWNsZQAAAAAAEwAAAAAAAAAOaW5zdXJhbmNlX2Z1bmQAAAAAABMAAAAAAAAACGRlcGxveWVyAAAAEwAAAAAAAAAGcGFyYW1zAAAAAAfQAAAAEE1hcmtldEluaXRQYXJhbXMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAANZ2V0X2FsbF9wb29scwAAAAAAAAAAAAABAAAD6gAAABM=",
        "AAAAAAAAAAAAAAANZ2V0X3Bvb2xfZGF0YQAAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAB9AAAAAIUG9vbERhdGEAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAOYWRkX2NvbGxhdGVyYWwAAAAAAAQAAAAAAAAABHVzZXIAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAIcmVmZXJyZXIAAAPoAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAOYXBwbHlfcG9vbF9zZXQAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAPY2FuY2VsX3Bvb2xfc2V0AAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAPZ2V0X21hcmtldF9kYXRhAAAAAAAAAAABAAAD6QAAB9AAAAAKTWFya2V0RGF0YQAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAQY2xlYXJfcG9vbF9mYXJtcwAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAQZ2V0X2dsb2JhbF9zdGF0ZQAAAAAAAAABAAAH0AAAAAtHbG9iYWxTdGF0ZQA=",
        "AAAAAAAAAAAAAAARcHJvcG9zZV9uZXdfYWRtaW4AAAAAAAABAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAARcXVldWVfaW5fcG9vbF9zZXQAAAAAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAtwb29sX2NvbmZpZwAAAAfQAAAAClBvb2xDb25maWcAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAARcmVtb3ZlX2NvbGxhdGVyYWwAAAAAAAAEAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAARc2ltdWxhdGVfd2l0aGRyYXcAAAAAAAAEAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAB9AAAAAOV2l0aGRyYXdSZXN1bHQAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAASZ2V0X2Zhcm1zX2NvbnRyYWN0AAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAAScmVmcmVzaF9vYmxpZ2F0aW9uAAAAAAABAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAASc2V0X2Zhcm1zX2NvbnRyYWN0AAAAAAABAAAAAAAAAA5mYXJtc19jb250cmFjdAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAASc2V0X3Bvb2xfZGVidF9mYXJtAAAAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAASdXBkYXRlX3Bvb2xfc3RhdHVzAAAAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAABBuZXdfc3RhdHVzX2ZsYWdzAAAABAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAATYXBwbHlfbWFya2V0X3VwZGF0ZQAAAAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAATZ2V0X3F1ZXVlZF9wb29sX3NldAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAADVF1ZXVlZFBvb2xTZXQAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAATZ2V0X3VzZXJfb2JsaWdhdGlvbgAAAAABAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAA+kAAAfQAAAACk9ibGlnYXRpb24AAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAUY2FuY2VsX21hcmtldF91cGRhdGUAAAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAUY2xlYXJfZmFybXNfY29udHJhY3QAAAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAUZGlzdHJpYnV0ZV9wb29sX2ZlZXMAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAUaXNzdWVfY292ZXJfYmFkX2RlYnQAAAABAAAAAAAAAAR1c2VyAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAUc2V0X3Bvb2xfc3VwcGx5X2Zhcm0AAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAUdXBkYXRlX21hcmtldF9zdGF0dXMAAAABAAAAAAAAAApuZXdfc3RhdHVzAAAAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAVYWNjZXB0X3Byb3Bvc2VkX2FkbWluAAAAAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAVc3VibWl0X3JlcXVlc3RzX2JhdGNoAAAAAAAAAwAAAAAAAAAEdXNlcgAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAAAAAAIcmVxdWVzdHMAAAPqAAAH0AAAAAdSZXF1ZXN0AAAAAAAAAAAIcmVmZXJyZXIAAAPoAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAWcXVldWVfaW5fbWFya2V0X3VwZGF0ZQAAAAAAAwAAAAAAAAARbmV3X21heF9wb3NpdGlvbnMAAAAAAAAEAAAAAAAAAB5uZXdfbWluX2NvbGxhdGVyYWxfdmFsdWVfY2VudHMAAAAAAAsAAAAAAAAAE25ld19iYWRfZGVidF9sb2NrX2QAAAAABgAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAYcmVmcmVzaF9vYmxpZ2F0aW9uX2Zhcm1zAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAZZGlzdHJpYnV0ZV9hbGxfcG9vbHNfZmVlcwAAAAAAAAAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAZZnVuZF91cGRhdGVfbWFya2V0X3N0YXR1cwAAAAAAAAEAAAAAAAAACm5ld19zdGF0dXMAAAAAAAQAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAZZ2V0X29yYWNsZV9wcmljZV9kZWNpbWFscwAAAAAAAAAAAAABAAAABA==",
        "AAAAAAAAAAAAAAAbZ2V0X21hcmtldF9xdWV1ZWRfaW5fdXBkYXRlAAAAAAAAAAABAAAD6QAAB9AAAAAMTWFya2V0VXBkYXRlAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAbZ2V0X3Bvb2xfYXNzZXRfb3JhY2xlX3ByaWNlAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAAAsAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAcY2xhaW1fY292ZXJfYmFkX2RlYnRfcmVzdWx0cwAAAAEAAAAAAAAABHVzZXIAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAgc2V0X29wZXJhdGlvbl9mZWVzX2JlbmVmaWNpYXJpZXMAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAA1iZW5lZmljaWFyaWVzAAAAAAAD7AAAABMAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAgc2V0X3Rha2VfcmF0ZV9mZWVzX2JlbmVmaWNpYXJpZXMAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAA1iZW5lZmljaWFyaWVzAAAAAAAD7AAAABMAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAQAAAP1EZWxlZ2F0ZWUgaWRlbnRpZmllciBmb3IgZmFybSBzdGFrZXMuCgpTdXBwb3J0cyBtdWx0aXBsZSBzdGFrZSBpZGVudGl0aWVzIHBlciBvd25lciBhZGRyZXNzOgotIFNpbXBsZToganVzdCBvd25lciBhZGRyZXNzIChmb3IgY29udHJhY3RzIHdoZXJlIHVzZXIgaGFzIHNpbmdsZSBwb3NpdGlvbikKLSBXaXRoIHNlZWQ6IG93bmVyIGFkZHJlc3MgKyBzZWVkIChmb3IgY29udHJhY3RzIHdpdGggbXVsdGlwbGUgb2JsaWdhdGlvbnMgcGVyIHVzZXIpAAAAAAAAAAAAAAlEZWxlZ2F0ZWUAAAAAAAACAAAAE1RoZSBvd25lcidzIGFkZHJlc3MAAAAABW93bmVyAAAAAAAAEwAAADlPcHRpb25hbCBzZWVkIHRvIGRpc3Rpbmd1aXNoIG11bHRpcGxlIHBvc2l0aW9ucyBwZXIgb3duZXIAAAAAAAAEc2VlZAAAA+gAAAPuAAAAIA==",
        "AAAAAgAAAAAAAAAAAAAADkNvdmVyYWdlU3RhdHVzAAAAAAACAAAAAAAAAAAAAAAHUGVuZGluZwAAAAABAAAAAAAAAAVSZWFkeQAAAAAAAAEAAAAL",
        "AAAAAgAAAAAAAAAAAAAAEklzc3VlUmVxdWVzdFJlc3VsdAAAAAAAAgAAAAEAAAAAAAAACFJlY29yZGVkAAAAAQAAAAYAAAABAAAAAAAAAAlJbW1lZGlhdGUAAAAAAAABAAAACw==",
        "AAAAAgAAAApBc3NldCB0eXBlAAAAAAAAAAAABUFzc2V0AAAAAAAAAgAAAAEAAAAAAAAAB1N0ZWxsYXIAAAAAAQAAABMAAAABAAAAAAAAAAVPdGhlcgAAAAAAAAEAAAAR",
        "AAAAAQAAAC9QcmljZSBkYXRhIGZvciBhbiBhc3NldCBhdCBhIHNwZWNpZmljIHRpbWVzdGFtcAAAAAAAAAAACVByaWNlRGF0YQAAAAAAAAIAAAAAAAAABXByaWNlAAAAAAAACwAAAAAAAAAJdGltZXN0YW1wAAAAAAAABg==" ]),
      options
    )
  }
  public readonly fromJSON = {
    repay: this.txFromJSON<Result<void>>,
        borrow: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        upgrade: this.txFromJSON<null>,
        get_pool: this.txFromJSON<Result<Pool>>,
        withdraw: this.txFromJSON<Result<void>>,
        liquidate: this.txFromJSON<Result<void>>,
        flash_loan: this.txFromJSON<Result<void>>,
        refresh_pool: this.txFromJSON<Result<void>>,
        get_all_pools: this.txFromJSON<Array<string>>,
        get_pool_data: this.txFromJSON<Result<PoolData>>,
        add_collateral: this.txFromJSON<Result<void>>,
        apply_pool_set: this.txFromJSON<Result<void>>,
        cancel_pool_set: this.txFromJSON<Result<void>>,
        get_market_data: this.txFromJSON<Result<MarketData>>,
        clear_pool_farms: this.txFromJSON<Result<void>>,
        get_global_state: this.txFromJSON<GlobalState>,
        propose_new_admin: this.txFromJSON<Result<void>>,
        queue_in_pool_set: this.txFromJSON<Result<void>>,
        remove_collateral: this.txFromJSON<Result<void>>,
        simulate_withdraw: this.txFromJSON<Result<WithdrawResult>>,
        get_farms_contract: this.txFromJSON<Option<string>>,
        refresh_obligation: this.txFromJSON<Result<void>>,
        set_farms_contract: this.txFromJSON<Result<void>>,
        set_pool_debt_farm: this.txFromJSON<Result<void>>,
        update_pool_status: this.txFromJSON<Result<void>>,
        apply_market_update: this.txFromJSON<Result<void>>,
        get_queued_pool_set: this.txFromJSON<Result<QueuedPoolSet>>,
        get_user_obligation: this.txFromJSON<Result<Obligation>>,
        cancel_market_update: this.txFromJSON<Result<void>>,
        clear_farms_contract: this.txFromJSON<Result<void>>,
        distribute_pool_fees: this.txFromJSON<Result<void>>,
        issue_cover_bad_debt: this.txFromJSON<Result<void>>,
        set_pool_supply_farm: this.txFromJSON<Result<void>>,
        update_market_status: this.txFromJSON<Result<void>>,
        accept_proposed_admin: this.txFromJSON<Result<void>>,
        submit_requests_batch: this.txFromJSON<Result<void>>,
        queue_in_market_update: this.txFromJSON<Result<void>>,
        refresh_obligation_farms: this.txFromJSON<Result<void>>,
        distribute_all_pools_fees: this.txFromJSON<Result<void>>,
        fund_update_market_status: this.txFromJSON<Result<void>>,
        get_oracle_price_decimals: this.txFromJSON<u32>,
        get_market_queued_in_update: this.txFromJSON<Result<MarketUpdate>>,
        get_pool_asset_oracle_price: this.txFromJSON<Result<i128>>,
        claim_cover_bad_debt_results: this.txFromJSON<Result<void>>,
        set_operation_fees_beneficiaries: this.txFromJSON<Result<void>>,
        set_take_rate_fees_beneficiaries: this.txFromJSON<Result<void>>
  }
}