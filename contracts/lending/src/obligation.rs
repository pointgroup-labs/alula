use {
    crate::{
        constants::{LCError, BPS_FACTOR, HEALTH_FACTOR_THRESHOLD_BPS},
        contract::get_asset_price,
        pool::PoolConfig,
        storage::{self, get_global_state, PoolAddress},
    },
    core::borrow,
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{contracttype, token, Address, Env, Map},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct Obligation {
    /// Deposited collateral for the obligation, unique by deposit pool address
    pub deposits: Map<PoolAddress, DepositObligation>,
    /// Borrowed liquidity for the obligation, unique by borrow pool address
    pub borrows: Map<PoolAddress, BorrowObligation>,
    // /// Last update to collateral, liquidity, or their market values
    // pub last_update: u64,
    // /// Market value of deposits
    // pub deposited_value: i128,
    // /// Market value of deposits
    // pub borrowed_value: i128,
}

impl Obligation {
    pub fn new(e: &Env) -> Self {
        Self {
            deposits: Map::new(e),
            borrows: Map::new(e),
        }
    }

    pub fn accrue_interest(&mut self, e: &Env) -> Result<(), LCError> {
        for (pool_address, mut borrow_obligation) in self.borrows.iter() {
            borrow_obligation.accrue_interest(e, &pool_address)?;
            // WARN: I am not sure if this will work, tbh
            self.borrows.set(pool_address, borrow_obligation);
        }

        Ok(())
    }

    pub fn is_healthy(&self, e: &Env) -> Result<bool, LCError> {
        Ok(self.compute_health_factor_bps(e)? >= HEALTH_FACTOR_THRESHOLD_BPS)
    }

    pub fn is_empty(&self) -> bool {
        self.deposits.is_empty() && self.borrows.is_empty()
    }

    fn compute_health_factor_bps(&self, e: &Env) -> Result<i128, LCError> {
        let liquidation_threshold_bps = get_global_state(e).liquidation_threshold_bps;

        let (mut collateral_value_sum, mut borrowed_value_sum) = (0i128, 0i128);

        for (collateral_pool_address, deposit_obligation) in self.deposits.iter() {
            let DepositObligation {
                shares, collateral, ..
            } = deposit_obligation;

            let Some(collateral_pool) = storage::get_pool(e, &collateral_pool_address) else {
                return Err(LCError::InternalError);
            };

            let shares_to_tokens = shares
                .checked_mul(collateral_pool.available + collateral_pool.total_borrowed)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_div(collateral_pool.total_shares)
                .ok_or(LCError::OverOrUnderflow)?;

            let total_tokens = shares_to_tokens + collateral;

            let asset_price = get_asset_price(e, &collateral_pool.token_ticker)?;

            collateral_value_sum = collateral_value_sum
                .checked_add(
                    asset_price
                        .checked_mul(total_tokens)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;
        }

        for (borrow_pool_address, borrow_obligation) in self.borrows.iter() {
            let Some(borrow_pool) = storage::get_pool(e, &borrow_pool_address) else {
                return Err(LCError::InternalError);
            };

            let borrowed = borrow_obligation.borrowed;

            let borrowed_asset_price = get_asset_price(e, &borrow_pool.token_ticker)?;

            borrowed_value_sum = borrowed_value_sum
                .checked_add(
                    borrowed_asset_price
                        .checked_mul(borrowed)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;
        }

        if borrowed_value_sum == 0 {
            // If nothing is borrowed - it's the healthiest obligation it can be
            return Ok(i128::MAX);
        }

        let numerator = collateral_value_sum
            .checked_mul(liquidation_threshold_bps)
            .ok_or(LCError::OverOrUnderflow)?;
        let health_factor_bps = numerator
            .checked_div(borrowed_value_sum)
            .ok_or(LCError::OverOrUnderflow)?;

        Ok(health_factor_bps)
    }

    /// Deposits assets on an obligation per pool
    pub fn deposit(&mut self, pool_address: &Address, shares_amount: i128) -> Result<(), LCError> {
        self.adjust_shares(pool_address, shares_amount)
    }

    /// Borrows assets on an obligation per pool
    pub fn borrow(&mut self, pool_address: &Address, borrowed_amount: i128) -> Result<(), LCError> {
        self.adjust_borrowed(pool_address, borrowed_amount)
    }

    /// Deposits collateral assets on an obligation per pool
    pub fn deposit_collateral(
        &mut self,
        pool_address: &Address,
        collateral_amount: i128,
    ) -> Result<(), LCError> {
        self.adjust_collateral(pool_address, collateral_amount)
    }

    /// Withdraws assets from an obligation per pool
    pub fn withdraw(&mut self, pool_address: &Address, shares_amount: i128) -> Result<(), LCError> {
        let mut deposit_obligation = self
            .deposits
            .get(pool_address.clone())
            .ok_or(LCError::ObligationDoesNotExist)?;

        if deposit_obligation.shares < shares_amount {
            return Err(LCError::WithdrawOverBalance);
        }

        deposit_obligation.adjust_shares(-shares_amount)?;
        if deposit_obligation.is_empty() {
            self.deposits.remove(pool_address.clone());
        } else {
            self.deposits.set(pool_address.clone(), deposit_obligation);
        }

        Ok(())
    }

    /// Withdraws collateral assets from an obligation per pool
    pub fn withdraw_collateral(
        &mut self,
        pool_address: &Address,
        collateral_amount: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();

        if deposit_obligation.collateral < collateral_amount {
            return Err(LCError::WithdrawOverBalance);
        }

        deposit_obligation.adjust_collateral(-collateral_amount)?;
        if deposit_obligation.is_empty() {
            self.deposits.remove(pool_address.clone());
        } else {
            self.deposits.set(pool_address.clone(), deposit_obligation);
        }

        Ok(())
    }

    /// Repays the debt on a specific obligation per pool. Since `repaid_amount` can exceed the debt -
    /// the real repaid amount is calculated as `min(debt, repaid_amount)`
    ///
    /// # Returns
    /// [`Result::Ok(real_repaid_amount)`] in success and [`Err(LCError)`] in failure
    pub fn repay(&mut self, pool_address: &Address, repaid_amount: i128) -> Result<i128, LCError> {
        let mut borrow_obligation = self
            .borrows
            .get(pool_address.clone())
            .ok_or(LCError::ObligationDoesNotExist)?;

        let real_repaid_amount = i128::min(
            repaid_amount,
            borrow_obligation.borrowed + borrow_obligation.unpaid_interest,
        );
        if real_repaid_amount == (borrow_obligation.borrowed + borrow_obligation.unpaid_interest) {
            self.borrows.remove(pool_address.clone());
        } else {
            borrow_obligation.adjust_borrowed(-real_repaid_amount)?;
            self.borrows.set(pool_address.clone(), borrow_obligation);
        }

        Ok(real_repaid_amount)
    }

    /// Liquidates unhealthy borrow
    ///
    /// # WARN
    /// This modifies collateral pools' data in the storage and sends tokens to the liquidator as
    /// a side effect
    pub fn liquidate(
        &mut self,
        e: &Env,
        pool_address: &Address,
        liquidator: &Address,
        amount: i128,
    ) -> Result<i128, LCError> {
        let Some(borrow_obligation) = self.borrows.get(pool_address.clone()) else {
            return Err(LCError::ObligationDoesNotExist);
        };

        let Some(pool) = storage::get_pool(e, pool_address) else {
            return Err(LCError::PoolDoesNotExist);
        };

        let PoolConfig {
            liquidation_close_factor_bps,
            liquidation_incentive_bps,
            ..
        } = pool.config;

        let borrowed_amount = borrow_obligation.borrowed;
        let liquidatable_bps = amount
            .fixed_div_floor(borrowed_amount, BPS_FACTOR)
            .ok_or(LCError::OverOrUnderflow)?;

        if liquidatable_bps > liquidation_close_factor_bps {
            // TODO: What's the best way to set `close_factor_bps` value?
            return Err(LCError::LiquidationExceedsCloseFactor);
        }

        let borrowed_asset_price = get_asset_price(e, &pool.token_ticker)?;

        let liquidation_value = borrowed_asset_price
            .checked_mul(amount)
            .ok_or(LCError::OverOrUnderflow)?;

        let mut desired_collateral_value_to_redeem = liquidation_value
            .fixed_mul_floor(BPS_FACTOR + liquidation_incentive_bps, BPS_FACTOR)
            .ok_or(LCError::OverOrUnderflow)?;

        // TODO: It seems more reasonable to take all collateral from pools first
        // and only then to start taking available amount
        for (collateral_pool_address, mut deposit_obligation) in self.deposits.iter() {
            let DepositObligation {
                collateral, shares, ..
            } = deposit_obligation;

            let Some(mut collateral_pool) = storage::get_pool(e, pool_address) else {
                // NB: This is a major invariant breakage - not a wrong input error
                return Err(LCError::PoolDoesNotExist);
            };

            let total_deposit_liquidity = collateral_pool
                .total_borrowed
                .checked_add(collateral_pool.available)
                .ok_or(LCError::OverOrUnderflow)?;

            let tokens_in_shares = shares
                .checked_mul(total_deposit_liquidity)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_div(collateral_pool.total_shares)
                .ok_or(LCError::OverOrUnderflow)?;

            let deposit = i128::min(tokens_in_shares, collateral_pool.available);

            let collateral_asset_price = get_asset_price(e, &collateral_pool.token_ticker)?;

            let collateral_value = collateral
                .checked_mul(collateral_asset_price)
                .ok_or(LCError::OverOrUnderflow)?;

            let deposit_value = deposit
                .checked_mul(collateral_asset_price)
                .ok_or(LCError::OverOrUnderflow)?;

            // Check if collateral alone can cover the purchase of the debt
            if desired_collateral_value_to_redeem <= collateral_value {
                let collateral_tokens_to_take = desired_collateral_value_to_redeem
                    .checked_div(collateral_asset_price)
                    .ok_or(LCError::OverOrUnderflow)?;

                deposit_obligation.adjust_collateral(-collateral_tokens_to_take)?;
                collateral_pool.adjust_total_collateral(-collateral_tokens_to_take)?;

                storage::set_pool(e, &collateral_pool_address, &collateral_pool);

                let token_client = token::Client::new(e, &collateral_pool_address);
                token_client.transfer(
                    &e.current_contract_address(),
                    liquidator,
                    &collateral_tokens_to_take,
                );

                desired_collateral_value_to_redeem = 0;

                break;
            } else {
                // If collateral isn't sufficient - try to cover the debt with available tokens from the pool
                let value_left = desired_collateral_value_to_redeem - collateral_value;

                let deposit_value_to_take = if deposit_value >= value_left {
                    value_left
                } else {
                    deposit_value
                };

                let deposit_tokens_to_take = deposit_value_to_take
                    .checked_div(collateral_asset_price)
                    .ok_or(LCError::OverOrUnderflow)?;

                let shares_to_burn =
                    collateral_pool.compute_shares_amount(deposit_tokens_to_take)?;

                deposit_obligation.adjust_shares(-shares_to_burn)?;
                collateral_pool.adjust_available(-deposit_tokens_to_take)?;
                collateral_pool.adjust_total_shares(-shares_to_burn)?;

                let token_client = token::Client::new(e, &collateral_pool_address);
                token_client.transfer(
                    &e.current_contract_address(),
                    liquidator,
                    &deposit_tokens_to_take,
                );

                desired_collateral_value_to_redeem = desired_collateral_value_to_redeem
                    .checked_sub(deposit_value_to_take)
                    .ok_or(LCError::OverOrUnderflow)?;

                if desired_collateral_value_to_redeem < 0 {
                    return Err(LCError::InternalError);
                } else if desired_collateral_value_to_redeem == 0 {
                    break;
                }
            }
        }

        let liquidated_amount = amount
            .checked_sub(
                desired_collateral_value_to_redeem
                    .checked_div(borrowed_asset_price)
                    .ok_or(LCError::OverOrUnderflow)?,
            )
            .ok_or(LCError::OverOrUnderflow)?;

        if liquidated_amount < 0 {
            return Err(LCError::InternalError);
        }

        Ok(liquidated_amount)
    }

    fn adjust_shares(
        &mut self,
        pool_address: &Address,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();
        deposit_obligation.adjust_shares(adjusting_amount)?;
        self.deposits.set(pool_address.clone(), deposit_obligation);

        Ok(())
    }

    fn adjust_borrowed(
        &mut self,
        pool_address: &Address,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        let mut borrow_obligation = self.borrows.get(pool_address.clone()).unwrap_or_default();
        borrow_obligation.adjust_borrowed(adjusting_amount)?;
        self.borrows.set(pool_address.clone(), borrow_obligation);

        Ok(())
    }

    fn adjust_collateral(
        &mut self,
        pool_address: &Address,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();
        deposit_obligation.adjust_collateral(adjusting_amount)?;
        self.deposits.set(pool_address.clone(), deposit_obligation);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[contracttype]
pub struct BorrowObligation {
    /// The initial amount of the borrowed token
    pub borrowed: i128,
    /// The amount of unpaid interest
    pub unpaid_interest: i128,
    /// The numerical value that is used to determine the scaling factor required for updating the position amount
    /// with interest, i.e. new_borrowed = (current_accrual \ last_accrual) * borrowed
    pub last_accrual: i128,
}

impl BorrowObligation {
    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.borrowed == 0
    }

    pub fn adjust_borrowed(&mut self, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = self
            .borrowed
            .checked_add(adjusting_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_amount < 0 {
            // TODO: event/log the specific issue
            // This shouldn't be a specific `[LCError]` variant,
            // since it's a broken invariant and not a cause of a bad input
            return Err(LCError::InternalError);
        }

        self.borrowed = new_amount;

        Ok(())
    }

    /// Accrues interest on a borrow obligation
    ///
    /// # WARN
    /// This modifies the pool's data in the storage
    pub fn accrue_interest(&mut self, e: &Env, pool_address: &Address) -> Result<(), LCError> {
        let mut pool = storage::get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;
        pool.accrue_interest(e)?;

        let prev_debt = self.borrowed + self.unpaid_interest;
        let new_debt = prev_debt
            .checked_mul(pool.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(self.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?;

        let new_unpaid_interest = new_debt
            .checked_sub(self.borrowed)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_unpaid_interest < 0 {
            return Err(LCError::InternalError);
        }

        self.unpaid_interest = new_unpaid_interest;
        self.last_accrual = pool.last_accrual;

        storage::set_pool(e, pool_address, &pool);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[contracttype]
pub struct DepositObligation {
    pub collateral: i128,
    pub shares: i128,
}

impl DepositObligation {
    pub fn adjust_shares(&mut self, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = self
            .shares
            .checked_add(adjusting_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_amount < 0 {
            // TODO: event/log the specific issue
            // This shouldn't be a specific `[LCError]` variant,
            // since it's a broken invariant and not a cause of a bad input
            return Err(LCError::InternalError);
        }

        self.shares = new_amount;

        Ok(())
    }

    pub fn adjust_collateral(&mut self, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = self
            .collateral
            .checked_add(adjusting_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_amount < 0 {
            // TODO: event/log the specific issue
            // This shouldn't be a specific `[LCError]` variant,
            // since it's a broken invariant and not a cause of a bad input
            return Err(LCError::InternalError);
        }

        self.collateral = new_amount;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.shares == 0 && self.collateral == 0
    }
}
