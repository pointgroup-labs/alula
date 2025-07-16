use {
    crate::{
        constants::{ACCRUAL_INIT, BPS_FACTOR, HEALTH_FACTOR_THRESHOLD_BPS},
        contract::get_asset_price,
        events,
        math_utils::MathUtils,
        pool::{Pool, PoolConfig},
        storage::{self, get_global_state},
        LCError,
    },
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{contracttype, Address, Env, Map},
};

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct Obligation {
    /// The obligation's user
    pub user: Address,
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
    pub fn new(e: &Env, user: Address) -> Self {
        Self {
            user,
            deposits: Map::new(e),
            borrows: Map::new(e),
        }
    }

    /// Accrues interest on all borrows for the obligation
    ///
    /// # WARNING
    /// Modifies the obligation's pools storage data, but **DOESN'T** modify the obligation's storage data
    pub fn accrue_interest(&mut self, e: &Env) -> Result<(), LCError> {
        for (pool_address, mut borrow_obligation) in self.borrows.iter() {
            borrow_obligation.accrue_interest(e, &pool_address)?;
            // TODO: Check if you can modify and iterate through [`soroban_sdk::Map`] at the same time
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
                events::pool_is_missing_in_storage(e, &collateral_pool_address);

                return Err(LCError::InternalError);
            };

            let shares_to_tokens = if collateral_pool.total_shares != 0 {
                shares
                    .checked_mul(collateral_pool.available + collateral_pool.total_borrowed)
                    .map_over_or_underflow()?
                    .checked_div(collateral_pool.total_shares)
                    .map_over_or_underflow()?
            } else {
                0
            };

            let total_tokens = shares_to_tokens + collateral;

            let asset_price = get_asset_price(e, &collateral_pool.token_ticker)?;

            collateral_value_sum = collateral_value_sum
                .checked_add(
                    asset_price
                        .checked_mul(total_tokens)
                        .map_over_or_underflow()?,
                )
                .map_over_or_underflow()?;
        }

        for (borrow_pool_address, borrow_obligation) in self.borrows.iter() {
            let total_debt = borrow_obligation.total_debt()?;

            let Some(borrow_pool) = storage::get_pool(e, &borrow_pool_address) else {
                events::pool_is_missing_in_storage(e, &borrow_pool_address);

                return Err(LCError::InternalError);
            };

            let borrowed_asset_price = get_asset_price(e, &borrow_pool.token_ticker)?;

            borrowed_value_sum = borrowed_value_sum
                .checked_add(
                    borrowed_asset_price
                        .checked_mul(total_debt)
                        .map_over_or_underflow()?,
                )
                .map_over_or_underflow()?;
        }

        if borrowed_value_sum == 0 {
            // If nothing is borrowed - it's the healthiest obligation it can be
            return Ok(i128::MAX);
        }

        let numerator = collateral_value_sum
            .checked_mul(liquidation_threshold_bps)
            .map_over_or_underflow()?;
        let health_factor_bps = numerator
            .checked_div(borrowed_value_sum)
            .map_over_or_underflow()?;

        Ok(health_factor_bps)
    }

    /// Deposits assets on an obligation per pool
    pub fn deposit(
        &mut self,
        e: &Env,
        pool_address: &Address,
        amount: i128,
    ) -> Result<(), LCError> {
        self.adjust_shares(e, pool_address, amount)
    }

    /// Borrows assets on an obligation per pool
    pub fn borrow(&mut self, e: &Env, pool_address: &Address, amount: i128) -> Result<(), LCError> {
        self.adjust_borrowed(e, pool_address, amount)
    }

    /// Adds collateral assets on an obligation per pool
    pub fn add_collateral(
        &mut self,
        e: &Env,
        pool_address: &Address,
        amount: i128,
    ) -> Result<(), LCError> {
        self.adjust_collateral(e, pool_address, amount)
    }

    /// Withdraws assets from an obligation per pool
    pub fn withdraw(
        &mut self,
        e: &Env,
        pool_address: &Address,
        shares_amount: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self
            .deposits
            .get(pool_address.clone())
            .ok_or(LCError::ObligationDoesNotExist)?;

        if deposit_obligation.shares < shares_amount {
            return Err(LCError::WithdrawOverBalance);
        }

        deposit_obligation.adjust_shares(e, -shares_amount)?;

        if deposit_obligation.is_empty() {
            self.deposits.remove(pool_address.clone());
        } else {
            self.deposits.set(pool_address.clone(), deposit_obligation);
        }

        Ok(())
    }

    /// Removes collateral assets from an obligation per pool
    pub fn remove_collateral(
        &mut self,
        e: &Env,
        pool_address: &Address,
        amount: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();

        if deposit_obligation.collateral < amount {
            return Err(LCError::WithdrawOverBalance);
        }

        deposit_obligation.adjust_collateral(e, -amount)?;

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
    pub fn repay(
        &mut self,
        e: &Env,
        pool_address: &Address,
        amount: i128,
    ) -> Result<i128, LCError> {
        let mut borrow_obligation = self
            .borrows
            .get(pool_address.clone())
            .ok_or(LCError::ObligationDoesNotExist)?;

        let total_debt = borrow_obligation.total_debt()?;

        let real_repaid_amount = i128::min(amount, total_debt);

        if real_repaid_amount == total_debt {
            self.borrows.remove(pool_address.clone());
        } else {
            if real_repaid_amount <= borrow_obligation.unpaid_interest {
                borrow_obligation.adjust_unpaid_interest(e, -real_repaid_amount)?;
            } else {
                let to_remove_from_borrowed =
                    real_repaid_amount - borrow_obligation.unpaid_interest;
                borrow_obligation.adjust_borrowed(e, -to_remove_from_borrowed)?;
                borrow_obligation.adjust_unpaid_interest(e, -borrow_obligation.unpaid_interest)?;
            }

            self.borrows.set(pool_address.clone(), borrow_obligation);
        }

        Ok(real_repaid_amount)
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
    ) -> Result<LiquidationValues, LCError> {
        let Some(mut borrow_obligation) = self.borrows.get(borrow_pool_address.clone()) else {
            return Err(LCError::BorrowDoesNotExist);
        };
        let Some(mut collateral_obligation) = self.deposits.get(collateral_pool_address.clone())
        else {
            return Err(LCError::DepositDoesNotExist);
        };

        let PoolConfig {
            liquidation_close_factor_bps,
            liquidation_incentive_bps,
            ..
        } = borrow_pool.config;

        let total_debt = borrow_obligation.total_debt()?;

        // 'liquidatable_bps' == ((amount * 10_000) / total_debt)
        let liquidatable_bps = amount
            .fixed_div_floor(total_debt, BPS_FACTOR)
            .map_over_or_underflow()?;

        if liquidatable_bps > liquidation_close_factor_bps {
            // TODO: What's the best way to set `close_factor_bps` value?
            return Err(LCError::LiquidationExceedsCloseFactor);
        }

        let borrowed_price = get_asset_price(e, &borrow_pool.token_ticker)?;
        let liquidation_value = amount.checked_mul(borrowed_price).map_over_or_underflow()?;

        // Value, which liquidator would like to receive if a full liquidation takes place
        // 'liquidation_value_with_incentive' == (liquidation_value * (10_000 + liquidation_incentive_bps)) / 10_000
        let liquidation_value_with_incentive = liquidation_value
            .fixed_mul_floor(BPS_FACTOR + liquidation_incentive_bps, BPS_FACTOR)
            .map_over_or_underflow()?;

        let collateral_price = get_asset_price(e, &collateral_pool.token_ticker)?;

        let full_collateral_amount = collateral_obligation.collateral;
        let full_collateral_value = full_collateral_amount
            .checked_mul(collateral_price)
            .map_over_or_underflow()?;

        let liquidation_values = if full_collateral_value >= liquidation_value_with_incentive {
            let collateral_amount_sold = liquidation_value_with_incentive
                .checked_div(collateral_price)
                .map_over_or_underflow()?;

            LiquidationValues {
                liquidated_amount: amount,
                collateral_amount_sold,
                shares_amount_sold: 0,
                tokens_from_sold_shares: 0,
            }
        } else {
            let value_left = liquidation_value_with_incentive - full_collateral_value; // safe

            let full_collateral_shares = collateral_obligation.shares;
            let tokens_from_shares =
                collateral_pool.compute_tokens_from_shares(e, full_collateral_shares)?;

            let available_tokens_from_shares =
                i128::min(collateral_pool.available, tokens_from_shares);

            let tokens_from_shares_value = available_tokens_from_shares
                .checked_mul(collateral_price)
                .map_over_or_underflow()?;

            if tokens_from_shares_value >= value_left {
                let tokens_from_sold_shares = value_left
                    .checked_div(collateral_price)
                    .map_over_or_underflow()?;
                let shares_amount_sold =
                    collateral_pool.compute_shares_from_tokens(tokens_from_sold_shares)?;

                LiquidationValues {
                    liquidated_amount: amount,
                    collateral_amount_sold: full_collateral_amount,
                    shares_amount_sold,
                    tokens_from_sold_shares,
                }
            } else {
                // The case when full liquidation cannot take place because of not enough available amount in the pool
                // TODO: Rewrite with using cTokens
                let collateral_value_sum = full_collateral_value
                    .checked_add(tokens_from_shares_value)
                    .map_over_or_underflow()?;

                let tokens_per_collateral = collateral_value_sum
                    .checked_div(collateral_price)
                    .map_over_or_underflow()?;

                let numerator = BPS_FACTOR - liquidation_incentive_bps; // safe
                let denominator = BPS_FACTOR;

                // Liquidator cannot receive the entire desired value of collateral,
                // so only a proportional amount of tokens must be repaid
                let tokens_per_collateral_minus_incentive = tokens_per_collateral
                    .checked_mul(numerator)
                    .map_over_or_underflow()?
                    .checked_div(denominator)
                    .map_over_or_underflow()?;

                LiquidationValues {
                    liquidated_amount: tokens_per_collateral_minus_incentive,
                    collateral_amount_sold: full_collateral_amount,
                    shares_amount_sold: full_collateral_shares,
                    tokens_from_sold_shares: available_tokens_from_shares,
                }
            }
        };

        let unpaid_interest_repaid = i128::min(
            borrow_obligation.unpaid_interest,
            liquidation_values.liquidated_amount,
        );

        let borrow_repaid = liquidation_values.liquidated_amount - unpaid_interest_repaid; // safe
        borrow_obligation.adjust_borrowed(e, -borrow_repaid)?;
        borrow_obligation.adjust_unpaid_interest(e, -unpaid_interest_repaid)?;

        collateral_obligation.adjust_collateral(e, -liquidation_values.collateral_amount_sold)?;
        collateral_obligation.adjust_shares(e, -liquidation_values.shares_amount_sold)?;

        self.borrows
            .set(borrow_pool_address.clone(), borrow_obligation);

        self.deposits
            .set(collateral_pool_address.clone(), collateral_obligation);

        Ok(liquidation_values)
    }

    pub fn get_shares(&self, pool_address: &Address) -> Result<i128, LCError> {
        let Some(deposit_obligation) = self.deposits.get(pool_address.clone()) else {
            return Err(LCError::DepositDoesNotExist);
        };

        Ok(deposit_obligation.shares)
    }

    pub fn get_borrowed(&self, pool_address: &Address) -> Result<i128, LCError> {
        let Some(borrow_obligation) = self.borrows.get(pool_address.clone()) else {
            return Err(LCError::BorrowDoesNotExist);
        };

        Ok(borrow_obligation.borrowed)
    }

    pub fn get_collateral(&self, pool_address: &Address) -> Result<i128, LCError> {
        let Some(deposit_obligation) = self.deposits.get(pool_address.clone()) else {
            return Err(LCError::DepositDoesNotExist);
        };

        Ok(deposit_obligation.collateral)
    }

    /// Saves\updates obligation in the contract's storage
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn set(&self, e: &Env) {
        storage::set_obligation(e, &self.user, self);
    }

    /// Tries to get the user's obligation from the contract's storage
    ///
    /// # Returns
    /// - `[Ok(Obligation)]` if a pool with the given address exists in the contract's storage
    /// - `[Err(LCError::ObligationDoesNotExist)]` otherwise
    pub fn try_get(e: &Env, user: &Address) -> Result<Self, LCError> {
        storage::get_obligation(e, user).ok_or(LCError::ObligationDoesNotExist)
    }

    /// Removes obligation from the contract's storage
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn remove(self, e: &Env) {
        storage::remove_obligation(e, &self.user);
    }

    fn adjust_shares(
        &mut self,
        e: &Env,
        pool_address: &Address,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();
        deposit_obligation.adjust_shares(e, adjusting_amount)?;
        self.deposits.set(pool_address.clone(), deposit_obligation);

        Ok(())
    }

    fn adjust_borrowed(
        &mut self,
        e: &Env,
        pool_address: &Address,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        let mut borrow_obligation = self
            .borrows
            .get(pool_address.clone())
            .unwrap_or(BorrowObligation::new());
        borrow_obligation.adjust_borrowed(e, adjusting_amount)?;
        self.borrows.set(pool_address.clone(), borrow_obligation);

        Ok(())
    }

    fn adjust_collateral(
        &mut self,
        e: &Env,
        pool_address: &Address,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();
        deposit_obligation.adjust_collateral(e, adjusting_amount)?;
        self.deposits.set(pool_address.clone(), deposit_obligation);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
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
    fn new() -> Self {
        Self {
            borrowed: 0,
            unpaid_interest: 0,
            last_accrual: ACCRUAL_INIT,
        }
    }

    #[allow(unused)]
    pub fn is_empty(&self) -> bool {
        self.borrowed == 0
    }

    pub fn total_debt(&self) -> Result<i128, LCError> {
        self.borrowed
            .checked_add(self.unpaid_interest)
            .map_over_or_underflow()
    }

    pub fn adjust_borrowed(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let old_amount = self.borrowed;
        let new_amount = old_amount
            .checked_add(adjusting_amount)
            .map_over_or_underflow()?;

        if new_amount < 0 {
            events::obligation_amount_becomes_negative(e, old_amount, new_amount);

            return Err(LCError::InternalError);
        }

        self.borrowed = new_amount;

        Ok(())
    }

    pub fn adjust_unpaid_interest(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        let old_amount = self.unpaid_interest;
        let new_amount = old_amount
            .checked_add(adjusting_amount)
            .map_over_or_underflow()?;

        if new_amount < 0 {
            events::obligation_amount_becomes_negative(e, old_amount, new_amount);

            return Err(LCError::InternalError);
        }

        self.unpaid_interest = new_amount;

        Ok(())
    }

    /// Accrues interest on a borrow obligation
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn accrue_interest(&mut self, e: &Env, pool_address: &Address) -> Result<(), LCError> {
        let mut pool = storage::get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;
        pool.accrue_interest(e)?;

        let prev_debt = self.borrowed + self.unpaid_interest;
        let new_debt = prev_debt
            .checked_mul(pool.last_accrual)
            .map_over_or_underflow()?
            .checked_div(self.last_accrual)
            .map_over_or_underflow()?;

        let old_unpaid_interest = self.unpaid_interest;
        let new_unpaid_interest = new_debt
            .checked_sub(self.borrowed)
            .map_over_or_underflow()?;

        if new_unpaid_interest < 0 {
            events::obligation_amount_becomes_negative(e, old_unpaid_interest, new_unpaid_interest);

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
    // TODO: Refactor this as we do with [`Pool::adjust_field`]
    pub fn adjust_shares(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let old_amount = self.shares;
        let new_amount = old_amount
            .checked_add(adjusting_amount)
            .map_over_or_underflow()?;

        if new_amount < 0 {
            events::obligation_amount_becomes_negative(e, old_amount, new_amount);

            return Err(LCError::InternalError);
        }

        self.shares = new_amount;

        Ok(())
    }

    pub fn adjust_collateral(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let old_amount = self.collateral;
        let new_amount = old_amount
            .checked_add(adjusting_amount)
            .map_over_or_underflow()?;

        if new_amount < 0 {
            events::obligation_amount_becomes_negative(e, old_amount, new_amount);

            return Err(LCError::InternalError);
        }

        self.collateral = new_amount;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.shares == 0 && self.collateral == 0
    }
}

pub struct LiquidationValues {
    /// The amount of tokens repaid by the liquidator
    pub liquidated_amount: i128,
    /// The number of the borrower's collateral tokens that are taken by the liquidator
    pub collateral_amount_sold: i128,
    /// The number of available pool tokens that are taken from the borrower's shares
    pub shares_amount_sold: i128,
    /// The number of tokens that correspond to the sold shares
    pub tokens_from_sold_shares: i128,
}
