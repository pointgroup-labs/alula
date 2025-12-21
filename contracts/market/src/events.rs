use soroban_sdk::{Address, Env, String, Symbol, contractevent};

use crate::{
    obligation::{
        AddCollateralResult, BorrowResult, DepositResult, LiquidationResult, Obligation,
        ObligationKey, RemoveCollateralResult, RepayResult, WithdrawResult,
    },
    pool::{Pool, PoolConfig},
};

// --- Contract's Methods Events ---

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
struct InitializeMultiplyPairEvent {
    #[topic]
    deposit_pool_address: Address,
    #[topic]
    borrow_pool_address: Address,
}

#[contractevent]
struct QueueInPoolConfigUpdate {
    #[topic]
    pool_address: Address,
    #[topic]
    pool_config: PoolConfig,
}

#[contractevent]
struct CancelPoolConfigUpdate {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct ApplyPoolConfigUpdate {
    #[topic]
    pool_address: Address,
}

#[contractevent]
struct BootstrapPoolEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    sponsor: Address,
    amount: i128,
    period: (u64, u64),
}

#[contractevent]
struct DepositEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    deposit_result: DepositResult,
}

// TODO: TO BE REMOVED
#[contractevent]
struct SwapEvent {
    #[topic]
    user: Address,
    #[topic]
    token_in: Address,
    #[topic]
    token_out: Address,
    amount_in: i128,
    amount_out: i128,
    received_amount: i128,
}

#[contractevent]
struct BorrowEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    borrow_result: BorrowResult,
}

#[contractevent]
struct AddCollateralEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    add_collateral_result: AddCollateralResult,
}

#[contractevent]
struct RepayEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
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
    liquidation_result: LiquidationResult,
}

#[contractevent]
struct RemoveCollateralEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    remove_collateral_result: RemoveCollateralResult,
}

#[contractevent]
struct WithdrawEvent {
    #[topic]
    pool_address: Address,
    #[topic]
    obligation_key: ObligationKey,
    withdraw_result: WithdrawResult,
}

#[contractevent]
struct FlashLoanEvent {
    #[topic]
    contract: Address,
    #[topic]
    pool_address: Address,
    amount: i128,
    fees_paid: i128,
}

#[contractevent]
struct DepositWithLeverageEvent {
    #[topic]
    obligation_key: ObligationKey,
    #[topic]
    deposit_pool_address: Address,
    #[topic]
    borrow_pool_address: Address,
    original_amount: i128,
    leverage_multiplier: u32,
    total_deposited_amount: i128,
    total_borrowed_amount: i128,
}

#[contractevent]
struct WithdrawFromLeveragedEvent {
    #[topic]
    obligation_key: ObligationKey,
    #[topic]
    deposit_pool_address: Address,
    #[topic]
    borrow_pool_address: Address,
    withdrawn_to_wallet_amount: i128,
    deposit_reduced_amount: i128,
    borrow_reduced_amount: i128,
}

#[contractevent]
struct AccrueInterestEvent {
    #[topic]
    user: Address,
}

// ----- Internal Error Events -----

#[contractevent]
struct LedgerTimestampError {
    current_timestamp: u64,
    stored_timestamp: u64,
}

#[contractevent]
struct LeveragedPositionBadDebt {
    #[topic]
    user: Address,
    #[topic]
    deposit_pool_address: Address,
    #[topic]
    borrow_pool_address: Address,
    deposited_amount: i128,
    borrowed_amount: i128,
    deposited_amount_swapped: i128,
}

#[contractevent]
struct LeverageExceedsBorrowCapacity {
    #[topic]
    user: Address,
    #[topic]
    flash_borrow_amount: i128,
    flash_repay_amount: i128,
    max_healthy_borrow_amount: i128,
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
struct ObligationIsMissingInStorage {
    #[topic]
    obligation_key: ObligationKey,
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
struct ReceivedUnexpectedSwapAmount {
    #[topic]
    user: Address,
    #[topic]
    token_in: Address,
    #[topic]
    token_out: Address,
    amount_in: i128,
    amount_out: i128,
    expected_amount_in: i128,
    expected_amount_out: i128,
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
struct ReferrerIsUnexpectedlyMissing {}

// --- Methods that abstract how events are published ---

pub fn deposit(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    deposit_result: DepositResult,
) {
    DepositEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
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

pub fn initialize_multiply_pair(
    e: &Env,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
) {
    InitializeMultiplyPairEvent {
        deposit_pool_address: deposit_pool_address.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
    }
    .publish(e);
}

pub fn queue_in_pool_config_update(e: &Env, pool_address: Address, pool_config: PoolConfig) {
    QueueInPoolConfigUpdate { pool_address, pool_config }.publish(e);
}

pub fn cancel_pool_config_update(e: &Env, pool_address: Address) {
    CancelPoolConfigUpdate { pool_address }.publish(e);
}

pub fn apply_pool_config_update(e: &Env, pool_address: Address) {
    ApplyPoolConfigUpdate { pool_address }.publish(e);
}

pub fn bootstrap_pool(
    e: &Env,
    pool_address: &Address,
    sponsor: &Address,
    amount: i128,
    period: (u64, u64),
) {
    BootstrapPoolEvent {
        pool_address: pool_address.clone(),
        sponsor: sponsor.clone(),
        amount,
        period,
    }
    .publish(e);
}

pub fn swap(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    received_amount: i128,
) {
    SwapEvent {
        user: user.clone(),
        token_in: token_in.clone(),
        token_out: token_out.clone(),
        amount_in,
        amount_out,
        received_amount,
    }
    .publish(e);
}

pub fn borrow(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    borrow_result: BorrowResult,
) {
    BorrowEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        borrow_result,
    }
    .publish(e);
}

pub fn add_collateral(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    add_collateral_result: AddCollateralResult,
) {
    AddCollateralEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        add_collateral_result,
    }
    .publish(e);
}

pub fn repay(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    repay_result: RepayResult,
) {
    RepayEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
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
    liquidation_result: LiquidationResult,
) {
    LiquidateEvent {
        liquidator: liquidator.clone(),
        borrower_obligation_key: borrower_obligation_key.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
        collateral_pool_address: collateral_pool_address.clone(),
        liquidation_result,
    }
    .publish(e);
}

pub fn remove_collateral(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    remove_collateral_result: RemoveCollateralResult,
) {
    RemoveCollateralEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        remove_collateral_result,
    }
    .publish(e);
}

pub fn withdraw(
    e: &Env,
    pool_address: &Address,
    obligation_key: &ObligationKey,
    withdraw_result: WithdrawResult,
) {
    WithdrawEvent {
        pool_address: pool_address.clone(),
        obligation_key: obligation_key.clone(),
        withdraw_result,
    }
    .publish(e);
}

pub fn flash_loan(
    e: &Env,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
    fees_paid: i128,
) {
    FlashLoanEvent {
        contract: contract.clone(),
        pool_address: pool_address.clone(),
        amount,
        fees_paid,
    }
    .publish(e);
}

#[allow(clippy::too_many_arguments)]
pub fn deposit_with_leverage(
    e: &Env,
    obligation_key: &ObligationKey,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    original_amount: i128,
    leverage_multiplier: u32,
    total_deposited_amount: i128,
    total_borrowed_amount: i128,
) {
    DepositWithLeverageEvent {
        obligation_key: obligation_key.clone(),
        deposit_pool_address: deposit_pool_address.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
        original_amount,
        leverage_multiplier,
        total_deposited_amount,
        total_borrowed_amount,
    }
    .publish(e);
}

pub fn withdraw_from_leveraged(
    e: &Env,
    obligation_key: &ObligationKey,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    withdrawn_to_wallet_amount: i128,
    deposit_reduced_amount: i128,
    borrow_reduced_amount: i128,
) {
    WithdrawFromLeveragedEvent {
        obligation_key: obligation_key.clone(),
        deposit_pool_address: deposit_pool_address.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
        withdrawn_to_wallet_amount,
        deposit_reduced_amount,
        borrow_reduced_amount,
    }
    .publish(e);
}

/// Emitted when the current ledger timestamp unexpectedly precedes the previously kept in the
/// storage timestamp
pub fn current_ledger_timestamp_smaller_than_stored_timestamp(
    e: &Env,
    current_timestamp: u64,
    stored_timestamp: u64,
) {
    LedgerTimestampError { current_timestamp, stored_timestamp }.publish(e);
}

/// Emitted when a leveraged position incurs bad debt
pub fn leveraged_position_bad_debt(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    deposited_amount: i128,
    borrowed_amount: i128,
    deposited_amount_swapped: i128,
) {
    LeveragedPositionBadDebt {
        user: user.clone(),
        deposit_pool_address: deposit_pool_address.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
        deposited_amount,
        borrowed_amount,
        deposited_amount_swapped,
    }
    .publish(e);
}

pub fn leverage_borrow_exceeds_borrowing_capacity(
    e: &Env,
    user: &Address,
    flash_borrow_amount: i128,
    flash_repay_amount: i128,
    max_healthy_borrow_amount: i128,
) {
    LeverageExceedsBorrowCapacity {
        user: user.clone(),
        flash_borrow_amount,
        flash_repay_amount,
        max_healthy_borrow_amount,
    }
    .publish(e);
}

/// Emitted when a pool's utilization ratio exceeds a predefined limit
pub fn utilization_ratio_exceeds_limit(
    e: &Env,
    utilization_ratio_bps: i128,
    utilization_ratio_limit_bps: i128,
) {
    UtilizationRatioExceedsLimit { utilization_ratio_bps, utilization_ratio_limit_bps }.publish(e);
}

/// Emitted when an attempt is made to interact with a loan pool that does not exist in storage
pub fn pool_is_unexpectedly_missing_in_storage(e: &Env, pool_address: &Address) {
    PoolIsMissingInStorage { pool_address: pool_address.clone() }.publish(e);
}

/// Emitted when an attempt is made to interact with an obligation that does not exist in storage
pub fn obligation_is_unexpectedly_missing_in_storage(e: &Env, obligation_key: &ObligationKey) {
    ObligationIsMissingInStorage { obligation_key: obligation_key.clone() }.publish(e);
}

/// Emitted when a pool's total amount of tokens unexpectedly attempts to become negative
pub fn obligation_amount_becomes_negative(e: &Env, old_amount: i128, new_amount: i128) {
    ObligationAmntBecomesNegative { old_amount, new_amount }.publish(e);
}

/// Emitted when a pool's total amount of tokens unexpectedly attempts to become negative
pub fn pool_amount_becomes_negative(e: &Env, old_amount: i128, new_amount: i128) {
    PoolAmountBecomesNegative { old_amount, new_amount }.publish(e);
}

/// Emitted when the total shares in a pool are found to be less than an individual user's shares
pub fn pool_total_shares_smaller_than_individual_user_shares(
    e: &Env,
    total_shares: i128,
    individual_shares: i128,
) {
    PoolInconsistentTotalShares { total_shares, individual_shares }.publish(e);
}

/// Emitted when the total shares in a pool are found to be less than the total tokens amount
pub fn pool_total_shares_smaller_than_total_tokens(
    e: &Env,
    total_shares: i128,
    total_tokens: i128,
) {
    PoolInconsistentTotalTokens { total_shares, total_tokens }.publish(e);
}

/// Emitted when pool state becomes generally inconsistent
pub fn pool_contains_inconsistent_state(e: &Env, pool: &Pool) {
    PoolContainsInconsistentState { pool: pool.clone() }.publish(e);
}

/// Emitted when obligation unexpectedly becomes empty
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

/// Emitted when calculated interest(either for borrow or supply position) is negative
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

/// Emitted when an unexpected amount has been received after a deterministic swap operation via a
/// swap provider
#[allow(clippy::too_many_arguments)]
pub fn received_unexpected_swap_amount(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    expected_amount_in: i128,
    expected_amount_out: i128,
) {
    ReceivedUnexpectedSwapAmount {
        user: user.clone(),
        token_in: token_in.clone(),
        token_out: token_out.clone(),
        amount_in,
        amount_out,
        expected_amount_in,
        expected_amount_out,
    }
    .publish(e);
}

pub fn inconsistent_immediate_insurance_fund_coverage(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    balance_diff: i128,
    debt_amount: i128,
) {
    InconsistentImmediateCoverage {
        obligation_key: obligation_key.clone(),
        pool_address: pool_address.clone(),
        balance_diff,
        debt_amount,
    }
    .publish(e);
}

pub fn insurance_fund_missing_request(
    e: &Env,
    obligation_key: &ObligationKey,
    pool_address: &Address,
    request_id: u64,
) {
    InsuranceFundMissingRequest {
        obligation_key: obligation_key.clone(),
        pool_address: pool_address.clone(),
        request_id,
    }
    .publish(e);
}

pub fn referrer_is_unexpectedly_missing(e: &Env) {
    ReferrerIsUnexpectedlyMissing {}.publish(e);
}

// --- Helper Functions  ---

#[contractevent]
struct DbgEvent {
    #[topic]
    pub symbol: Symbol,
}

pub fn dbg(e: &Env, symbol: Symbol) {
    DbgEvent { symbol }.publish(e);
}
