use soroban_sdk::{Address, Env, Symbol};

// ---- Contract's Methods Events ----

/// Emitted when the contract is constructed
///
/// - topics - `["constructor", admin: Address]`
/// - data - `[liquidation_threshold_percent: i128]`
pub fn constructor(e: &Env, admin: &Address, liquidation_threshold_percent: i128) {
    let topics = (Symbol::new(e, "constructor"), admin);
    let data = (liquidation_threshold_percent,);

    e.events().publish(topics, data);
}

/// Emitted when depositing tokens
///
/// - topics - `["deposit", pool_address: Address, user: Address]`
/// - data - `[amount: i128, shares_issued: i128]`
pub fn deposit(e: &Env, pool_address: &Address, user: &Address, amount: i128, shares_issued: i128) {
    let topics = (Symbol::new(e, "deposit"), pool_address, user);
    let data = (amount, shares_issued);

    e.events().publish(topics, data);
}

/// Emitted when a loan pool is initialized
///
/// - topics - `["initialize_pool", token_address: Address]`
/// - data - `[pool_address: Address, token_ticker: Symbol]`
pub fn initialize_pool(
    e: &Env,
    token_address: &Address,
    pool_address: &Address,
    token_ticker: &Symbol,
) {
    let topics = (Symbol::new(e, "initialize_pool"), token_address);
    let data = (pool_address, token_ticker);

    e.events().publish(topics, data);
}

/// Emitted when tokens are swapped
///
/// - topics - `["swap", user: Address, token_in: Address, token_out: Address]`
/// - data - `[amount_in: i128, amount_out: i128, received_amount: i128]`
pub fn swap(
    e: &Env,
    user: &Address,
    token_in: &Address,
    token_out: &Address,
    amount_in: i128,
    amount_out: i128,
    received_amount: i128,
) {
    let topics = (Symbol::new(e, "swap"), user, token_in, token_out);
    let data = (amount_in, amount_out, received_amount);

    e.events().publish(topics, data);
}

/// Emitted when tokens are borrowed from a pool
///
/// - topics - `["borrow", pool_address: Address, user: Address]`
/// - data - `[amount: i128]`
pub fn borrow(e: &Env, pool_address: &Address, user: &Address, amount: i128) {
    let topics = (Symbol::new(e, "borrow"), pool_address, user);
    let data = (amount,);

    e.events().publish(topics, data);
}

/// Emitted when collateral is added to a pool
///
/// - topics - `["add_collateral", pool_address: Address, user: Address]`
/// - data - `[amount: i128]`
pub fn add_collateral(e: &Env, pool_address: &Address, user: &Address, amount: i128) {
    let topics = (Symbol::new(e, "add_collateral"), pool_address, user);
    let data = (amount,);

    e.events().publish(topics, data);
}

/// Emitted when borrowed tokens are repaid
///
/// - topics - `["repay", pool_address: Address, user: Address]`
/// - data - `[amount: i128]`
pub fn repay(e: &Env, pool_address: &Address, user: &Address, amount: i128) {
    let topics = (Symbol::new(e, "repay"), pool_address, user);
    let data = (amount,);

    e.events().publish(topics, data);
}

/// Emitted when a borrower's position is liquidated
///
/// - topics - `["liquidate", liquidator: Address, borrower: Address, borrow_pool: Address, collateral_pool: Address]`
/// - data - `[liquidated_amount: i128, collateral_seized_amount: i128]`
pub fn liquidate(
    e: &Env,
    liquidator: &Address,
    borrower: &Address,
    borrow_pool_address: &Address,
    collateral_pool_address: &Address,
    liquidated_amount: i128,
    collateral_seized_amount: i128,
) {
    let topics = (
        Symbol::new(e, "liquidate"),
        liquidator,
        borrower,
        borrow_pool_address,
        collateral_pool_address,
    );
    let data = (liquidated_amount, collateral_seized_amount);

    e.events().publish(topics, data);
}

/// Emitted when collateral tokens are removed from a pool
///
/// - topics - `["remove_collateral", pool_address: Address, user: Address]`
/// - data - `[amount: i128]`
pub fn remove_collateral(e: &Env, pool_address: &Address, user: &Address, amount: i128) {
    let topics = (Symbol::new(e, "remove_collateral"), pool_address, user);
    let data = (amount,);

    e.events().publish(topics, data);
}

/// Emitted when deposited tokens are withdrawn from a pool
///
/// - topics - `["withdraw", pool_address: Address, user: Address]`
/// - data - `[amount: i128]`
pub fn withdraw(e: &Env, pool_address: &Address, user: &Address, amount: i128) {
    let topics = (Symbol::new(e, "withdraw"), pool_address, user);
    let data = (amount,);

    e.events().publish(topics, data);
}

/// Emitted when a flash loan is initiated
///
/// - topics - `["flash_loan", contract: Address, pool_address: Address]`
/// - data - `[amount: i128, fees_paid: i128]`
pub fn flash_loan(
    e: &Env,
    contract: &Address,
    pool_address: &Address,
    amount: i128,
    fees_paid: i128,
) {
    let topics = (Symbol::new(e, "flash_loan"), contract, pool_address);
    let data = (amount, fees_paid);

    e.events().publish(topics, data);
}

/// Emitted when tokens are deposited with leverage
///
/// - topics - `["deposit_with_leverage", user: Address, deposit_pool: Address, borrow_pool: Address]`
/// - data - `[original_amount: i128, leverage_multiplier: u32, total_deposited_amount: i128, total_borrowed_amount: i128]`
#[allow(clippy::too_many_arguments)]
pub fn deposit_with_leverage(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    original_amount: i128,
    leverage_multiplier: u32,
    total_deposited_amount: i128,
    total_borrowed_amount: i128,
) {
    let topics = (
        Symbol::new(e, "deposit_with_leverage"),
        user,
        deposit_pool_address,
        borrow_pool_address,
    );
    let data = (
        original_amount,
        leverage_multiplier,
        total_deposited_amount,
        total_borrowed_amount,
    );

    e.events().publish(topics, data);
}

/// Emitted when a leveraged deposit position is withdrawn
///
/// - topics - `["withdraw_from_leveraged", user: Address, deposit_pool: Address, borrow_pool: Address]`
/// - data - `[amount: i128, actual_amount_withdrawn: i128]`
pub fn withdraw_from_leveraged(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    amount: i128,
    actual_amount_withdrawn: i128,
) {
    let topics = (
        Symbol::new(e, "withdraw_from_leveraged"),
        user,
        deposit_pool_address,
        borrow_pool_address,
    );
    let data = (amount, actual_amount_withdrawn);

    e.events().publish(topics, data);
}

/// Emitted when a user's obligation interest is accrued
///
/// - topics - `["accrue_interest", user: Address]`
pub fn accrue_interest(e: &Env, user: &Address) {
    let topics = (Symbol::new(e, "accrue_interest"), user);

    e.events().publish(topics, ());
}

// ----- Internal Error Events -----

/// Emitted when a leveraged position incurs bad debt. This typically happens when the value
/// of the collateral for a leveraged loan drops significantly, making it insufficient
/// to cover the borrowed amount, even after accounting for the initial deposit
///
/// - topics - `["leveraged_position_bad_debt", user: Address, deposit_pool_address: Address, borrow_pool_address: Address]`
/// - data - `[deposited_amount: i128, borrowed_amount: i128, borrowed_amount_swapped: i128]`
pub fn leveraged_position_bad_debt(
    e: &Env,
    user: &Address,
    deposit_pool_address: &Address,
    borrow_pool_address: &Address,
    deposited_amount: i128,
    borrowed_amount: i128,
    borrowed_amount_swapped: i128,
) {
    let topics = (
        Symbol::new(e, "leveraged_position_bad_debt"),
        user,
        deposit_pool_address,
        borrow_pool_address,
    );
    let data = (deposited_amount, borrowed_amount, borrowed_amount_swapped);

    e.events().publish(topics, data);
}

/// Emitted when a pool's utilization ratio exceeds a predefined limit
///
/// - topics - `["utilization_ration_exceeds_limit"]`
/// - data - `[utilization_ratio_bps: i128, utilization_ratio_limit_bps: i128]`
pub fn utilization_ration_exceeds_limit(
    e: &Env,
    utilization_ratio_bps: i128,
    utilization_ratio_limit_bps: i128,
) {
    // TODO: This can happen when `total_borrowed` amount on a pool accrued over time by itself, so, maybe, treat it as a regular error?
    let topics = (Symbol::new(e, "utilization_ration_exceeds_limit"),);
    let data = (utilization_ratio_bps, utilization_ratio_limit_bps);

    e.events().publish(topics, data);
}

/// Emitted when an attempt is made to interact with a loan pool that does not exist in storage.
/// This indicates a potential issue with the provided pool address or a data inconsistency
///
/// - topics - `["pool_is_missing_in_storage", pool_address: Address]`
/// - data - `[]`
pub fn pool_is_missing_in_storage(e: &Env, pool_address: &Address) {
    let topics = (Symbol::new(e, "pool_is_missing_in_storage"), pool_address);
    let data = ();

    e.events().publish(topics, data);
}

/// Emitted when an obligation's borrowed amount unexpectedly attempts to become negative.
/// This is an anomalous condition that should not occur under normal operation and
/// likely indicates an error in calculation or logic
///
/// - topics - `["obligation_borrowed_amount_turns_negative"]`
/// - data - `[old_amount: i128, new_amount: i128]`
pub fn obligation_amount_becomes_negative(e: &Env, old_amount: i128, new_amount: i128) {
    let topics = (Symbol::new(e, "obligation_borrowed_amount_turns_negative"),);
    let data = (old_amount, new_amount);

    e.events().publish(topics, data);
}

/// Emitted when a pool's total amount of tokens unexpectedly attempts to become negative.
/// This signifies a critical error, as a pool should always have a non-negative balance
///
/// - topics - `["pool_amount_becomes_negative"]`
/// - data - `[old_amount: i128, new_amount: i128]`
pub fn pool_amount_becomes_negative(e: &Env, old_amount: i128, new_amount: i128) {
    let topics = (Symbol::new(e, "pool_amount_becomes_negative"),);
    let data = (old_amount, new_amount);

    e.events().publish(topics, data);
}

/// Emitted when the total shares in a pool are found to be less than an individual user's shares.
/// This indicates a severe logical inconsistency or a potential corruption of state,
/// as individual shares should never exceed the total available shares in the pool
///
/// - topics - `["pool_total_shares_smaller_than_individual_user_shares"]`
/// - data - `[total_shares: i128, individual_shares: i128]`
pub fn pool_total_shares_smaller_than_individual_user_shares(
    e: &Env,
    total_shares: i128,
    individual_shares: i128,
) {
    let topics = (Symbol::new(
        e,
        "pool_total_shares_smaller_than_individual_user_shares",
    ),);
    let data = (total_shares, individual_shares);

    e.events().publish(topics, data);
}

// TODO: Write simple macro for this and pass `&str` there as input
pub fn dbg(e: &Env, symbol: Symbol) {
    let topics = (symbol,);
    let data = ();

    e.events().publish(topics, data);
}
