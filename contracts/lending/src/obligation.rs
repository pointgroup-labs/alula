use core::borrow;

use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, Map, Vec, contracttype};

use crate::{
    LCError,
    constants::BPS_FACTOR,
    contract::get_asset_price,
    events,
    math_utils::MathUtils,
    pool::{Pool, PoolConfig},
    storage,
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
    /// Creates a new obligation for the user
    ///
    /// # WARNING
    /// Modifies the obligation's pools storage data by appending the user's address to the
    /// obligation's list
    pub fn new(e: &Env, user: Address) -> Self {
        storage::register_obligation(e, &user);

        Self {
            user,
            deposits: Map::new(e),
            borrows: Map::new(e),
        }
    }

    /// Accrues interest on all obligation-related pools
    ///
    /// # WARNING
    /// Modifies the obligation's pools storage data
    pub fn accrue_interest(&self, e: &Env) -> Result<(), LCError> {
        for borrow_pool_address in self.borrows.keys() {
            accrue_interest_on_pool(e, &borrow_pool_address)?;
        }

        for deposit_pool_address in self.deposits.keys() {
            accrue_interest_on_pool(e, &deposit_pool_address)?;
        }

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        self.deposits.is_empty() && self.borrows.is_empty()
    }

    /// Computes the current borrowed assets summed value per obligation (each asset value is scaled
    /// with the corresponding `liability factor` value)
    fn compute_borrowed_value_scaled_w_liability_factors(&self, e: &Env) -> Result<i128, LCError> {
        let mut borrowed_value_sum = 0_i128;

        for (borrow_pool_address, borrow_obligation) in self.borrows.iter() {
            let BorrowObligation { d_tokens, .. } = borrow_obligation;

            let borrow_pool = storage::get_pool(e, &borrow_pool_address).ok_or_else(|| {
                events::pool_is_missing_in_storage(e, &borrow_pool_address);

                LCError::InternalError
            })?;

            let borrowed_tokens = borrow_pool.compute_tokens_from_d_tokens(e, d_tokens)?;
            let borrowed_asset_price = get_asset_price(e, &borrow_pool.token_ticker)?;

            let new_value_term = borrowed_asset_price
                .checked_mul(borrowed_tokens)
                .map_over_or_underflow()?;
            let new_value_term_scaled_w_liability_factor = new_value_term
                .fixed_mul_ceil(borrow_pool.config.liability_factor_bps, BPS_FACTOR)
                .map_over_or_underflow()?;

            borrowed_value_sum = borrowed_value_sum
                .checked_add(new_value_term_scaled_w_liability_factor)
                .map_over_or_underflow()?;
        }

        Ok(borrowed_value_sum)
    }

    /// Computes the current collateral assets summed value(deposit shares + plain collateral) per
    /// obligation
    ///
    /// # Important
    /// This function is agnostic regardless of a borrow pool, hence, the resulting value isn't
    /// scaled with any `open LTV` percent
    fn compute_collateral_value(&self, e: &Env) -> Result<i128, LCError> {
        let mut collateral_value_sum = 0_i128;

        for (collateral_pool_address, deposit_obligation) in self.deposits.iter() {
            let DepositObligation {
                j_tokens,
                collateral,
                ..
            } = deposit_obligation;

            let collateral_pool =
                storage::get_pool(e, &collateral_pool_address).ok_or_else(|| {
                    events::pool_is_missing_in_storage(e, &collateral_pool_address);

                    LCError::InternalError
                })?;

            let deposited_tokens = collateral_pool.compute_tokens_from_j_tokens(e, j_tokens)?;
            let total_collateral_tokens = deposited_tokens
                .checked_add(collateral)
                .map_over_or_underflow()?;
            let collateral_asset_price = get_asset_price(e, &collateral_pool.token_ticker)?;

            let new_value_term = collateral_asset_price
                .checked_mul(total_collateral_tokens)
                .map_over_or_underflow()?;

            collateral_value_sum = collateral_value_sum
                .checked_add(new_value_term)
                .map_over_or_underflow()?;
        }

        Ok(collateral_value_sum)
    }

    /// Computes the max healthy amount of the collateral token (that is used as a deposit or as a
    /// collateral) that can be removed so that the obligation's LTV is equal to the `open LTV`
    /// parameter on the pool
    pub fn compute_max_healthy_collateral_removed_amount(
        &self,
        e: &Env,
        pool_address: &Address,
    ) -> Result<i128, LCError> {
        let pool = storage::get_pool(e, pool_address).ok_or_else(|| {
            events::pool_is_missing_in_storage(e, pool_address);

            LCError::PoolDoesNotExist
        })?;

        let collateral_value = self.compute_collateral_value(e)?;
        let collateral_value_scaled = collateral_value
            .fixed_mul_floor(pool.config.open_ltv_bps, BPS_FACTOR)
            .map_over_or_underflow()?;
        let borrowed_value_scaled = self.compute_borrowed_value_scaled_w_liability_factors(e)?;

        let max_healthy_removed_amount = if collateral_value <= borrowed_value_scaled {
            // Since the scaled borrowed assets value exceeds the scaled collateral assets value,
            // the collateral removal is prohibited
            0
        } else {
            let collateral_asset_price = get_asset_price(e, &pool.token_ticker)?;
            let value_left = collateral_value_scaled - borrowed_value_scaled; // safe

            // Contrary to [`compute_max_healthy_collateral_removed_amount`], there's no need
            // to scale down the value left
            value_left
                .checked_div(collateral_asset_price)
                .map_over_or_underflow()?
        };

        Ok(max_healthy_removed_amount)
    }

    /// Computes the max healthy amount of the token that can be borrowed and that
    /// doesn't exceed the `open LTV` parameter on the pool
    pub fn compute_max_healthy_borrow_added_amount(
        &self,
        e: &Env,
        pool_address: &Address,
    ) -> Result<i128, LCError> {
        let pool = storage::get_pool(e, pool_address).ok_or_else(|| {
            events::pool_is_missing_in_storage(e, pool_address);

            LCError::PoolDoesNotExist
        })?;

        let collateral_value = self.compute_collateral_value(e)?;
        let collateral_value_scaled = collateral_value
            .fixed_mul_floor(pool.config.open_ltv_bps, BPS_FACTOR)
            .map_over_or_underflow()?;
        let borrowed_value_scaled = self.compute_borrowed_value_scaled_w_liability_factors(e)?;

        let max_healthy_added_amount = if borrowed_value_scaled >= collateral_value_scaled {
            // Since the scaled borrowed assets value exceeds the scaled collateral assets value,
            // the borrow is prohibited
            0
        } else {
            let borrow_asset_price = get_asset_price(e, &pool.token_ticker)?;
            let value_left = collateral_value_scaled - borrowed_value_scaled; // safe

            let tokens_per_value_left = value_left
                .checked_div(borrow_asset_price)
                .map_over_or_underflow()?;

            // Since the liability factor can exceed 100%, the real amount of a token
            // that can be borrowed is divided by it
            tokens_per_value_left
                .fixed_div_floor(pool.config.liability_factor_bps, BPS_FACTOR)
                .map_over_or_underflow()?
        };

        Ok(max_healthy_added_amount)
    }

    pub fn get_all(e: &Env) -> Vec<Address> {
        storage::get_all_obligations(e)
    }

    /// Deposits assets on an obligation per pool
    pub fn deposit(
        &mut self,
        e: &Env,
        pool_address: &Address,
        j_tokens_issued: i128,
        deposited_tokens: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();

        deposit_obligation.adjust_j_tokens(e, j_tokens_issued)?;
        deposit_obligation.adjust_deposited(e, deposited_tokens)?;

        self.deposits.set(pool_address.clone(), deposit_obligation);

        Ok(())
    }

    /// Borrows assets on an obligation per pool
    pub fn borrow(
        &mut self,
        e: &Env,
        pool_address: &Address,
        d_tokens_issued: i128,
        borrowed_tokens: i128,
    ) -> Result<(), LCError> {
        let mut borrow_obligation = self.borrows.get(pool_address.clone()).unwrap_or_default();

        borrow_obligation.adjust_d_tokens(e, d_tokens_issued)?;
        borrow_obligation.adjust_borrowed(e, borrowed_tokens)?;

        self.borrows.set(pool_address.clone(), borrow_obligation);

        Ok(())
    }

    /// Adds collateral assets on an obligation per pool
    pub fn add_collateral(
        &mut self,
        e: &Env,
        pool_address: &Address,
        collateral_tokens: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();

        deposit_obligation.adjust_collateral(e, collateral_tokens)?;

        self.deposits.set(pool_address.clone(), deposit_obligation);

        Ok(())
    }

    /// Withdraws assets from an obligation per pool
    pub fn withdraw(
        &mut self,
        e: &Env,
        pool_address: &Address,
        j_tokens_removed: i128,
        deposited_tokens_removed: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self
            .deposits
            .get(pool_address.clone())
            .ok_or(LCError::ObligationDoesNotExist)?;

        if deposit_obligation.j_tokens < j_tokens_removed {
            return Err(LCError::WithdrawOverBalance);
        }

        deposit_obligation.adjust_j_tokens(e, -j_tokens_removed)?;
        deposit_obligation.adjust_deposited(e, -deposited_tokens_removed)?;

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
        collateral: i128,
    ) -> Result<(), LCError> {
        let mut deposit_obligation = self.deposits.get(pool_address.clone()).unwrap_or_default();

        if deposit_obligation.collateral < collateral {
            return Err(LCError::CollateralRemovalOverbalance);
        }

        deposit_obligation.adjust_collateral(e, -collateral)?;

        if deposit_obligation.is_empty() {
            self.deposits.remove(pool_address.clone());
        } else {
            self.deposits.set(pool_address.clone(), deposit_obligation);
        }

        Ok(())
    }

    /// Repays the debt on a specific obligation per pool. Since `repaid_amount` can exceed the debt
    /// - the real repaid amount is calculated as `min(debt, repaid_amount)`
    ///
    /// # Returns
    /// [`Result::Ok((d_tokens_burnt, real_repaid_amount))`] in success and [`Err(LCError)`] in failure
    pub fn repay(&mut self, e: &Env, pool: &Pool, amount: i128) -> Result<(i128, i128), LCError> {
        let mut borrow_obligation = self
            .borrows
            .get(pool.pool_address.clone())
            .ok_or(LCError::ObligationDoesNotExist)?;

        let total_debt_tokens = pool.compute_tokens_from_d_tokens(e, borrow_obligation.d_tokens)?;
        let initially_borrowed = borrow_obligation.borrowed;

        if total_debt_tokens < initially_borrowed {
            // TODO: Add an event

            return Err(LCError::InternalError);
        }

        let real_repaid_amount = i128::min(total_debt_tokens, amount);

        let d_tokens_burnt = if real_repaid_amount == total_debt_tokens {
            self.borrows.remove(pool.pool_address.clone());

            borrow_obligation.d_tokens
        } else {
            let d_tokens_burnt = pool.compute_d_tokens_from_tokens(e, real_repaid_amount)?;
            borrow_obligation.adjust_d_tokens(e, -d_tokens_burnt);

            let unpaid_interest = total_debt_tokens - initially_borrowed; // safe
            if unpaid_interest >= real_repaid_amount {
                let diff = unpaid_interest - real_repaid_amount; // safe
                borrow_obligation.adjust_borrowed(e, -diff);
            }

            self.borrows
                .set(pool.pool_address.clone(), borrow_obligation);

            d_tokens_burnt
        };

        Ok((d_tokens_burnt, real_repaid_amount))
    }

    /// Liquidates unhealthy borrow
    #[allow(unused)]
    pub fn liquidate(
        &mut self,
        e: &Env,
        borrow_pool_address: &Address,
        collateral_pool_address: &Address,
        borrow_pool: &Pool,
        collateral_pool: &Pool,
        d_tokens_burnt: i128,
    ) -> Result<LiquidationValues, LCError> {
        // TODO: Refactor when markets are a thing

        todo!()
    }

    pub fn get_j_tokens(&self, pool_address: &Address) -> Result<i128, LCError> {
        let deposit_obligation = self
            .deposits
            .get(pool_address.clone())
            .ok_or(LCError::DepositDoesNotExist)?;

        Ok(deposit_obligation.j_tokens)
    }

    pub fn get_unpaid_interest(&self, e: &Env, pool_address: &Address) -> Result<i128, LCError> {
        let borrow_obligation = self
            .borrows
            .get(pool_address.clone())
            .ok_or(LCError::DepositDoesNotExist)?;
        let borrow_pool = Pool::try_get(&e, pool_address)?;

        let total_debt =
            borrow_pool.compute_tokens_from_d_tokens(&e, borrow_obligation.d_tokens)?;
        let borrowed = borrow_obligation.borrowed;

        if total_debt < borrowed {
            // TODO: Add an event?

            return Err(LCError::InternalError);
        }
        let unpaid_interest = total_debt - borrowed; // safe

        Ok(unpaid_interest)
    }

    pub fn get_borrowed(&self, pool_address: &Address) -> Result<i128, LCError> {
        let Some(borrow_obligation) = self.borrows.get(pool_address.clone()) else {
            return Err(LCError::BorrowDoesNotExist);
        };

        Ok(borrow_obligation.borrowed)
    }

    pub fn get_total_debt(&self, e: &Env, pool_address: &Address) -> Result<i128, LCError> {
        let borrow_obligation = self
            .borrows
            .get(pool_address.clone())
            .ok_or_else(|| LCError::BorrowDoesNotExist)?;
        let borrow_pool = Pool::try_get(e, pool_address).map_err(|_| {
            // TODO: Add an event?

            LCError::InternalError
        })?;

        let total_debt = borrow_pool.compute_tokens_from_d_tokens(e, borrow_obligation.d_tokens)?;

        Ok(total_debt)
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
    /// - [`Ok(Obligation)`] if a pool with the given address exists in the contract's storage
    /// - [`Err(LCError::ObligationDoesNotExist)`] otherwise
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
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
#[contracttype]
pub struct BorrowObligation {
    /// TODO: add a comment ....
    pub d_tokens: i128,
    /// Accumulated value of initially borrowed tokens
    pub borrowed: i128,
}

impl BorrowObligation {
    fn new() -> Self {
        Self {
            d_tokens: 0,
            borrowed: 0,
        }
    }

    pub fn adjust_d_tokens(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = adjust_obligation_field(e, self.d_tokens, adjusting_amount)?;
        self.d_tokens = new_amount;

        Ok(())
    }

    pub fn adjust_borrowed(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = adjust_obligation_field(e, self.borrowed, adjusting_amount)?;
        self.borrowed = new_amount;

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
#[contracttype]
pub struct DepositObligation {
    /// TODO: Comment
    pub j_tokens: i128,
    /// Accumulated value of collateral that doesn't accrue interest
    pub collateral: i128,
    /// Accumulated value of initially deposited tokens. E.g., if a user initially deposited 100
    /// tokens, the time passed, which caused 2 tokens to be accrued, and the user deposited 20
    /// more tokens - this value will be equal to 120
    pub deposited: i128,
}

impl DepositObligation {
    pub fn new() -> Self {
        Self {
            collateral: 0,
            j_tokens: 0,
            deposited: 0,
        }
    }

    pub fn adjust_j_tokens(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = adjust_obligation_field(e, self.j_tokens, adjusting_amount)?;
        self.j_tokens = new_amount;

        Ok(())
    }

    pub fn adjust_deposited(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = adjust_obligation_field(e, self.deposited, adjusting_amount)?;
        self.deposited = new_amount;

        Ok(())
    }

    pub fn adjust_collateral(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = adjust_obligation_field(e, self.collateral, adjusting_amount)?;
        self.collateral = new_amount;

        Ok(())
    }

    pub fn is_empty(&self) -> bool {
        if self.j_tokens == 0 {
            if self.deposited != 0 {
                // TODO: Invariant breakage
            }
        }

        self.j_tokens == 0 && self.collateral == 0
    }
}

/// Adjusts a field on the obligation's structs
///
/// # Returns
/// `Ok(new_amount)` if adjusting doesn't lead to a new amount being negative.
/// `Err(LCError::InternalError otherwise
fn adjust_obligation_field(
    e: &Env,
    current_value: i128,
    adjusting_amount: i128,
) -> Result<i128, LCError> {
    let new_amount = current_value
        .checked_add(adjusting_amount)
        .map_over_or_underflow()?;

    if new_amount < 0 {
        events::obligation_amount_becomes_negative(e, current_value, new_amount);

        return Err(LCError::InternalError);
    }

    Ok(new_amount)
}

/// Accrues interest on a pool
///
/// # WARNING
/// Modifies the contract's storage
fn accrue_interest_on_pool(e: &Env, pool_address: &Address) -> Result<(), LCError> {
    let mut pool = storage::get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    pool.accrue_interest(e)?;
    pool.set(e);

    Ok(())
}

// TODO: Move this somewhere else when working on liquidation
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

#[cfg(test)]
mod tests {
    use soroban_sdk::{Address, Env, Map, testutils::Address as _};

    use super::*;

    fn create_test_env() -> Env {
        let env = Env::default();
        env.mock_all_auths();
        env
    }

    fn create_test_obligation(env: &Env, user: Address) -> Obligation {
        let deposits = Map::new(env);
        let borrows = Map::new(env);

        Obligation {
            user,
            deposits,
            borrows,
        }
    }

    #[test]
    fn test_obligation_is_empty() {
        let env = create_test_env();
        let user = Address::generate(&env);
        let obligation = create_test_obligation(&env, user);

        assert!(obligation.is_empty());
    }

    // #[test]
    // fn test_deposit_obligation_is_empty() {
    //     let deposit_obligation = DepositObligation {
    //         shares: 0,
    //         collateral: 0,
    //     };

    //     assert!(deposit_obligation.is_empty());

    //     let non_empty_deposit = DepositObligation {
    //         shares: 1,
    //         collateral: 0,
    //     };

    //     assert!(!non_empty_deposit.is_empty());
    // }

    // #[test]
    // fn test_borrow_obligation_successful_adjustments() {
    //     let env = create_test_env();
    //     let mut borrow_obligation = BorrowObligation {
    //         borrowed: 100,
    //         unpaid_interest: 50,
    //         last_accrual: 1_000_000,
    //     };

    //     // Test successful borrowed adjustment
    //     let result = borrow_obligation.adjust_borrowed(&env, 25);
    //     assert!(result.is_ok());
    //     assert_eq!(borrow_obligation.borrowed, 125);

    //     // Test successful unpaid interest adjustment
    //     let result = borrow_obligation.adjust_unpaid_interest(&env, 10);
    //     assert!(result.is_ok());
    //     assert_eq!(borrow_obligation.unpaid_interest, 60);
    // }

    // #[test]
    // fn test_deposit_obligation_successful_adjustments() {
    //     let env = create_test_env();
    //     let mut deposit_obligation = DepositObligation {
    //         shares: 100,
    //         collateral: 50,
    //     };

    //     // Test successful shares adjustment
    //     let result = deposit_obligation.adjust_shares(&env, 25);
    //     assert!(result.is_ok());
    //     assert_eq!(deposit_obligation.shares, 125);

    //     // Test successful collateral adjustment
    //     let result = deposit_obligation.adjust_collateral(&env, 10);
    //     assert!(result.is_ok());
    //     assert_eq!(deposit_obligation.collateral, 60);
    // }
}
