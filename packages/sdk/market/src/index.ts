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
  100: {message:"InvalidInitialization"},
  101: {message:"PoolDoesNotExist"},
  102: {message:"InvalidLoanPoolConfig"},
  103: {message:"NotEnoughPoolFunds"},
  104: {message:"DepositPoolDoesNotExist"},
  105: {message:"BorrowPoolDoesNotExist"},
  106: {message:"CollateralPoolDoesNotExist"},
  107: {message:"PoolAlreadyContainsQueuedInConfigUpdate"},
  108: {message:"PoolDoesNotHaveQueuedInConfigUpdate"},
  109: {message:"PoolConfigUpdateIsNotYetApplicable"},
  110: {message:"OperationForbiddenOnPool"},
  111: {message:"InvalidBootstrapPeriod"},
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
  602: {message:"ObligationContainsOpenCoverBadDebtRequests"},
  603: {message:"BadDebtCoverageCriterionIsNotMet"},
  604: {message:"AssetCannotBeUsedAsCollateral"},
  605: {message:"LiquidationExcessiveDemandedCollateral"},
  700: {message:"InvalidLeverageInputs"},
  701: {message:"InvalidSwapSlippage"},
  702: {message:"MultiplyPairAlreadyExists"},
  703: {message:"MultiplyPairDoesNotExist"},
  704: {message:"LeveragePositionContainsBadDebt"},
  705: {message:"InconsistentDepositWithLeverage"}
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


export interface MultiplyPair {
  borrow_pool: string;
  deposit_pool: string;
  max_leverage_multiplier: u32;
  seed: Buffer;
}


export interface ObligationKey {
  seed: Option<Buffer>;
  user: string;
}


export interface Obligation {
  borrows: Map<string, BorrowPosition>;
  deposits: Map<string, DepositPosition>;
  insurance_fund_requests_ids: Map<readonly [string, u64], void>;
  positions_count: u32;
}


export interface BorrowPosition {
  d_tokens: i128;
  originally_borrowed: i128;
}


export interface DepositPosition {
  collateral: i128;
  j_tokens: i128;
  last_scarcity_withdraw_ts: u64;
  originally_deposited: i128;
}


export interface OperationFees {
  fee_sum: i128;
  referrer_fee: Option<i128>;
}


export interface DepositResult {
  deposited: i128;
  j_tokens_to_issue: i128;
  operation_fees: OperationFees;
}


export interface BorrowResult {
  borrower_new_debt: i128;
  borrower_to_receive: i128;
  d_tokens_to_issue: i128;
  operation_fees: OperationFees;
}


export interface AddCollateralResult {
  added_collateral: i128;
  operation_fees: OperationFees;
}


export interface WithdrawResult {
  deposit_decrease: i128;
  j_tokens_to_burn: i128;
  operation_fees: OperationFees;
  withdrawer_to_receive: i128;
}


export interface RepayResult {
  amount_to_send_back: i128;
  d_tokens_to_burn: i128;
  debt_repaid: i128;
  operation_fees: OperationFees;
}


export interface RemoveCollateralResult {
  collateral_decrease: i128;
  collateral_remover_to_receive: i128;
  operation_fees: OperationFees;
}


export interface LiquidationResult {
  d_tokens_burned: i128;
  debt_repaid: i128;
  j_tokens_seized: i128;
  plain_collateral_seized: i128;
  tokens_from_j_tokens_seized: i128;
}


export interface Pool {
  bootstrap_periods: Map<readonly [u64, u64], PoolBootstrapPeriod>;
  borrow_apr_bps: i128;
  config: PoolConfig;
  farm_debt: Option<Buffer>;
  farm_supply: Option<Buffer>;
  interest_rate_modifier: i128;
  last_accrual_timestamp: u64;
  name: string;
  operation_fees_sum: i128;
  pool_address: string;
  supply_apr_bps: i128;
  take_rate_fees_sum: i128;
  target_utilization_ratio_bps: i128;
  token_address: string;
  token_symbol: string;
  total_available: i128;
  total_borrowed: i128;
  total_collateral: i128;
  total_d_tokens: i128;
  total_j_tokens: i128;
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
  withdraw_scarcity_fee_sc_bps: u32;
}


export interface PoolStatus {
  flags: u32;
}


export interface PoolConfig {
  accrual_model: AccrualModel;
  fee_config: PoolFeeConfig;
  health_config: PoolHealthConfig;
  interest_rate_model: InterestRateModel;
  ir_reactivity_constant: u32;
  status: PoolStatus;
}


export interface PoolHealthConfig {
  close_ltv_bps: i128;
  insolvency_ltv_bps: i128;
  liability_factor_bps: i128;
  liquidation_close_factor_bps: i128;
  max_liquidation_incentive_bps: i128;
  open_ltv_bps: i128;
  supply_limit: i128;
  utilization_ratio_limit_bps: i128;
  withdraw_scarcity_cooldown_s: u64;
  withdraw_scarcity_limit_bps: i128;
}


export interface PoolBootstrapPeriod {
  remaining_amount: i128;
  total_amount: i128;
}


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
  RefreshFarms = 6,
}


export interface GlobalState {
  admin: string;
  deployer: string;
  insolvency_ltv_bps: i128;
  is_owned: boolean;
  max_positions: u32;
  min_collateral_value_cents: i128;
  name: string;
  oracle: string;
  status: u32;
  update_in_queue_period: Option<u64>;
}

export type MarketStatus = {tag: "Active", values: void} | {tag: "BorrowFrozen", values: void} | {tag: "BorrowFrozenByAdmin", values: void} | {tag: "DepositFrozen", values: void} | {tag: "DepositFrozenByAdmin", values: void} | {tag: "Frozen", values: void} | {tag: "FrozenByAdmin", values: void};


export interface PoolUpdate {
  new_config: PoolConfig;
  queued_in_timestamp: u64;
}

export type DataKey = {tag: "Name", values: void} | {tag: "Admin", values: void} | {tag: "Oracle", values: void} | {tag: "Accrual", values: void} | {tag: "IsOwned", values: void} | {tag: "AllPools", values: void} | {tag: "GlobalState", values: void} | {tag: "DeployerHost", values: void} | {tag: "MaxPositions", values: void} | {tag: "MarketStatus", values: void} | {tag: "FarmsContract", values: void} | {tag: "ConfigUpdate", values: readonly [string]} | {tag: "Pool", values: readonly [string]} | {tag: "InsuranceFund", values: void} | {tag: "AllObligations", values: void} | {tag: "InsolvencyLtvBps", values: void} | {tag: "AllMultiplyPairs", values: void} | {tag: "EarnObligationSeed", values: void} | {tag: "MinCollateralValueCents", values: void} | {tag: "UpdateInQueuePeriod", values: void} | {tag: "Obligation", values: readonly [ObligationKey]} | {tag: "MultiplyPair", values: readonly [readonly [string, string]]} | {tag: "ProposedAdmin", values: void};


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
  asset_decimals: u32;
  global_state: GlobalState;
  multiply_pairs: Array<MultiplyPair>;
  oracle_price_decimals: u32;
  pools_data: Array<PoolData>;
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

export type IssueRequestResult = {tag: "Recorded", values: readonly [u64]} | {tag: "Immediate", values: readonly [i128]};

export type CoverageStatus = {tag: "Pending", values: void} | {tag: "Ready", values: readonly [i128]};


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
   */
  upgrade: ({new_wasm_hash}: {new_wasm_hash: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a submit_requests_batch transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  submit_requests_batch: ({user, requests, referrer}: {user: string, requests: Array<Request>, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_global_state transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_global_state: (options?: MethodOptions) => Promise<AssembledTransaction<GlobalState>>

  /**
   * Construct and simulate a update_market transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_market: ({new_max_positions, new_min_collateral_value_cents}: {new_max_positions: u32, new_min_collateral_value_cents: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a update_market_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_market_status: ({new_status}: {new_status: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a fund_update_market_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  fund_update_market_status: ({new_status}: {new_status: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a initialize_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_pool: ({token_address, salt, pool_config}: {token_address: string, salt: Option<Buffer>, pool_config: Option<PoolConfig>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<string>>>

  /**
   * Construct and simulate a initialize_multiply_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  initialize_multiply_pair: ({deposit_pool_address, borrow_pool_address}: {deposit_pool_address: string, borrow_pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a queue_in_pool_config_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  queue_in_pool_config_update: ({pool_address, new_pool_config}: {pool_address: string, new_pool_config: PoolConfig}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a cancel_pool_config_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  cancel_pool_config_update: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a apply_pool_config_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  apply_pool_config_update: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_pool_config_queued_in_update transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_config_queued_in_update: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<PoolUpdate>>>

  /**
   * Construct and simulate a set_take_rate_fees_beneficiaries transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_take_rate_fees_beneficiaries: ({pool_address, beneficiaries}: {pool_address: string, beneficiaries: Map<string, u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_operation_fees_beneficiaries transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_operation_fees_beneficiaries: ({pool_address, beneficiaries}: {pool_address: string, beneficiaries: Map<string, u32>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a bootstrap_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  bootstrap_pool: ({pool_address, sponsor, amount, start_period, end_period}: {pool_address: string, sponsor: string, amount: i128, start_period: u64, end_period: u64}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit_earn transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_earn: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a borrow transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  borrow: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a swap transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  swap: ({user, token_in, token_out, amount_in}: {user: string, token_in: string, token_out: string, amount_in: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a donate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  donate: ({user, pool_address, amount}: {user: string, pool_address: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a add_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  add_collateral: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a remove_collateral transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  remove_collateral: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a repay transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  repay: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a liquidate transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  liquidate: ({liquidator, borrower, borrower_obligation_seed, borrow_pool_address, collateral_pool_address, repay_amount, min_demanded_collateral_amount}: {liquidator: string, borrower: string, borrower_obligation_seed: Option<Buffer>, borrow_pool_address: string, collateral_pool_address: string, repay_amount: i128, min_demanded_collateral_amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a simulate_withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  simulate_withdraw: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<WithdrawResult>>>

  /**
   * Construct and simulate a simulate_earn_withdraw transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  simulate_earn_withdraw: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<WithdrawResult>>>

  /**
   * Construct and simulate a withdraw_earn transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_earn: ({user, pool_address, amount, referrer}: {user: string, pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a flash_loan transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  flash_loan: ({contract, caller, pool_address, amount}: {contract: string, caller: string, pool_address: string, amount: i128}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a deposit_with_leverage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  deposit_with_leverage: ({user, deposit_pool_address, borrow_pool_address, deposit_as_margin, amount, leverage_multiplier, referrer}: {user: string, deposit_pool_address: string, borrow_pool_address: string, deposit_as_margin: boolean, amount: i128, leverage_multiplier: u32, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a withdraw_from_leveraged transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  withdraw_from_leveraged: ({user, deposit_pool_address, borrow_pool_address, amount, referrer}: {user: string, deposit_pool_address: string, borrow_pool_address: string, amount: i128, referrer: Option<string>}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a issue_cover_bad_debt transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  issue_cover_bad_debt: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a issue_cover_bad_debt_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  issue_cover_bad_debt_pair: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a claim_cover_bad_debt_results transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_cover_bad_debt_results: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a claim_cover_bad_debt_result_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  claim_cover_bad_debt_result_pair: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_pool_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  distribute_pool_fees: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a distribute_all_pools_fees transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  distribute_all_pools_fees: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_asset_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_asset_decimals: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_oracle_price_decimals transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_oracle_price_decimals: (options?: MethodOptions) => Promise<AssembledTransaction<u32>>

  /**
   * Construct and simulate a get_pool_asset_oracle_price transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_asset_oracle_price: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<i128>>>

  /**
   * Construct and simulate a get_user_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_user_obligation: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a refresh_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_obligation: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_earn_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_earn_obligation: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_multiply_pair_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_multiply_pair_obligation: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_pool: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_earn_user_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_earn_user_obligation: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a get_multiply_pair_obligation transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_multiply_pair_obligation: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Obligation>>>

  /**
   * Construct and simulate a get_pool transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<Pool>>>

  /**
   * Construct and simulate a get_all_pools transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_pools: (options?: MethodOptions) => Promise<AssembledTransaction<Array<string>>>

  /**
   * Construct and simulate a get_market_data transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_market_data: (options?: MethodOptions) => Promise<AssembledTransaction<Result<MarketData>>>

  /**
   * Construct and simulate a get_all_obligations transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_obligations: (options?: MethodOptions) => Promise<AssembledTransaction<Array<ObligationKey>>>

  /**
   * Construct and simulate a get_multiply_pair transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_multiply_pair: ({deposit_pool_address, borrow_pool_address}: {deposit_pool_address: string, borrow_pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<MultiplyPair>>>

  /**
   * Construct and simulate a get_all_multiply_pairs transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_all_multiply_pairs: (options?: MethodOptions) => Promise<AssembledTransaction<Array<MultiplyPair>>>

  /**
   * Construct and simulate a get_pool_data transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_pool_data: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<PoolData>>>

  /**
   * Construct and simulate a set_farms_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_farms_contract: ({farms_contract}: {farms_contract: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a clear_farms_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  clear_farms_contract: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a get_farms_contract transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  get_farms_contract: (options?: MethodOptions) => Promise<AssembledTransaction<Option<string>>>

  /**
   * Construct and simulate a set_pool_supply_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_pool_supply_farm: ({pool_address, farm_id}: {pool_address: string, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a set_pool_debt_farm transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  set_pool_debt_farm: ({pool_address, farm_id}: {pool_address: string, farm_id: Buffer}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a clear_pool_farms transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  clear_pool_farms: ({pool_address}: {pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_obligation_farms transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_obligation_farms: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_earn_obligation_farms transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_earn_obligation_farms: ({user}: {user: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a refresh_multiply_pair_farms transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  refresh_multiply_pair_farms: ({user, deposit_pool_address, borrow_pool_address}: {user: string, deposit_pool_address: string, borrow_pool_address: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a reset_storage transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  reset_storage: (options?: MethodOptions) => Promise<AssembledTransaction<null>>

  /**
   * Construct and simulate a update_pool_status transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  update_pool_status: ({pool_address, new_status_flags}: {pool_address: string, new_status_flags: u32}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a propose_new_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  propose_new_admin: ({new_admin}: {new_admin: string}, options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

  /**
   * Construct and simulate a accept_proposed_admin transaction. Returns an `AssembledTransaction` object which will have a `result` field containing the result of the simulation. If this transaction changes contract state, you will need to call `signAndSend()` on the returned object.
   */
  accept_proposed_admin: (options?: MethodOptions) => Promise<AssembledTransaction<Result<void>>>

}
export class Client extends ContractClient {
  static async deploy<T = Client>(
        /** Constructor/Initialization Args for the contract's `__constructor` method */
        {name, admin, oracle, insurance_fund, deployer, max_positions, min_collateral_value_cents, insolvency_ltv_bps, update_in_queue_period}: {name: string, admin: string, oracle: string, insurance_fund: string, deployer: string, max_positions: u32, min_collateral_value_cents: i128, insolvency_ltv_bps: i128, update_in_queue_period: Option<u64>},
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
    return ContractClient.deploy({name, admin, oracle, insurance_fund, deployer, max_positions, min_collateral_value_cents, insolvency_ltv_bps, update_in_queue_period}, options)
  }
  constructor(public readonly options: ContractClientOptions) {
    super(
      new ContractSpec([ "AAAAAgAAAAAAAAAAAAAADEFjY3J1YWxNb2RlbAAAAAEAAAAAAAAAAAAAAApDb21wb3VuZGVkAAA=",
        "AAAAAAAAAAAAAAAHdXBncmFkZQAAAAABAAAAAAAAAA1uZXdfd2FzbV9oYXNoAAAAAAAD7gAAACAAAAAA",
        "AAAAAAAAAAAAAAAVc3VibWl0X3JlcXVlc3RzX2JhdGNoAAAAAAAAAwAAAAAAAAAEdXNlcgAAABMAAAAAAAAACHJlcXVlc3RzAAAD6gAAB9AAAAAHUmVxdWVzdAAAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAQZ2V0X2dsb2JhbF9zdGF0ZQAAAAAAAAABAAAH0AAAAAtHbG9iYWxTdGF0ZQA=",
        "AAAAAAAAAAAAAAANdXBkYXRlX21hcmtldAAAAAAAAAIAAAAAAAAAEW5ld19tYXhfcG9zaXRpb25zAAAAAAAABAAAAAAAAAAebmV3X21pbl9jb2xsYXRlcmFsX3ZhbHVlX2NlbnRzAAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAUdXBkYXRlX21hcmtldF9zdGF0dXMAAAABAAAAAAAAAApuZXdfc3RhdHVzAAAAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAZZnVuZF91cGRhdGVfbWFya2V0X3N0YXR1cwAAAAAAAAEAAAAAAAAACm5ld19zdGF0dXMAAAAAAAQAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAPaW5pdGlhbGl6ZV9wb29sAAAAAAMAAAAAAAAADXRva2VuX2FkZHJlc3MAAAAAAAATAAAAAAAAAARzYWx0AAAD6AAAA+4AAAAgAAAAAAAAAAtwb29sX2NvbmZpZwAAAAPoAAAH0AAAAApQb29sQ29uZmlnAAAAAAABAAAD6QAAABMAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAYaW5pdGlhbGl6ZV9tdWx0aXBseV9wYWlyAAAAAgAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAbcXVldWVfaW5fcG9vbF9jb25maWdfdXBkYXRlAAAAAAIAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAAD25ld19wb29sX2NvbmZpZwAAAAfQAAAAClBvb2xDb25maWcAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAZY2FuY2VsX3Bvb2xfY29uZmlnX3VwZGF0ZQAAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAYYXBwbHlfcG9vbF9jb25maWdfdXBkYXRlAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAgZ2V0X3Bvb2xfY29uZmlnX3F1ZXVlZF9pbl91cGRhdGUAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAfQAAAAClBvb2xVcGRhdGUAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAgc2V0X3Rha2VfcmF0ZV9mZWVzX2JlbmVmaWNpYXJpZXMAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAA1iZW5lZmljaWFyaWVzAAAAAAAD7AAAABMAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAgc2V0X29wZXJhdGlvbl9mZWVzX2JlbmVmaWNpYXJpZXMAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAA1iZW5lZmljaWFyaWVzAAAAAAAD7AAAABMAAAAEAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAOYm9vdHN0cmFwX3Bvb2wAAAAAAAUAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAAB3Nwb25zb3IAAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAxzdGFydF9wZXJpb2QAAAAGAAAAAAAAAAplbmRfcGVyaW9kAAAAAAAGAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAHZGVwb3NpdAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAhyZWZlcnJlcgAAA+gAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAMZGVwb3NpdF9lYXJuAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAIcmVmZXJyZXIAAAPoAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAGYm9ycm93AAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAhyZWZlcnJlcgAAA+gAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAEc3dhcAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAh0b2tlbl9pbgAAABMAAAAAAAAACXRva2VuX291dAAAAAAAABMAAAAAAAAACWFtb3VudF9pbgAAAAAAAAsAAAABAAAD6QAAAAsAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAGZG9uYXRlAAAAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAOYWRkX2NvbGxhdGVyYWwAAAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAARcmVtb3ZlX2NvbGxhdGVyYWwAAAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAhyZWZlcnJlcgAAA+gAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAFcmVwYXkAAAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAhyZWZlcnJlcgAAA+gAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAJbGlxdWlkYXRlAAAAAAAABwAAAAAAAAAKbGlxdWlkYXRvcgAAAAAAEwAAAAAAAAAIYm9ycm93ZXIAAAATAAAAAAAAABhib3Jyb3dlcl9vYmxpZ2F0aW9uX3NlZWQAAAPoAAAD7gAAACAAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAXY29sbGF0ZXJhbF9wb29sX2FkZHJlc3MAAAAAEwAAAAAAAAAMcmVwYXlfYW1vdW50AAAACwAAAAAAAAAebWluX2RlbWFuZGVkX2NvbGxhdGVyYWxfYW1vdW50AAAAAAALAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAId2l0aGRyYXcAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAhyZWZlcnJlcgAAA+gAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAARc2ltdWxhdGVfd2l0aGRyYXcAAAAAAAAEAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAhyZWZlcnJlcgAAA+gAAAATAAAAAQAAA+kAAAfQAAAADldpdGhkcmF3UmVzdWx0AAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAWc2ltdWxhdGVfZWFybl93aXRoZHJhdwAAAAAABAAAAAAAAAAEdXNlcgAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAAIcmVmZXJyZXIAAAPoAAAAEwAAAAEAAAPpAAAH0AAAAA5XaXRoZHJhd1Jlc3VsdAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAANd2l0aGRyYXdfZWFybgAAAAAAAAQAAAAAAAAABHVzZXIAAAATAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAKZmxhc2hfbG9hbgAAAAAABAAAAAAAAAAIY29udHJhY3QAAAATAAAAAAAAAAZjYWxsZXIAAAAAABMAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAAAAAAABmFtb3VudAAAAAAACwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAVZGVwb3NpdF93aXRoX2xldmVyYWdlAAAAAAAABwAAAAAAAAAEdXNlcgAAABMAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAAAAABFkZXBvc2l0X2FzX21hcmdpbgAAAAAAAAEAAAAAAAAABmFtb3VudAAAAAAACwAAAAAAAAATbGV2ZXJhZ2VfbXVsdGlwbGllcgAAAAAEAAAAAAAAAAhyZWZlcnJlcgAAA+gAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAXd2l0aGRyYXdfZnJvbV9sZXZlcmFnZWQAAAAABQAAAAAAAAAEdXNlcgAAABMAAAAAAAAAFGRlcG9zaXRfcG9vbF9hZGRyZXNzAAAAEwAAAAAAAAATYm9ycm93X3Bvb2xfYWRkcmVzcwAAAAATAAAAAAAAAAZhbW91bnQAAAAAAAsAAAAAAAAACHJlZmVycmVyAAAD6AAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAUaXNzdWVfY292ZXJfYmFkX2RlYnQAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAZaXNzdWVfY292ZXJfYmFkX2RlYnRfcGFpcgAAAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAcY2xhaW1fY292ZXJfYmFkX2RlYnRfcmVzdWx0cwAAAAEAAAAAAAAABHVzZXIAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAgY2xhaW1fY292ZXJfYmFkX2RlYnRfcmVzdWx0X3BhaXIAAAADAAAAAAAAAAR1c2VyAAAAEwAAAAAAAAAUZGVwb3NpdF9wb29sX2FkZHJlc3MAAAATAAAAAAAAABNib3Jyb3dfcG9vbF9hZGRyZXNzAAAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAUZGlzdHJpYnV0ZV9wb29sX2ZlZXMAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAZZGlzdHJpYnV0ZV9hbGxfcG9vbHNfZmVlcwAAAAAAAAAAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
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
        "AAAAAAAAAAAAAAASc2V0X2Zhcm1zX2NvbnRyYWN0AAAAAAABAAAAAAAAAA5mYXJtc19jb250cmFjdAAAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAUY2xlYXJfZmFybXNfY29udHJhY3QAAAAAAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAASZ2V0X2Zhcm1zX2NvbnRyYWN0AAAAAAAAAAAAAQAAA+gAAAAT",
        "AAAAAAAAAAAAAAAUc2V0X3Bvb2xfc3VwcGx5X2Zhcm0AAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAASc2V0X3Bvb2xfZGVidF9mYXJtAAAAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAdmYXJtX2lkAAAAA+4AAAAgAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAQY2xlYXJfcG9vbF9mYXJtcwAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAYcmVmcmVzaF9vYmxpZ2F0aW9uX2Zhcm1zAAAAAQAAAAAAAAAEdXNlcgAAABMAAAABAAAD6QAAA+0AAAAAAAAH0AAAAAdNQ0Vycm9yAA==",
        "AAAAAAAAAAAAAAAdcmVmcmVzaF9lYXJuX29ibGlnYXRpb25fZmFybXMAAAAAAAABAAAAAAAAAAR1c2VyAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAAbcmVmcmVzaF9tdWx0aXBseV9wYWlyX2Zhcm1zAAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAAAAABRkZXBvc2l0X3Bvb2xfYWRkcmVzcwAAABMAAAAAAAAAE2JvcnJvd19wb29sX2FkZHJlc3MAAAAAEwAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAANcmVzZXRfc3RvcmFnZQAAAAAAAAAAAAAA",
        "AAAAAAAAAAAAAAANX19jb25zdHJ1Y3RvcgAAAAAAAAkAAAAAAAAABG5hbWUAAAAQAAAAAAAAAAVhZG1pbgAAAAAAABMAAAAAAAAABm9yYWNsZQAAAAAAEwAAAAAAAAAOaW5zdXJhbmNlX2Z1bmQAAAAAABMAAAAAAAAACGRlcGxveWVyAAAAEwAAAAAAAAANbWF4X3Bvc2l0aW9ucwAAAAAAAAQAAAAAAAAAGm1pbl9jb2xsYXRlcmFsX3ZhbHVlX2NlbnRzAAAAAAALAAAAAAAAABJpbnNvbHZlbmN5X2x0dl9icHMAAAAAAAsAAAAAAAAAFnVwZGF0ZV9pbl9xdWV1ZV9wZXJpb2QAAAAAA+gAAAAGAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAASdXBkYXRlX3Bvb2xfc3RhdHVzAAAAAAACAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAABBuZXdfc3RhdHVzX2ZsYWdzAAAABAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAAAAAAAAAAAAARcHJvcG9zZV9uZXdfYWRtaW4AAAAAAAABAAAAAAAAAAluZXdfYWRtaW4AAAAAAAATAAAAAQAAA+kAAAPtAAAAAAAAB9AAAAAHTUNFcnJvcgA=",
        "AAAAAAAAAAAAAAAVYWNjZXB0X3Byb3Bvc2VkX2FkbWluAAAAAAAAAAAAAAEAAAPpAAAD7QAAAAAAAAfQAAAAB01DRXJyb3IA",
        "AAAABAAAAAAAAAAAAAAAB01DRXJyb3IAAAAAMAAAAAAAAAANSW50ZXJuYWxFcnJvcgAAAAAAAAAAAAAAAAAAE05lZ2F0aXZlSW5wdXRBbW91bnQAAAAAAQAAAAAAAAAXRGVwZW5kZW5jeUNvbnRyYWN0RXJyb3IAAAAAAgAAAAAAAAAQTWFya2V0SXNOb3RPd25lZAAAAAMAAAAAAAAAF0JvcnJvd0ZvcmJpZGRlbk9uTWFya2V0AAAAAAQAAAAAAAAAGERlcG9zaXRGb3JiaWRkZW5Pbk1hcmtldAAAAAUAAAAAAAAADk1hcmtldElzRnJvemVuAAAAAAAGAAAAAAAAABNJbnZhbGlkTWFya2V0VXBkYXRlAAAAAAcAAAAAAAAAGUludmFsaWRNYXJrZXRTdGF0dXNVcGRhdGUAAAAAAAAIAAAAAAAAABRJbmNvcnJlY3RSZXF1ZXN0VHlwZQAAAAkAAAAAAAAAD092ZXJPclVuZGVyZmxvdwAAAAAKAAAAAAAAABBUb29NYW55UG9zaXRpb25zAAAACwAAAAAAAAAaTWluQ29sbGF0ZXJhbFZhbHVlSXNOb3RNZXQAAAAAAAwAAAAAAAAAFUludmFsaWRJbml0aWFsaXphdGlvbgAAAAAAAGQAAAAAAAAAEFBvb2xEb2VzTm90RXhpc3QAAABlAAAAAAAAABVJbnZhbGlkTG9hblBvb2xDb25maWcAAAAAAABmAAAAAAAAABJOb3RFbm91Z2hQb29sRnVuZHMAAAAAAGcAAAAAAAAAF0RlcG9zaXRQb29sRG9lc05vdEV4aXN0AAAAAGgAAAAAAAAAFkJvcnJvd1Bvb2xEb2VzTm90RXhpc3QAAAAAAGkAAAAAAAAAGkNvbGxhdGVyYWxQb29sRG9lc05vdEV4aXN0AAAAAABqAAAAAAAAACdQb29sQWxyZWFkeUNvbnRhaW5zUXVldWVkSW5Db25maWdVcGRhdGUAAAAAawAAAAAAAAAjUG9vbERvZXNOb3RIYXZlUXVldWVkSW5Db25maWdVcGRhdGUAAAAAbAAAAAAAAAAiUG9vbENvbmZpZ1VwZGF0ZUlzTm90WWV0QXBwbGljYWJsZQAAAAAAbQAAAAAAAAAYT3BlcmF0aW9uRm9yYmlkZGVuT25Qb29sAAAAbgAAAAAAAAAWSW52YWxpZEJvb3RzdHJhcFBlcmlvZAAAAAAAbwAAAAAAAAAWT2JsaWdhdGlvbkRvZXNOb3RFeGlzdAAAAAAAyAAAAAAAAAAbRGVwb3NpdFBvc2l0aW9uRG9lc05vdEV4aXN0AAAAAMkAAAAAAAAAGkJvcnJvd1Bvc2l0aW9uRG9lc05vdEV4aXN0AAAAAADKAAAAAAAAABlXaXRoZHJhd1NjYXJjaXR5T3ZlckxpbWl0AAAAAAAAywAAAAAAAAAWU2NhcmNpdHlDb29sZG93blBlcmlvZAAAAAAAzAAAAAAAAAAcQm9ycm93UG9zaXRpb25Gb3JBc3NldEV4aXN0cwAAAM0AAAAAAAAAHURlcG9zaXRQb3NpdGlvbkZvckFzc2V0RXhpc3RzAAAAAAAAzgAAAAAAAAAXUG9vbFN1cHBseUxpbWl0RXhjZWVkZWQAAAABkAAAAAAAAAAfUG9vbFV0aWxpemF0aW9uUmF0aW9DYXBFeGNlZWRlZAAAAAGRAAAAAAAAABtPcmFjbGVEb2VzTm90S25vd0Fzc2V0UHJpY2UAAAAB9AAAAAAAAAAQT3JhY2xlU3RhbGVQcmljZQAAAfUAAAAAAAAAGEludmFsaWRMaXF1aWRhdGlvbklucHV0cwAAAlgAAAAAAAAAE09ibGlnYXRpb25Jc0hlYWx0aHkAAAACWQAAAAAAAAAqT2JsaWdhdGlvbkNvbnRhaW5zT3BlbkNvdmVyQmFkRGVidFJlcXVlc3RzAAAAAAJaAAAAAAAAACBCYWREZWJ0Q292ZXJhZ2VDcml0ZXJpb25Jc05vdE1ldAAAAlsAAAAAAAAAHUFzc2V0Q2Fubm90QmVVc2VkQXNDb2xsYXRlcmFsAAAAAAACXAAAAAAAAAAmTGlxdWlkYXRpb25FeGNlc3NpdmVEZW1hbmRlZENvbGxhdGVyYWwAAAAAAl0AAAAAAAAAFUludmFsaWRMZXZlcmFnZUlucHV0cwAAAAAAArwAAAAAAAAAE0ludmFsaWRTd2FwU2xpcHBhZ2UAAAACvQAAAAAAAAAZTXVsdGlwbHlQYWlyQWxyZWFkeUV4aXN0cwAAAAAAAr4AAAAAAAAAGE11bHRpcGx5UGFpckRvZXNOb3RFeGlzdAAAAr8AAAAAAAAAH0xldmVyYWdlUG9zaXRpb25Db250YWluc0JhZERlYnQAAAACwAAAAAAAAAAfSW5jb25zaXN0ZW50RGVwb3NpdFdpdGhMZXZlcmFnZQAAAALB",
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
        "AAAABQAAAAAAAAAAAAAAD1Byb3Bvc2VOZXdBZG1pbgAAAAABAAAAEXByb3Bvc2VfbmV3X2FkbWluAAAAAAAAAQAAAAAAAAAJbmV3X2FkbWluAAAAAAAAEwAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAE0FjY2VwdEFkbWluUHJvcG9zYWwAAAAAAQAAABVhY2NlcHRfYWRtaW5fcHJvcG9zYWwAAAAAAAAAAAAAAg==",
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
        "AAAABQAAAAAAAAAAAAAAHUluY29uc2lzdGVudEltbWVkaWF0ZUNvdmVyYWdlAAAAAAAAAQAAAB9pbmNvbnNpc3RlbnRfaW1tZWRpYXRlX2NvdmVyYWdlAAAAAAQAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAMYmFsYW5jZV9kaWZmAAAACwAAAAEAAAAAAAAAC2RlYnRfYW1vdW50AAAAAAsAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAG0luc3VyYW5jZUZ1bmRNaXNzaW5nUmVxdWVzdAAAAAABAAAAHmluc3VyYW5jZV9mdW5kX21pc3NpbmdfcmVxdWVzdAAAAAAAAwAAAAAAAAAOb2JsaWdhdGlvbl9rZXkAAAAAB9AAAAANT2JsaWdhdGlvbktleQAAAAAAAAEAAAAAAAAADHBvb2xfYWRkcmVzcwAAABMAAAABAAAAAAAAAApyZXF1ZXN0X2lkAAAAAAAGAAAAAQAAAAI=",
        "AAAABQAAAAAAAAAAAAAAEkR1cGxpY2F0ZVJlcXVlc3RJZAAAAAAAAQAAABRkdXBsaWNhdGVfcmVxdWVzdF9pZAAAAAMAAAAAAAAADm9ibGlnYXRpb25fa2V5AAAAAAfQAAAADU9ibGlnYXRpb25LZXkAAAAAAAABAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAQAAAAAAAAAKcmVxdWVzdF9pZAAAAAAABgAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAADUNsYWltTWlzbWF0Y2gAAAAAAAABAAAADmNsYWltX21pc21hdGNoAAAAAAAFAAAAAAAAAA5vYmxpZ2F0aW9uX2tleQAAAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAACnJlcXVlc3RfaWQAAAAAAAYAAAAAAAAAAAAAAA9hcHByb3ZlZF9hbW91bnQAAAAACwAAAAAAAAAAAAAAD2FjdHVhbF9yZWNlaXZlZAAAAAALAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAAHVJlZmVycmVySXNVbmV4cGVjdGVkbHlNaXNzaW5nAAAAAAAAAQAAACByZWZlcnJlcl9pc191bmV4cGVjdGVkbHlfbWlzc2luZwAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAHU9ibGlnYXRpb25GYXJtc1JlZnJlc2hlZEV2ZW50AAAAAAAAAQAAACBvYmxpZ2F0aW9uX2Zhcm1zX3JlZnJlc2hlZF9ldmVudAAAAAMAAAAAAAAABHVzZXIAAAATAAAAAQAAAAAAAAAQbnVtX3N1cHBseV9mYXJtcwAAAAQAAAAAAAAAAAAAAA5udW1fZGVidF9mYXJtcwAAAAAABAAAAAAAAAAC",
        "AAAABQAAAAAAAAAAAAAAFUZhcm1zQ29udHJhY3RTZXRFdmVudAAAAAAAAAEAAAAYZmFybXNfY29udHJhY3Rfc2V0X2V2ZW50AAAAAQAAAAAAAAAOZmFybXNfY29udHJhY3QAAAAAABMAAAABAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAGUZhcm1zQ29udHJhY3RDbGVhcmVkRXZlbnQAAAAAAAABAAAAHGZhcm1zX2NvbnRyYWN0X2NsZWFyZWRfZXZlbnQAAAAAAAAAAg==",
        "AAAABQAAAAAAAAAAAAAAFVBvb2xGYXJtc0NsZWFyZWRFdmVudAAAAAAAAAEAAAAYcG9vbF9mYXJtc19jbGVhcmVkX2V2ZW50AAAAAQAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAC",
        "AAAABQAAAAAAAAAAAAAAEFBvb2xGYXJtU2V0RXZlbnQAAAABAAAAE3Bvb2xfZmFybV9zZXRfZXZlbnQAAAAAAwAAAAAAAAAMcG9vbF9hZGRyZXNzAAAAEwAAAAEAAAAAAAAAB2Zhcm1faWQAAAAD7gAAACAAAAAAAAAAAAAAAAlmYXJtX2tpbmQAAAAAAAARAAAAAAAAAAI=",
        "AAAABQAAAAAAAAAAAAAACERiZ0V2ZW50AAAAAQAAAAlkYmdfZXZlbnQAAAAAAAABAAAAAAAAAAZzeW1ib2wAAAAAABEAAAABAAAAAg==",
        "AAAAAQAAAAAAAAAAAAAAFkFubnVhbFBlcmNlbnRhZ2VZaWVsZHMAAAAAAAIAAAAAAAAACmJvcnJvd19icHMAAAAAAAQAAAAAAAAACnN1cHBseV9icHMAAAAAAAQ=",
        "AAAAAgAAAAAAAAAAAAAAEUludGVyZXN0UmF0ZU1vZGVsAAAAAAAAAQAAAAEAAAAAAAAABktpbmtlZAAAAAAAAQAAB9AAAAAOS2lua2VkSVJDb25maWcAAA==",
        "AAAAAQAAAAAAAAAAAAAADktpbmtlZElSQ29uZmlnAAAAAAAGAAAARkJhc2UgQVBSIHRoYXQgaXMgYWNjcnVlZCByZWdhcmRsZXNzIG9mIHRoZSB1dGlsaXphdGlvbiByYXRpbyBvZiBhIHBvb2wAAAAAAAxiYXNlX2Fwcl9icHMAAAALAAAARUFQUiB0aGF0IGlzIGFjY3J1ZWQgd2hlbiB0aGUgdXRpbGl6YXRpb24gcmF0aW8gaXMgYXQgdGhlIGtpbmsgMSB2YWx1ZQAAAAAAAA1raW5rMV9hcHJfYnBzAAAAAAAACwAAABhLaW5rIDEgdXRpbGl6YXRpb24gcmF0aW8AAAAMa2luazFfdXJfYnBzAAAACwAAAEVBUFIgdGhhdCBpcyBhY2NydWVkIHdoZW4gdGhlIHV0aWxpemF0aW9uIHJhdGlvIGlzIGF0IHRoZSBraW5rIDIgdmFsdWUAAAAAAAANa2luazJfYXByX2JwcwAAAAAAAAsAAAAYS2luayAyIHV0aWxpemF0aW9uIHJhdGlvAAAADGtpbmsyX3VyX2JwcwAAAAsAAAA5QVBSIHRoYXQgaXMgYWNjcnVlZCB3aGVuIHRoZSB1dGlsaXphdGlvbiByYXRpbyBpcyBhdCAxMDAlAAAAAAAAC21heF9hcHJfYnBzAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAADE11bHRpcGx5UGFpcgAAAAQAAAAAAAAAC2JvcnJvd19wb29sAAAAABMAAAAAAAAADGRlcG9zaXRfcG9vbAAAABMAAAAAAAAAF21heF9sZXZlcmFnZV9tdWx0aXBsaWVyAAAAAAQAAAAAAAAABHNlZWQAAAPuAAAAIA==",
        "AAAAAQAAAAAAAAAAAAAADU9ibGlnYXRpb25LZXkAAAAAAAACAAAAAAAAAARzZWVkAAAD6AAAA+4AAAAgAAAAAAAAAAR1c2VyAAAAEw==",
        "AAAAAQAAAAAAAAAAAAAACk9ibGlnYXRpb24AAAAAAAQAAAAAAAAAB2JvcnJvd3MAAAAD7AAAABMAAAfQAAAADkJvcnJvd1Bvc2l0aW9uAAAAAAAAAAAACGRlcG9zaXRzAAAD7AAAABMAAAfQAAAAD0RlcG9zaXRQb3NpdGlvbgAAAAAAAAAAG2luc3VyYW5jZV9mdW5kX3JlcXVlc3RzX2lkcwAAAAPsAAAD7QAAAAIAAAATAAAABgAAA+0AAAAAAAAAAAAAAA9wb3NpdGlvbnNfY291bnQAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAADkJvcnJvd1Bvc2l0aW9uAAAAAAACAAAAAAAAAAhkX3Rva2VucwAAAAsAAAAAAAAAE29yaWdpbmFsbHlfYm9ycm93ZWQAAAAACw==",
        "AAAAAQAAAAAAAAAAAAAAD0RlcG9zaXRQb3NpdGlvbgAAAAAEAAAAAAAAAApjb2xsYXRlcmFsAAAAAAALAAAAAAAAAAhqX3Rva2VucwAAAAsAAAAAAAAAGWxhc3Rfc2NhcmNpdHlfd2l0aGRyYXdfdHMAAAAAAAAGAAAAAAAAABRvcmlnaW5hbGx5X2RlcG9zaXRlZAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAADU9wZXJhdGlvbkZlZXMAAAAAAAACAAAAAAAAAAdmZWVfc3VtAAAAAAsAAAAAAAAADHJlZmVycmVyX2ZlZQAAA+gAAAAL",
        "AAAAAQAAAAAAAAAAAAAADURlcG9zaXRSZXN1bHQAAAAAAAADAAAAAAAAAAlkZXBvc2l0ZWQAAAAAAAALAAAAAAAAABFqX3Rva2Vuc190b19pc3N1ZQAAAAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAADEJvcnJvd1Jlc3VsdAAAAAQAAAAAAAAAEWJvcnJvd2VyX25ld19kZWJ0AAAAAAAACwAAAAAAAAATYm9ycm93ZXJfdG9fcmVjZWl2ZQAAAAALAAAAAAAAABFkX3Rva2Vuc190b19pc3N1ZQAAAAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAAE0FkZENvbGxhdGVyYWxSZXN1bHQAAAAAAgAAAAAAAAAQYWRkZWRfY29sbGF0ZXJhbAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAADldpdGhkcmF3UmVzdWx0AAAAAAAEAAAAAAAAABBkZXBvc2l0X2RlY3JlYXNlAAAACwAAAAAAAAAQal90b2tlbnNfdG9fYnVybgAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAAAAAAAAAAAFXdpdGhkcmF3ZXJfdG9fcmVjZWl2ZQAAAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAC1JlcGF5UmVzdWx0AAAAAAQAAAAAAAAAE2Ftb3VudF90b19zZW5kX2JhY2sAAAAACwAAAAAAAAAQZF90b2tlbnNfdG9fYnVybgAAAAsAAAAAAAAAC2RlYnRfcmVwYWlkAAAAAAsAAAAAAAAADm9wZXJhdGlvbl9mZWVzAAAAAAfQAAAADU9wZXJhdGlvbkZlZXMAAAA=",
        "AAAAAQAAAAAAAAAAAAAAFlJlbW92ZUNvbGxhdGVyYWxSZXN1bHQAAAAAAAMAAAAAAAAAE2NvbGxhdGVyYWxfZGVjcmVhc2UAAAAACwAAAAAAAAAdY29sbGF0ZXJhbF9yZW1vdmVyX3RvX3JlY2VpdmUAAAAAAAALAAAAAAAAAA5vcGVyYXRpb25fZmVlcwAAAAAH0AAAAA1PcGVyYXRpb25GZWVzAAAA",
        "AAAAAQAAAAAAAAAAAAAAEUxpcXVpZGF0aW9uUmVzdWx0AAAAAAAABQAAAAAAAAAPZF90b2tlbnNfYnVybmVkAAAAAAsAAAAAAAAAC2RlYnRfcmVwYWlkAAAAAAsAAAAAAAAAD2pfdG9rZW5zX3NlaXplZAAAAAALAAAAAAAAABdwbGFpbl9jb2xsYXRlcmFsX3NlaXplZAAAAAALAAAAAAAAABt0b2tlbnNfZnJvbV9qX3Rva2Vuc19zZWl6ZWQAAAAACw==",
        "AAAAAQAAAAAAAAAAAAAABFBvb2wAAAAUAAAAAAAAABFib290c3RyYXBfcGVyaW9kcwAAAAAAA+wAAAPtAAAAAgAAAAYAAAAGAAAH0AAAABNQb29sQm9vdHN0cmFwUGVyaW9kAAAAAAAAAAAOYm9ycm93X2Fwcl9icHMAAAAAAAsAAAAAAAAABmNvbmZpZwAAAAAH0AAAAApQb29sQ29uZmlnAAAAAAAAAAAACWZhcm1fZGVidAAAAAAAA+gAAAPuAAAAIAAAAAAAAAALZmFybV9zdXBwbHkAAAAD6AAAA+4AAAAgAAAAAAAAABZpbnRlcmVzdF9yYXRlX21vZGlmaWVyAAAAAAALAAAAAAAAABZsYXN0X2FjY3J1YWxfdGltZXN0YW1wAAAAAAAGAAAAAAAAAARuYW1lAAAAEAAAAAAAAAASb3BlcmF0aW9uX2ZlZXNfc3VtAAAAAAALAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAA5zdXBwbHlfYXByX2JwcwAAAAAACwAAAAAAAAASdGFrZV9yYXRlX2ZlZXNfc3VtAAAAAAALAAAAAAAAABx0YXJnZXRfdXRpbGl6YXRpb25fcmF0aW9fYnBzAAAACwAAAAAAAAANdG9rZW5fYWRkcmVzcwAAAAAAABMAAAAAAAAADHRva2VuX3N5bWJvbAAAABAAAAAAAAAAD3RvdGFsX2F2YWlsYWJsZQAAAAALAAAAAAAAAA50b3RhbF9ib3Jyb3dlZAAAAAAACwAAAAAAAAAQdG90YWxfY29sbGF0ZXJhbAAAAAsAAAAAAAAADnRvdGFsX2RfdG9rZW5zAAAAAAALAAAAAAAAAA50b3RhbF9qX3Rva2VucwAAAAAACw==",
        "AAAAAQAAAAAAAAAAAAAADVBvb2xGZWVDb25maWcAAAAAAAAMAAAAAAAAABZhZGRfY29sbGF0ZXJhbF9mZWVfYnBzAAAAAAAEAAAAAAAAAA5ib3Jyb3dfZmVlX2JwcwAAAAAABAAAAAAAAAAPZGVwb3NpdF9mZWVfYnBzAAAAAAQAAAAAAAAAEmZsYXNoX2xvYW5fZmVlX2JwcwAAAAAABAAAAAAAAAAbb3BlcmF0aW9uX2ZlZV9iZW5lZmljaWFyaWVzAAAAA+gAAAPsAAAAEwAAAAQAAAAAAAAACXJlZmVycmVycwAAAAAAA+gAAAPsAAAAEwAAAAQAAAAAAAAAGXJlbW92ZV9jb2xsYXRlcmFsX2ZlZV9icHMAAAAAAAAEAAAAAAAAAA1yZXBheV9mZWVfYnBzAAAAAAAABAAAAAAAAAAXdGFrZV9yYXRlX2JlbmVmaWNpYXJpZXMAAAAD6AAAA+wAAAATAAAABAAAAAAAAAANdGFrZV9yYXRlX2JwcwAAAAAAAAQAAAAAAAAAEHdpdGhkcmF3X2ZlZV9icHMAAAAEAAAAAAAAABx3aXRoZHJhd19zY2FyY2l0eV9mZWVfc2NfYnBzAAAABA==",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xTdGF0dXMAAAAAAAEAAAAAAAAABWZsYWdzAAAAAAAABA==",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xDb25maWcAAAAAAAYAAAAAAAAADWFjY3J1YWxfbW9kZWwAAAAAAAfQAAAADEFjY3J1YWxNb2RlbAAAAAAAAAAKZmVlX2NvbmZpZwAAAAAH0AAAAA1Qb29sRmVlQ29uZmlnAAAAAAAAAAAAAA1oZWFsdGhfY29uZmlnAAAAAAAH0AAAABBQb29sSGVhbHRoQ29uZmlnAAAAAAAAABNpbnRlcmVzdF9yYXRlX21vZGVsAAAAB9AAAAARSW50ZXJlc3RSYXRlTW9kZWwAAAAAAAAAAAAAFmlyX3JlYWN0aXZpdHlfY29uc3RhbnQAAAAAAAQAAAAAAAAABnN0YXR1cwAAAAAH0AAAAApQb29sU3RhdHVzAAA=",
        "AAAAAQAAAAAAAAAAAAAAEFBvb2xIZWFsdGhDb25maWcAAAAKAAAAAAAAAA1jbG9zZV9sdHZfYnBzAAAAAAAACwAAAAAAAAASaW5zb2x2ZW5jeV9sdHZfYnBzAAAAAAALAAAAAAAAABRsaWFiaWxpdHlfZmFjdG9yX2JwcwAAAAsAAAAAAAAAHGxpcXVpZGF0aW9uX2Nsb3NlX2ZhY3Rvcl9icHMAAAALAAAAAAAAAB1tYXhfbGlxdWlkYXRpb25faW5jZW50aXZlX2JwcwAAAAAAAAsAAAAAAAAADG9wZW5fbHR2X2JwcwAAAAsAAAAAAAAADHN1cHBseV9saW1pdAAAAAsAAAAAAAAAG3V0aWxpemF0aW9uX3JhdGlvX2xpbWl0X2JwcwAAAAALAAAAAAAAABx3aXRoZHJhd19zY2FyY2l0eV9jb29sZG93bl9zAAAABgAAAAAAAAAbd2l0aGRyYXdfc2NhcmNpdHlfbGltaXRfYnBzAAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAE1Bvb2xCb290c3RyYXBQZXJpb2QAAAAAAgAAAAAAAAAQcmVtYWluaW5nX2Ftb3VudAAAAAsAAAAAAAAADHRvdGFsX2Ftb3VudAAAAAs=",
        "AAAAAQAAAAAAAAAAAAAAB1JlcXVlc3QAAAAAAwAAAAAAAAAGYW1vdW50AAAAAAALAAAAAAAAAAxwb29sX2FkZHJlc3MAAAATAAAAAAAAAAxyZXF1ZXN0X3R5cGUAAAAE",
        "AAAAAwAAAAAAAAAAAAAAC1JlcXVlc3RUeXBlAAAAAAcAAAAAAAAAB0RlcG9zaXQAAAAAAAAAAAAAAAAGQm9ycm93AAAAAAABAAAAAAAAAAhXaXRoZHJhdwAAAAIAAAAAAAAABVJlcGF5AAAAAAAAAwAAAAAAAAANQWRkQ29sbGF0ZXJhbAAAAAAAAAQAAAAAAAAAEFJlbW92ZUNvbGxhdGVyYWwAAAAFAAAAAAAAAAxSZWZyZXNoRmFybXMAAAAG",
        "AAAAAQAAAAAAAAAAAAAAC0dsb2JhbFN0YXRlAAAAAAoAAAAAAAAABWFkbWluAAAAAAAAEwAAAAAAAAAIZGVwbG95ZXIAAAATAAAAAAAAABJpbnNvbHZlbmN5X2x0dl9icHMAAAAAAAsAAAAAAAAACGlzX293bmVkAAAAAQAAAAAAAAANbWF4X3Bvc2l0aW9ucwAAAAAAAAQAAAAAAAAAGm1pbl9jb2xsYXRlcmFsX3ZhbHVlX2NlbnRzAAAAAAALAAAAAAAAAARuYW1lAAAAEAAAAAAAAAAGb3JhY2xlAAAAAAATAAAAAAAAAAZzdGF0dXMAAAAAAAQAAAAAAAAAFnVwZGF0ZV9pbl9xdWV1ZV9wZXJpb2QAAAAAA+gAAAAG",
        "AAAAAgAAAAAAAAAAAAAADE1hcmtldFN0YXR1cwAAAAcAAAAAAAAAAAAAAAZBY3RpdmUAAAAAAAAAAAAAAAAADEJvcnJvd0Zyb3plbgAAAAAAAAAAAAAAE0JvcnJvd0Zyb3plbkJ5QWRtaW4AAAAAAAAAAAAAAAANRGVwb3NpdEZyb3plbgAAAAAAAAAAAAAAAAAAFERlcG9zaXRGcm96ZW5CeUFkbWluAAAAAAAAAAAAAAAGRnJvemVuAAAAAAAAAAAAAAAAAA1Gcm96ZW5CeUFkbWluAAAA",
        "AAAAAQAAAAAAAAAAAAAAClBvb2xVcGRhdGUAAAAAAAIAAAAAAAAACm5ld19jb25maWcAAAAAB9AAAAAKUG9vbENvbmZpZwAAAAAAAAAAABNxdWV1ZWRfaW5fdGltZXN0YW1wAAAAAAY=",
        "AAAAAgAAAAAAAAAAAAAAB0RhdGFLZXkAAAAAFwAAAAAAAAAAAAAABE5hbWUAAAAAAAAAAAAAAAVBZG1pbgAAAAAAAAAAAAAAAAAABk9yYWNsZQAAAAAAAAAAAAAAAAAHQWNjcnVhbAAAAAAAAAAAAAAAAAdJc093bmVkAAAAAAAAAAAAAAAACEFsbFBvb2xzAAAAAAAAAAAAAAALR2xvYmFsU3RhdGUAAAAAAAAAAAAAAAAMRGVwbG95ZXJIb3N0AAAAAAAAAAAAAAAMTWF4UG9zaXRpb25zAAAAAAAAAAAAAAAMTWFya2V0U3RhdHVzAAAAAAAAAAAAAAANRmFybXNDb250cmFjdAAAAAAAAAEAAAAAAAAADENvbmZpZ1VwZGF0ZQAAAAEAAAATAAAAAQAAAAAAAAAEUG9vbAAAAAEAAAATAAAAAAAAAAAAAAANSW5zdXJhbmNlRnVuZAAAAAAAAAAAAAAAAAAADkFsbE9ibGlnYXRpb25zAAAAAAAAAAAAAAAAABBJbnNvbHZlbmN5THR2QnBzAAAAAAAAAAAAAAAQQWxsTXVsdGlwbHlQYWlycwAAAAAAAAAAAAAAEkVhcm5PYmxpZ2F0aW9uU2VlZAAAAAAAAAAAAAAAAAAXTWluQ29sbGF0ZXJhbFZhbHVlQ2VudHMAAAAAAAAAAAAAAAATVXBkYXRlSW5RdWV1ZVBlcmlvZAAAAAABAAAAAAAAAApPYmxpZ2F0aW9uAAAAAAABAAAH0AAAAA1PYmxpZ2F0aW9uS2V5AAAAAAAAAQAAAAAAAAAMTXVsdGlwbHlQYWlyAAAAAQAAA+0AAAACAAAAEwAAABMAAAAAAAAAAAAAAA1Qcm9wb3NlZEFkbWluAAAA",
        "AAAAAQAAAAAAAAAAAAAACFBvb2xEYXRhAAAABwAAAAAAAAADYXB5AAAAB9AAAAAWQW5udWFsUGVyY2VudGFnZVlpZWxkcwAAAAAAAAAAABVkX3Rva2VuX3JhdGVfY2VpbF9icHMAAAAAAAALAAAAAAAAABZqX3Rva2VuX3JhdGVfZmxvb3JfYnBzAAAAAAALAAAAAAAAABJvcmFjbGVfYXNzZXRfcHJpY2UAAAAAAAsAAAAAAAAABHBvb2wAAAfQAAAABFBvb2wAAAAAAAAAGHRvdGFsX2F2YWlsYWJsZV9hZGp1c3RlZAAAAAsAAAAAAAAADHRvdGFsX3N1cHBseQAAAAs=",
        "AAAAAQAAAAAAAAAAAAAACk1hcmtldERhdGEAAAAAAAUAAAAAAAAADmFzc2V0X2RlY2ltYWxzAAAAAAAEAAAAAAAAAAxnbG9iYWxfc3RhdGUAAAfQAAAAC0dsb2JhbFN0YXRlAAAAAAAAAAAObXVsdGlwbHlfcGFpcnMAAAAAA+oAAAfQAAAADE11bHRpcGx5UGFpcgAAAAAAAAAVb3JhY2xlX3ByaWNlX2RlY2ltYWxzAAAAAAAABAAAAAAAAAAKcG9vbHNfZGF0YQAAAAAD6gAAB9AAAAAIUG9vbERhdGE=",
        "AAAAAQAAAP1EZWxlZ2F0ZWUgaWRlbnRpZmllciBmb3IgZmFybSBzdGFrZXMuCgpTdXBwb3J0cyBtdWx0aXBsZSBzdGFrZSBpZGVudGl0aWVzIHBlciBvd25lciBhZGRyZXNzOgotIFNpbXBsZToganVzdCBvd25lciBhZGRyZXNzIChmb3IgY29udHJhY3RzIHdoZXJlIHVzZXIgaGFzIHNpbmdsZSBwb3NpdGlvbikKLSBXaXRoIHNlZWQ6IG93bmVyIGFkZHJlc3MgKyBzZWVkIChmb3IgY29udHJhY3RzIHdpdGggbXVsdGlwbGUgb2JsaWdhdGlvbnMgcGVyIHVzZXIpAAAAAAAAAAAAAAlEZWxlZ2F0ZWUAAAAAAAACAAAAE1RoZSBvd25lcidzIGFkZHJlc3MAAAAABW93bmVyAAAAAAAAEwAAADlPcHRpb25hbCBzZWVkIHRvIGRpc3Rpbmd1aXNoIG11bHRpcGxlIHBvc2l0aW9ucyBwZXIgb3duZXIAAAAAAAAEc2VlZAAAA+gAAAPuAAAAIA==",
        "AAAAAgAAAAAAAAAAAAAAEklzc3VlUmVxdWVzdFJlc3VsdAAAAAAAAgAAAAEAAAAAAAAACFJlY29yZGVkAAAAAQAAAAYAAAABAAAAAAAAAAlJbW1lZGlhdGUAAAAAAAABAAAACw==",
        "AAAAAgAAAAAAAAAAAAAADkNvdmVyYWdlU3RhdHVzAAAAAAACAAAAAAAAAAAAAAAHUGVuZGluZwAAAAABAAAAAAAAAAVSZWFkeQAAAAAAAAEAAAAL",
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
        fund_update_market_status: this.txFromJSON<Result<void>>,
        initialize_pool: this.txFromJSON<Result<string>>,
        initialize_multiply_pair: this.txFromJSON<Result<void>>,
        queue_in_pool_config_update: this.txFromJSON<Result<void>>,
        cancel_pool_config_update: this.txFromJSON<Result<void>>,
        apply_pool_config_update: this.txFromJSON<Result<void>>,
        get_pool_config_queued_in_update: this.txFromJSON<Result<PoolUpdate>>,
        set_take_rate_fees_beneficiaries: this.txFromJSON<Result<void>>,
        set_operation_fees_beneficiaries: this.txFromJSON<Result<void>>,
        bootstrap_pool: this.txFromJSON<Result<void>>,
        deposit: this.txFromJSON<Result<void>>,
        deposit_earn: this.txFromJSON<Result<void>>,
        borrow: this.txFromJSON<Result<void>>,
        swap: this.txFromJSON<Result<i128>>,
        donate: this.txFromJSON<Result<void>>,
        add_collateral: this.txFromJSON<Result<void>>,
        remove_collateral: this.txFromJSON<Result<void>>,
        repay: this.txFromJSON<Result<void>>,
        liquidate: this.txFromJSON<Result<void>>,
        withdraw: this.txFromJSON<Result<void>>,
        simulate_withdraw: this.txFromJSON<Result<WithdrawResult>>,
        simulate_earn_withdraw: this.txFromJSON<Result<WithdrawResult>>,
        withdraw_earn: this.txFromJSON<Result<void>>,
        flash_loan: this.txFromJSON<Result<void>>,
        deposit_with_leverage: this.txFromJSON<Result<void>>,
        withdraw_from_leveraged: this.txFromJSON<Result<void>>,
        issue_cover_bad_debt: this.txFromJSON<Result<void>>,
        issue_cover_bad_debt_pair: this.txFromJSON<Result<void>>,
        claim_cover_bad_debt_results: this.txFromJSON<Result<void>>,
        claim_cover_bad_debt_result_pair: this.txFromJSON<Result<void>>,
        distribute_pool_fees: this.txFromJSON<Result<void>>,
        distribute_all_pools_fees: this.txFromJSON<Result<void>>,
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
        set_farms_contract: this.txFromJSON<Result<void>>,
        clear_farms_contract: this.txFromJSON<Result<void>>,
        get_farms_contract: this.txFromJSON<Option<string>>,
        set_pool_supply_farm: this.txFromJSON<Result<void>>,
        set_pool_debt_farm: this.txFromJSON<Result<void>>,
        clear_pool_farms: this.txFromJSON<Result<void>>,
        refresh_obligation_farms: this.txFromJSON<Result<void>>,
        refresh_earn_obligation_farms: this.txFromJSON<Result<void>>,
        refresh_multiply_pair_farms: this.txFromJSON<Result<void>>,
        reset_storage: this.txFromJSON<null>,
        update_pool_status: this.txFromJSON<Result<void>>,
        propose_new_admin: this.txFromJSON<Result<void>>,
        accept_proposed_admin: this.txFromJSON<Result<void>>
  }
}