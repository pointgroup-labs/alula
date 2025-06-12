use {
    crate::constants::{
        LCError, BPS_IN_PERCENT, DEFAULT_BASE_RATE_PER_SECOND, DEFAULT_CLOSE_FACTOR,
        DEFAULT_LIQUIDATION_SPREAD, DEFAULT_OPTIMAL_UTILIZATION_RATIO, DEFAULT_RESERVE_RATIO,
        DEFAULT_SLOPE1, DEFAULT_SLOPE2,
    },
    soroban_sdk::{contracttype, Address, Symbol},
};

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
#[derive(Debug)]
pub struct Pool {
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
    /// The numerical value that is used to determine the scaling factor required for updating the position amount
    /// with interest, i.e. new_borrowed = (current_accrual \ last_accrual) * borrowed
    pub last_accrual: i128,
    /// The timestamp of the last accrual re-calculation
    pub last_accrual_timestamp: u64,
}

impl Pool {
    pub fn adjust_total_shares(&mut self, adjusting_amount: i128) -> Result<(), LCError> {
        let new_total_shares = self
            .total_shares
            .checked_add(adjusting_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_total_shares < 0 {
            // TODO: log/event
            return Err(LCError::InternalError);
        }

        self.total_shares = new_total_shares;

        Ok(())
    }

    pub fn adjust_available(&mut self, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = self
            .total_shares
            .checked_add(adjusting_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_amount < 0 {
            // TODO: log/event
            return Err(LCError::InternalError);
        }

        self.available = new_amount;

        Ok(())
    }

    pub fn adjust_total_borrowed(&mut self, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = self
            .total_borrowed
            .checked_add(adjusting_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_amount < 0 {
            // TODO: log/event
            return Err(LCError::InternalError);
        }

        self.total_borrowed = new_amount;

        Ok(())
    }

    pub fn adjust_total_collateral(&mut self, adjusting_amount: i128) -> Result<(), LCError> {
        let new_amount = self
            .total_collateral
            .checked_add(adjusting_amount)
            .ok_or(LCError::OverOrUnderflow)?;

        if new_amount < 0 {
            // TODO: log/event
            return Err(LCError::InternalError);
        }

        self.total_collateral = new_amount;

        Ok(())
    }

    /// Computes shares amount which must be issued for\burnt from a depositor based on the deposited\withdrawn amount
    pub fn compute_shares_amount(&self, amount: i128) -> Result<i128, LCError> {
        let shares_amount = if self.total_shares == 0 {
            amount
        } else {
            assert!(
                (self.available + self.total_borrowed) >= self.total_shares,
                "Total shares amount must never be smaller than the total liquidity amount"
            );
            /*
            This must hold:
                issued_shares / (issued_shares + prev_total_shares) = deposited_amount / (deposited_amount + prev_total_borrowed + prev_available)
            Which implies:
                issued_shares = prev_total_shares * (deposited_amount / (prev_total_borrowed + prev_available))
            */
            amount
                .checked_mul(self.total_shares)
                .ok_or(LCError::OverOrUnderflow)?
                .checked_div(self.total_borrowed + self.available)
                .ok_or(LCError::OverOrUnderflow)?
        };

        Ok(shares_amount)
    }
}

#[contracttype]
#[derive(Debug, Clone, Copy)]
pub struct PoolConfig {
    /// Base interest rate applied regardless of utilization, expressed per second
    /// in 1/`SCALED_ONE` units. Must be positive.
    pub base_rate_per_second: i128,
    /// Positive Optimal Utilization Ratio
    pub optimal_utilization_ratio_bps: i128,
    /// Interest rate slope before reaching optimal utilization ratio.
    /// Controls how aggressively rates increase with utilization below the optimal point.
    pub slope1: i128,
    /// Interest rate slope after exceeding optimal utilization ratio.
    /// Controls how aggressively rates increase with utilization above the optimal point.
    pub slope2: i128,
    /// Percentage of interest payments allocated to protocol reserves.
    pub reserve_ratio_bps: i128,
    /// Maximum percentage of a borrower's debt that can be liquidated.
    pub liquidation_close_factor_bps: i128,
    /// Additional discount given to liquidators when purchasing collateral.
    pub liquidation_incentive_bps: i128,
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
        }
    }
}

impl PoolConfig {
    pub fn validate(&self) -> Result<(), &str> {
        let &PoolConfig {
            optimal_utilization_ratio_bps,
            slope1,
            slope2,
            reserve_ratio_bps,
            liquidation_close_factor_bps,
            liquidation_incentive_bps,
            ..
        } = self;

        if optimal_utilization_ratio_bps <= 0 {
            return Err("Optimal utilization ratio must be greater than 0%");
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

        if slope1 >= slope2 {
            return Err("slope1 must be less than slope2 for kinked model to work");
        }

        Ok(())
    }
}

fn is_valid_percent(value: i128) -> bool {
    (0..100 * BPS_IN_PERCENT).contains(&value)
}

#[contracttype]
#[derive(Debug)]
pub struct Accrual {
    pub timestamp: u64,
    pub borrow_accrual: i128,
    pub deposit_accrual: i128,
}
