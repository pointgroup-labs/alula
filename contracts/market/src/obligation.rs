use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, BytesN, Env, Map, Vec, contracttype};

use crate::{
    constants::*,
    error::MCError,
    events,
    math_utils::MathUtils,
    oracle::get_asset_price,
    pool::{Pool, PoolHealthConfig},
    storage,
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct ObligationKey {
    pub user: Address,
    pub seed: Option<BytesN<32>>,
}

impl ObligationKey {
    pub fn new(user: Address) -> Self {
        Self { user, seed: None }
    }

    pub fn new_with_seed(user: Address, seed: BytesN<32>) -> Self {
        Self { user, seed: Some(seed) }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct Obligation {
    /// Deposited collateral for the obligation, unique by deposit pool address
    pub deposits: Map<Address, DepositObligation>,
    /// Borrowed liquidity for the obligation, unique by borrow pool address
    pub borrows: Map<Address, BorrowObligation>,
    // /// Last update to collateral, liquidity, or their market values
    // pub last_update: u64,
    // /// Market value of deposits
    // pub deposited_value: i128,
    // /// Market value of deposits
    // pub borrowed_value: i128,
}

impl Obligation {
    /// Creates a new obligation for the specified obligation key
    ///
    /// # WARNING
    /// Modifies the obligation's pools storage data by appending the user's address to the
    /// obligation's list
    pub fn new(e: &Env, obligation_key: &ObligationKey) -> Self {
        storage::register_obligation(e, obligation_key);

        Self { deposits: Map::new(e), borrows: Map::new(e) }
    }

    /// Accrues interest on all obligation-related pools
    ///
    /// # WARNING
    /// Modifies the obligation's pools storage data.
    /// Also, accruing interest on an obligation should precede pool retrieval
    pub fn accrue_interest(&self, e: &Env) -> Result<(), MCError> {
        for borrow_pool_address in self.borrows.keys() {
            accrue_interest_on_pool(e, &borrow_pool_address)?;
        }

        for deposit_pool_address in self.deposits.keys() {
            accrue_interest_on_pool(e, &deposit_pool_address)?;
        }

        Ok(())
    }

    /// # Returns
    ///
    /// [`Result::Ok(false)`] if obligation *can* be liquidated,
    /// [`Result::Ok(true)`] if obligation *cannot* be liquidated,
    /// [`Result::Err(MMError)`] if any error occurred during calculation
    pub fn is_healthy(&self, e: &Env) -> Result<bool, MCError> {
        // TODO: Maybe, somehow cache these values?
        let is_healthy = self.compute_collateral_value_scaled_w_close_ltv(e)?
            >= self.compute_debt_value_scaled_w_liability_factors(e)?;

        Ok(is_healthy)
    }

    pub fn is_empty(&self) -> bool {
        self.deposits.is_empty() && self.borrows.is_empty()
    }

    pub fn require_non_healthy(&self, e: &Env) -> Result<(), MCError> {
        if self.is_healthy(e)? {
            return Err(MCError::LiquidatedPositionIsHealthy);
        }

        Ok(())
    }

    /// Computes the max healthy amount of the collateral token(that is used as a deposit or as a
    /// collateral) that can be removed so that the obligation's LTV is equal to the `open LTV`
    /// parameter on the pool
    pub fn compute_max_healthy_collateral_removed_amount(
        &self,
        e: &Env,
        pool: &Pool,
    ) -> Result<i128, MCError> {
        self.compute_max_health_factor_decreasing_amount(
            e,
            pool,
            pool.config.health_config.open_ltv_bps,
        )
    }

    /// Computes the max healthy amount of the token that can be borrowed and that
    /// doesn't exceed the `open LTV` parameter on the pool
    pub fn compute_max_healthy_debt_added_amount(
        &self,
        e: &Env,
        pool: &Pool,
    ) -> Result<i128, MCError> {
        self.compute_max_health_factor_decreasing_amount(
            e,
            pool,
            pool.config.health_config.liability_factor_bps,
        )
    }

    /// Computes the current collateral assets summed value(deposit shares + plain collateral) per
    /// obligation
    pub fn compute_collateral_value(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_obligation) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_collateral_value_scaled(
                e,
                &pool,
                &deposit_obligation,
                BPS_FACTOR,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the current collateral assets summed value(deposit shares + plain collateral) per
    /// obligation, scaling each value with the appropriate `open_ltv_bps` value
    fn compute_collateral_value_scaled_w_open_ltv(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_obligation) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_collateral_value_scaled(
                e,
                &pool,
                &deposit_obligation,
                pool.config.health_config.open_ltv_bps,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the current collateral assets summed value(deposit shares + plain collateral) per
    /// obligation, scaling each value with the appropriate `close_ltv_bps` value
    fn compute_collateral_value_scaled_w_close_ltv(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_obligation) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_collateral_value_scaled(
                e,
                &pool,
                &deposit_obligation,
                pool.config.health_config.close_ltv_bps,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the current debt assets summed value per
    /// obligation
    pub fn compute_debt_value(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_obligation) in self.borrows.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term =
                Self::compute_pool_debt_value_scaled(e, &pool, &deposit_obligation, BPS_FACTOR)?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the current debt assets summed value per
    /// obligation, scaling each value with the appropriate `liability_factor_bps` value
    fn compute_debt_value_scaled_w_liability_factors(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_obligation) in self.borrows.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_debt_value_scaled(
                e,
                &pool,
                &deposit_obligation,
                pool.config.health_config.liability_factor_bps,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the maximum healthy amount of the token that can be additionally added to the debt
    /// or removed from the collateral, scaled with the corresponding coefficient(`open_ltv_bps`
    /// or `liability_factor_bps`, etc)
    fn compute_max_health_factor_decreasing_amount(
        &self,
        e: &Env,
        pool: &Pool,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let collateral_value_scaled = self.compute_collateral_value_scaled_w_open_ltv(e)?;
        let debt_value_scaled = self.compute_debt_value_scaled_w_liability_factors(e)?;

        let max_amount = if collateral_value_scaled <= debt_value_scaled {
            // Since the scaled borrowed assets value exceeds the scaled collateral assets value,
            // any health factor decreasing operation is prohibited
            0
        } else {
            let asset_price = get_asset_price(e, &pool.token_address)?;
            let value_left = collateral_value_scaled - debt_value_scaled;

            // ----
            // 'value_left' = amount * scalar_bps(i.e. liability_factor_bps or open_ltv_bps) *
            // asset_price , implies:
            // 'amount' = value_left / (scalar_bps * asset_price)
            // ----

            let numerator = value_left;
            let denominator =
                asset_price.fixed_mul_floor(scalar_bps, BPS_FACTOR).map_over_or_underflow()?;

            numerator.checked_div(denominator).map_over_or_underflow()?
        };

        Ok(max_amount)
    }

    /// Computes obligation's collateral pool's asset total(collateral + deposit) value scaled
    /// with provided `scalar_bps` value
    fn compute_pool_collateral_value_scaled(
        e: &Env,
        pool: &Pool,
        deposit_obligation: &DepositObligation,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let &DepositObligation { j_tokens, collateral, .. } = deposit_obligation;

        let supply = pool.compute_tokens_from_j_tokens(e, j_tokens)?;
        let total_collateral_tokens = supply.checked_add(collateral).map_over_or_underflow()?;

        Self::compute_asset_value_scaled(e, total_collateral_tokens, pool, scalar_bps)
    }

    /// Computes obligation's debt pool's asset value scaled
    /// with provided `scalar_bps` value
    fn compute_pool_debt_value_scaled(
        e: &Env,
        pool: &Pool,
        borrow_obligation: &BorrowObligation,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let &BorrowObligation { d_tokens, .. } = borrow_obligation;
        let debt = pool.compute_tokens_from_j_tokens(e, d_tokens)?;

        Self::compute_asset_value_scaled(e, debt, pool, scalar_bps)
    }

    fn compute_asset_value_scaled(
        e: &Env,
        amount: i128,
        pool: &Pool,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let price = get_asset_price(e, &pool.token_address)?;
        let value = amount.checked_mul(price).map_over_or_underflow()?;
        let value_scaled = value.fixed_mul_floor(scalar_bps, BPS_FACTOR).map_over_or_underflow()?;

        Ok(value_scaled)
    }

    /// # Returns
    ///
    /// [`Map<ObligationKey, ()>`] containing all obligation keys in the market
    pub fn get_all(e: &Env) -> Map<ObligationKey, ()> {
        storage::get_all_obligations(e)
    }

    /// Deposits assets on an obligation per pool
    pub fn deposit(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<DepositResult, MCError> {
        let mut deposit_obligation =
            self.deposits.get(pool.pool_address.clone()).unwrap_or_default();

        let computed_fees = compute_fees(
            original_amount,
            pool.config.fee_config.deposit_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        let deposited_tokens_minus_fee =
            original_amount.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;
        let j_tokens_to_issue = pool.compute_j_tokens_from_tokens(e, deposited_tokens_minus_fee)?;

        deposit_obligation.adjust_deposited(e, deposited_tokens_minus_fee)?;
        deposit_obligation.adjust_j_tokens(e, j_tokens_to_issue)?;

        self.deposits.set(pool.pool_address.clone(), deposit_obligation);

        Ok(DepositResult {
            j_tokens_to_issue,
            deposited: deposited_tokens_minus_fee,
            computed_fees,
        })
    }

    /// Borrows assets on an obligation
    pub fn borrow(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<BorrowResult, MCError> {
        let max_healthy_borrow_added_amount =
            self.compute_max_healthy_debt_added_amount(e, pool)?;
        let real_borrowed_amount = i128::min(max_healthy_borrow_added_amount, original_amount);

        pool.require_borrow_preserves_ur_cap(e, real_borrowed_amount)?;

        // WARN: This can potentially create a borrow obligation with 0ed fields
        let mut borrow_obligation = self.borrows.get(pool.pool_address.clone()).unwrap_or_default();

        let computed_fees = compute_fees(
            real_borrowed_amount,
            pool.config.fee_config.borrow_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        // 'what borrower receives' = 'borrower debt' - 'fees'
        let borrower_to_receive =
            real_borrowed_amount.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;
        let d_tokens_to_issue = pool.compute_d_tokens_from_tokens(e, real_borrowed_amount)?;

        borrow_obligation.adjust_d_tokens(e, d_tokens_to_issue)?;
        borrow_obligation.adjust_borrowed(e, real_borrowed_amount)?;

        self.borrows.set(pool.pool_address.clone(), borrow_obligation);

        Ok(BorrowResult {
            d_tokens_to_issue,
            borrower_to_receive,
            borrower_new_debt: real_borrowed_amount,
            computed_fees,
        })
    }

    /// Adds collateral assets on an obligation per pool
    pub fn add_collateral(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<AddCollateralResult, MCError> {
        let mut deposit_obligation =
            self.deposits.get(pool.pool_address.clone()).unwrap_or_default();

        let computed_fees = compute_fees(
            original_amount,
            pool.config.fee_config.add_collateral_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        let added_collateral =
            original_amount.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;
        deposit_obligation.adjust_collateral(e, added_collateral)?;

        self.deposits.set(pool.pool_address.clone(), deposit_obligation);

        Ok(AddCollateralResult { added_collateral, computed_fees })
    }

    /// Withdraws assets from an obligation per pool
    pub fn withdraw(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<WithdrawResult, MCError> {
        let mut deposit_obligation =
            self.deposits.get(pool.pool_address.clone()).ok_or(MCError::ObligationDoesNotExist)?;

        let max_healthy_withdrawn_amount =
            self.compute_max_healthy_collateral_removed_amount(e, pool)?;
        let all_deposit = pool.compute_tokens_from_j_tokens(e, deposit_obligation.j_tokens)?;
        let deposit_decrease =
            i128::min(i128::min(original_amount, max_healthy_withdrawn_amount), all_deposit);

        if deposit_decrease > pool.total_available {
            // TODO: Add failing test here
            return Err(MCError::NotEnoughPoolFunds);
        }

        let above_ur_cap_withdraw_fee = {
            let new_ur_bps = {
                let new_total = pool.total_supply()? - deposit_decrease; // safe

                if new_total == 0 {
                    BPS_FACTOR
                } else {
                    pool.total_borrowed
                        .fixed_div_ceil(new_total, BPS_FACTOR)
                        .map_over_or_underflow()?
                }
            };

            let diff_bps = if new_ur_bps > pool.config.health_config.utilization_ratio_limit_bps {
                let deposit_decrease_to_available_bps = deposit_decrease
                    .fixed_div_ceil(
                        pool.total_available_minus_accumulated_reserve_fees()?,
                        BPS_FACTOR,
                    )
                    .map_over_or_underflow()?;

                if deposit_decrease_to_available_bps
                    > pool.config.health_config.withdraw_scarcity_limit_bps
                {
                    return Err(MCError::OverScarcityLimitWithdraw);
                } else {
                    let last_scarcity_withdraw_ts = deposit_obligation.last_scarcity_withdraw_ts;
                    let scarcity_withdraw_cooldown =
                        pool.config.health_config.withdraw_scarcity_cooldown;
                    let current_timestamp = e.ledger().timestamp();

                    if current_timestamp
                        < last_scarcity_withdraw_ts
                            .checked_add(scarcity_withdraw_cooldown)
                            .map_over_or_underflow()?
                    {
                        return Err(MCError::ScarcityCooldownPeriod);
                    }

                    deposit_obligation.last_scarcity_withdraw_ts = current_timestamp;
                }

                new_ur_bps - pool.config.health_config.utilization_ratio_limit_bps // safe
            } else {
                0
            } as u32;

            diff_bps
                .checked_mul(pool.config.fee_config.over_ur_withdraw_fee_scalar)
                .map_over_or_underflow()?
        };

        let withdraw_fee_bps = pool
            .config
            .fee_config
            .withdraw_fee_bps
            .checked_add(above_ur_cap_withdraw_fee)
            .map_over_or_underflow()?;

        let computed_fees =
            compute_fees(deposit_decrease, withdraw_fee_bps, pool.config.fee_config.host_fee_bps)?;

        let withdrawer_to_receive =
            deposit_decrease.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;

        let j_tokens_to_burn = pool.compute_j_tokens_from_tokens(e, deposit_decrease)?;

        let mut received_interest =
            all_deposit.checked_sub(deposit_obligation.deposited).map_over_or_underflow()?;

        if received_interest < 0 {
            events::computed_interest_is_negative(
                e,
                &pool.pool_address,
                j_tokens_to_burn,
                deposit_decrease,
                received_interest,
                all_deposit,
            );

            received_interest = 0;
            // return Err(MCError::InternalError);
        }

        if deposit_decrease >= received_interest {
            let deposited_diff = deposit_decrease - received_interest; // safe

            deposit_obligation
                .adjust_deposited(e, deposited_diff.checked_neg().map_over_or_underflow()?)?;
        }

        deposit_obligation
            .adjust_j_tokens(e, j_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;

        if deposit_obligation.is_empty() {
            self.deposits.remove(pool.pool_address.clone());
        } else {
            self.deposits.set(pool.pool_address.clone(), deposit_obligation);
        }

        Ok(WithdrawResult {
            j_tokens_to_burn,
            deposit_decrease,
            withdrawer_to_receive,
            computed_fees,
        })
    }

    /// Removes collateral assets from an obligation per pool
    pub fn remove_collateral(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<RemoveCollateralResult, MCError> {
        let mut deposit_obligation =
            self.deposits.get(pool.pool_address.clone()).unwrap_or_default();

        let max_possible_collateral_removed_amount =
            self.compute_max_healthy_collateral_removed_amount(e, pool)?;
        let collateral_decrease = i128::min(
            i128::min(original_amount, max_possible_collateral_removed_amount),
            deposit_obligation.collateral,
        );

        let computed_fees = compute_fees(
            collateral_decrease,
            pool.config.fee_config.remove_collateral_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        let collateral_remover_to_receive =
            collateral_decrease.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;

        deposit_obligation
            .adjust_collateral(e, collateral_decrease.checked_neg().map_over_or_underflow()?)?;

        if deposit_obligation.is_empty() {
            self.deposits.remove(pool.pool_address.clone());
        } else {
            self.deposits.set(pool.pool_address.clone(), deposit_obligation);
        }

        Ok(RemoveCollateralResult {
            collateral_decrease,
            collateral_remover_to_receive,
            computed_fees,
        })
    }

    /// Repays the debt on a specific obligation per pool. Since `repaid_amount` can exceed the debt
    /// — the real repaid amount is calculated as `min(debt, repaid_amount)`
    ///
    /// # Returns
    /// [`Result::Ok((real_repaid_amount, d_tokens_burnt))`] in success and
    /// [`Err(MCError)`] in failure
    pub fn repay(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<RepayResult, MCError> {
        let mut borrow_obligation =
            self.borrows.get(pool.pool_address.clone()).ok_or(MCError::ObligationDoesNotExist)?;

        let all_debt = pool.compute_tokens_from_d_tokens(e, borrow_obligation.d_tokens)?;
        let all_debt_fees = compute_fees(
            all_debt,
            pool.config.fee_config.repay_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?
        .fee_sum;

        let amount_to_repay_all_debt =
            all_debt.checked_add(all_debt_fees).map_over_or_underflow()?;

        let amount_to_take_from_borrower = i128::min(original_amount, amount_to_repay_all_debt);

        let computed_fees = compute_fees(
            amount_to_take_from_borrower,
            pool.config.fee_config.repay_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        let debt_decrease = amount_to_take_from_borrower
            .checked_sub(computed_fees.fee_sum)
            .map_over_or_underflow()?;
        let d_tokens_to_burn = pool.compute_d_tokens_from_tokens(e, debt_decrease)?;

        let mut unpaid_interest =
            all_debt.checked_sub(borrow_obligation.borrowed).map_over_or_underflow()?;
        if unpaid_interest < 0 {
            events::computed_interest_is_negative(
                e,
                &pool.pool_address,
                d_tokens_to_burn,
                debt_decrease,
                unpaid_interest,
                all_debt,
            );

            unpaid_interest = 0;
            // return Err(MCError::InternalError);
        }

        if debt_decrease >= unpaid_interest {
            let borrowed_diff = debt_decrease - unpaid_interest; // safe
            borrow_obligation
                .adjust_borrowed(e, borrowed_diff.checked_neg().map_over_or_underflow()?)?;
        }

        borrow_obligation
            .adjust_d_tokens(e, d_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;

        if borrow_obligation.is_empty() {
            self.borrows.remove(pool.pool_address.clone());
        } else {
            self.borrows.set(pool.pool_address.clone(), borrow_obligation);
        }

        let amount_to_send_back = if amount_to_take_from_borrower < original_amount {
            original_amount - amount_to_take_from_borrower
        } else {
            0
        };

        Ok(RepayResult {
            d_tokens_to_burn,
            debt_repaid: debt_decrease,
            amount_to_send_back,
            computed_fees,
        })
    }

    /// Liquidates unhealthy borrow
    pub fn liquidate(
        &mut self,
        e: &Env,
        borrow_pool_address: &Address,
        collateral_pool_address: &Address,
        borrow_pool: &Pool,
        collateral_pool: &Pool,
        amount: i128,
    ) -> Result<LiquidationValues, MCError> {
        let (mut collateral_obligation, mut borrow_obligation) = (
            self.deposits
                .get(collateral_pool_address.clone())
                .ok_or(MCError::DepositDoesNotExist)?,
            self.borrows.get(borrow_pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?,
        );

        let PoolHealthConfig { liquidation_close_factor_bps, liquidation_incentive_bps, .. } =
            borrow_pool.config.health_config;

        let borrow_obligation_d_tokens = borrow_obligation.d_tokens;
        let borrow_obligation_d_tokens_as_tokens =
            borrow_pool.compute_tokens_from_d_tokens(e, borrow_obligation_d_tokens)?;

        let collateral_obligation_j_tokens = collateral_obligation.j_tokens;
        let collateral_obligation_j_tokens_as_tokens =
            collateral_pool.compute_tokens_from_j_tokens(e, collateral_obligation_j_tokens)?;

        // 'liquidatable_bps' == ((amount * 10_000) / total_debt)
        let liquidatable_bps = amount
            .fixed_div_floor(borrow_obligation_d_tokens_as_tokens, BPS_FACTOR)
            .map_over_or_underflow()?;
        if liquidatable_bps > liquidation_close_factor_bps {
            // TODO: What's the best way to set `close_factor_bps` value?
            return Err(MCError::LiquidationExceedsCloseFactor);
        }

        let borrow_price = get_asset_price(e, &borrow_pool.token_address)?;
        let liquidation_value = amount.checked_mul(borrow_price).map_over_or_underflow()?;

        // Value, which liquidator would like to receive if a full liquidation takes place
        // 'liquidation_value_with_incentive' == (liquidation_value * (10_000 +
        // liquidation_incentive_bps)) / 10_000
        let liquidation_value_with_incentive = liquidation_value
            .fixed_mul_floor(BPS_FACTOR + liquidation_incentive_bps, BPS_FACTOR)
            .map_over_or_underflow()?;

        let collateral_price = get_asset_price(e, &collateral_pool.token_address)?;
        let full_collateral_amount = collateral_obligation.collateral;
        let full_collateral_value =
            full_collateral_amount.checked_mul(collateral_price).map_over_or_underflow()?;

        let liquidation_values = if full_collateral_value >= liquidation_value_with_incentive {
            let collateral_amount_sold = liquidation_value_with_incentive
                .checked_div(collateral_price)
                .map_over_or_underflow()?;
            let d_tokens_repaid = borrow_pool.compute_d_tokens_from_tokens(e, amount)?;

            LiquidationValues {
                liquidated_amount: amount,
                d_tokens_repaid,
                collateral_amount_sold,
                j_tokens_amount_sold: 0,
                tokens_from_sold_j_tokens: 0,
            }
        } else {
            let value_left = liquidation_value_with_incentive - full_collateral_value; // safe

            let full_collateral_j_tokens = collateral_obligation.j_tokens;
            let j_tokens_as_tokens =
                collateral_pool.compute_tokens_from_j_tokens(e, full_collateral_j_tokens)?;
            let available_tokens_from_j_tokens =
                i128::min(collateral_pool.total_available, j_tokens_as_tokens);

            let tokens_from_j_tokens_value = available_tokens_from_j_tokens
                .checked_mul(collateral_price)
                .map_over_or_underflow()?;

            if tokens_from_j_tokens_value >= value_left {
                let tokens_from_sold_j_tokens =
                    value_left.checked_div(collateral_price).map_over_or_underflow()?;
                let j_tokens_amount_sold =
                    collateral_pool.compute_j_tokens_from_tokens(e, tokens_from_sold_j_tokens)?;
                let d_tokens_repaid = borrow_pool.compute_d_tokens_from_tokens(e, amount)?;

                LiquidationValues {
                    liquidated_amount: amount,
                    d_tokens_repaid,
                    collateral_amount_sold: full_collateral_amount,
                    j_tokens_amount_sold,
                    tokens_from_sold_j_tokens,
                }
            } else {
                // The case when full liquidation cannot take place because of not enough available
                // amount in the pool
                let collateral_value_sum = full_collateral_value
                    .checked_add(tokens_from_j_tokens_value)
                    .map_over_or_underflow()?;
                let tokens_per_collateral =
                    collateral_value_sum.checked_div(collateral_price).map_over_or_underflow()?;

                let numerator = BPS_FACTOR - liquidation_incentive_bps; // safe
                let denominator = BPS_FACTOR;

                // Liquidator cannot receive the entire desired value of collateral,
                // so only a proportional amount of tokens must be repaid
                let tokens_per_collateral_minus_incentive = tokens_per_collateral
                    .checked_mul(numerator)
                    .map_over_or_underflow()?
                    .checked_div(denominator)
                    .map_over_or_underflow()?;
                let d_tokens_repaid = borrow_pool
                    .compute_d_tokens_from_tokens(e, tokens_per_collateral_minus_incentive)?;

                LiquidationValues {
                    liquidated_amount: tokens_per_collateral_minus_incentive,
                    d_tokens_repaid,
                    collateral_amount_sold: full_collateral_amount,
                    j_tokens_amount_sold: full_collateral_j_tokens,
                    tokens_from_sold_j_tokens: available_tokens_from_j_tokens,
                }
            }
        };

        let unpaid_interest = borrow_obligation_d_tokens_as_tokens - borrow_obligation.borrowed;
        let borrowed_diff = if liquidation_values.liquidated_amount >= unpaid_interest {
            liquidation_values.liquidated_amount - unpaid_interest // safe
        } else {
            0
        };

        let received_interest =
            collateral_obligation_j_tokens_as_tokens - collateral_obligation.deposited;
        let deposited_diff = if liquidation_values.tokens_from_sold_j_tokens >= received_interest {
            liquidation_values.tokens_from_sold_j_tokens - received_interest // safe
        } else {
            0
        };

        borrow_obligation
            .adjust_borrowed(e, borrowed_diff.checked_neg().map_over_or_underflow()?)?;
        borrow_obligation.adjust_d_tokens(
            e,
            liquidation_values.d_tokens_repaid.checked_neg().map_over_or_underflow()?,
        )?;

        collateral_obligation
            .adjust_deposited(e, deposited_diff.checked_neg().map_over_or_underflow()?)?;
        collateral_obligation.adjust_j_tokens(
            e,
            liquidation_values.j_tokens_amount_sold.checked_neg().map_over_or_underflow()?,
        )?;
        collateral_obligation.adjust_collateral(
            e,
            liquidation_values.collateral_amount_sold.checked_neg().map_over_or_underflow()?,
        )?;

        self.borrows.set(borrow_pool_address.clone(), borrow_obligation);
        self.deposits.set(collateral_pool_address.clone(), collateral_obligation);
        // TODO: Remove empty obligations?

        Ok(liquidation_values)
    }

    /// Covers the obligation's bad debt by returning the list of the obligation's positions
    pub fn cover_bad_debt(&self, e: &Env) -> Result<CoverBadDebtResult, MCError> {
        let collateral_value = self.compute_collateral_value(e)?;
        let debt_value = self.compute_debt_value(e)?;

        if collateral_value >= debt_value {
            return Err(MCError::PositionDoesNotHaveBadDebt);
        }

        let mut borrows_to_be_compensated: Vec<(Address, i128)> = Vec::new(e);
        for (pool_address, borrow_obligation) in self.borrows.iter() {
            borrows_to_be_compensated.push_back((pool_address, borrow_obligation.d_tokens));
        }

        let mut collaterals_to_remove: Vec<(Address, i128, i128)> = Vec::new(e);
        for (pool_address, deposit_obligation) in self.deposits.iter() {
            collaterals_to_remove.push_back((
                pool_address,
                deposit_obligation.j_tokens,
                deposit_obligation.collateral,
            ));
        }

        Ok(CoverBadDebtResult { borrows_to_be_compensated, collaterals_to_remove })
    }

    /// Returns the amount of `jTokens` that the obligation has in the specified pool
    pub fn get_j_tokens(&self, pool_address: &Address) -> Result<i128, MCError> {
        let deposit_obligation =
            self.deposits.get(pool_address.clone()).ok_or(MCError::DepositDoesNotExist)?;

        Ok(deposit_obligation.j_tokens)
    }

    /// Calculates the interest that the obligation received from the deposit pool
    pub fn get_received_interest(&self, e: &Env, pool_address: &Address) -> Result<i128, MCError> {
        let deposit_obligation =
            self.deposits.get(pool_address.clone()).ok_or(MCError::DepositDoesNotExist)?;
        let deposit_pool = Pool::try_get(e, pool_address)?;

        let total_supply =
            deposit_pool.compute_j_tokens_from_tokens(e, deposit_obligation.j_tokens)?;
        let deposited = deposit_obligation.deposited;

        if total_supply < deposited {
            // TODO: Add an event?
            return Err(MCError::InternalError);
        }

        let unpaid_interest = total_supply - deposited; // safe

        Ok(unpaid_interest)
    }

    /// Calculates the interest that the obligation owes to the borrow pool
    pub fn get_unpaid_interest(&self, e: &Env, pool_address: &Address) -> Result<i128, MCError> {
        let borrow_obligation =
            self.borrows.get(pool_address.clone()).ok_or(MCError::DepositDoesNotExist)?;
        let borrow_pool = Pool::try_get(e, pool_address)?;

        let total_debt = borrow_pool.compute_tokens_from_d_tokens(e, borrow_obligation.d_tokens)?;
        let borrowed = borrow_obligation.borrowed;

        if total_debt < borrowed {
            // TODO: Add an event?
            return Err(MCError::InternalError);
        }
        let unpaid_interest = total_debt - borrowed; // safe

        Ok(unpaid_interest)
    }

    /// Returns the amount of `dTokens` that the obligation has in the specified pool
    pub fn get_borrowed(&self, pool_address: &Address) -> Result<i128, MCError> {
        let Some(borrow_obligation) = self.borrows.get(pool_address.clone()) else {
            return Err(MCError::BorrowDoesNotExist);
        };

        Ok(borrow_obligation.borrowed)
    }

    /// Returns the total debt (including interest) that the obligation has in the specified pool
    /// (in tokens, not in dTokens)
    pub fn get_total_debt(&self, e: &Env, pool_address: &Address) -> Result<i128, MCError> {
        let borrow_obligation =
            self.borrows.get(pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?;

        let borrow_pool = Pool::try_get(e, pool_address).map_err(|_| {
            events::pool_is_missing_in_storage(e, pool_address);
            MCError::InternalError
        })?;

        let total_debt = borrow_pool.compute_tokens_from_d_tokens(e, borrow_obligation.d_tokens)?;

        Ok(total_debt)
    }

    /// Returns the amount of collateral that the obligation has in the specified pool
    pub fn get_collateral(&self, pool_address: &Address) -> Result<i128, MCError> {
        let Some(deposit_obligation) = self.deposits.get(pool_address.clone()) else {
            return Err(MCError::DepositDoesNotExist);
        };

        Ok(deposit_obligation.collateral)
    }

    /// Saves/updates obligation in the contract's storage
    ///
    /// # Arguments
    /// * `user` - obligation's user address. **MUST** equal the original user address
    /// * `seed` - obligation's user seed. **MUST** equal the original obligation seed
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn set(&self, e: &Env, obligation_key: &ObligationKey) {
        storage::set_obligation(e, obligation_key, self);
    }

    /// Tries to get the user's obligation from the contract's storage
    ///
    /// # Returns
    /// - [`Ok(Obligation)`] if a pool with the given address exists in the contract's storage
    /// - [`Err(MCError::ObligationDoesNotExist)`] otherwise
    pub fn try_get(e: &Env, obligation_key: &ObligationKey) -> Result<Self, MCError> {
        storage::get_obligation(e, obligation_key).ok_or(MCError::ObligationDoesNotExist)
    }

    /// Removes obligation from the contract's storage
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn remove(self, e: &Env, obligation_key: &ObligationKey) {
        storage::remove_obligation(e, obligation_key);
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[contracttype]
pub struct BorrowObligation {
    /// Amount of the total debt shares that the obligation contains
    pub d_tokens: i128,
    /// Accumulated value of initially borrowed tokens
    pub borrowed: i128,
}

impl BorrowObligation {
    pub fn adjust_d_tokens(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.d_tokens, adjusting_amount)?;
        self.d_tokens = new_amount;

        Ok(())
    }

    pub fn adjust_borrowed(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.borrowed, adjusting_amount)?;
        self.borrowed = new_amount;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.d_tokens == 0 && self.borrowed == 0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[contracttype]
pub struct DepositObligation {
    /// A share of total supplied tokens in the pool that obligation contains
    pub j_tokens: i128,
    /// Accumulated value of collateral that doesn't accrue interest
    pub collateral: i128,
    /// Accumulated value of initially deposited tokens. E.g., if a user initially deposited 100
    /// tokens, the time passed, which caused 2 tokens to be accrued, and the user deposited 20
    /// more tokens - this value will be equal to 120
    pub deposited: i128,
    /// Timestamp of when the last scarcity withdraw took place
    pub last_scarcity_withdraw_ts: u64,
}

impl DepositObligation {
    pub fn new() -> Self {
        Self { collateral: 0, j_tokens: 0, deposited: 0, last_scarcity_withdraw_ts: 0 }
    }

    pub fn adjust_j_tokens(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.j_tokens, adjusting_amount)?;
        self.j_tokens = new_amount;

        Ok(())
    }

    pub fn adjust_deposited(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.deposited, adjusting_amount)?;
        self.deposited = new_amount;

        Ok(())
    }

    pub fn adjust_collateral(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.collateral, adjusting_amount)?;
        self.collateral = new_amount;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        if self.j_tokens == 0 && self.deposited != 0 {
            // TODO: Invariant breakage
        }

        self.j_tokens == 0 && self.collateral == 0
    }
}

/// Adjusts a field on the obligation's structs
///
/// # Returns
/// `Ok(new_amount)` if adjusting doesn't lead to a new amount being negative.
/// `Err(MCError::InternalError)` otherwise
fn adjust_obligation_field(
    e: &Env,
    current_value: i128,
    adjusting_amount: i128,
) -> Result<i128, MCError> {
    let new_amount = current_value.checked_add(adjusting_amount).map_over_or_underflow()?;

    if new_amount < 0 {
        events::obligation_amount_becomes_negative(e, current_value, new_amount);

        return Err(MCError::InternalError);
    }

    Ok(new_amount)
}

/// Accrues interest on a pool
///
/// # WARNING
/// Modifies the contract's storage
fn accrue_interest_on_pool(e: &Env, pool_address: &Address) -> Result<(), MCError> {
    let mut pool = storage::get_pool(e, pool_address).ok_or(MCError::PoolDoesNotExist)?;

    pool.accrue_interest(e)?;
    pool.set(e);

    Ok(())
}

/// Computes fees for any operations
///
/// # Arguments
/// * `original_amount` - original operation amount
/// * `operation_fee_bps` - percentage of the original amount that is segregated for fees
/// * `host_fee_bps` - percentage of the operation fee that is segregated for the host lending
///   platform
pub fn compute_fees(
    original_amount: i128,
    operation_fee_bps: u32,
    host_fee_bps: u32,
) -> Result<ComputedFees, MCError> {
    let fee_sum = original_amount
        .fixed_mul_floor(operation_fee_bps as i128, BPS_FACTOR)
        .map_over_or_underflow()?;
    let host_fee =
        fee_sum.fixed_mul_floor(host_fee_bps as i128, BPS_FACTOR).map_over_or_underflow()?;
    let market_fee = fee_sum.checked_sub(host_fee).map_over_or_underflow()?;

    Ok(ComputedFees { fee_sum, market_fee, host_fee })
}

#[contracttype]
#[derive(Clone)]
/// Generally represents computed fees issued by any possible operation on a market
pub struct ComputedFees {
    /// Sum of `market_fee` and `host_fee`
    pub fee_sum: i128,
    /// Fee segregated to the market admin
    pub market_fee: i128,
    /// Fee segregated to the protocol host
    pub host_fee: i128,
}

#[contracttype]
/// [`Obligation::deposit`] resulting data
pub struct DepositResult {
    /// Amount of `jTokens` to issue that represent the `originally_deposited` amount in the pool
    pub j_tokens_to_issue: i128,
    /// Amount of originally deposited tokens(minus all fees)
    pub deposited: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
/// [`Obligation::borrow`] resulting data
pub struct BorrowResult {
    /// Amount of `dTokens` to issue that represent the `borrower_new_debt` amount in the pool
    pub d_tokens_to_issue: i128,
    /// Amount of debt(in tokens) that is added to the borrower's obligation
    pub borrower_new_debt: i128,
    /// Amount of tokens to receive by the borrower(`borrower_new_debt` minus fees)
    pub borrower_to_receive: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
/// [`Obligation::add_collateral`] resulting data
pub struct AddCollateralResult {
    /// Amount of tokens added as collateral(with subtracted fees)
    pub added_collateral: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
/// [`Obligation::withdraw`] resulting data
pub struct WithdrawResult {
    /// Amount of `jTokens` to burn that represent the `deposit_decreased_amount` amount in the
    /// pool
    pub j_tokens_to_burn: i128,
    /// Amount of the original deposit(in tokens) that is removed from the `DepositObligation`
    pub deposit_decrease: i128,
    /// Amount of tokens to receive by the withdrawer(`deposit_decreased_amount` minus fees)
    pub withdrawer_to_receive: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
#[derive(Clone)]
/// [`Obligation::repay`] resulting data
pub struct RepayResult {
    /// Amount of `dTokens` to issue that represent the `real_repaid` amount in the pool
    pub d_tokens_to_burn: i128,
    /// Amount of the debt that is repaid
    pub debt_repaid: i128,
    /// Excess amount given by the borrower that is sent back
    pub amount_to_send_back: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
/// [`Obligation::remove_collateral`] resulting data
pub struct RemoveCollateralResult {
    /// Amount of collateral tokens removed
    pub collateral_decrease: i128,
    /// Amount of collateral tokens received by the collateral remover(accounting subtracted fees)
    pub collateral_remover_to_receive: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
/// [`Obligation::cover_bad_debt`] resulting data
pub struct CoverBadDebtResult {
    /// `(pool address, borrower dTokens)` pairs for each bad debt obligation borrows
    pub borrows_to_be_compensated: Vec<(Address, i128)>,
    /// `(pool address, borrower jTokens, borrower collateral)` tuples for each bad debt obligation
    /// collateral
    pub collaterals_to_remove: Vec<(Address, i128, i128)>,
}

// TODO: Move this somewhere else when working on liquidation
pub struct LiquidationValues {
    /// The amount of tokens repaid by the liquidator
    pub liquidated_amount: i128,
    /// The amount of dTokens repaid by the liquidator
    pub d_tokens_repaid: i128,
    /// The number of the borrower's collateral tokens that are taken by the liquidator
    pub collateral_amount_sold: i128,
    /// The number of available pool tokens that are taken from the borrower's jTokens
    pub j_tokens_amount_sold: i128,
    /// The number of tokens that correspond to the sold jTokens
    pub tokens_from_sold_j_tokens: i128,
}
