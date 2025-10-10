use soroban_sdk::{Address, Env, IntoVal, String, Symbol, Val, Vec as SorobanVec, contractevent};

use crate::{
    obligation::{
        AddCollateralResult, BorrowResult, DepositResult, ObligationKey, RemoveCollateralResult,
        RepayResult, WithdrawResult,
    },
    pool::Pool,
};

// TODO: It's not clear which data we must include in topics. It'll become clear
// when implementing event subscriptions. Blend includes both addresses and names in topics

// --- Contract's Methods Events ---

#[contractevent]
pub struct ConstructorEvent {
    #[topic]
    pub admin: Address,
    #[topic]
    pub name: String,
    pub oracle: Address,
}

#[contractevent]
pub struct DepositEvent {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub obligation_key: ObligationKey, /* TODO: If we emit events with an obligation key, we'd better have a public method */
    // that returns them, right?
    pub deposit_result: DepositResult,
}

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InitializePoolEvent {
    #[topic]
    pub token_address: Address,
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub token_ticker: Symbol,
}

// TODO: Should we still keep a public `swap` endpoint?
/// Emitted when tokens are swapped
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SwapEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub token_in: Address,
    #[topic]
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out: i128,
    pub received_amount: i128,
}

#[contractevent]
pub struct BorrowEvent {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub obligation_key: ObligationKey,
    pub borrow_result: BorrowResult,
}

#[contractevent]
pub struct AddCollateralEvent {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub obligation_key: ObligationKey,
    pub add_collateral_result: AddCollateralResult,
}

#[contractevent]
pub struct RepayEvent {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub obligation_key: ObligationKey,
    pub repay_result: RepayResult,
}

#[contractevent]
pub struct LiquidateEvent {
    #[topic]
    pub liquidator: Address,
    #[topic]
    pub borrower_obligation_key: ObligationKey,
    #[topic]
    pub borrow_pool_address: Address,
    #[topic]
    pub collateral_pool_address: Address,
    // TODO: Introduce `LiquidateResult`
    pub liquidated_amount: i128,
    pub collateral_seized_amount: i128,
}

#[contractevent]
pub struct RemoveCollateralEvent {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub obligation_key: ObligationKey,
    pub remove_collateral_result: RemoveCollateralResult,
}

#[contractevent]
pub struct WithdrawEvent {
    #[topic]
    pub pool_address: Address,
    #[topic]
    pub obligation_key: ObligationKey,
    pub withdraw_result: WithdrawResult,
}

#[contractevent]
pub struct FlashLoanEvent {
    #[topic]
    pub contract: Address,
    #[topic]
    pub pool_address: Address,
    // TODO: Introduce `FlashLoanResult`
    pub amount: i128,
    pub fees_paid: i128,
}

#[contractevent]
pub struct DepositWithLeverageEvent {
    #[topic]
    pub obligation_key: ObligationKey,
    #[topic]
    pub deposit_pool_address: Address,
    #[topic]
    pub borrow_pool_address: Address,
    // TODO: DepositWithLeverageResult
    pub original_amount: i128,
    pub leverage_multiplier: u32,
    pub total_deposited_amount: i128,
    pub total_borrowed_amount: i128,
}

#[contractevent]
pub struct WithdrawFromLeveragedEvent {
    #[topic]
    pub user: Address,
    #[topic]
    pub deposit_pool_address: Address,
    #[topic]
    pub borrow_pool_address: Address,
    // TODO: WithdrawFromLeveragedResult
    pub amount: i128,
    pub actual_amount_withdrawn: i128,
}

// TODO: Should this event even exist?
#[contractevent]
pub struct AccrueInterestEvent {
    #[topic]
    pub user: Address,
}

// ----- Internal Error Events -----

/// Emitted when the current ledger timestamp unexpectedly precedes the previously kept in the
/// storage timestamp
#[contractevent]
pub struct LedgerTimestampError {
    pub current_timestamp: u64,
    pub stored_timestamp: u64,
}

/// Emitted when a leveraged position incurs bad debt
#[contractevent]
pub struct LeveragedPositionBadDebt {
    #[topic]
    pub user: Address,
    #[topic]
    pub deposit_pool_address: Address,
    #[topic]
    pub borrow_pool_address: Address,
    pub deposited_amount: i128,
    pub borrowed_amount: i128,
    pub deposited_amount_swapped: i128,
}

/// Emitted when a pool's utilization ratio exceeds a predefined limit
#[contractevent]
pub struct UtilizationRatioExceedsLimit {
    pub utilization_ratio_bps: i128,
    pub utilization_ratio_limit_bps: i128,
}

/// Emitted when an attempt is made to interact with a loan pool that does not exist in storage
#[contractevent]
pub struct PoolIsMissingInStorage {
    #[topic]
    pub pool_address: Address,
}

/// Emitted when an attempt is made to interact with an obligation that does not exist in storage
#[contractevent]
pub struct ObligationIsMissingInStorage {
    #[topic]
    pub obligation_key: ObligationKey,
}

/// Emitted when an obligation's borrowed amount unexpectedly attempts to become negative
///
/// // TODO: fix;
// #[contractevent]
// pub struct ObligationAmountBecomesNegative {
//     #[topic]
//     pub obligation_key: ObligationKey,
//     // pub old_amount: i128,
//     // pub new_amount: i128,
// }

/// Emitted when a pool's total amount of tokens unexpectedly attempts to become negative
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolAmountBecomesNegative {
    pub old_amount: i128,
    pub new_amount: i128,
}

// TODO: Fix

// /// Emitted when the total shares in a pool are found to be less than an individual user's shares
// #[contractevent]
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct PoolTotalSharesSmallerThanIndividualUserShares {
//     pub total_shares: i128,
//     pub individual_shares: i128,
// }

// /// Emitted when the total shares in a pool are found to be less than the total tokens amount
// #[contractevent]
// #[derive(Clone, Debug, Eq, PartialEq)]
// pub struct PoolTotalSharesSmallerThanTotalTokens {
//     pub total_shares: i128,
//     pub total_tokens: i128,
// }

/// Emitted when pool state becomes generally inconsistent
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PoolContainsInconsistentState {
    pub pool: Pool,
}

/// Emitted when obligation unexpectedly becomes empty
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObligationIsUnexpectedlyEmpty {
    #[topic]
    pub obligation_key: ObligationKey,
    #[topic]
    pub pool_address: Address,
}

/// Emitted when calculated interest(either for borrow or supply position) is negative
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComputedInterestIsNegative {
    #[topic]
    pub pool_address: Address,
    pub shares: i128,
    pub tokens_from_shares: i128,
    pub computed_interest: i128,
    pub tokens_from_all_shares: i128,
}

/// Emitted when an unexpected amount has been received after a deterministic swap operation via a
/// swap provider
#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReceivedUnexpectedSwapAmount {
    #[topic]
    pub user: Address,
    #[topic]
    pub token_in: Address,
    #[topic]
    pub token_out: Address,
    pub amount_in: i128,
    pub amount_out: i128,
    pub expected_amount_in: i128,
    pub expected_amount_out: i128,
}

pub fn constructor(e: &Env, admin: &Address, name: &String, oracle: &Address) {
    ConstructorEvent { admin: admin.clone(), name: name.clone(), oracle: oracle.clone() }
        .publish(e);
}

/// Helper to publish the DepositEvent.
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

/// Helper to publish the InitializePoolEvent.
pub fn initialize_pool(
    e: &Env,
    token_address: &Address,
    pool_address: &Address,
    token_ticker: &Symbol,
) {
    InitializePoolEvent {
        token_address: token_address.clone(),
        pool_address: pool_address.clone(),
        token_ticker: token_ticker.clone(),
    }
    .publish(e);
}

/// Helper to publish the SwapEvent.
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

/// Helper to publish the BorrowEvent.
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

/// Helper to publish the AddCollateralEvent.
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

/// Helper to publish the RepayEvent.
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

/// Helper to publish the LiquidateEvent.
#[allow(clippy::too_many_arguments)]
pub fn liquidate(
    e: &Env,
    liquidator: &Address,
    borrower_obligation_key: &ObligationKey,
    borrow_pool_address: &Address,
    collateral_pool_address: &Address,
    liquidated_amount: i128,
    collateral_seized_amount: i128,
) {
    LiquidateEvent {
        liquidator: liquidator.clone(),
        borrower_obligation_key: borrower_obligation_key.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
        collateral_pool_address: collateral_pool_address.clone(),
        liquidated_amount,
        collateral_seized_amount,
    }
    .publish(e);
}

/// Helper to publish the RemoveCollateralEvent.
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

/// Helper to publish the WithdrawEvent.
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

/// Helper to publish the FlashLoanEvent.
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

/// Helper to publish the DepositWithLeverageEvent.
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

/// Helper to publish the WithdrawFromLeveragedEvent.
pub fn withdraw_from_leveraged(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    amount: i128,
    actual_amount_withdrawn: i128,
) {
    WithdrawFromLeveragedEvent {
        user: user.clone(),
        deposit_pool_address: deposit_pool_address.clone(),
        borrow_pool_address: borrow_pool_address.clone(),
        amount,
        actual_amount_withdrawn,
    }
    .publish(e);
}

/// Helper to publish the AccrueInterestEvent.
pub fn accrue_interest(e: &Env, user: &Address) {
    AccrueInterestEvent { user: user.clone() }.publish(e);
}

/// Helper to publish the LedgerTimestampError.
pub fn current_ledger_timestamp_smaller_than_stored_timestamp(
    e: &Env,
    current_timestamp: u64,
    stored_timestamp: u64,
) {
    LedgerTimestampError { current_timestamp, stored_timestamp }.publish(e);
}

/// Helper to publish the LeveragedPositionBadDebt.
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

/// Helper to publish the UtilizationRatioExceedsLimit.
pub fn utilization_ratio_exceeds_limit(
    e: &Env,
    utilization_ratio_bps: i128,
    utilization_ratio_limit_bps: i128,
) {
    UtilizationRatioExceedsLimit { utilization_ratio_bps, utilization_ratio_limit_bps }.publish(e);
}

/// Helper to publish the PoolIsMissingInStorage.
pub fn pool_is_missing_in_storage(e: &Env, pool_address: &Address) {
    PoolIsMissingInStorage { pool_address: pool_address.clone() }.publish(e);
}

/// Helper to publish the ObligationIsMissingInStorage.
pub fn obligation_is_missing_in_storage(e: &Env, obligation_key: &ObligationKey) {
    ObligationIsMissingInStorage { obligation_key: obligation_key.clone() }.publish(e);
}

/// Helper to publish the ObligationAmountBecomesNegative.
pub fn obligation_amount_becomes_negative(e: &Env, _old_amount: i128, _new_amount: i128) {
    // ObligationAmountBecomesNegative { old_amount, new_amount }.publish(e);
}

/// Helper to publish the PoolAmountBecomesNegative.
pub fn pool_amount_becomes_negative(e: &Env, old_amount: i128, new_amount: i128) {
    PoolAmountBecomesNegative { old_amount, new_amount }.publish(e);
}

/// Helper to publish the PoolTotalSharesSmallerThanIndividualUserShares.
pub fn pool_total_shares_smaller_than_individual_user_shares(
    _e: &Env,
    _total_shares: i128,
    _individual_shares: i128,
) {
    // PoolTotalSharesSmallerThanIndividualUserShares { total_shares, individual_shares }.publish(e);
}

/// Helper to publish the PoolTotalSharesSmallerThanTotalTokens.
pub fn pool_total_shares_smaller_than_total_tokens(
    _e: &Env,
    _total_shares: i128,
    _total_tokens: i128,
) {
    // PoolTotalSharesSmallerThanTotalTokens { total_shares, total_tokens }.publish(e);
}

/// Helper to publish the PoolContainsInconsistentState.
pub fn pool_contains_inconsistent_state(e: &Env, pool: &Pool) {
    PoolContainsInconsistentState { pool: pool.clone() }.publish(e);
}

/// Helper to publish the ObligationIsUnexpectedlyEmpty.
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

/// Helper to publish the ComputedInterestIsNegative.
pub fn computed_interest_is_negative(
    e: &Env,
    pool_address: &Address,
    shares: i128,
    tokens_from_shares: i128,
    computed_interest: i128,
    tokens_from_all_shares: i128,
) {
    ComputedInterestIsNegative {
        pool_address: pool_address.clone(),
        shares,
        tokens_from_shares,
        computed_interest,
        tokens_from_all_shares,
    }
    .publish(e);
}

/// Helper to publish the ReceivedUnexpectedSwapAmount.
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

// --- Helper Functions  ---

#[contractevent]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DbgEvent {
    #[topic]
    pub symbol: Symbol,
}

/// Helper to publish the DbgEvent.
pub fn dbg(e: &Env, symbol: Symbol) {
    DbgEvent { symbol }.publish(e);
}
