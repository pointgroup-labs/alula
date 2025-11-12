use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Bytes, BytesN, Env, Map, Vec, contracttype};

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
    pub deposits: Map<Address, DepositPosition>,
    /// Borrowed liquidity for the obligation, unique by borrow pool address
    pub borrows: Map<Address, BorrowPosition>,
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

    /// # Returns
    ///
    /// [`Map<ObligationKey, ()>`] containing all obligation keys in the market
    pub fn get_all(e: &Env) -> Map<ObligationKey, ()> {
        storage::get_all_obligations(e)
    }

    /// Removes obligation from the contract's storage
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn remove(self, e: &Env, obligation_key: &ObligationKey) {
        storage::remove_obligation(e, obligation_key);
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
    /// [`Result::Ok(false)`] if obligation **CAN** be liquidated,
    /// [`Result::Ok(true)`] if obligation **CANNOT** be liquidated,
    /// [`Result::Err(MMError)`] if any error occurred during calculation
    pub fn is_healthy(&self, e: &Env) -> Result<bool, MCError> {
        // TODO: Maybe, somehow cache these values?
        let is_healthy = self.compute_collateral_value_scaled_w_close_ltvs(e)?
            >= self.compute_debt_value_scaled_w_liability_factors(e)?;

        Ok(is_healthy)
    }

    pub fn is_empty(&self) -> bool {
        self.deposits.is_empty() && self.borrows.is_empty()
    }

    pub fn borrow_exists(&self) -> bool {
        !self.borrows.is_empty()
    }

    pub fn deposit_exists(&self) -> bool {
        !self.deposits.is_empty()
    }

    // ------ `require_X` Circuit Breakers ------

    pub fn require_non_healthy(&self, e: &Env) -> Result<(), MCError> {
        if self.is_healthy(e)? {
            return Err(MCError::LiquidatedObligationIsHealthy);
        }

        Ok(())
    }

    pub fn require_no_borrow_position_exists(&self, pool_address: &Address) -> Result<(), MCError> {
        if self.borrows.contains_key(pool_address.clone()) {
            return Err(MCError::BorrowPositionForAssetExists);
        }

        Ok(())
    }

    pub fn require_no_deposit_position_exists(
        &self,
        pool_address: &Address,
    ) -> Result<(), MCError> {
        // MEGA_WARN: Must be 'deposits' Add failing tests for this
        if self.deposits.contains_key(pool_address.clone()) {
            return Err(MCError::DepositPositionForAssetExists);
        }

        Ok(())
    }

    pub fn require_is_liquidatable_pair(
        &self,
        borrow_pool_address: &Address,
        collateral_pool_address: &Address,
    ) -> Result<(), MCError> {
        if !self.borrows.contains_key(borrow_pool_address.clone())
            || !self.deposits.contains_key(collateral_pool_address.clone())
        {
            return Err(MCError::PairIsNotLiquidatable);
        }

        Ok(())
    }

    // ------ Health Factor Removing Computations ------

    /// Computes the current collateral assets summed value(deposit shares + plain collateral) per
    /// obligation
    pub fn compute_collateral_value(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_position) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_collateral_value_scaled(
                e,
                &pool,
                &deposit_position,
                BPS_FACTOR,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
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

    /// Computes the maximum healthy amount of the token that can be additionally added to the debt
    /// or removed from the collateral, scaled with the corresponding coefficient(`open_ltv_bps`
    /// or `liability_factor_bps`, etc)
    fn compute_max_health_factor_decreasing_amount(
        &self,
        e: &Env,
        pool: &Pool,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let collateral_value_scaled = self.compute_collateral_value_scaled_w_open_ltvs(e)?;
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

    /// Computes the current debt assets summed value per
    /// obligation, scaling each value with the appropriate `liability_factor_bps` value
    fn compute_debt_value_scaled_w_liability_factors(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, borrow_position) in self.borrows.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_debt_value_scaled(
                e,
                &pool,
                &borrow_position,
                pool.config.health_config.liability_factor_bps,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the current debt assets summed value per
    /// obligation
    pub fn compute_debt_value(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, borrow_position) in self.borrows.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term =
                Self::compute_pool_debt_value_scaled(e, &pool, &borrow_position, BPS_FACTOR)?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the current collateral assets summed value(deposit shares + plain collateral) per
    /// obligation, scaling each value with the appropriate `close_ltv_bps` value
    fn compute_collateral_value_scaled_w_close_ltvs(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_position) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_collateral_value_scaled(
                e,
                &pool,
                &deposit_position,
                pool.config.health_config.close_ltv_bps,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes the current collateral assets summed value(deposit shares + plain collateral) per
    /// obligation, scaling each value with the appropriate `open_ltv_bps` value
    fn compute_collateral_value_scaled_w_open_ltvs(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_position) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term = Self::compute_pool_collateral_value_scaled(
                e,
                &pool,
                &deposit_position,
                pool.config.health_config.open_ltv_bps,
            )?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    /// Computes obligation's collateral pool's asset total(collateral + deposit) value scaled
    /// with provided `scalar_bps` value
    fn compute_pool_collateral_value_scaled(
        e: &Env,
        pool: &Pool,
        deposit_position: &DepositPosition,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let &DepositPosition { j_tokens, collateral, .. } = deposit_position;

        let supply = pool.compute_tokens_from_j_tokens_floor(e, j_tokens)?;
        let total_collateral_tokens = supply.checked_add(collateral).map_over_or_underflow()?;

        Self::compute_asset_value_scaled(e, total_collateral_tokens, pool, scalar_bps)
    }

    /// Computes obligation's debt pool's asset value scaled
    /// with provided `scalar_bps` value
    fn compute_pool_debt_value_scaled(
        e: &Env,
        pool: &Pool,
        borrow_position: &BorrowPosition,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let &BorrowPosition { d_tokens, .. } = borrow_position;
        let debt = pool.compute_tokens_from_d_tokens_ceil(e, d_tokens)?;

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

    // ------ Operations ------

    /// Deposits assets on an obligation per pool
    pub fn deposit(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<DepositResult, MCError> {
        let mut deposit_position = self.deposits.get(pool.pool_address.clone()).unwrap_or_default();

        let computed_fees = compute_fees(
            original_amount,
            pool.config.fee_config.deposit_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        let deposited_tokens_minus_fee =
            original_amount.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;
        let j_tokens_to_issue =
            pool.compute_j_tokens_from_tokens_floor(e, deposited_tokens_minus_fee)?;

        deposit_position.adjust_deposited(e, deposited_tokens_minus_fee)?;
        deposit_position.adjust_j_tokens(e, j_tokens_to_issue)?;

        self.deposits.set(pool.pool_address.clone(), deposit_position);

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
        let mut borrow_position = self.borrows.get(pool.pool_address.clone()).unwrap_or_default();

        let computed_fees = compute_fees(
            real_borrowed_amount,
            pool.config.fee_config.borrow_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        // 'what borrower receives' = 'borrower debt' - 'fees'
        let borrower_to_receive =
            real_borrowed_amount.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;
        let d_tokens_to_issue = pool.compute_d_tokens_from_tokens_ceil(e, real_borrowed_amount)?;

        borrow_position.adjust_d_tokens(e, d_tokens_to_issue)?;
        borrow_position.adjust_borrowed(e, real_borrowed_amount)?;

        self.borrows.set(pool.pool_address.clone(), borrow_position);

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
        let mut deposit_position = self.deposits.get(pool.pool_address.clone()).unwrap_or_default();

        let computed_fees = compute_fees(
            original_amount,
            pool.config.fee_config.add_collateral_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;

        let added_collateral =
            original_amount.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;
        deposit_position.adjust_collateral(e, added_collateral)?;

        self.deposits.set(pool.pool_address.clone(), deposit_position);

        Ok(AddCollateralResult { added_collateral, computed_fees })
    }

    /// Withdraws assets from an obligation per pool
    pub fn withdraw(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
    ) -> Result<WithdrawResult, MCError> {
        let mut deposit_position =
            self.deposits.get(pool.pool_address.clone()).ok_or(MCError::ObligationDoesNotExist)?;

        let all_deposit = pool.compute_tokens_from_j_tokens_floor(e, deposit_position.j_tokens)?;
        let deposit_decrease = if self.borrow_exists() {
            let max_healthy_withdrawn_amount =
                self.compute_max_healthy_collateral_removed_amount(e, pool)?;
            i128::min(i128::min(original_amount, max_healthy_withdrawn_amount), all_deposit)
        } else {
            i128::min(all_deposit, original_amount)
        };
        // TODO: Add failing test here regarding insurance fund
        // if deposit_decrease > pool.total_available {
        if deposit_decrease > pool.total_available()? {
            return Err(MCError::NotEnoughPoolFunds);
        }

        let withdraw_scarcity_fee =
            compute_withdraw_scarcity_fee_bps(e, pool, deposit_decrease, &mut deposit_position)?;
        let withdraw_fee_bps = pool
            .config
            .fee_config
            .withdraw_fee_bps
            .checked_add(withdraw_scarcity_fee)
            .map_over_or_underflow()?;
        let computed_fees =
            compute_fees(deposit_decrease, withdraw_fee_bps, pool.config.fee_config.host_fee_bps)?;
        let withdrawer_to_receive =
            deposit_decrease.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;

        let j_tokens_to_burn = if deposit_decrease == all_deposit {
            deposit_position.j_tokens
        } else {
            i128::min(
                deposit_position.j_tokens,
                pool.compute_j_tokens_from_tokens_ceil(e, deposit_decrease)?,
            )
        };

        let all_deposit_ceil =
            pool.compute_tokens_from_j_tokens_ceil(e, deposit_position.j_tokens)?;
        let received_interest =
            all_deposit_ceil.checked_sub(deposit_position.deposited).map_over_or_underflow()?;

        if received_interest < 0 {
            events::computed_interest_is_negative(
                e,
                &pool.pool_address,
                j_tokens_to_burn,
                deposit_decrease,
                received_interest,
                all_deposit,
            );

            return Err(MCError::InternalError);
        } else if deposit_decrease >= received_interest {
            if deposit_decrease == all_deposit {
                deposit_position.adjust_deposited(
                    e,
                    deposit_position.deposited.checked_neg().map_over_or_underflow()?,
                )?;
            } else {
                let deposited_diff = deposit_decrease - received_interest; // safe

                deposit_position
                    .adjust_deposited(e, deposited_diff.checked_neg().map_over_or_underflow()?)?;
            }
        }

        deposit_position
            .adjust_j_tokens(e, j_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;
        if deposit_position.is_empty() {
            self.deposits.remove(pool.pool_address.clone());
        } else {
            self.deposits.set(pool.pool_address.clone(), deposit_position);
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
        let mut deposit_position =
            self.deposits.get(pool.pool_address.clone()).ok_or(MCError::CollateralDoesNotExist)?;

        let collateral_decrease = if self.borrow_exists() {
            let max_possible_collateral_removed_amount =
                self.compute_max_healthy_collateral_removed_amount(e, pool)?;
            i128::min(
                i128::min(original_amount, max_possible_collateral_removed_amount),
                deposit_position.collateral,
            )
        } else {
            i128::min(original_amount, deposit_position.collateral)
        };

        let computed_fees = compute_fees(
            collateral_decrease,
            pool.config.fee_config.remove_collateral_fee_bps,
            pool.config.fee_config.host_fee_bps,
        )?;
        let collateral_remover_to_receive =
            collateral_decrease.checked_sub(computed_fees.fee_sum).map_over_or_underflow()?;

        deposit_position
            .adjust_collateral(e, collateral_decrease.checked_neg().map_over_or_underflow()?)?;
        if deposit_position.is_empty() {
            self.deposits.remove(pool.pool_address.clone());
        } else {
            self.deposits.set(pool.pool_address.clone(), deposit_position);
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
        let mut borrow_position =
            self.borrows.get(pool.pool_address.clone()).ok_or(MCError::ObligationDoesNotExist)?;

        let all_debt = pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;
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

        let d_tokens_to_burn = if amount_to_take_from_borrower == amount_to_repay_all_debt {
            borrow_position.d_tokens
        } else {
            pool.compute_d_tokens_from_tokens_floor(e, debt_decrease)?
        };

        let unpaid_interest =
            all_debt.checked_sub(borrow_position.borrowed).map_over_or_underflow()?;
        if unpaid_interest < 0 {
            events::computed_interest_is_negative(
                e,
                &pool.pool_address,
                d_tokens_to_burn,
                debt_decrease,
                unpaid_interest,
                all_debt,
            );

            return Err(MCError::InternalError);
        } else if debt_decrease >= unpaid_interest {
            let borrowed_diff = debt_decrease - unpaid_interest; // safe
            borrow_position
                .adjust_borrowed(e, borrowed_diff.checked_neg().map_over_or_underflow()?)?;
        }

        borrow_position
            .adjust_d_tokens(e, d_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;

        if borrow_position.is_empty() {
            self.borrows.remove(pool.pool_address.clone());
        } else {
            self.borrows.set(pool.pool_address.clone(), borrow_position);
        }

        let amount_to_send_back = if amount_to_take_from_borrower < original_amount {
            original_amount - amount_to_take_from_borrower // safe
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
        borrow_pool: &Pool,
        collateral_pool: &Pool,
        amount: i128,
        min_collateral_amount: i128,
    ) -> Result<LiquidationResult2, MCError> {
        let (mut deposit_position, mut borrow_position) = (
            self.deposits
                .get(collateral_pool.pool_address.clone())
                .ok_or(MCError::PairIsNotLiquidatable)?,
            self.borrows
                .get(borrow_pool.pool_address.clone())
                .ok_or(MCError::PairIsNotLiquidatable)?,
        );

        let insolvency_ltv_bps = storage::get_insolvency_ltv_bps(e);
        let min_collateral_value = storage::get_min_collateral_value(e);

        let obligation_debt_value = self.compute_debt_value(e)?;
        let obligation_collateral_value = self.compute_collateral_value(e)?;
        let obligation_debt_value_w_liability_factors =
            self.compute_debt_value_scaled_w_liability_factors(e)?;
        let obligation_collateral_value_w_close_ltvs =
            self.compute_collateral_value_scaled_w_close_ltvs(e)?;

        if obligation_debt_value_w_liability_factors <= obligation_collateral_value_w_close_ltvs {
            return Err(MCError::LiquidatedObligationIsHealthy);
        }

        let unparameterized_ltv_bps = obligation_debt_value
            .fixed_div_ceil(obligation_collateral_value, BPS_FACTOR)
            .map_over_or_underflow()?;
        let is_solvent = unparameterized_ltv_bps < insolvency_ltv_bps;

        let liquidation_incentive_bps = borrow_pool.config.health_config.liquidation_incentive_bps;
        let liquidation_close_factor_bps =
            borrow_pool.config.health_config.liquidation_close_factor_bps;

        let borrowed_asset_price = get_asset_price(e, &borrow_pool.token_address)?;
        let collateral_asset_price = get_asset_price(e, &collateral_pool.token_address)?;

        let position_debt =
            borrow_pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;
        let position_collateral_tokens_from_j_tokens =
            collateral_pool.compute_tokens_from_j_tokens_floor(e, deposit_position.j_tokens)?;
        let position_collateral_sum = deposit_position
            .collateral
            .checked_add(position_collateral_tokens_from_j_tokens)
            .map_over_or_underflow()?;

        let (mut collateral_to_sell_to_liquidator, liquidated_amount) = if is_solvent {
            let liquidated_amount = amount;

            // 1. Check if liquidation doesn't exceed the close factor
            let liquidated_borrow_bps = liquidated_amount
                .fixed_div_ceil(position_debt, BPS_FACTOR)
                .map_over_or_underflow()?;
            if liquidated_borrow_bps > liquidation_close_factor_bps {
                return Err(MCError::LiquidationExceedsCloseFactor);
            }

            // 2. Count the maximum amount of sold collateral that improves LTV
            // ----
            // 'received_collateral_amount' < ('obligation_collateral_value' * 'borrowed_asset_repaid_amount' * 'borrowed_asset_price') /
            //                          / ('obligation_debt_value' * 'collateral_asset_price'),
            // must hold for the position to become healthier
            // ----
            let liquidated_value =
                liquidated_amount.checked_mul(borrowed_asset_price).map_over_or_underflow()?;

            let numerator = obligation_collateral_value
                .checked_mul(liquidated_value)
                .map_over_or_underflow()?;
            let denominator = obligation_debt_value
                .checked_mul(collateral_asset_price)
                .map_over_or_underflow()?;
            let max_ltv_improving_bonus =
                numerator.checked_div(denominator).map_over_or_underflow()?;

            let collateral_value_to_redeem_with_max_incentive = liquidated_value
                .fixed_mul_floor(
                    BPS_FACTOR.checked_add(liquidation_incentive_bps).map_over_or_underflow()?,
                    BPS_FACTOR,
                )
                .map_over_or_underflow()?;
            let redeemed_collateral_amount_with_max_incentive =
                collateral_value_to_redeem_with_max_incentive
                    .checked_div(collateral_asset_price)
                    .map_over_or_underflow()?;

            // 4. Find the amount of collateral to give away that obey all LTV improving constraints
            let collateral_to_sell_to_liquidator = position_collateral_sum
                .min(max_ltv_improving_bonus)
                .min(redeemed_collateral_amount_with_max_incentive);

            (collateral_to_sell_to_liquidator, liquidated_amount)
        } else {
            let liquidated_amount = amount.min(position_debt);
            let liquidated_value =
                liquidated_amount.checked_mul(borrowed_asset_price).map_over_or_underflow()?;

            let collateral_value_to_redeem_with_max_incentive = liquidated_value
                .fixed_mul_floor(
                    BPS_FACTOR.checked_add(liquidation_incentive_bps).map_over_or_underflow()?,
                    BPS_FACTOR,
                )
                .map_over_or_underflow()?;
            let redeemed_collateral_amount_with_max_incentive =
                collateral_value_to_redeem_with_max_incentive
                    .checked_div(collateral_asset_price)
                    .map_over_or_underflow()?;

            let collateral_to_sell_to_liquidator =
                position_collateral_sum.min(redeemed_collateral_amount_with_max_incentive);

            (collateral_to_sell_to_liquidator, liquidated_amount)
        };

        let mut is_collateral_drained = false;
        if position_collateral_sum > collateral_to_sell_to_liquidator {
            let collateral_left = position_collateral_sum - collateral_to_sell_to_liquidator; // safe 
            let collateral_value_left =
                collateral_left.checked_mul(collateral_asset_price).map_over_or_underflow()?;

            if collateral_value_left < min_collateral_value {
                // If collateral that's left is worth less than the configured `min_collateral_value` in the market,
                // the liquidator additionally receives all of the collateral that's left
                collateral_to_sell_to_liquidator += collateral_left;
                is_collateral_drained = true;
            }
        }

        // Verify that collateral amount is sufficient for a liquidator
        if collateral_to_sell_to_liquidator < min_collateral_amount {
            return Err(MCError::LiquidationMinCollateralTooBig);
        }

        // Distribute liquidator incentive between plain collateral and received `jTokens`
        let (collateral_seized, j_tokens_seized) = if collateral_to_sell_to_liquidator
            > deposit_position.collateral
        {
            let left_as_j_tokens = collateral_to_sell_to_liquidator - deposit_position.collateral; // safe
            let j_tokens = collateral_pool
                .compute_j_tokens_from_tokens_floor(e, left_as_j_tokens)?
                .min(deposit_position.j_tokens);

            (deposit_position.collateral, j_tokens)
        } else {
            (collateral_to_sell_to_liquidator, 0)
        };

        // -- Adjust obligation positions --

        deposit_position
            .adjust_collateral(&e, collateral_seized.checked_neg().map_over_or_underflow()?)?;
        deposit_position
            .adjust_j_tokens(e, j_tokens_seized.checked_neg().map_over_or_underflow()?)?;

        // TODO: Update received interest
        // borrow_position.adjust_d_tokens(e, adjusting_amount)

        todo!()
    }

    /// Liquidates unhealthy borrow
    pub fn liquidate2(
        &mut self,
        e: &Env,
        borrow_pool_address: &Address,
        collateral_pool_address: &Address,
        borrow_pool: &Pool,
        collateral_pool: &Pool,
        amount: i128,
    ) -> Result<LiquidationResult, MCError> {
        let (mut collateral_obligation, mut borrow_position) = (
            self.deposits
                .get(collateral_pool_address.clone())
                .ok_or(MCError::DepositDoesNotExist)?,
            self.borrows.get(borrow_pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?,
        );

        let PoolHealthConfig { liquidation_close_factor_bps, liquidation_incentive_bps, .. } =
            borrow_pool.config.health_config;

        let borrow_position_d_tokens = borrow_position.d_tokens;
        let borrow_position_d_tokens_as_tokens =
            borrow_pool.compute_tokens_from_d_tokens_floor(e, borrow_position_d_tokens)?;

        let collateral_obligation_j_tokens = collateral_obligation.j_tokens;
        let collateral_obligation_j_tokens_as_tokens = collateral_pool
            .compute_tokens_from_j_tokens_floor(e, collateral_obligation_j_tokens)?;

        // 'liquidatable_bps' == ((amount * 10_000) / total_debt)
        let liquidatable_bps = amount
            .fixed_div_floor(borrow_position_d_tokens_as_tokens, BPS_FACTOR)
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
            let d_tokens_repaid = borrow_pool.compute_d_tokens_from_tokens_floor(e, amount)?;

            LiquidationResult {
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
                collateral_pool.compute_tokens_from_j_tokens_floor(e, full_collateral_j_tokens)?;
            let available_tokens_from_j_tokens =
                i128::min(collateral_pool.total_available, j_tokens_as_tokens);

            let tokens_from_j_tokens_value = available_tokens_from_j_tokens
                .checked_mul(collateral_price)
                .map_over_or_underflow()?;

            if tokens_from_j_tokens_value >= value_left {
                let tokens_from_sold_j_tokens =
                    value_left.checked_div(collateral_price).map_over_or_underflow()?;
                let j_tokens_amount_sold = collateral_pool
                    .compute_j_tokens_from_tokens_floor(e, tokens_from_sold_j_tokens)?;
                let d_tokens_repaid = borrow_pool.compute_d_tokens_from_tokens_floor(e, amount)?;

                LiquidationResult {
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
                    .compute_d_tokens_from_tokens_floor(e, tokens_per_collateral_minus_incentive)?;

                LiquidationResult {
                    liquidated_amount: tokens_per_collateral_minus_incentive,
                    d_tokens_repaid,
                    collateral_amount_sold: full_collateral_amount,
                    j_tokens_amount_sold: full_collateral_j_tokens,
                    tokens_from_sold_j_tokens: available_tokens_from_j_tokens,
                }
            }
        };

        let unpaid_interest = borrow_position_d_tokens_as_tokens - borrow_position.borrowed;
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

        borrow_position.adjust_borrowed(e, borrowed_diff.checked_neg().map_over_or_underflow()?)?;
        borrow_position.adjust_d_tokens(
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

        self.borrows.set(borrow_pool_address.clone(), borrow_position);
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
        for (pool_address, borrow_position) in self.borrows.iter() {
            borrows_to_be_compensated.push_back((pool_address, borrow_position.d_tokens));
        }

        let mut collaterals_to_remove: Vec<(Address, i128, i128)> = Vec::new(e);
        for (pool_address, deposit_position) in self.deposits.iter() {
            collaterals_to_remove.push_back((
                pool_address,
                deposit_position.j_tokens,
                deposit_position.collateral,
            ));
        }

        Ok(CoverBadDebtResult { borrows_to_be_compensated, collaterals_to_remove })
    }

    // ------ Getters ------

    /// Returns the amount of `jTokens` that the obligation has in the specified pool
    pub fn get_j_tokens(&self, pool_address: &Address) -> Result<i128, MCError> {
        let deposit_position =
            self.deposits.get(pool_address.clone()).ok_or(MCError::DepositDoesNotExist)?;

        Ok(deposit_position.j_tokens)
    }

    /// Calculates the interest that the obligation received from the deposit pool
    pub fn get_received_interest(&self, e: &Env, pool_address: &Address) -> Result<i128, MCError> {
        let deposit_position =
            self.deposits.get(pool_address.clone()).ok_or(MCError::DepositDoesNotExist)?;
        let deposit_pool = Pool::try_get(e, pool_address)?;

        let total_supply =
            deposit_pool.compute_j_tokens_from_tokens_ceil(e, deposit_position.j_tokens)?;
        let deposited = deposit_position.deposited;

        if total_supply < deposited {
            events::pool_contains_inconsistent_state(e, &deposit_pool);

            return Err(MCError::InternalError);
        }
        let unpaid_interest = total_supply - deposited; // safe

        Ok(unpaid_interest)
    }

    /// Calculates the interest that the obligation owes to the borrow pool
    pub fn get_unpaid_interest(&self, e: &Env, pool_address: &Address) -> Result<i128, MCError> {
        let borrow_position =
            self.borrows.get(pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?;
        let borrow_pool = Pool::try_get(e, pool_address)?;

        let total_debt =
            borrow_pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;
        let borrowed = borrow_position.borrowed;

        if total_debt < borrowed {
            events::pool_contains_inconsistent_state(e, &borrow_pool);

            return Err(MCError::InternalError);
        }
        let unpaid_interest = total_debt - borrowed; // safe

        Ok(unpaid_interest)
    }

    /// Returns the amount of `dTokens` that the obligation has in the specified pool
    pub fn get_borrowed(&self, pool_address: &Address) -> Result<i128, MCError> {
        let Some(borrow_position) = self.borrows.get(pool_address.clone()) else {
            return Err(MCError::BorrowDoesNotExist);
        };

        Ok(borrow_position.borrowed)
    }

    /// Returns the total debt (including interest) that the obligation has in the specified pool
    /// (in tokens, not in dTokens)
    pub fn get_total_debt(&self, e: &Env, pool_address: &Address) -> Result<i128, MCError> {
        let borrow_position =
            self.borrows.get(pool_address.clone()).ok_or(MCError::BorrowDoesNotExist)?;
        let borrow_pool = Pool::try_get(e, pool_address)?;

        let total_debt =
            borrow_pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;

        Ok(total_debt)
    }

    /// Returns the amount of collateral that the obligation has in the specified pool
    pub fn get_collateral(&self, pool_address: &Address) -> Result<i128, MCError> {
        let Some(deposit_position) = self.deposits.get(pool_address.clone()) else {
            return Err(MCError::DepositDoesNotExist);
        };

        Ok(deposit_position.collateral)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[contracttype]
pub struct BorrowPosition {
    /// Amount of the total debt shares that the obligation contains
    pub d_tokens: i128,
    /// Accumulated value of initially borrowed tokens
    pub borrowed: i128,
}

impl BorrowPosition {
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[contracttype]
pub struct DepositPosition {
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

impl DepositPosition {
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

/// Used to generate a unique seed for `Earn` obligation
/// See [`compute_earn_obligation_seed`]
const EARN_OBLIGATION_SEED_STR: &str = "EV";

/// Computes 'Earn' seed and caches it if it hasn't been computed yet, or gets it from the storage otherwise
///
/// # Returns
/// [`BytesN<32>`] bytes used as an obligation seed to distinguish unique users' obligations
pub fn get_earn_obligation_seed(e: &Env) -> BytesN<32> {
    if let Some(stored_seed) = storage::get_earn_obligation_seed(e) {
        // TODO: Add tests that verify that caching actually takes place
        stored_seed
    } else {
        let computed_seed = compute_earn_obligation_seed(e);
        storage::set_earn_obligation_seed(e, &computed_seed);

        computed_seed
    }
}

fn compute_earn_obligation_seed(e: &Env) -> BytesN<32> {
    let mut seed = Bytes::new(e);
    seed.extend_from_slice(EARN_OBLIGATION_SEED_STR.as_bytes());
    e.crypto().keccak256(&seed).into()
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

/// Computes additional withdraw scarcity fee(in basis) points that is charged when pool's utilization ratio
/// exceeds utilization ratio limit. E.g., (2% of origination fee + 4.5% of scarcity fee, etc.).
///
/// # WARNING
/// This updates `deposit_position.last_scarcity_withdraw_ts` on `DepositPosition` in case of scarcity withdraw
fn compute_withdraw_scarcity_fee_bps(
    e: &Env,
    pool: &Pool,
    deposit_decrease: i128,
    deposit_position: &mut DepositPosition,
) -> Result<u32, MCError> {
    let current_utilization_ratio_bps = pool.compute_utilization_ratio_bps()?;
    let new_utilization_ratio_bps = {
        let new_total_supply = pool.total_supply()? - deposit_decrease; // safe

        if new_total_supply == 0 {
            BPS_FACTOR
        } else {
            pool.total_borrowed
                .fixed_div_ceil(new_total_supply, BPS_FACTOR)
                .map_over_or_underflow()?
        }
    };

    let utilization_ratio_diff_bps = if new_utilization_ratio_bps
        > pool.config.health_config.utilization_ratio_limit_bps
    {
        // If withdraw leads to a scarcity state - update the last scarcity withdraw timestamp
        // per deposit obligation
        let deposit_decrease_to_total_supply_bps = deposit_decrease
            .fixed_div_ceil(pool.total_supply()?, BPS_FACTOR)
            .map_over_or_underflow()?;

        let withdraw_scarcity_limit_bps = if current_utilization_ratio_bps
            < pool.config.health_config.utilization_ratio_limit_bps
        {
            let remaining_utilization_ratio = pool.config.health_config.utilization_ratio_limit_bps
                - current_utilization_ratio_bps; // safe

            remaining_utilization_ratio
                .checked_add(pool.config.health_config.withdraw_scarcity_limit_bps)
                .map_over_or_underflow()?
        } else {
            pool.config.health_config.withdraw_scarcity_limit_bps
        };
        if deposit_decrease_to_total_supply_bps > withdraw_scarcity_limit_bps {
            return Err(MCError::WithdrawScarcityOverLimit);
        }

        let last_scarcity_withdraw_ts = deposit_position.last_scarcity_withdraw_ts;
        let scarcity_withdraw_cooldown = pool.config.health_config.withdraw_scarcity_cooldown_s;
        let current_timestamp = e.ledger().timestamp();

        if current_timestamp
            < last_scarcity_withdraw_ts
                .checked_add(scarcity_withdraw_cooldown)
                .map_over_or_underflow()?
        {
            return Err(MCError::ScarcityCooldownPeriod);
        }
        deposit_position.last_scarcity_withdraw_ts = current_timestamp;

        new_utilization_ratio_bps - pool.config.health_config.utilization_ratio_limit_bps // safe
    } else {
        0
    };
    let fee = utilization_ratio_diff_bps
        .fixed_mul_ceil(pool.config.fee_config.withdraw_scarcity_fee_sc_bps as i128, BPS_FACTOR)
        .map_over_or_underflow()? as u32;

    Ok(fee)
}

#[contracttype]
#[derive(Clone)]
/// Generally represents computed fees issued by any possible operation on a market
pub struct ComputedFees {
    /// Sum of `market_fee` and `host_fee`
    pub fee_sum: i128,
    /// Fee segregated to the market admin
    pub market_fee: i128,
    /// Fee segregated to the protocol host(market deployer)
    pub host_fee: i128,
}

#[contracttype]
/// [`Obligation::deposit`] resulting data
pub struct DepositResult {
    /// Amount of `jTokens` to issue that represent the `deposited` amount in the pool
    pub j_tokens_to_issue: i128,
    /// Amount of originally deposited tokens(minus all possible fees)
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
    /// Amount of tokens to receive by the borrower(`borrower_new_debt` minus all fees)
    pub borrower_to_receive: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
/// [`Obligation::add_collateral`] resulting data
pub struct AddCollateralResult {
    /// Amount of tokens added as collateral(minus all possible fees)
    pub added_collateral: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
/// [`Obligation::withdraw`] resulting data
pub struct WithdrawResult {
    /// Amount of `jTokens` to burn that represent the `deposit_decreased_amount` amount in the
    /// pool
    pub j_tokens_to_burn: i128,
    /// Amount of the original deposit(in tokens) that is removed from the `DepositPosition`
    pub deposit_decrease: i128,
    /// Amount of tokens to receive by the withdrawer(`deposit_decreased_amount` minus fees)
    pub withdrawer_to_receive: i128,
    pub computed_fees: ComputedFees,
}

#[contracttype]
#[derive(Clone)]
/// [`Obligation::repay`] resulting data
pub struct RepayResult {
    /// Amount of `dTokens` to burn that represent the `real_repaid` amount in the pool
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

pub struct LiquidationResult2 {
    /// The amount of dTokens that are burned from the borrower's borrow position
    pub d_tokens_repaid: i128,
    /// The amount of jTokens seized from the borrower's position and given away to the liquidator
    /// in case the borrower's position doesn't contain enough plain collateral to cover the liquidation expenses
    pub received_plain_collateral: i128,
    /// The amount of jTokens seized from the borrower's position and given away the liquidator
    /// in case if borrower's position doesn't contains enough plain collateral to cover the liquidation expenses
    pub received_j_tokens: i128,
    /// The amount of tokens representing the `received_j_tokens`
    pub tokens_from_received_j_tokens: i128,
}

// TODO: Move this somewhere else when working on liquidation
pub struct LiquidationResult {
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
#[cfg(test)]
mod tests {
    use soroban_sdk::{BytesN, Env};

    use super::*;

    #[test]
    fn test_computes_earn_obligation_seed_with_valid_address() {
        let e = Env::default();

        let seed = compute_earn_obligation_seed(&e);

        assert_ne!(seed, BytesN::from_array(&e, &[0; 32]));
    }

    #[test]
    fn test_computes_different_seeds_for_different_addresses() {
        let e = Env::default();

        let seed_1 = compute_earn_obligation_seed(&e);
        let seed_2 = compute_earn_obligation_seed(&e);

        assert_eq!(seed_1, seed_2);
    }
}
