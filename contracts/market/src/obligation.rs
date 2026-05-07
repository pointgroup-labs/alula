use farms_interface::Delegatee;
use soroban_fixed_point_math::{FixedPoint, SorobanFixedPoint};
use soroban_sdk::{Address, Bytes, BytesN, Env, I256, Map, contracttype};

use crate::{
    constants::*,
    error::MCError,
    events,
    math_utils::MathUtils,
    oracle,
    pool::{Pool, PoolFeeConfig},
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

    pub fn require_auth(&self) {
        self.user.require_auth();
    }
}

impl From<&ObligationKey> for Delegatee {
    fn from(key: &ObligationKey) -> Self {
        match &key.seed {
            Some(seed) => (key.user.clone(), seed.clone()).into(),
            None => key.user.clone().into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct Obligation {
    // Deposited collateral for the obligation, unique by deposit pool address
    pub deposits: Map<Address, DepositPosition>,
    // Borrowed liquidity for the obligation, unique by borrow pool address
    pub borrows: Map<Address, BorrowPosition>,
    // Count of non-empty positions
    pub positions_count: u32,
    // Active Insurance Fund requests for this obligation. Key: (pool, request_id).
    // Value: per-request expiry deadline (unix seconds) at which a still-Pending
    // request becomes permissionlessly cancelable. Stored per-request so that
    // newly recorded requests cannot push out the cancellation eligibility of
    // older ones (the pool-level deadline is only used as a withdraw-gate).
    pub insurance_fund_requests_ids: Map<(Address, u64), u64>,
    // // Market value of obligation's collateral
    // pub collateral_value: i128,
    // // Last update to collateral, liquidity, or their market values
    // pub last_update: u64,
    // // Market value of deposits
    // pub borrowed_value: i128,
}

impl Obligation {
    // Creates a new obligation for the specified obligation key
    //
    // # WARNING
    // Modifies the obligation's pools storage data by appending the user's address to the
    // obligation's list
    pub fn new(e: &Env, obligation_key: &ObligationKey) -> Self {
        storage::register_obligation(e, obligation_key);

        Self {
            deposits: Map::new(e),
            borrows: Map::new(e),
            positions_count: 0,
            insurance_fund_requests_ids: Map::new(e),
        }
    }

    // Verifies that the new deposit position doesn't exceed the max allowed number of positions and
    // creates one
    pub fn try_create_deposit_position(&mut self, e: &Env) -> Result<DepositPosition, MCError> {
        if self.positions_count >= storage::get_max_positions(e) {
            return Err(MCError::TooManyPositions);
        }
        self.positions_count += 1;

        Ok(DepositPosition::default())
    }

    // Verifies that the new borrow position doesn't exceed the max allowed number of positions and
    // creates one
    pub fn try_create_borrow_position(&mut self, e: &Env) -> Result<BorrowPosition, MCError> {
        if self.positions_count >= storage::get_max_positions(e) {
            return Err(MCError::TooManyPositions);
        }
        self.positions_count += 1;

        Ok(BorrowPosition::default())
    }

    // Removes the deposit position for the obligation
    pub fn try_remove_deposit_position(
        &mut self,
        e: &Env,
        pool_address: &Address,
    ) -> Result<(), MCError> {
        self.deposits.remove(pool_address.clone());
        if self.positions_count == 0 {
            events::positions_count_becomes_negative(e, pool_address, self);

            return Err(MCError::InternalError);
        }
        self.positions_count -= 1;

        Ok(())
    }

    // Removes the borrow position for the obligation
    pub fn try_remove_borrow_position(
        &mut self,
        e: &Env,
        pool_address: &Address,
    ) -> Result<(), MCError> {
        self.borrows.remove(pool_address.clone());
        if self.positions_count == 0 {
            events::positions_count_becomes_negative(e, pool_address, self);

            return Err(MCError::InternalError);
        }
        self.positions_count -= 1;

        Ok(())
    }

    // Saves/updates obligation in the contract's storage
    //
    // # Arguments
    // * `user` - obligation's user address. **MUST** equal the original user address
    // * `seed` - obligation's user seed. **MUST** equal the original obligation seed
    //
    // # WARNING
    // Modifies the contract's storage
    pub fn set(&self, e: &Env, obligation_key: &ObligationKey) {
        storage::set_obligation(e, obligation_key, self);
    }

    // Tries to get the user's obligation from the contract's storage
    //
    // # Returns
    // - [`Ok(Obligation)`] if a pool with the given address exists in the contract's storage
    // - [`Err(MCError::ObligationDoesNotExist)`] otherwise
    pub fn try_get(e: &Env, obligation_key: &ObligationKey) -> Result<Self, MCError> {
        storage::get_obligation(e, obligation_key).ok_or(MCError::ObligationDoesNotExist)
    }

    // # Returns
    //
    // [`Map<ObligationKey, ()>`] containing all obligation keys in the market
    pub fn get_all(e: &Env) -> Map<ObligationKey, ()> {
        storage::get_all_obligations(e)
    }

    // Removes obligation from the contract's storage
    //
    // # WARNING
    // Modifies the contract's storage
    pub fn remove(self, e: &Env, obligation_key: &ObligationKey) {
        storage::remove_obligation(e, obligation_key);
    }

    // Accrues interest on all obligation-related pools
    //
    // # WARNING
    // Modifies the obligation's pools storage data.
    // Also, accruing interest on an obligation should precede pool retrieval
    pub fn accrue_interest(&self, e: &Env) -> Result<(), MCError> {
        for borrow_pool_address in self.borrows.keys() {
            accrue_interest_on_pool(e, &borrow_pool_address)?;
        }

        for deposit_pool_address in self.deposits.keys() {
            accrue_interest_on_pool(e, &deposit_pool_address)?;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.deposits.is_empty()
            && self.borrows.is_empty()
            && self.insurance_fund_requests_ids.is_empty()
    }

    pub fn borrow_exists(&self) -> bool {
        !self.borrows.is_empty()
    }

    // ------ `require_` circuits ------

    pub fn require_borrow_exists(&self) -> Result<(), MCError> {
        if !self.borrow_exists() {
            return Err(MCError::BadDebtCoverageCriterionIsNotMet);
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
        if self.deposits.contains_key(pool_address.clone()) {
            return Err(MCError::DepositPositionForAssetExists);
        }

        Ok(())
    }

    pub fn require_no_active_cover_bad_debt_requests_exists(&self) -> Result<(), MCError> {
        if !self.insurance_fund_requests_ids.is_empty() {
            return Err(MCError::ObligationContainsOpenCoverBadDebtRequests);
        }

        Ok(())
    }

    // ------ Health Factor Removing Computations ------

    // Computes the current collateral assets summed value(deposit shares + plain collateral) per
    // obligation with floor rounding
    pub fn compute_collateral_value(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0i128;
        for (pool_address, deposit_position) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

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

    // Computes the max healthy amount of the token that can be borrowed and that
    // doesn't exceed the `open LTV` parameter on the pool
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

    // Computes the max healthy amount of the collateral token(that is used as a deposit or as a
    // collateral) that can be removed so that the obligation's LTV is equal to the `open LTV`
    // parameter on the pool
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

    // Computes the maximum healthy amount of the token that can be additionally added to the debt
    // or removed from the collateral, scaled with the corresponding coefficient(`open_ltv_bps` for removed collateral
    // or `liability_factor_bps` for added borrow)
    fn compute_max_health_factor_decreasing_amount(
        &self,
        e: &Env,
        pool: &Pool,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let (collateral_value_scaled, amount_of_borrow_backing_collateral_positions) =
            self.compute_collateral_value_scaled_w_open_ltvs(e)?;
        let debt_value_scaled = self.compute_debt_value_scaled_w_liability_factors(e)?;

        let max_amount = if collateral_value_scaled <= debt_value_scaled {
            // Since the scaled borrowed assets value exceeds the scaled collateral assets value,
            // any health factor decreasing operation is prohibited
            0
        } else {
            let asset_price: i128 = oracle::get_asset_price(e, &pool.token_address)?;

            let value_left = collateral_value_scaled - debt_value_scaled; // safe
            let min_collateral_value_threshold = compute_min_collateral_threshold_scaled(e)?;
            let min_collateral_value_requirement = min_collateral_value_threshold
                .checked_mul(amount_of_borrow_backing_collateral_positions as i128)
                .map_over_or_underflow()?;
            let borrow_backing_value_left =
                i128::max(value_left.saturating_sub(min_collateral_value_requirement), 0);

            // ----
            // borrow_backing_value_left = (amount * asset_price / 10^decimals) * (scalar_bps / BPS_FACTOR)
            // ----
            // implies:
            // ----
            // amount = (borrow_backing_value_left * 10^decimals * BPS_FACTOR) / (asset_price * scalar_bps)
            // ----

            let target_value = I256::from_i128(e, borrow_backing_value_left);
            let decimals_multiplier = I256::from_i128(e, 10_i128.pow(pool.token_decimals));
            let bps_factor = I256::from_i128(e, BPS_FACTOR);
            let numerator = target_value.mul(&decimals_multiplier).mul(&bps_factor);

            let asset_price = I256::from_i128(e, asset_price);
            let scalar_bps = I256::from_i128(e, scalar_bps);
            let denominator = asset_price.mul(&scalar_bps);

            numerator.div(&denominator).to_i128().map_over_or_underflow()?
        };

        Ok(max_amount)
    }

    // Computes the current debt assets summed value per
    // obligation, scaling each value with the appropriate `liability_factor_bps` value
    fn compute_debt_value_scaled_w_liability_factors(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, borrow_position) in self.borrows.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

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

    // Computes the current debt assets summed value per
    // obligation
    pub fn compute_debt_value(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, borrow_position) in self.borrows.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let new_value_term =
                Self::compute_pool_debt_value_scaled(e, &pool, &borrow_position, BPS_FACTOR)?;

            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok(value_sum)
    }

    // Computes the current collateral assets summed value(deposit shares + plain collateral) per
    // obligation, scaling each value with the appropriate `close_ltv_bps` value
    fn compute_collateral_value_scaled_w_close_ltvs(&self, e: &Env) -> Result<i128, MCError> {
        let mut value_sum = 0_i128;

        for (pool_address, deposit_position) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

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

    // Computes the current collateral assets summed value(deposit shares + plain collateral) per
    // obligation, scaling each value with the appropriate `open_ltv_bps` value.
    // As a second value, it returns the amount of open positions that are used as collateral that can back up borrows
    fn compute_collateral_value_scaled_w_open_ltvs(&self, e: &Env) -> Result<(i128, u32), MCError> {
        let mut value_sum = 0i128;
        let mut positions_with_non_zero_close_ltv_count = 0u32;

        for (pool_address, deposit_position) in self.deposits.iter() {
            let pool = Pool::try_get(e, &pool_address).map_err(|_| {
                events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            if pool.config.health_config.close_ltv_bps.is_positive() {
                positions_with_non_zero_close_ltv_count += 1;
            }
            let new_value_term = Self::compute_pool_collateral_value_scaled(
                e,
                &pool,
                &deposit_position,
                pool.config.health_config.open_ltv_bps,
            )?;
            value_sum = value_sum.checked_add(new_value_term).map_over_or_underflow()?;
        }

        Ok((value_sum, positions_with_non_zero_close_ltv_count))
    }

    // Computes obligation's collateral pool's asset total(collateral + deposit) value scaled
    // with provided `scalar_bps` value
    fn compute_pool_collateral_value_scaled(
        e: &Env,
        pool: &Pool,
        deposit_position: &DepositPosition,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let &DepositPosition { j_tokens, collateral, .. } = deposit_position;

        let supply = pool.compute_tokens_from_j_tokens_floor(e, j_tokens)?;
        let total_collateral_tokens = supply.checked_add(collateral).map_over_or_underflow()?;

        Self::compute_asset_value_scaled_floor(e, total_collateral_tokens, pool, scalar_bps)
    }

    // Computes obligation's debt pool's asset value scaled
    // with provided `scalar_bps` value
    fn compute_pool_debt_value_scaled(
        e: &Env,
        pool: &Pool,
        borrow_position: &BorrowPosition,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let &BorrowPosition { d_tokens, .. } = borrow_position;
        let debt = pool.compute_tokens_from_d_tokens_ceil(e, d_tokens)?;

        Self::compute_asset_value_scaled_ceil(e, debt, pool, scalar_bps)
    }

    fn compute_asset_value_scaled_floor(
        e: &Env,
        amount: i128,
        pool: &Pool,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let price = I256::from_i128(e, oracle::get_asset_price(e, &pool.token_address)?);
        let amount = I256::from_i128(e, amount);

        let numerator = amount.mul(&price);
        let denominator = I256::from_i128(e, 10_i128.pow(pool.token_decimals));

        let value = numerator.div(&denominator).to_i128().map_over_or_underflow()?;
        let value_scaled = value.fixed_mul_floor(scalar_bps, BPS_FACTOR).map_over_or_underflow()?;

        Ok(value_scaled)
    }

    fn compute_asset_value_scaled_ceil(
        e: &Env,
        amount: i128,
        pool: &Pool,
        scalar_bps: i128,
    ) -> Result<i128, MCError> {
        let price = I256::from_i128(e, oracle::get_asset_price(e, &pool.token_address)?);
        let amount = I256::from_i128(e, amount);

        let numerator = amount.mul(&price);
        let denominator = I256::from_i128(e, 10_i128.pow(pool.token_decimals));

        let value = numerator
            .fixed_div_ceil(e, &denominator, &I256::from_i128(e, 1))
            .to_i128()
            .map_over_or_underflow()?;
        let value_scaled = value.fixed_mul_ceil(scalar_bps, BPS_FACTOR).map_over_or_underflow()?;

        Ok(value_scaled)
    }

    // ------ Operations ------

    // Deposits assets on an obligation per pool
    pub fn deposit(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
        referrer: &Option<Address>,
    ) -> Result<DepositResult, MCError> {
        let mut deposit_position = self
            .deposits
            .get(pool.pool_address.clone())
            .map_or_else(|| self.try_create_deposit_position(e), Ok)?;

        let operation_fees = compute_operation_fees(
            original_amount,
            pool.config.fee_config.deposit_fee_bps,
            referrer,
            &pool.config.fee_config,
        )?;
        let deposited_tokens_minus_fee =
            original_amount.checked_sub(operation_fees.fee_sum).map_over_or_underflow()?;
        let (j_tokens_to_issue, new_originally_deposited) =
            pool.compute_j_tokens_from_tokens_floor(e, deposited_tokens_minus_fee)?;
        if j_tokens_to_issue <= 0 {
            return Err(MCError::NonPositiveSharesAmount);
        }

        deposit_position.adjust_originally_deposited(e, new_originally_deposited)?;
        deposit_position.adjust_j_tokens(e, j_tokens_to_issue)?;

        self.deposits.set(pool.pool_address.clone(), deposit_position);

        Ok(DepositResult {
            j_tokens_to_issue,
            deposited: deposited_tokens_minus_fee,
            operation_fees,
        })
    }

    // Borrows assets on an obligation
    pub fn borrow(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
        referrer: &Option<Address>,
    ) -> Result<BorrowResult, MCError> {
        let max_healthy_borrow_added_amount =
            self.compute_max_healthy_debt_added_amount(e, pool)?;
        if max_healthy_borrow_added_amount <= 0 {
            return Err(MCError::UnhealthyOperation);
        }

        // `[i128::MAX]` indicates capped borrow
        let real_borrowed_amount = if original_amount == i128::MAX {
            let max_prserving_ur_cap_borrow =
                pool.compute_available_utilization_ratio_capped_borrow(e)?;

            max_healthy_borrow_added_amount.min(max_prserving_ur_cap_borrow)
        } else {
            pool.require_borrow_preserves_ur_cap(e, original_amount)?;
            if original_amount > max_healthy_borrow_added_amount {
                return Err(MCError::UnhealthyOperation);
            }

            original_amount
        };

        let mut borrow_position = self
            .borrows
            .get(pool.pool_address.clone())
            .map_or_else(|| self.try_create_borrow_position(e), Ok)?;

        let operation_fees = compute_operation_fees(
            real_borrowed_amount,
            pool.config.fee_config.borrow_fee_bps,
            referrer,
            &pool.config.fee_config,
        )?;

        // 'what borrower receives' = 'borrower debt' - 'fees'
        let borrower_to_receive =
            real_borrowed_amount.checked_sub(operation_fees.fee_sum).map_over_or_underflow()?;
        let (d_tokens_to_issue, new_originally_borrowed) =
            pool.compute_d_tokens_from_tokens_ceil(e, real_borrowed_amount)?;
        if d_tokens_to_issue <= 0 {
            events::minting_non_positive_d_tokens_on_borrow(
                e,
                pool.pool_address.clone(),
                d_tokens_to_issue,
                real_borrowed_amount,
            );

            // 2 consecutive `i128::MAX` withdrawals will lead to this
            return Err(MCError::NonPositiveSharesAmount);
        }

        borrow_position.adjust_d_tokens(e, d_tokens_to_issue)?;
        borrow_position.adjust_originally_borrowed(e, new_originally_borrowed)?;

        self.borrows.set(pool.pool_address.clone(), borrow_position);

        Ok(BorrowResult {
            d_tokens_to_issue,
            borrower_to_receive,
            borrower_new_debt: real_borrowed_amount,
            operation_fees,
        })
    }

    // Adds collateral assets on an obligation per pool
    pub fn add_collateral(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
        referrer: &Option<Address>,
    ) -> Result<AddCollateralResult, MCError> {
        let mut deposit_position = self
            .deposits
            .get(pool.pool_address.clone())
            .map_or_else(|| self.try_create_deposit_position(e), Ok)?;

        let operation_fees = compute_operation_fees(
            original_amount,
            pool.config.fee_config.add_collateral_fee_bps,
            referrer,
            &pool.config.fee_config,
        )?;

        let added_collateral =
            original_amount.checked_sub(operation_fees.fee_sum).map_over_or_underflow()?;
        deposit_position.adjust_collateral(e, added_collateral)?;

        self.deposits.set(pool.pool_address.clone(), deposit_position);

        Ok(AddCollateralResult { added_collateral, operation_fees })
    }

    // Withdraws assets from an obligation per pool
    pub fn withdraw(
        &mut self,
        e: &Env,
        pool: &Pool,
        provided_amount: i128,
        referrer: &Option<Address>,
    ) -> Result<WithdrawResult, MCError> {
        let mut deposit_position = self
            .deposits
            .get(pool.pool_address.clone())
            .ok_or(MCError::DepositPositionDoesNotExist)?;

        let all_deposit = pool.compute_tokens_from_j_tokens_floor(e, deposit_position.j_tokens)?;
        let amount_capped_to_deposit = all_deposit.min(provided_amount);

        let deposit_decrease = if self.borrow_exists() {
            let max_healthy = self.compute_max_healthy_collateral_removed_amount(e, pool)?;

            if provided_amount == i128::MAX {
                max_healthy.min(all_deposit).min(pool.total_available()?)
            } else if amount_capped_to_deposit > max_healthy {
                return Err(MCError::UnhealthyOperation);
            } else {
                amount_capped_to_deposit
            }
        } else if provided_amount == i128::MAX {
            amount_capped_to_deposit.min(pool.total_available()?)
        } else {
            amount_capped_to_deposit
        };
        pool.require_total_available(deposit_decrease)?;

        let is_all_withdrawn = deposit_decrease == all_deposit;

        let withdraw_scarcity_fee_bps =
            compute_withdraw_scarcity_fee_bps(e, pool, deposit_decrease, &deposit_position)?;
        if withdraw_scarcity_fee_bps > 0 {
            deposit_position.last_scarcity_withdraw_ts = e.ledger().timestamp();
        }

        let withdraw_fee_bps = pool
            .config
            .fee_config
            .withdraw_fee_bps
            .checked_add(withdraw_scarcity_fee_bps)
            .map_over_or_underflow()?;
        let operation_fees = compute_operation_fees(
            deposit_decrease,
            withdraw_fee_bps,
            referrer,
            &pool.config.fee_config,
        )?;
        let withdrawer_to_receive =
            deposit_decrease.checked_sub(operation_fees.fee_sum).map_over_or_underflow()?;

        let j_tokens_to_burn = if is_all_withdrawn {
            deposit_position.j_tokens
        } else {
            pool.compute_j_tokens_from_tokens_ceil(deposit_decrease)?.min(deposit_position.j_tokens)
        };
        if j_tokens_to_burn <= 0 {
            events::burning_non_positive_j_tokens_on_withdraw(
                e,
                pool.pool_address.clone(),
                j_tokens_to_burn,
                deposit_decrease,
            );

            return Err(MCError::NonPositiveSharesAmount);
        }

        let all_deposit_ceil =
            pool.compute_tokens_from_j_tokens_ceil(e, deposit_position.j_tokens)?;
        let received_interest = all_deposit_ceil
            .checked_sub(deposit_position.originally_deposited)
            .map_over_or_underflow()?;
        if received_interest.is_negative() {
            events::computed_interest_is_negative(
                e,
                &pool.pool_address,
                deposit_position.j_tokens,
                all_deposit_ceil,
                received_interest,
            );
        }

        let new_originally_deposited = if is_all_withdrawn {
            0
        } else {
            let new_total_j_tokens = pool.total_j_tokens - j_tokens_to_burn; // safe
            let new_total_supply = pool.total_supply()? - deposit_decrease; // safe
            let new_j_tokens = deposit_position.j_tokens - j_tokens_to_burn; // safe

            Pool::compute_tokens_from_shares_floor(
                e,
                new_j_tokens,
                new_total_j_tokens,
                new_total_supply,
            )?
        };

        deposit_position
            .adjust_j_tokens(e, j_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;
        deposit_position.set_originally_deposited(e, new_originally_deposited)?;

        if deposit_position.is_empty() {
            self.try_remove_deposit_position(e, &pool.pool_address)?;
        } else {
            self.deposits.set(pool.pool_address.clone(), deposit_position);
        }

        Ok(WithdrawResult {
            j_tokens_to_burn,
            deposit_decrease,
            withdrawer_to_receive,
            operation_fees,
        })
    }

    // Removes collateral assets from an obligation per pool
    pub fn remove_collateral(
        &mut self,
        e: &Env,
        pool: &Pool,
        original_amount: i128,
        referrer: &Option<Address>,
    ) -> Result<RemoveCollateralResult, MCError> {
        let mut deposit_position = self
            .deposits
            .get(pool.pool_address.clone())
            .ok_or(MCError::DepositPositionDoesNotExist)?;

        let collateral_decrease = if self.borrow_exists() {
            let max_possible_collateral_removed_amount =
                self.compute_max_healthy_collateral_removed_amount(e, pool)?;

            original_amount
                .min(max_possible_collateral_removed_amount)
                .min(deposit_position.collateral)
        } else {
            original_amount.min(deposit_position.collateral)
        };

        let operation_fees = compute_operation_fees(
            collateral_decrease,
            pool.config.fee_config.remove_collateral_fee_bps,
            referrer,
            &pool.config.fee_config,
        )?;
        let collateral_remover_to_receive =
            collateral_decrease.checked_sub(operation_fees.fee_sum).map_over_or_underflow()?;

        deposit_position
            .adjust_collateral(e, collateral_decrease.checked_neg().map_over_or_underflow()?)?;

        if deposit_position.is_empty() {
            self.try_remove_deposit_position(e, &pool.pool_address)?;
        } else {
            self.deposits.set(pool.pool_address.clone(), deposit_position);
        }

        Ok(RemoveCollateralResult {
            collateral_decrease,
            collateral_remover_to_receive,
            operation_fees,
        })
    }

    // Repays the debt on a specific obligation per pool. Since `repaid_amount` can exceed the debt
    // — the real repaid amount is calculated as `min(debt, repaid_amount)`
    //
    // # Returns
    // [`Result::Ok((real_repaid_amount, d_tokens_burnt))`] in success and
    // [`Err(MCError)`] in failure
    pub fn repay(
        &mut self,
        e: &Env,
        pool: &Pool,
        provided_amount: i128,
        referrer: &Option<Address>,
    ) -> Result<RepayResult, MCError> {
        let mut borrow_position = self
            .borrows
            .get(pool.pool_address.clone())
            .ok_or(MCError::BorrowPositionDoesNotExist)?;

        let all_debt = pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;
        let all_debt_fees = compute_operation_fees(
            all_debt,
            pool.config.fee_config.repay_fee_bps,
            referrer,
            &pool.config.fee_config,
        )?;
        let amount_to_repay_all_debt =
            all_debt.checked_add(all_debt_fees.fee_sum).map_over_or_underflow()?;

        let (is_all_repaid, amount_to_take_from_borrower, operation_fees) =
            if provided_amount < amount_to_repay_all_debt {
                let operation_fees = compute_operation_fees(
                    provided_amount,
                    pool.config.fee_config.repay_fee_bps,
                    referrer,
                    &pool.config.fee_config,
                )?;
                (false, provided_amount, operation_fees)
            } else {
                (true, amount_to_repay_all_debt, all_debt_fees)
            };
        let debt_decrease_in_tokens = amount_to_take_from_borrower
            .checked_sub(operation_fees.fee_sum)
            .map_over_or_underflow()?;
        let d_tokens_to_burn = if is_all_repaid {
            borrow_position.d_tokens
        } else {
            pool.compute_d_tokens_from_tokens_floor(debt_decrease_in_tokens)?
        };
        if d_tokens_to_burn <= 0 {
            return Err(MCError::NonPositiveSharesAmount);
        }

        let unpaid_interest =
            all_debt.checked_sub(borrow_position.originally_borrowed).map_over_or_underflow()?;
        if unpaid_interest.is_negative() {
            events::computed_interest_is_negative(
                e,
                &pool.pool_address,
                borrow_position.d_tokens,
                all_debt,
                unpaid_interest,
            );
        }

        let new_originally_borrowed = if is_all_repaid {
            0
        } else {
            let new_total_d_tokens = pool.total_d_tokens - d_tokens_to_burn; // safe
            let new_total_borrowed = pool.total_borrowed - debt_decrease_in_tokens; // safe
            let new_d_tokens = borrow_position.d_tokens - d_tokens_to_burn; // safe

            Pool::compute_tokens_from_shares_floor(
                e,
                new_d_tokens,
                new_total_d_tokens,
                new_total_borrowed,
            )?
        };
        borrow_position.set_originally_borrowed(e, new_originally_borrowed)?;

        if is_all_repaid {
            self.try_remove_borrow_position(e, &pool.pool_address)?;
        } else {
            borrow_position
                .adjust_d_tokens(e, d_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;
            self.borrows.set(pool.pool_address.clone(), borrow_position);
        }
        let amount_to_send_back = if amount_to_take_from_borrower < provided_amount {
            provided_amount - amount_to_take_from_borrower // safe
        } else {
            0
        };

        Ok(RepayResult {
            d_tokens_to_burn,
            debt_repaid: debt_decrease_in_tokens,
            amount_to_send_back,
            operation_fees,
        })
    }

    pub fn liquidate(
        &mut self,
        e: &Env,
        borrow_pool: &Pool,
        collateral_pool: &Pool,
        repay_amount: i128,
        demanded_collateral_amount: i128,
    ) -> Result<LiquidationResult, MCError> {
        let (mut deposit_position, mut borrow_position) = (
            self.deposits
                .get(collateral_pool.pool_address.clone())
                .ok_or(MCError::DepositPositionDoesNotExist)?,
            self.borrows
                .get(borrow_pool.pool_address.clone())
                .ok_or(MCError::BorrowPositionDoesNotExist)?,
        );

        let (obligation_debt_value_w_liability_factors, obligation_collateral_value_w_close_ltvs) = (
            self.compute_debt_value_scaled_w_liability_factors(e)?,
            self.compute_collateral_value_scaled_w_close_ltvs(e)?,
        );
        if obligation_debt_value_w_liability_factors <= obligation_collateral_value_w_close_ltvs {
            return Err(MCError::ObligationIsHealthy);
        }

        let (obligation_debt_value, obligation_collateral_value) =
            (self.compute_debt_value(e)?, self.compute_collateral_value(e)?);
        let unparameterized_ltv_bps = obligation_debt_value
            .fixed_div_ceil(obligation_collateral_value, BPS_FACTOR)
            .map_over_or_underflow()?;
        let (max_liquidation_incentive_bps, liquidation_close_factor_bps) = (
            borrow_pool
                .config
                .health_config
                .max_liquidation_incentive_bps
                .min(collateral_pool.config.health_config.max_liquidation_incentive_bps),
            borrow_pool.config.health_config.liquidation_close_factor_bps,
        );

        let (insolvency_ltv_bps, min_collateral_value_threshold) =
            (storage::get_insolvency_ltv_bps(e), compute_min_collateral_threshold_scaled(e)?);
        let is_solvent = unparameterized_ltv_bps < insolvency_ltv_bps;

        let position_debt =
            borrow_pool.compute_tokens_from_d_tokens_ceil(e, borrow_position.d_tokens)?;
        let liquidated_amount = repay_amount.min(position_debt);
        let liquidated_value = oracle::get_asset_value_floor(
            e,
            liquidated_amount,
            &borrow_pool.token_address,
            borrow_pool.token_decimals,
        )?;

        let position_collateral_tokens_from_j_tokens =
            collateral_pool.compute_tokens_from_j_tokens_floor(e, deposit_position.j_tokens)?;
        let position_collateral_sum = deposit_position
            .collateral
            .checked_add(position_collateral_tokens_from_j_tokens)
            .map_over_or_underflow()?;

        // -- Count max possible redeemed collateral with liquidation incentive --

        let redeemed_collateral_amount_with_max_incentive = {
            let collateral_amount = oracle::get_asset_amount_floor(
                e,
                liquidated_value,
                &collateral_pool.token_address,
                collateral_pool.token_decimals,
            )?;

            collateral_amount
                .fixed_mul_floor(
                    BPS_FACTOR
                        .checked_add(max_liquidation_incentive_bps)
                        .map_over_or_underflow()?,
                    BPS_FACTOR,
                )
                .map_over_or_underflow()?
        };

        let mut collateral_to_sell_to_liquidator = if is_solvent {
            // -- LTV-improving scenario --

            // Check if the liquidation doesn't exceed the close factor
            let liquidated_borrow_bps = liquidated_amount
                .fixed_div_ceil(position_debt, BPS_FACTOR)
                .map_over_or_underflow()?;
            if liquidated_borrow_bps > liquidation_close_factor_bps {
                return Err(MCError::InvalidLiquidationInputs);
            }

            let max_ltv_improving_collateral_seized = {
                // Count the maximum amount of sold collateral that improves LTV:
                // ----
                // (old_debt_value / old_collateral_value) > (old_debt_value - debt_value_repaid) / (old_collateral_value - collateral_value_lost),
                // , implies:
                // collateral_value_lost < (old_collateral_value / old_debt_value) * debt_value_repaid
                // ----
                // must hold for the position to become healthier
                let unparameterized_ltv_sc9 = obligation_collateral_value
                    .fixed_div_floor(obligation_debt_value, SCALAR_9)
                    .map_over_or_underflow()?;

                let max_collateral_received_value = liquidated_value
                    .fixed_mul_floor(unparameterized_ltv_sc9, SCALAR_9)
                    .map_over_or_underflow()?;

                // Take 99.9% to guarantee LTV improvement
                let strict_max_collateral_received_value = max_collateral_received_value
                    .fixed_mul_floor(999, 1000)
                    .map_over_or_underflow()?;

                oracle::get_asset_amount_floor(
                    e,
                    strict_max_collateral_received_value,
                    &collateral_pool.token_address,
                    collateral_pool.token_decimals,
                )?
            };

            position_collateral_sum
                .min(max_ltv_improving_collateral_seized)
                .min(redeemed_collateral_amount_with_max_incentive)
        } else {
            // -- Insolvency scenario - the obligation is given away completely to liquidators to decrease the amount of `bad debt`
            // in the market --

            position_collateral_sum.min(redeemed_collateral_amount_with_max_incentive)
        };

        let collateral_left = position_collateral_sum - collateral_to_sell_to_liquidator; // safe
        let collateral_value_left = oracle::get_asset_value_floor(
            e,
            collateral_left,
            &collateral_pool.token_address,
            collateral_pool.token_decimals,
        )?;

        let is_all_collateral_drained = if collateral_value_left < min_collateral_value_threshold {
            // If collateral(both plain collateral and supply shares) that's left is worth
            // less than the configured `min_collateral_value_cents` on the market, the liquidator
            // additionally receives all of the collateral that's left
            collateral_to_sell_to_liquidator += collateral_left;

            true
        } else {
            false
        };

        if collateral_to_sell_to_liquidator < demanded_collateral_amount {
            return Err(MCError::LiquidationExcessiveDemandedCollateral);
        }

        let is_plain_collateral_sufficient =
            deposit_position.collateral >= collateral_to_sell_to_liquidator;

        // -- Adjust Deposit Position --

        let tokens_from_j_tokens_seized = if is_plain_collateral_sufficient {
            0
        } else {
            collateral_to_sell_to_liquidator - deposit_position.collateral
        };

        // Distribute liquidator incentive between plain collateral and received `jTokens`
        let (plain_collateral_seized, j_tokens_seized) = if is_plain_collateral_sufficient {
            (collateral_to_sell_to_liquidator, 0)
        } else {
            let j_tokens = if is_all_collateral_drained {
                // Complete collateral liquidation
                deposit_position.j_tokens
            } else {
                collateral_pool
                    .compute_j_tokens_from_tokens_ceil(tokens_from_j_tokens_seized)?
                    .min(deposit_position.j_tokens)
            };

            (deposit_position.collateral, j_tokens)
        };

        if j_tokens_seized.is_positive() {
            let new_j_tokens = deposit_position.j_tokens - j_tokens_seized; // safe
            let new_originally_deposited = Pool::compute_tokens_from_shares_floor(
                e,
                new_j_tokens,
                collateral_pool.total_j_tokens,
                collateral_pool.total_supply()?,
            )?;

            deposit_position.set_originally_deposited(e, new_originally_deposited)?;
        }

        deposit_position
            .adjust_collateral(e, plain_collateral_seized.checked_neg().map_over_or_underflow()?)?;
        deposit_position
            .adjust_j_tokens(e, j_tokens_seized.checked_neg().map_over_or_underflow()?)?;

        // -- Adjust Borrow Position --

        let is_all_debt_liquidated = liquidated_amount == position_debt;

        let (d_tokens_to_burn, new_originally_borrowed) = if is_all_debt_liquidated {
            // All gone
            (borrow_position.d_tokens, 0)
        } else {
            let d_tokens_to_burn =
                borrow_pool.compute_d_tokens_from_tokens_floor(liquidated_amount)?;
            let unpaid_interest = position_debt
                .checked_sub(borrow_position.originally_borrowed)
                .map_over_or_underflow()?;

            if unpaid_interest.is_negative() {
                events::computed_interest_is_negative(
                    e,
                    &borrow_pool.pool_address,
                    borrow_position.d_tokens,
                    position_debt,
                    unpaid_interest,
                );
            }

            let new_originally_borrowed = if liquidated_amount > unpaid_interest {
                let new_total_d_tokens = borrow_pool.total_d_tokens - d_tokens_to_burn; // safe
                let new_total_borrowed = borrow_pool.total_borrowed - liquidated_amount; // safe
                let new_d_tokens = borrow_position.d_tokens - d_tokens_to_burn; // safe

                Pool::compute_tokens_from_shares_ceil(
                    e,
                    new_d_tokens,
                    new_total_d_tokens,
                    new_total_borrowed,
                )?
            } else {
                borrow_position.originally_borrowed
            };

            (d_tokens_to_burn, new_originally_borrowed)
        };

        if d_tokens_to_burn <= 0 {
            return Err(MCError::NonPositiveSharesAmount);
        }

        borrow_position
            .adjust_d_tokens(e, d_tokens_to_burn.checked_neg().map_over_or_underflow()?)?;
        borrow_position.set_originally_borrowed(e, new_originally_borrowed)?;

        if deposit_position.is_empty() {
            self.try_remove_deposit_position(e, &collateral_pool.pool_address)?;
        } else {
            self.deposits.set(collateral_pool.pool_address.clone(), deposit_position);
        }

        if borrow_position.is_empty() {
            self.try_remove_borrow_position(e, &borrow_pool.pool_address)?;
        } else {
            self.borrows.set(borrow_pool.pool_address.clone(), borrow_position);
        }

        let amount_to_send_back = if liquidated_amount < repay_amount {
            repay_amount - liquidated_amount // safe
        } else {
            0
        };

        Ok(LiquidationResult {
            j_tokens_seized,
            amount_to_send_back,
            debt_repaid: liquidated_amount,
            d_tokens_burned: d_tokens_to_burn,
            plain_collateral_seized,
            tokens_from_j_tokens_seized,
        })
    }

    // Increases `jTokens` amount for the liquidator's obligation if the plain collateral wasn't sufficient to cover the liquidation
    pub fn liquidation_increase_j_tokens(
        &mut self,
        e: &Env,
        collateral_pool: &Pool,
        j_tokens_amount: i128,
    ) -> Result<(), MCError> {
        let mut deposit_position = self
            .deposits
            .get(collateral_pool.pool_address.clone())
            .map_or_else(|| self.try_create_deposit_position(e), Ok)?;

        deposit_position.adjust_j_tokens(e, j_tokens_amount)?;
        self.deposits.set(collateral_pool.pool_address.clone(), deposit_position);

        Ok(())
    }

    pub fn require_no_liquidatable_collateral_exists(&self, e: &Env) -> Result<(), MCError> {
        for (pool_address, deposit_position) in self.deposits.iter() {
            let DepositPosition { j_tokens, collateral, .. } = deposit_position;
            let pool = &storage::get_pool(e, &pool_address).ok_or_else(|| {
                events::pool_is_unexpectedly_missing_in_storage(e, &pool_address);

                MCError::InternalError
            })?;

            let all_collateral = pool
                .compute_tokens_from_j_tokens_floor(e, j_tokens)?
                .checked_add(collateral)
                .map_over_or_underflow()?;
            let collateral_value = oracle::get_asset_value_floor(
                e,
                all_collateral,
                &pool.token_address,
                pool.token_decimals,
            )?;

            let min_collateral_value_threshold = compute_min_collateral_threshold_scaled(e)?;
            if collateral_value > min_collateral_value_threshold {
                return Err(MCError::BadDebtCoverageCriterionIsNotMet);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[contracttype]
pub struct BorrowPosition {
    // Amount of the total debt shares that the obligation contains
    pub d_tokens: i128,
    // Originally borrowed token amount. I.e., if the user borrows 100 tokens and 20 tokens
    // have been accrued with time as additional debt - this value remains 100. If, after that, the user repays the amount
    // that exceeds the debt accrual(like 30) - the value becomes 90. If the user instead borrows 10 tokens, the
    // value increases to 110. In any other case, it doesn't change. Its only purpose is to track the amount
    // of accrued unpaid interest
    pub originally_borrowed: i128,
}

impl BorrowPosition {
    pub fn adjust_d_tokens(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.d_tokens, adjusting_amount)?;
        self.d_tokens = new_amount;

        Ok(())
    }

    pub fn set_originally_borrowed(&mut self, e: &Env, new_amount: i128) -> Result<(), MCError> {
        if new_amount.is_negative() {
            events::obligation_amount_becomes_negative(e, self.originally_borrowed, new_amount);

            return Err(MCError::InternalError);
        }
        self.originally_borrowed = new_amount;

        Ok(())
    }

    pub fn adjust_originally_borrowed(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.originally_borrowed, adjusting_amount)?;
        self.originally_borrowed = new_amount;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.d_tokens == 0 && self.originally_borrowed == 0
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[contracttype]
pub struct DepositPosition {
    // A share of total supplied tokens in the pool that obligation contains
    pub j_tokens: i128,
    // Accumulated value of collateral that doesn't accrue interest
    pub collateral: i128,
    // Originally deposited token amount. I.e., if the user deposits 100 tokens and 20 tokens
    // have been accrued with time - this value remains 100. If, after that, the user withdraws the amount
    // that exceeds the accrual(like 30) - the value becomes 90 (same goes for when `j_tokens` are seized
    // as collateral by a liquidator. If the user instead deposits 10 tokens, the
    // value increases to 110. In any other case, it doesn't change. Its only purpose is to track the amount
    // of received supply interest
    pub originally_deposited: i128,
    // Timestamp of when the last scarcity withdraw took place
    pub last_scarcity_withdraw_ts: u64,
}

impl DepositPosition {
    pub fn adjust_j_tokens(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.j_tokens, adjusting_amount)?;
        self.j_tokens = new_amount;

        Ok(())
    }

    pub fn set_originally_deposited(&mut self, e: &Env, new_amount: i128) -> Result<(), MCError> {
        if new_amount.is_negative() {
            events::obligation_amount_becomes_negative(e, self.originally_deposited, new_amount);

            return Err(MCError::InternalError);
        }

        self.originally_deposited = new_amount;

        Ok(())
    }

    pub fn adjust_originally_deposited(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.originally_deposited, adjusting_amount)?;
        self.originally_deposited = new_amount;

        Ok(())
    }

    pub fn adjust_collateral(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), MCError> {
        let new_amount = adjust_obligation_field(e, self.collateral, adjusting_amount)?;
        self.collateral = new_amount;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.j_tokens == 0 && self.collateral == 0
    }
}

// Used to generate a unique seed for `Earn` obligation
// See [`compute_earn_obligation_seed`]
const EARN_OBLIGATION_SEED_STR: &str = "EV";

// Computes 'Earn' seed and caches it if it hasn't been computed yet, or gets it from the storage otherwise
//
// # Returns
// [`BytesN<32>`] bytes used as an obligation seed to distinguish unique users' obligations
pub fn get_earn_obligation_seed(e: &Env) -> BytesN<32> {
    if let Some(stored_seed) = storage::get_earn_obligation_seed(e) {
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

// Adjusts a field on the obligation's structs
//
// # Returns
// `Ok(new_amount)` if adjusting doesn't lead to a new amount being negative.
// `Err(MCError::InternalError)` otherwise
fn adjust_obligation_field(
    e: &Env,
    current_value: i128,
    adjusting_amount: i128,
) -> Result<i128, MCError> {
    let new_amount = current_value.checked_add(adjusting_amount).map_over_or_underflow()?;

    if new_amount.is_negative() {
        events::obligation_amount_becomes_negative(e, current_value, new_amount);

        return Err(MCError::InternalError);
    }

    Ok(new_amount)
}

// Accrues interest on a pool
//
// # WARNING
// Modifies the contract's storage
fn accrue_interest_on_pool(e: &Env, pool_address: &Address) -> Result<(), MCError> {
    let mut pool = storage::get_pool(e, pool_address).ok_or(MCError::PoolDoesNotExist)?;

    pool.accrue_interest(e)?;
    pool.set(e);

    Ok(())
}

// Computes the operation's one-time fees
pub fn compute_operation_fees(
    original_amount: i128,
    fee_bps: u32,
    referrer: &Option<Address>,
    pool_fee_config: &PoolFeeConfig,
) -> Result<OperationFees, MCError> {
    if fee_bps == 0 || original_amount == 0 {
        return Ok(OperationFees { fee_sum: 0, referrer_fee: 0 });
    }

    // -- Calculate Total Fee --

    let fee_sum =
        original_amount.fixed_mul_ceil(fee_bps as i128, BPS_FACTOR).map_over_or_underflow()?;

    // -- Calculate the Referrer Split --

    let referrer_fee = if let Some(referrer_addr) = referrer
        && let Some(referrers_map) = &pool_fee_config.referrers
        && let Some(referrer_share_bps) = referrers_map.get(referrer_addr.clone())
        && referrer_share_bps != 0
    {
        fee_sum.fixed_mul_ceil(referrer_share_bps as i128, BPS_FACTOR).map_over_or_underflow()?
    } else {
        0
    };

    Ok(OperationFees { fee_sum, referrer_fee })
}

// Computes additional withdraw scarcity fee(in basis) points that is charged when pool's utilization ratio
// exceeds utilization ratio limit. E.g., (2% of origination fee + 4.5% of scarcity fee, etc.)
fn compute_withdraw_scarcity_fee_bps(
    e: &Env,
    pool: &Pool,
    deposit_decrease: i128,
    deposit_position: &DepositPosition,
) -> Result<u32, MCError> {
    let pool_total_supply = pool.total_supply()?;
    if pool_total_supply == 0 {
        return Ok(0);
    }

    let current_utilization_ratio_bps = pool.compute_utilization_ratio_bps()?;
    let new_utilization_ratio_bps = {
        let new_total_supply = pool_total_supply - deposit_decrease; // safe

        if new_total_supply == 0 {
            // Happens only when the last withdrawal takes place in a pool
            0
        } else {
            pool.total_borrowed
                .fixed_div_ceil(new_total_supply, BPS_FACTOR)
                .map_over_or_underflow()?
        }
    };

    if new_utilization_ratio_bps == BPS_FACTOR {
        return Ok(pool.config.fee_config.withdraw_max_scarcity_fee_bps);
    }

    let utilization_ratio_diff_bps = if new_utilization_ratio_bps
        > pool.config.health_config.utilization_ratio_limit_bps
    {
        let deposit_decrease_to_total_supply_bps = deposit_decrease
            .fixed_div_ceil(pool_total_supply, BPS_FACTOR)
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

        new_utilization_ratio_bps - pool.config.health_config.utilization_ratio_limit_bps // safe
    } else {
        return Ok(0);
    };

    let max_utilization_ratio_diff_bps =
        BPS_FACTOR - pool.config.health_config.utilization_ratio_limit_bps; // safe
    let fee = utilization_ratio_diff_bps
        .fixed_mul_ceil(
            pool.config.fee_config.withdraw_max_scarcity_fee_bps as i128,
            max_utilization_ratio_diff_bps,
        )
        .unwrap() as u32; // safe

    Ok(fee)
}

#[contracttype]
#[derive(Clone)]
// Represents operational one-time fees
pub struct OperationFees {
    // Fee sum
    pub fee_sum: i128,
    // Fee, immediately sent to the referrer if one is present
    pub referrer_fee: i128,
}

#[contracttype]
// [`Obligation::deposit`] resulting data
pub struct DepositResult {
    // Amount of `jTokens` to issue that represent the `deposited` amount in the pool
    pub j_tokens_to_issue: i128,
    // Amount of originally deposited tokens(minus all possible fees)
    pub deposited: i128,
    pub operation_fees: OperationFees,
}

#[contracttype]
// [`Obligation::borrow`] resulting data
pub struct BorrowResult {
    // Amount of `dTokens` to issue that represent the `borrower_new_debt` amount in the pool
    pub d_tokens_to_issue: i128,
    // Amount of debt(in tokens) that is added to the borrower's obligation
    pub borrower_new_debt: i128,
    // Amount of tokens to receive by the borrower(`borrower_new_debt` minus all fees)
    pub borrower_to_receive: i128,
    pub operation_fees: OperationFees,
}

#[contracttype]
// [`Obligation::add_collateral`] resulting data
pub struct AddCollateralResult {
    // Amount of tokens added as collateral(minus all possible fees)
    pub added_collateral: i128,
    pub operation_fees: OperationFees,
}

#[contracttype]
// [`Obligation::withdraw`] resulting data
pub struct WithdrawResult {
    // Amount of `jTokens` to burn that represent the `deposit_decreased_amount` amount in the
    // pool
    pub j_tokens_to_burn: i128,
    // Amount of the original deposit(in tokens) that is removed from the `DepositPosition`
    pub deposit_decrease: i128,
    // Amount of tokens to receive by the withdrawer(`deposit_decreased_amount` minus fees)
    pub withdrawer_to_receive: i128,
    pub operation_fees: OperationFees,
}

#[contracttype]
#[derive(Clone)]
// [`Obligation::repay`] resulting data
pub struct RepayResult {
    // Amount of `dTokens` to burn that represent the `real_repaid` amount in the pool
    pub d_tokens_to_burn: i128,
    // Amount of the debt that is repaid
    pub debt_repaid: i128,
    // Excess amount given by the borrower that is sent back
    pub amount_to_send_back: i128,
    pub operation_fees: OperationFees,
}

#[contracttype]
// [`Obligation::remove_collateral`] resulting data
pub struct RemoveCollateralResult {
    // Amount of collateral tokens removed
    pub collateral_decrease: i128,
    // Amount of collateral tokens received by the collateral remover(accounting subtracted fees)
    pub collateral_remover_to_receive: i128,
    pub operation_fees: OperationFees,
}

#[contracttype]
pub struct LiquidationResult {
    // The amount of debt tokens repaid by the liquidator
    pub debt_repaid: i128,
    // Excess amount given by the liquidator that is sent back
    pub amount_to_send_back: i128,
    // The amount of `dTokens` that are burned from the borrower's borrow position
    pub d_tokens_burned: i128,
    // The amount of plain collateral seized from the borrower's obligation and transferred to the liquidator
    pub plain_collateral_seized: i128,
    // The amount of `jTokens` seized from the borrower's obligation and given away to the liquidator's obligation
    // in case the borrower's position doesn't contain enough plain collateral to cover the liquidation expenses
    pub j_tokens_seized: i128,
    // The amount of tokens representing the `j_tokens_seized` computed via ceiling
    pub tokens_from_j_tokens_seized: i128,
}

pub fn compute_min_collateral_threshold_scaled(e: &Env) -> Result<i128, MCError> {
    let min_collat_cents = storage::get_min_collateral_value_cents(e);

    let oracle_decimals = oracle::get_oracle_price_decimals(e);
    let total_scale = i128::pow(10, oracle_decimals);

    let threshold = min_collat_cents
        .checked_mul(total_scale)
        .map_over_or_underflow()?
        .checked_div(100)
        .map_over_or_underflow()?;

    Ok(threshold)
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
