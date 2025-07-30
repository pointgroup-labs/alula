use {
    crate::{
        constants::{
            BPS_IN_PERCENT, DEFAULT_BASE_RATE_PER_SECOND, DEFAULT_CLOSE_FACTOR,
            DEFAULT_LIQUIDATION_SPREAD, DEFAULT_LIQUIDATION_THRESHOLD,
            DEFAULT_OPTIMAL_UTILIZATION_RATIO, DEFAULT_RESERVE_RATIO, DEFAULT_SLOPE1,
            DEFAULT_SLOPE2, DEFAULT_SUPPLY_LIMIT, DEFAULT_UTILIZATION_RATIO_LIMIT,
        },
        events,
        math_utils::MathUtils,
        storage, LCError,
    },
    soroban_fixed_point_math::FixedPoint,
    soroban_sdk::{contracttype, symbol_short, Address, Env, String, Symbol, Vec},
};

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MultiplyPair {
    /// Address of a pool in a pair for a leveraged deposit
    pub deposit_pool: Address,
    /// Address of a pool in a pair for a leveraged borrow
    pub borrow_pool: Address,
}

#[contracttype]
#[derive(Debug, Eq, PartialEq)]
pub struct Pool {
    /// The address of the loan pool
    pub pool_address: Address,
    /// The address of the token associated with the pool
    pub token_address: Address,
    /// The ticker symbol of the associated token, which is used to identify the token in the pool
    pub token_ticker: Symbol,
    /// The total amount of borrowed assets. This value increases with interest rate accrual
    pub total_borrowed: i128,
    /// The total amount of deposited assets that accrue interest
    pub total_shares: i128,
    /// The currently available for borrowing tokens
    pub available: i128,
    /// The total amount of deposited collateral assets that don't accrue interest
    pub total_collateral: i128,
    /// Configuration settings for the pool
    pub config: PoolConfig,
    /// The numerical value that is used to determine the scaling factor required for updating the borrowed amount
    /// with interest, i.e. new_borrowed = (current_accrual \ last_accrual) * borrowed
    pub last_accrual: i128,
    /// The timestamp of the last accrual re-calculation
    pub last_accrual_timestamp: u64,
    /// The result of `TokenClient::name(&self)` invocation: `native` string for XLM SAC and the SAC's native asset code
    /// and asset issuer concatenated with `:` for other SACs(e.g, "AQUA:GAHPYWLK6YRN7CVYZOO4H3VDRZ7PVF5UJGLZCSPAEIKJE2XSWF5LAGER")
    pub name: String,
}

impl Pool {
    fn adjust_field(e: &Env, current_value: i128, adjusting_amount: i128) -> Result<i128, LCError> {
        let new_amount = current_value
            .checked_add(adjusting_amount)
            .map_over_or_underflow()?;

        if new_amount < 0 {
            // TODO: Just an ad-hoc fix. When switching to bTokens this issue should likely be gone
            events::pool_amount_becomes_negative(e, current_value, new_amount);

            return Ok(0);
        }

        Ok(new_amount)
    }

    pub fn adjust_total_shares(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        events::dbg(e, symbol_short!("shares"));
        self.total_shares = Self::adjust_field(e, self.total_shares, adjusting_amount)?;
        Ok(())
    }

    pub fn adjust_available(&mut self, e: &Env, adjusting_amount: i128) -> Result<(), LCError> {
        events::dbg(e, symbol_short!("avail"));
        self.available = Self::adjust_field(e, self.available, adjusting_amount)?;
        Ok(())
    }

    pub fn adjust_total_borrowed(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        events::dbg(e, symbol_short!("borrowed"));
        self.total_borrowed = Self::adjust_field(e, self.total_borrowed, adjusting_amount)?;
        Ok(())
    }

    pub fn adjust_total_collateral(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), LCError> {
        events::dbg(e, symbol_short!("collat"));
        self.total_collateral = Self::adjust_field(e, self.total_collateral, adjusting_amount)?;
        Ok(())
    }

    /// Computes tokens amount proportional to the share of the of `shares` in the pool
    pub fn compute_tokens_from_shares(
        &self,
        e: &Env,
        shares_amount: i128,
    ) -> Result<i128, LCError> {
        if shares_amount == 0 {
            return Ok(0);
        }

        if self.total_shares < shares_amount {
            events::pool_total_shares_smaller_than_individual_user_shares(
                e,
                self.total_shares,
                shares_amount,
            );

            return Err(LCError::InternalError);
        }

        let total_supply = self.total_supply()?;
        let tokens_amount = total_supply
            .checked_mul(shares_amount)
            .map_over_or_underflow()?
            .checked_div(self.total_shares)
            .map_over_or_underflow()?;

        Ok(tokens_amount)
    }

    /// Computes shares amount which must be issued for\burnt from a depositor based on the deposited\withdrawn amount
    pub fn compute_shares_from_tokens(&self, tokens_amount: i128) -> Result<i128, LCError> {
        if tokens_amount == 0 {
            return Ok(0);
        }

        let shares_amount = if self.total_shares == 0 {
            tokens_amount
        } else {
            let total = self
                .available
                .checked_add(self.total_borrowed)
                .map_over_or_underflow()?;

            // assert!(
            //     total >= self.total_shares,
            //     "Total shares amount must never be smaller than the total liquidity amount"
            // );
            if total < self.total_shares {
                return Err(LCError::InternalError);
            }

            /*
            This must hold when issuing new shares:
                shares_to_issue / (shares_to_issue + prev_total_shares) = deposited_amount / (deposited_amount + prev_total_borrowed + prev_available)
            Which implies:
                shares_to_issue = prev_total_shares * (deposited_amount / (prev_total_borrowed + prev_available))

            This must hold when burning issued shares:
                shares_to_burn = prev_total_shares * (withdrawn_amount / (prev_total_borrowed + prev_available))
            */
            self.total_shares
                .fixed_div_ceil(total, tokens_amount)
                .map_over_or_underflow()?
        };

        Ok(shares_amount)
    }

    /// Calculates total supply (available + total_borrowed)
    pub fn total_supply(&self) -> Result<i128, LCError> {
        self.available
            .checked_add(self.total_borrowed)
            .map_over_or_underflow()
    }

    /// Checks if the pool is empty
    pub fn is_empty(&self) -> bool {
        self.total_shares == 0
            && self.total_borrowed == 0
            && self.available == 0
            && self.total_collateral == 0
    }

    /// Tries to get the pool from the contract's storage
    ///
    /// # Returns
    /// - `[Ok(Pool)]` if a pool with the given address exists in the contract's storage
    /// - `[Err(LCError::PoolDoesNotExist)]` otherwise
    pub fn try_get(e: &Env, pool_address: &Address) -> Result<Self, LCError> {
        storage::get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)
    }

    pub fn get_all(e: &Env) -> Vec<PoolAddress> {
        storage::get_all_pools(e)
    }

    pub fn get_all_multiply_pairs(e: &Env) -> Vec<MultiplyPair> {
        storage::get_all_multiply_pairs(e)
    }

    pub fn register_multiply_pair(e: &Env, pair: MultiplyPair) -> u32 {
        storage::register_multiply_pair(e, pair)
    }

    pub fn exists(e: &Env, address: &PoolAddress) -> bool {
        storage::pool_exists(e, address)
    }

    /// Refreshes the pool with the contract's storage data
    pub fn refresh(&mut self, e: &Env) -> Result<(), LCError> {
        let Some(refreshed_pool) = storage::get_pool(e, &self.pool_address) else {
            events::pool_is_missing_in_storage(e, &self.pool_address);

            return Err(LCError::InternalError);
        };

        *self = refreshed_pool;

        Ok(())
    }

    /// Saves\updates pool in the contract's storage
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn set(&self, e: &Env) {
        storage::set_pool(e, &self.pool_address, self);
    }

    /// Registers pool in the pool's list
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn register(&self, e: &Env) -> u32 {
        storage::register_pool(e, &self.pool_address.clone())
    }
}

#[contracttype]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PoolConfig {
    /// Base interest rate applied regardless of utilization, expressed per second
    /// in 1/`SCALED_ONE` units. Must be positive
    pub base_rate_per_second: i128,
    /// Positive Optimal Utilization Ratio
    pub optimal_utilization_ratio_bps: i128,
    /// Interest rate slope before reaching optimal utilization ratio
    /// Controls how aggressively rates increase with utilization below the optimal point
    pub slope1: i128,
    /// Interest rate slope after exceeding optimal utilization ratio
    /// Controls how aggressively rates increase with utilization above the optimal point
    pub slope2: i128,
    /// Percentage of interest payments allocated to protocol reserves
    pub reserve_ratio_bps: i128,
    /// Maximum percentage of a borrower's debt that can be liquidated
    pub liquidation_close_factor_bps: i128,
    /// Additional discount given to liquidators when purchasing collateral
    pub liquidation_incentive_bps: i128,
    /// The maximum amount of supplied tokens that can be supplied in the pool(i.e., `available` + `total_borrowed`)
    /// 0 denotes unlimited supply
    pub supply_limit: i128,
    /// The maximum utilization ratio that is allowed to be reached via borrowing
    pub utilization_ratio_limit_bps: i128,
    /// The maximum percentage of an asset's value that can be borrowed in basis points(e.g, 7000 = 70%, etc)
    /// with respect to a total obligation's collateral value
    pub open_ltv_bps: i128,
    /// The maximum percentage of an asset's value that can be held in an individual obligation in basis points
    /// with respect to a total obligation's collateral value. LTV greater than that makes borrow position eligible to liquidation
    pub close_ltv_bps: i128,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            slope1: DEFAULT_SLOPE1,
            slope2: DEFAULT_SLOPE2,
            base_rate_per_second: DEFAULT_BASE_RATE_PER_SECOND,
            reserve_ratio_bps: DEFAULT_RESERVE_RATIO * BPS_IN_PERCENT,
            optimal_utilization_ratio_bps: DEFAULT_OPTIMAL_UTILIZATION_RATIO * BPS_IN_PERCENT,
            liquidation_close_factor_bps: DEFAULT_CLOSE_FACTOR * BPS_IN_PERCENT,
            liquidation_incentive_bps: DEFAULT_LIQUIDATION_SPREAD * BPS_IN_PERCENT,
            supply_limit: DEFAULT_SUPPLY_LIMIT,
            utilization_ratio_limit_bps: DEFAULT_UTILIZATION_RATIO_LIMIT * BPS_IN_PERCENT,
            open_ltv_bps: DEFAULT_LIQUIDATION_THRESHOLD * BPS_IN_PERCENT,
            close_ltv_bps: DEFAULT_LIQUIDATION_THRESHOLD * BPS_IN_PERCENT,
        }
    }
}

impl PoolConfig {
    pub fn validate(&self) -> Result<(), &str> {
        let &PoolConfig {
            base_rate_per_second,
            optimal_utilization_ratio_bps,
            slope1,
            slope2,
            reserve_ratio_bps,
            liquidation_close_factor_bps,
            liquidation_incentive_bps,
            supply_limit,
            utilization_ratio_limit_bps,
            open_ltv_bps,
            close_ltv_bps,
        } = self;

        if base_rate_per_second < 0 {
            return Err("Base rate per second must be non-negative");
        }

        if optimal_utilization_ratio_bps <= 0
            || optimal_utilization_ratio_bps > 100 * BPS_IN_PERCENT
        {
            return Err("Optimal utilization ratio must be between 0% and 100%");
        }

        if supply_limit < 0 {
            return Err("Supply limit must be non-negative");
        }

        if !is_valid_percent(utilization_ratio_limit_bps) {
            return Err("Utilization ratio limit must be between 0% and 100%");
        }

        if utilization_ratio_limit_bps <= optimal_utilization_ratio_bps {
            return Err("Utilization ratio limit must exceed optimal utilization ratio");
        }

        if !is_valid_percent(reserve_ratio_bps) {
            return Err("Reserve ratio must be between 0% and 100%");
        }

        if !is_valid_percent(liquidation_close_factor_bps) {
            return Err("Liquidation close factor must be between 0% and 100%");
        }

        if !is_valid_percent(liquidation_incentive_bps) {
            return Err("Liquidation incentive must be between 0% and 100%");
        }

        if slope1 < 0 || slope2 < 0 {
            return Err("Interest rate slopes must be non-negative");
        }

        if slope1 >= slope2 {
            return Err("slope1 must be less than slope2 for kinked model to work");
        }

        if !is_valid_percent(open_ltv_bps) {
            return Err("Open LTV must be between 0% and 100%");
        }

        if !is_valid_percent(close_ltv_bps) {
            return Err("Close LTV must be between 0% and 100%");
        }

        if close_ltv_bps < open_ltv_bps {
            return Err("Open LTV mustn't be bigger than close LTV");
        }

        Ok(())
    }
}

fn is_valid_percent(value: i128) -> bool {
    (0..=100 * BPS_IN_PERCENT).contains(&value)
}

#[contracttype]
#[derive(Debug)]
pub struct Accrual {
    pub timestamp: u64,
    pub borrow_accrual: i128,
    pub deposit_accrual: i128,
}
