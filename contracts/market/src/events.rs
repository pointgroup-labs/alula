use soroban_sdk::{Address, BytesN, Env, Map, String, Symbol, Vec, contractevent};

use crate::{
    obligation::{
        AddCollateralResult, BorrowResult, DepositResult, LiquidationResult, Obligation,
        ObligationKey, RemoveCollateralResult, RepayResult, WithdrawResult,
    },
    pool::{Pool, PoolConfig},
    storage::MarketStatus,
};

// -- Contract's Methods Events --

#[contractevent]
struct InitializePoolEvent {
    #[topic]
    token_address: Address,
    #[topic]
    pool_address: Address,
    #[topic]
    token_symbol: String,
}

#[contractevent]
struct QueueInPoolSet {
    #[topic]
    pool_address: Address,
    #[topic]
    pool_config: PoolConfig,
}

#[contractevent]
struct CancelPoolSet {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct ApplyPoolSet {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct DepositEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    obligation: Obligation,
    deposit_result: DepositResult,
}

#[contractevent]
struct SwapExact {
    #[topic]
    swap_provider: Address,
    #[topic]
    user: Address,
    #[topic]
    path: Vec<Address>,
    amount_in: i128,
    min_amount_out: i128,
    received_amount: i128,
}

#[contractevent]
struct SwapForExact {
    #[topic]
    swap_provider: Address,
    #[topic]
    user: Address,
    path: Vec<Address>,
    max_amount_in: i128,
    amount_out: i128,
    sent_amount: i128,
}

#[contractevent]
struct BorrowEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    obligation: Obligation,
    borrow_result: BorrowResult,
}

#[contractevent]
struct AddCollateralEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    obligation: Obligation,
    add_collateral_result: AddCollateralResult,
}

#[contractevent]
struct RepayEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    obligation: Obligation,
    repay_result: RepayResult,
}

#[contractevent]
struct LiquidateEvent {
    #[topic]
    liquidator: Address,
    #[topic]
    borrower_obligation_key: ObligationKey,
    #[topic]
    borrow_pool_address: Address,
    #[topic]
    collateral_pool_address: Address,
    borrower_obligation: Option<Obligation>,
    liquidator_obligation: Option<Obligation>,
    liquidation_result: LiquidationResult,
}

#[contractevent]
struct RemoveCollateralEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    obligation: Option<Obligation>,
    remove_collateral_result: RemoveCollateralResult,
}

#[contractevent]
struct WithdrawEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    obligation: Option<Obligation>,
    withdraw_result: WithdrawResult,
}

#[contractevent]
struct FlashLoanEvent {
    #[topic]
    contract: Address,
    #[topic]
    pool_address: Address,
    #[topic]
    initiator: Address,
    amount: i128,
    fees_paid: i128,
}

#[contractevent]
struct ProposeNewAdmin {
    #[topic]
    new_admin: Address,
}

#[contractevent]
struct QueueInMarketConfigUpdate {
    new_max_positions: u32,
    new_min_collateral_value_cents: i128,
    new_bad_debt_lock_d: u64,
}

#[contractevent]
struct CancelMarketConfigUpdate {}

#[contractevent]
struct ApplyMarketConfigUpdate {
    new_max_positions: u32,
    new_min_collateral_value_cents: i128,
    new_bad_debt_lock_d: u64,
}

#[contractevent]
struct UpdateMarketStatus {
    new_status: MarketStatus,
}

#[contractevent]
struct FundUpdateMarketStatus {
    new_status: MarketStatus,
}

#[contractevent]
struct SetTakeRateBeneficiaries {
    #[topic]
    pool_address: Address,
    beneficiaries: Map<Address, u32>,
}

#[contractevent]
struct SetOperationFeesBeneficiaries {
    #[topic]
    pool_address: Address,
    beneficiaries: Map<Address, u32>,
}

#[contractevent]
struct IssueCoverBadDebt {
    #[topic]
    obligation_key: ObligationKey,
}

#[contractevent]
struct ClaimCoverBadDebtResults {
    #[topic]
    obligation_key: ObligationKey,
}

#[contractevent]
struct PoolBadDebtLocked {
    #[topic]
    pool_address: Address,
    deadline: u64,
}

#[contractevent]
struct PoolBadDebtUnlocked {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct DistributePoolFees {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct RefreshObligation {
    #[topic]
    obligation_key: ObligationKey,
}

#[contractevent]
struct RefreshPool {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct AcceptAdminProposal {}

#[contractevent]
struct FlashBorrow {
    pool_address: Address,
    user: Address,
    amount: i128,
}

#[contractevent]
struct FarmsContractSetEvent {
    #[topic]
    farms_contract: Address,
}

#[contractevent]
struct FarmsContractClearedEvent {}

#[contractevent]
struct PoolFarmsClearedEvent {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct PoolFarmSetEvent {
    #[topic]
    pool_address: Address,
    farm_id: BytesN<32>,
    farm_kind: Symbol,
}

// -- Internal Error Events --

#[contractevent]
struct LedgerTimestampError {
    current_timestamp: u64,
    stored_timestamp: u64,
}

#[contractevent]
struct UtilizationRatioExceedsLimit {
    utilization_ratio_bps: i128,
    utilization_ratio_limit_bps: i128,
}

#[contractevent]
struct PoolIsMissingInStorage {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct ObligationAmntBecomesNegative {
    old_amount: i128,
    new_amount: i128,
}

#[contractevent]
struct PoolAmountBecomesNegative {
    old_amount: i128,
    new_amount: i128,
}

#[contractevent]
struct PoolInconsistentTotalShares {
    total_shares: i128,
    individual_shares: i128,
}

#[contractevent]
struct PoolInconsistentTotalTokens {
    total_shares: i128,
    total_tokens: i128,
}

#[contractevent]
struct PoolContainsInconsistentState {
    pool: Pool,
}

#[contractevent]
struct ObligationIsUnexpectedlyEmpty {
    #[topic]
    obligation_key: ObligationKey,
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct ComputedInterestIsNegative {
    #[topic]
    pool_address: Address,
    position_shares: i128,
    tokens_from_shares_ceil: i128,
    computed_interest: i128,
}

#[contractevent]
struct PositionsCountBecomesNegative {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation: Obligation,
}

#[contractevent]
struct InconsistentImmediateCoverage {
    #[topic]
    obligation_key: ObligationKey,
    #[topic]
    pool_address: Address,
    #[topic]
    balance_diff: i128,
    debt_amount: i128,
}

#[contractevent]
struct InsuranceFundMissingRequest {
    #[topic]
    obligation_key: ObligationKey,
    #[topic]
    pool_address: Address,
    #[topic]
    request_id: u64,
}

#[contractevent]
struct DuplicateRequestId {
    #[topic]
    obligation_key: ObligationKey,
    #[topic]
    pool_address: Address,
    request_id: u64,
}

#[contractevent]
struct ClaimMismatch {
    #[topic]
    obligation_key: ObligationKey,
    #[topic]
    pool_address: Address,
    request_id: u64,
    approved_amount: i128,
    actual_received: i128,
}

#[contractevent]
struct ReferrerIsUnexpectedlyMissing {}

#[contractevent]
struct ReferrerFeeExceedsOpsFees {
    #[topic]
    pool_address: Address,
    operation_fees_sum: i128,
    referrer_fee: i128,
}

#[contractevent]
struct NonPositiveDTokensBorrow {
    #[topic]
    pool_address: Address,
    minted_d_tokens_amount: i128,
    real_borrowed_amount: i128,
}

#[contractevent]
struct NonPositiveJTokensWithdraw {
    #[topic]
    pool_address: Address,
    burned_j_tokens_amount: i128,
    deposit_decrease: i128,
}

#[contractevent]
struct ObligationFarmsRefreshedEvent {
    #[topic]
    user: Address,
    num_supply_farms: u32,
    num_debt_farms: u32,
}

#[contractevent]
struct InconsistentSwapReceived {
    #[topic]
    swap_provider: Address,
    path: Vec<Address>,
    received_amount: i128,
    min_amount_out: i128,
}

#[contractevent]
struct InconsistentSwapSent {
    #[topic]
    swap_provider: Address,
    path: Vec<Address>,
    sent_amount: i128,
    max_amount_in: i128,
}

// -- Methods that abstract away how events are published --

pub fn deposit(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    obligation: Obligation,
    deposit_result: DepositResult,
) {
    DepositEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        obligation,
        deposit_result,
    }
    .publish(e);
}

pub fn initialize_pool(
    e: &Env,
    token_address: &Address,
    pool_address: &Address,
    token_symbol: &String,
) {
    InitializePoolEvent {
        token_address: token_address.clone(),
        pool_address: pool_address.clone(),
        token_symbol: token_symbol.clone(),
    }
    .publish(e);
}

pub fn queue_in_pool_set(e: &Env, pool_address: Address, pool_config: PoolConfig) {
    QueueInPoolSet { pool_address, pool_config }.publish(e);
}

pub fn cancel_pool_set(e: &Env, pool_address: Address) {
    CancelPoolSet { pool_address }.publish(e);
}

pub fn apply_pool_set(e: &Env, pool_address: Address) {
    ApplyPoolSet { pool_address }.publish(e);
}

#[allow(clippy::too_many_arguments)]
pub fn swap_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    path: &Vec<Address>,
    amount_in: i128,
    min_amount_out: i128,
    received_amount: i128,
) {
    SwapExact {
        swap_provider: swap_provider.clone(),
        user: user.clone(),
        path: path.clone(),
        amount_in,
        min_amount_out,
        received_amount,
    }
    .publish(e);
}

#[allow(clippy::too_many_arguments)]
pub fn swap_for_exact(
    e: &Env,
    swap_provider: &Address,
    user: &Address,
    path: &Vec<Address>,
    max_amount_in: i128,
    amount_out: i128,
    sent_amount: i128,
) {
    SwapForExact {
        swap_provider: swap_provider.clone(),
        user: user.clone(),
        path: path.clone(),
        max_amount_in,
        amount_out,
        sent_amount,
    }
    .publish(e);
}

pub fn borrow(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    obligation: Obligation,
    borrow_result: BorrowResult,
) {
    BorrowEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        obligation,
        borrow_result,
    }
    .publish(e);
}

pub fn add_collateral(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    obligation: Obligation,
    add_collateral_result: AddCollateralResult,
) {
    AddCollateralEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        obligation,
        add_collateral_result,
    }
    .publish(e);
}

pub fn repay(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    obligation: Obligation,
    repay_result: RepayResult,
) {
    RepayEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        obligation,
        repay_result,
    }
    .publish(e);
}

#[allow(clippy::too_many_arguments)]
pub fn liquidate(
    e: &Env,
    liquidator: &Address,
    borrower_obligation_key: &ObligationKey,
    borrow_pool_address: &Address,
    collateral_pool_address: &Address,
    borrower_obligation: Option<Obligation>,
    liquidator_obligation: Option<Obligation>,
    liquidation_result: LiquidationResult,
) {
    LiquidateEvent {
        liquidator: liquidator.clone(),
        borrower_obligation_key: borrower_obligation_key.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
        collateral_pool_address: collateral_pool_address.clone(),
        borrower_obligation,
        liquidator_obligation,
        liquidation_result,
    }
    .publish(e);
}

pub fn remove_collateral(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    obligation: Option<Obligation>,
    remove_collateral_result: RemoveCollateralResult,
) {
    RemoveCollateralEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        obligation,
        remove_collateral_result,
    }
    .publish(e);
}

pub fn withdraw(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    obligation: Option<Obligation>,
    withdraw_result: WithdrawResult,
) {
    WithdrawEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        obligation,
        withdraw_result,
    }
    .publish(e);
}

pub fn flash_loan(
    e: &Env,
    initiator: &Address,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
    fees_paid: i128,
) {
    FlashLoanEvent {
        contract: contract.clone(),
        pool_address: pool_address.clone(),
        initiator: initiator.clone(),
        amount,
        fees_paid,
    }
    .publish(e);
}

pub fn flash_borrow(e: &Env, user: &Address, pool_address: &Address, amount: i128) {
    FlashBorrow { pool_address: pool_address.clone(), user: user.clone(), amount }.publish(e);
}

pub fn queue_in_market_config_update(
    e: &Env,
    new_max_positions: u32,
    new_min_collateral_value_cents: i128,
    new_bad_debt_lock_d: u64,
) {
    QueueInMarketConfigUpdate {
        new_max_positions,
        new_min_collateral_value_cents,
        new_bad_debt_lock_d,
    }
    .publish(e);
}

pub fn cancel_market_config_update(e: &Env) {
    CancelMarketConfigUpdate {}.publish(e);
}

pub fn apply_market_config_update(
    e: &Env,
    new_max_positions: u32,
    new_min_collateral_value_cents: i128,
    new_bad_debt_lock_d: u64,
) {
    ApplyMarketConfigUpdate {
        new_max_positions,
        new_min_collateral_value_cents,
        new_bad_debt_lock_d,
    }
    .publish(e);
}

pub fn update_market_status(e: &Env, new_status: MarketStatus) {
    UpdateMarketStatus { new_status }.publish(e);
}

pub fn fund_update_market_status(e: &Env, new_status: MarketStatus) {
    FundUpdateMarketStatus { new_status }.publish(e);
}

pub fn set_take_rate_fees_beneficiaries(
    e: &Env,
    pool_address: Address,
    beneficiaries: Map<Address, u32>,
) {
    SetTakeRateBeneficiaries { pool_address, beneficiaries }.publish(e);
}

pub fn set_operation_fees_beneficiaries(
    e: &Env,
    pool_address: Address,
    beneficiaries: Map<Address, u32>,
) {
    SetOperationFeesBeneficiaries { pool_address, beneficiaries }.publish(e);
}

pub fn issue_cover_bad_debt(e: &Env, obligation_key: ObligationKey) {
    IssueCoverBadDebt { obligation_key }.publish(e);
}

pub fn claim_cover_bad_debt_results(e: &Env, obligation_key: ObligationKey) {
    ClaimCoverBadDebtResults { obligation_key }.publish(e);
}

pub fn pool_bad_debt_locked(e: &Env, pool_address: &Address, deadline: u64) {
    PoolBadDebtLocked { pool_address: pool_address.clone(), deadline }.publish(e);
}

pub fn pool_bad_debt_unlocked(e: &Env, pool_address: &Address) {
    PoolBadDebtUnlocked { pool_address: pool_address.clone() }.publish(e);
}

pub fn distribute_pool_fees(e: &Env, pool_address: Address) {
    DistributePoolFees { pool_address }.publish(e);
}

pub fn refresh_obligation(e: &Env, obligation_key: ObligationKey) {
    RefreshObligation { obligation_key }.publish(e);
}

pub fn refresh_pool(e: &Env, pool_address: Address) {
    RefreshPool { pool_address }.publish(e);
}

pub fn propose_new_admin(e: &Env, new_admin: Address) {
    ProposeNewAdmin { new_admin }.publish(e);
}

pub fn accept_proposed_admin(e: &Env) {
    AcceptAdminProposal {}.publish(e);
}

// -- Internal Errors Events --

// Emitted when the current ledger timestamp unexpectedly precedes the previously kept in the
// storage timestamp
pub fn current_ledger_timestamp_smaller_than_stored_timestamp(
    e: &Env,
    current_timestamp: u64,
    stored_timestamp: u64,
) {
    LedgerTimestampError { current_timestamp, stored_timestamp }.publish(e);
}

// Emitted when a pool's utilization ratio exceeds a predefined limit
pub fn utilization_ratio_exceeds_limit(
    e: &Env,
    utilization_ratio_bps: i128,
    utilization_ratio_limit_bps: i128,
) {
    UtilizationRatioExceedsLimit { utilization_ratio_bps, utilization_ratio_limit_bps }.publish(e);
}

// Emitted when an attempt is made to interact with a loan pool that does not exist in storage
pub fn pool_is_unexpectedly_missing_in_storage(e: &Env, pool_address: &Address) {
    PoolIsMissingInStorage { pool_address: pool_address.clone() }.publish(e);
}

// Emitted when a pool's total amount of tokens unexpectedly attempts to become negative
pub fn obligation_amount_becomes_negative(e: &Env, old_amount: i128, new_amount: i128) {
    ObligationAmntBecomesNegative { old_amount, new_amount }.publish(e);
}

// Emitted when a pool's total amount of tokens unexpectedly attempts to become negative
pub fn pool_amount_becomes_negative(e: &Env, old_amount: i128, new_amount: i128) {
    PoolAmountBecomesNegative { old_amount, new_amount }.publish(e);
}

// Emitted when the total shares in a pool are found to be less than an individual user's shares
pub fn pool_total_shares_smaller_than_individual_user_shares(
    e: &Env,
    total_shares: i128,
    individual_shares: i128,
) {
    PoolInconsistentTotalShares { total_shares, individual_shares }.publish(e);
}

// Emitted when obligation unexpectedly becomes empty
pub fn obligation_is_unexpectedly_empty(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
) {
    ObligationIsUnexpectedlyEmpty {
        obligation_key: obligation_key.clone(),
        pool_address: pool_address.clone(),
    }
    .publish(e);
}

// Emitted when calculated interest(either for borrow or supply position) is negative
pub fn computed_interest_is_negative(
    e: &Env,
    pool_address: &Address,
    position_shares: i128,
    tokens_from_position_shares_ceil: i128,
    computed_interest: i128,
) {
    ComputedInterestIsNegative {
        position_shares,
        computed_interest,
        pool_address: pool_address.clone(),
        tokens_from_shares_ceil: tokens_from_position_shares_ceil,
    }
    .publish(e);
}

pub fn positions_count_becomes_negative(e: &Env, pool_address: &Address, obligation: &Obligation) {
    PositionsCountBecomesNegative {
        pool_address: pool_address.clone(),
        obligation: obligation.clone(),
    }
    .publish(e);
}

pub fn inconsistent_immediate_insurance_fund_coverage(
    e: &Env,
    obligation_key: ObligationKey,
    pool_address: Address,
    balance_diff: i128,
    debt_amount: i128,
) {
    InconsistentImmediateCoverage { obligation_key, pool_address, balance_diff, debt_amount }
        .publish(e)
}

// -- Farms Integration Events --

pub fn set_farms_contract(e: &Env, farms_contract: &Address) {
    FarmsContractSetEvent { farms_contract: farms_contract.clone() }.publish(e);
}

pub fn clear_farms_contract(e: &Env) {
    FarmsContractClearedEvent {}.publish(e);
}

pub fn pool_farm_set(e: &Env, pool_address: &Address, farm_id: &BytesN<32>, is_supply: bool) {
    let farm_kind = if is_supply { Symbol::new(e, "supply") } else { Symbol::new(e, "debt") };
    PoolFarmSetEvent { pool_address: pool_address.clone(), farm_id: farm_id.clone(), farm_kind }
        .publish(e);
}

pub fn pool_farms_cleared(e: &Env, pool_address: &Address) {
    PoolFarmsClearedEvent { pool_address: pool_address.clone() }.publish(e);
}

pub fn obligation_farms_refreshed(e: &Env, user: &Address, num_supply: u32, num_debt: u32) {
    ObligationFarmsRefreshedEvent {
        user: user.clone(),
        num_supply_farms: num_supply,
        num_debt_farms: num_debt,
    }
    .publish(e);
}

pub fn insurance_fund_missing_request(
    e: &Env,
    obligation_key: ObligationKey,
    pool_address: Address,
    request_id: u64,
) {
    InsuranceFundMissingRequest { obligation_key, pool_address, request_id }.publish(e);
}

pub fn insurance_fund_duplicate_request_id(
    e: &Env,
    obligation_key: ObligationKey,
    pool_address: Address,
    request_id: u64,
) {
    DuplicateRequestId { obligation_key, pool_address, request_id }.publish(e);
}

pub fn insurance_fund_claim_mismatch(
    e: &Env,
    obligation_key: ObligationKey,
    pool_address: Address,
    request_id: u64,
    approved_amount: i128,
    actual_received: i128,
) {
    ClaimMismatch { obligation_key, pool_address, request_id, approved_amount, actual_received }
        .publish(e);
}

pub fn referrer_is_unexpectedly_missing(e: &Env) {
    ReferrerIsUnexpectedlyMissing {}.publish(e);
}

pub fn referrer_fee_exceeds_operation_fees_sum(
    e: &Env,
    pool_address: Address,
    operation_fees_sum: i128,
    referrer_fee: i128,
) {
    ReferrerFeeExceedsOpsFees { pool_address, operation_fees_sum, referrer_fee }.publish(e);
}

pub fn minting_non_positive_d_tokens_on_borrow(
    e: &Env,
    pool_address: Address,
    minted_d_tokens_amount: i128,
    real_borrowed_amount: i128,
) {
    NonPositiveDTokensBorrow { pool_address, minted_d_tokens_amount, real_borrowed_amount }
        .publish(e);
}

pub fn burning_non_positive_j_tokens_on_withdraw(
    e: &Env,
    pool_address: Address,
    burned_j_tokens_amount: i128,
    deposit_decrease: i128,
) {
    NonPositiveJTokensWithdraw { pool_address, burned_j_tokens_amount, deposit_decrease }
        .publish(e);
}

pub fn inconsistent_swap_received_amount(
    e: &Env,
    swap_provider: &Address,
    path: &Vec<Address>,
    received_amount: i128,
    min_amount_out: i128,
) {
    InconsistentSwapReceived {
        swap_provider: swap_provider.clone(),
        path: path.clone(),
        received_amount,
        min_amount_out,
    }
    .publish(e);
}

pub fn inconsistent_swap_sent_amount(
    e: &Env,
    swap_provider: &Address,
    path: &Vec<Address>,
    sent_amount: i128,
    max_amount_in: i128,
) {
    InconsistentSwapSent {
        swap_provider: swap_provider.clone(),
        path: path.clone(),
        sent_amount,
        max_amount_in,
    }
    .publish(e);
}
