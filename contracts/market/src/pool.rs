use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, String, Symbol, Vec, contracttype};

use crate::{
    accrual::AccrualModel,
    constants::{
        BPS_FACTOR, BPS_IN_PERCENT, DEFAULT_ADD_COLLATERAL_FEE_BPS, DEFAULT_BORROW_FEE_BPS,
        DEFAULT_CLOSE_FACTOR, DEFAULT_CLOSE_LTV, DEFAULT_DEPOSIT_FEE_BPS,
        DEFAULT_FLASH_LOAN_FEE_BPS, DEFAULT_HOST_FEE_BPS, DEFAULT_LIABILITY_FACTOR,
        DEFAULT_LIQUIDATION_SPREAD, DEFAULT_OPEN_LTV, DEFAULT_REMOVE_COLLATERAL_FEE_BPS,
        DEFAULT_REPAY_FEE_BPS, DEFAULT_RESERVE_RATIO, DEFAULT_SUPPLY_LIMIT, DEFAULT_TAKE_RATE_BPS,
        DEFAULT_UTILIZATION_RATIO_LIMIT, DEFAULT_WITHDRAW_FEE_BPS, MAX_LIABILITY_FACTOR,
    },
    error::MCError,
    events,
    interest_rate_model::InterestRateModel,
    math_utils::MathUtils,
    storage,
};

#[contracttype]
#[derive(Debug, Eq, PartialEq)]
// TODO: Refactor?
pub struct Pool {
    /// The address of the loan pool
    pub pool_address: Address,
    /// The address of the token contract associated with the pool
    pub token_address: Address,
    /// The ticker symbol of the associated token
    pub token_ticker: Symbol,
    /// The total amount of borrowed assets. This value increases with interest rate accrual
    pub total_borrowed: i128,
    /// The total `dTokens` amount. Represents the sum of all debt shares distributed among debtors
    pub total_d_tokens: i128,
    /// The total `jTokens` amount. Represents the sum of all yielding interest collateral shares
    /// distributed among creditors
    pub total_j_tokens: i128,
    /// The total amount of currently available tokens for borrowing
    pub total_available: i128,
    /// The total amount of deposited collateral assets that don't accrue interest
    pub total_collateral: i128,
    /// Interest rate model(+configuration) used for interest rate calculation
    pub interest_rate_model: InterestRateModel,
    /// Accrual model used for accruing the `total_borrowed` amount on the pool based on the
    /// interest rate
    pub accrual_model: AccrualModel,
    /// Configuration settings for the pool
    pub config: PoolConfig,
    /// Fee configuration for the pool
    pub fee_config: PoolFeeConfig,
    /// Amount of tokens in the insurance reserve that can be used to cover a bad debt scenario
    pub accumulated_reserve_fee: i128,
    /// Amount of tokens that can be withdrawn by the market's admin as a fee
    pub accumulated_market_fee: i128,
    /// Amount of tokens that can be withdraw by the host platform admin as a fee
    pub accumulated_host_fee: i128,
    /// The timestamp of the last accrual re-calculation
    pub last_accrual_timestamp: u64,
    /// The result of `TokenClient::name(&self)` invocation: `native` string for XLM SAC and the
    /// SAC's native asset code and asset issuer concatenated with `:` for other SACs(e.g,
    /// "AQUA:GAHPYWLK6YRN7CVYZOO4H3VDRZ7PVF5UJGLZCSPAEIKJE2XSWF5LAGER")
    pub name: String,
}

impl Pool {
    fn adjust_field(e: &Env, current_value: i128, adjusting_amount: i128) -> Result<i128, MCError> {
        let new_amount = current_value
            .checked_add(adjusting_amount)
            .map_over_or_underflow()?;

        if new_amount < 0 {
            events::pool_amount_becomes_negative(e, current_value, new_amount);

            return Err(MCError::InternalError);
        }

        Ok(new_amount)
    }

    pub fn adjust_total_j_tokens(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = Self::adjust_field(e, self.total_j_tokens, adjusting_amount)?;
        self.total_j_tokens = new_amount;

        Ok(())
    }

    pub fn adjust_total_borrowed(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = Self::adjust_field(e, self.total_borrowed, adjusting_amount)?;
        self.total_borrowed = new_amount;

        Ok(())
    }

    pub fn adjust_total_available(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = Self::adjust_field(e, self.total_available, adjusting_amount)?;
        self.total_available = new_amount;

        Ok(())
    }

    pub fn adjust_total_d_tokens(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = Self::adjust_field(e, self.total_d_tokens, adjusting_amount)?;
        self.total_d_tokens = new_amount;

        Ok(())
    }

    pub fn adjust_total_collateral(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = Self::adjust_field(e, self.total_collateral, adjusting_amount)?;
        self.total_collateral = new_amount;

        Ok(())
    }

    pub fn adjust_accumulated_market_fee(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = Self::adjust_field(e, self.accumulated_market_fee, adjusting_amount)?;
        self.accumulated_market_fee = new_amount;

        Ok(())
    }

    pub fn adjust_accumulated_host_fee(
        &mut self,
        e: &Env,
        adjusting_amount: i128,
    ) -> Result<(), MCError> {
        let new_amount = Self::adjust_field(e, self.accumulated_host_fee, adjusting_amount)?;
        self.accumulated_host_fee = new_amount;

        Ok(())
    }

    // TODO: Add dTokenRate?

    pub fn compute_tokens_from_d_tokens(
        &self,
        e: &Env,
        d_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        // TODO: Check if there are some useful properties between jTokens and dTokens that
        // might cause some code re-usage
        let tokens = Self::compute_tokens_from_shares(
            e,
            d_tokens_amount,
            self.total_d_tokens,
            self.total_borrowed,
        )?;

        Ok(tokens)
    }

    pub fn compute_d_tokens_from_tokens(
        &self,
        e: &Env,
        tokens_amount: i128,
    ) -> Result<i128, MCError> {
        let d_tokens = Self::compute_shares_from_tokens(
            e,
            tokens_amount,
            self.total_d_tokens,
            self.total_borrowed,
        )?;

        Ok(d_tokens)
    }

    pub fn compute_tokens_from_j_tokens(
        &self,
        e: &Env,
        j_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        let tokens = Self::compute_tokens_from_shares(
            e,
            j_tokens_amount,
            self.total_j_tokens,
            self.total_supply()?,
        )?;

        Ok(tokens)
    }

    pub fn compute_j_tokens_from_tokens(
        &self,
        e: &Env,
        tokens_amount: i128,
    ) -> Result<i128, MCError> {
        let j_tokens = Self::compute_shares_from_tokens(
            e,
            tokens_amount,
            self.total_j_tokens,
            self.total_supply()?,
        )?;

        Ok(j_tokens)
    }

    pub fn require_available(&self, required: i128) -> Result<(), MCError> {
        if required > self.total_available_minus_accumulated_reserve_fees()? {
            return Err(MCError::NotEnoughPoolFunds);
        }

        Ok(())
    }

    pub fn require_does_not_exist(e: &Env, pool_address: &Address) -> Result<(), MCError> {
        if Self::exists(e, pool_address) {
            return Err(MCError::PoolAlreadyExists);
        }

        Ok(())
    }

    pub fn require_preserves_utilization_ratio_cap(
        &self,
        e: &Env,
        removed_available_amount: i128,
    ) -> Result<(), MCError> {
        let max_available_amount_to_remove =
            Self::compute_available_utilization_ratio_cap_borrow(self, e)?;

        if removed_available_amount > max_available_amount_to_remove {
            return Err(MCError::PoolUtilizationRatioCapExceeded);
        }

        Ok(())
    }

    /// Computes the maximum available amount for borrowing that doesn't exceed the utilization
    /// ratio limit on a pool
    pub fn compute_available_utilization_ratio_cap_borrow(&self, e: &Env) -> Result<i128, MCError> {
        let total_supply = self.total_supply()?; // likely a problem...
        let utilization_ratio = self.calculate_utilization_ratio_bps()?;

        if utilization_ratio > self.config.utilization_ratio_limit_bps {
            // NB: This can happen when the `total_borrowed` amount on a pool has accrued over time
            // by itself, so for now, we simply emit an event. We can agree to stop
            // accruing interest on a pool if this happens
            events::utilization_ratio_exceeds_limit(
                e,
                utilization_ratio,
                self.config.utilization_ratio_limit_bps,
            );

            return Ok(0);
        }
        let available_percentage_to_borrow_bps =
            self.config.utilization_ratio_limit_bps - utilization_ratio; // safe

        total_supply
            .fixed_mul_ceil(available_percentage_to_borrow_bps, BPS_FACTOR)
            .map_over_or_underflow()
    }

    /// Computes the number of tokens proportional to the given share of the tokens in the pool.
    /// Intended to be used for both `jTokens` and `dTokens` related calculations
    fn compute_tokens_from_shares(
        e: &Env,
        shares_amount: i128,
        total_shares_amount: i128,
        total_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        if shares_amount == 0 {
            return Ok(0);
        }

        if total_shares_amount < shares_amount {
            events::pool_total_shares_smaller_than_individual_user_shares(
                e,
                total_shares_amount,
                shares_amount,
            );

            return Err(MCError::InternalError);
        }

        let tokens_amount = total_tokens_amount
            .fixed_div_floor(total_shares_amount, shares_amount)
            .map_over_or_underflow()?;

        Ok(tokens_amount)
    }

    /// Computes the shares amount which must be issued or burnt from a specific obligation based on
    /// the provided tokens amount. Intended to be used for both `jTokens` and `dTokens` related
    /// calculations
    fn compute_shares_from_tokens(
        e: &Env,
        tokens_amount: i128,
        total_shares_amount: i128,
        total_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        // TODO: Is it always consistent with situations like:
        // I have the last shares and I remove them - total supply becomes zero. Check this
        if tokens_amount == 0 {
            return Ok(0);
        }

        let shares_amount = if total_shares_amount == 0 {
            // NB: Is it reasonable to make the initial amount smaller?
            tokens_amount
        } else {
            if total_shares_amount > total_tokens_amount {
                events::pool_total_shares_smaller_than_total_tokens(
                    e,
                    total_shares_amount,
                    total_tokens_amount,
                );

                return Err(MCError::InternalError);
            }

            /*
            This must hold when issuing new shares:
                shares_to_issue / (shares_to_issue + prev_total_shares) = tokens_added_amount / (tokens_added_amount + prev_total_tokens_amount)
            Which implies:
                shares_to_issue = prev_total_shares * (tokens_added_amount / prev_total_tokens_amount)

            This must hold when burning issued shares:
                shares_to_burn = prev_total_shares * (tokens_removed_amount / prev_total_tokens_amount)
            */
            total_shares_amount
                /* Using 'ceil' here has advantages when withdrawing\repaying small amounts of tokens.
                Namely, if the token amount is really small, with `floor`, the respective amount of
                shares to burn is 0, and doesn't make a difference  */
                .fixed_div_ceil(total_tokens_amount, tokens_amount)
                .map_over_or_underflow()?
        };

        Ok(shares_amount)
    }

    pub fn total_available_minus_accumulated_reserve_fees(&self) -> Result<i128, MCError> {
        // TODO: Can we use `saturating_sub` here instead of `checked_sub`?
        let res = self
            .total_available
            .saturating_sub(self.accumulated_reserve_fee);

        Ok(res)
    }

    /// Calculates total supply (available + total_borrowed - accumulated fees)
    pub fn total_supply(&self) -> Result<i128, MCError> {
        self.total_available
            .checked_add(self.total_borrowed)
            .map_over_or_underflow()?
            .checked_sub(self.accumulated_reserve_fee)
            .map_over_or_underflow()
    }

    /// Checks if the pool is empty
    pub fn is_empty(&self) -> bool {
        if self.total_j_tokens == 0 && self.total_available != 0 {
            // TODO: What to do in these cases?
        }

        if self.total_d_tokens == 0 && self.total_borrowed != 0 {
            // TODO: What to do in these cases?
        }

        self.total_j_tokens == 0
            && self.total_d_tokens == 0
            && self.total_borrowed == 0
            && self.total_available == 0
            && self.total_collateral == 0
        // TODO: && self.accumulated_reserve_fees == 0?
    }

    /// Tries to get the pool from the contract's storage
    ///
    /// # Returns
    /// - [`Ok(Pool)`] if a pool with the given address exists in the contract's storage
    /// - [`Err(MCError::PoolDoesNotExist)`] otherwise
    pub fn try_get(e: &Env, pool_address: &Address) -> Result<Self, MCError> {
        storage::get_pool(e, pool_address).ok_or(MCError::PoolDoesNotExist)
    }

    pub fn get_all(e: &Env) -> Vec<Address> {
        storage::get_all_pools(e)
    }

    fn exists(e: &Env, address: &Address) -> bool {
        storage::pool_exists(e, address)
    }

    /// Refreshes the pool with the contract's storage data
    pub fn refresh(&mut self, e: &Env) -> Result<(), MCError> {
        let Some(refreshed_pool) = storage::get_pool(e, &self.pool_address) else {
            events::pool_is_missing_in_storage(e, &self.pool_address);

            return Err(MCError::InternalError);
        };
        *self = refreshed_pool;

        Ok(())
    }

    /// Saves/updates pool in the contract's storage
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn set(&self, e: &Env) {
        storage::set_pool(e, &self.pool_address, self);
    }

    /// Registers pool in the pools list
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn register(&self, e: &Env) -> u32 {
        storage::register_pool(e, &self.pool_address.clone())
    }
}

#[contracttype]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PoolFeeConfig {
    pub borrow_fee_bps: u32,
    pub flash_loan_fee_bps: u32,

    pub deposit_fee_bps: u32,
    pub withdraw_fee_bps: u32,
    pub add_collateral_fee_bps: u32,
    pub remove_collateral_fee_bps: u32,
    pub repay_fee_bps: u32,

    // pub deposit_with_leverage_fee_bps: ?
    // pub withdraw_from_leveraged_fe_bps: ?
    pub take_rate_bps: u32,
    pub host_fee_bps: u32,
}

impl Default for PoolFeeConfig {
    fn default() -> Self {
        Self {
            borrow_fee_bps: DEFAULT_BORROW_FEE_BPS,
            flash_loan_fee_bps: DEFAULT_FLASH_LOAN_FEE_BPS,

            deposit_fee_bps: DEFAULT_DEPOSIT_FEE_BPS,
            withdraw_fee_bps: DEFAULT_WITHDRAW_FEE_BPS,
            add_collateral_fee_bps: DEFAULT_ADD_COLLATERAL_FEE_BPS,
            remove_collateral_fee_bps: DEFAULT_REMOVE_COLLATERAL_FEE_BPS,
            repay_fee_bps: DEFAULT_REPAY_FEE_BPS,

            host_fee_bps: DEFAULT_HOST_FEE_BPS,
            take_rate_bps: DEFAULT_TAKE_RATE_BPS,
        }
    }
}

impl PoolFeeConfig {
    pub fn validate(&self) -> Result<(), &str> {
        let &Self {
            deposit_fee_bps,
            borrow_fee_bps,
            add_collateral_fee_bps,
            withdraw_fee_bps,
            remove_collateral_fee_bps,
            flash_loan_fee_bps,
            take_rate_bps,
            repay_fee_bps,
            host_fee_bps,
        } = self;

        Ok(())
    }
}

#[contracttype]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
// NB: Soon, this config will likely become obsolete and will be
// replaced by a few adjacent configs
pub struct PoolConfig {
    /// Percentage of interest payments allocated to protocol reserves
    pub reserve_ratio_bps: i128,
    /// Maximum percentage of a borrower's debt that can be liquidated
    pub liquidation_close_factor_bps: i128,
    /// Additional discount given to liquidators when purchasing collateral
    pub liquidation_incentive_bps: i128,
    /// The maximum amount of supplied tokens that can be supplied in the pool(i.e., `available` +
    /// `total_borrowed`) 0 denotes unlimited supply
    pub supply_limit: i128,
    /// The maximum utilization ratio that is allowed to be reached via borrowing
    pub utilization_ratio_limit_bps: i128,
    /// The maximum percentage of an asset's value that can be borrowed in basis points(e.g, 7000 =
    /// 70%, etc) with respect to a total obligation's collateral value
    pub open_ltv_bps: i128,
    /// The maximum percentage of an asset's value that can be held in an individual obligation in
    /// basis points with respect to a total obligation's collateral value. LTV greater than
    /// that makes borrow position eligible to liquidation
    pub close_ltv_bps: i128,
    /// The factor used to calculate the current borrow limit by multiplying the collateral value
    /// by it before subtracting this value from the obligation's max borrow limit. Volatile
    /// assets' pools are expected to have this value set way above 100%
    pub liability_factor_bps: i128,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            reserve_ratio_bps: DEFAULT_RESERVE_RATIO * BPS_IN_PERCENT,
            liquidation_close_factor_bps: DEFAULT_CLOSE_FACTOR * BPS_IN_PERCENT,
            liquidation_incentive_bps: DEFAULT_LIQUIDATION_SPREAD * BPS_IN_PERCENT,
            supply_limit: DEFAULT_SUPPLY_LIMIT,
            utilization_ratio_limit_bps: DEFAULT_UTILIZATION_RATIO_LIMIT * BPS_IN_PERCENT,
            open_ltv_bps: DEFAULT_OPEN_LTV * BPS_IN_PERCENT,
            close_ltv_bps: DEFAULT_CLOSE_LTV * BPS_IN_PERCENT,
            liability_factor_bps: DEFAULT_LIABILITY_FACTOR * BPS_IN_PERCENT,
        }
    }
}

impl PoolConfig {
    pub fn validate(&self) -> Result<(), &str> {
        let &PoolConfig {
            reserve_ratio_bps,
            liquidation_close_factor_bps,
            liquidation_incentive_bps,
            supply_limit,
            utilization_ratio_limit_bps,
            open_ltv_bps,
            close_ltv_bps,
            liability_factor_bps,
        } = self;

        if supply_limit < 0 {
            return Err("Supply limit must be non-negative");
        }

        if !is_valid_percent(utilization_ratio_limit_bps) {
            return Err("Utilization ratio limit must be between 0% and 100%");
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

        if !(0..(100 * BPS_IN_PERCENT)).contains(&open_ltv_bps) {
            return Err("Open LTV must be between 0% and 100%");
        }

        if !is_valid_percent(close_ltv_bps) {
            return Err("Close LTV must be between 0% and 100%");
        }

        if close_ltv_bps < open_ltv_bps {
            return Err("Open LTV mustn't be bigger than close LTV");
        }

        if !(0..(MAX_LIABILITY_FACTOR * BPS_IN_PERCENT)).contains(&liability_factor_bps) {
            return Err("Invalid liability factor");
        }

        Ok(())
    }
}

fn is_valid_percent(value: i128) -> bool {
    (0..=100 * BPS_IN_PERCENT).contains(&value)
}
