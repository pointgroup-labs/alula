use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, String, Symbol, Vec, contracttype};

use crate::{
    accrual::AccrualModel,
    constants::*,
    error::MCError,
    events,
    interest_rate_model::InterestRateModel,
    math_utils::MathUtils,
    obligation::{
        AddCollateralResult, BorrowResult, ComputedFees, DepositResult, RemoveCollateralResult,
        RepayResult, WithdrawResult,
    },
    oracle::get_asset_price,
    storage::{self, PoolUpdate},
};

#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
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
    /// Amount of tokens in the insurance reserve that can be used to cover a bad debt scenario
    pub accumulated_reserve_fees: i128,
    /// Amount of tokens that can be withdrawn by the market's admin as a fee
    pub accumulated_market_fees: i128,
    /// Amount of tokens that can be withdrawn by the host platform admin as a fee
    pub accumulated_host_fees: i128,
    /// The result of `TokenClient::name(&self)` invocation: `native` string for XLM SAC and the
    /// SAC's native asset code and asset issuer concatenated with `:` for other SACs(e.g,
    /// "AQUA:GAHPYWLK6YRN7CVYZOO4H3VDRZ7PVF5UJGLZCSPAEIKJE2XSWF5LAGER")
    pub name: String,
    /// Configuration settings for the pool
    pub config: PoolConfig,
    /// The timestamp of the last accrual re-calculation
    pub last_accrual_timestamp: u64,
    // TODO: APY & APR?
}

macro_rules! generate_adjust_method {
    ($method_name:ident, $field:ident) => {
        pub fn $method_name(&mut self, e: &Env, amount: i128) -> Result<(), MCError> {
            let new_amount = self.$field.checked_add(amount).map_over_or_underflow()?;
            if new_amount < 0 {
                events::pool_amount_becomes_negative(e, self.$field, new_amount);
                return Err(MCError::InternalError);
            }
            self.$field = new_amount;
            Ok(())
        }
    };
}

impl Pool {
    generate_adjust_method!(adjust_total_j_tokens, total_j_tokens);
    generate_adjust_method!(adjust_total_borrowed, total_borrowed);
    generate_adjust_method!(adjust_total_available, total_available);
    generate_adjust_method!(adjust_total_d_tokens, total_d_tokens);
    generate_adjust_method!(adjust_total_collateral, total_collateral);
    generate_adjust_method!(adjust_accumulated_market_fees, accumulated_market_fees);
    generate_adjust_method!(adjust_accumulated_host_fees, accumulated_host_fees);
    generate_adjust_method!(adjust_accumulated_reserve_fees, accumulated_reserve_fees);

    fn adjust_fees(&mut self, e: &Env, fees: &ComputedFees) -> Result<(), MCError> {
        self.adjust_accumulated_host_fees(e, fees.host_fee)?;
        self.adjust_accumulated_market_fees(e, fees.market_fee)?;

        Ok(())
    }

    /// Deposits assets to the pool
    pub fn deposit(&mut self, e: &Env, deposit_result: &DepositResult) -> Result<(), MCError> {
        self.adjust_total_j_tokens(e, deposit_result.j_tokens_to_issue)?;
        self.adjust_total_available(e, deposit_result.deposited)?;

        self.adjust_fees(e, &deposit_result.computed_fees)?;

        Ok(())
    }

    /// Withdraws yielding assets from the pool
    pub fn withdraw(&mut self, e: &Env, withdraw_result: &WithdrawResult) -> Result<(), MCError> {
        self.adjust_total_available(
            e,
            withdraw_result.deposit_decrease.checked_neg().map_over_or_underflow()?,
        )?;
        self.adjust_total_j_tokens(
            e,
            withdraw_result.j_tokens_to_burn.checked_neg().map_over_or_underflow()?,
        )?;

        self.adjust_fees(e, &withdraw_result.computed_fees)?;

        Ok(())
    }

    /// Borrows assets from the pool
    pub fn borrow(&mut self, e: &Env, borrow_result: &BorrowResult) -> Result<(), MCError> {
        self.adjust_total_d_tokens(e, borrow_result.d_tokens_to_issue)?;
        self.adjust_total_borrowed(e, borrow_result.borrower_new_debt)?;
        self.adjust_total_available(
            e,
            borrow_result.borrower_new_debt.checked_neg().map_over_or_underflow()?,
        )?;

        self.adjust_fees(e, &borrow_result.computed_fees)?;

        Ok(())
    }

    /// Adds collateral to the pool
    pub fn add_collateral(
        &mut self,
        e: &Env,
        add_collateral_result: &AddCollateralResult,
    ) -> Result<(), MCError> {
        self.adjust_total_collateral(e, add_collateral_result.added_collateral)?;

        self.adjust_fees(e, &add_collateral_result.computed_fees)?;

        Ok(())
    }

    /// Repays debt to the pool
    pub fn repay(&mut self, e: &Env, repay_result: &RepayResult) -> Result<(), MCError> {
        self.adjust_total_d_tokens(
            e,
            repay_result.d_tokens_to_burn.checked_neg().map_over_or_underflow()?,
        )?;
        self.adjust_total_borrowed(
            e,
            repay_result.debt_repaid.checked_neg().map_over_or_underflow()?,
        )?;
        self.adjust_total_available(e, repay_result.debt_repaid)?;

        self.adjust_fees(e, &repay_result.computed_fees)?;

        Ok(())
    }

    /// Removes collateral from the pool
    pub fn remove_collateral(
        &mut self,
        e: &Env,
        remove_collateral_result: &RemoveCollateralResult,
    ) -> Result<(), MCError> {
        self.adjust_total_collateral(
            e,
            remove_collateral_result.collateral_decrease.checked_neg().map_over_or_underflow()?,
        )?;

        self.adjust_fees(e, &remove_collateral_result.computed_fees)?;

        Ok(())
    }

    // TODO: Add dTokenRate?

    pub fn compute_tokens_from_d_tokens(
        &self,
        e: &Env,
        d_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        // TODO: Check if there are some useful properties between jTokens and dTokens that
        // might cause some code re-usage | UPD: unlikely, for now
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

    pub fn require_remove_collateral_preserves_ur_cap(
        &self,
        e: &Env,
        removed_available_amount: i128,
    ) -> Result<(), MCError> {
        let max_available_amount_to_remove =
            Self::compute_available_utilization_ratio_cap_remove_collateral(self, e)?;

        if self.compute_utilization_ratio_bps()? != 0
            && removed_available_amount > max_available_amount_to_remove
        {
            return Err(MCError::PoolUtilizationRatioCapExceeded);
        }

        Ok(())
    }

    pub fn require_borrow_preserves_ur_cap(
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

    /// Computes the maximum available amount for collateral removal that doesn't exceed the
    /// utilization ratio limit on a pool
    pub fn compute_available_utilization_ratio_cap_remove_collateral(
        &self,
        e: &Env,
    ) -> Result<i128, MCError> {
        let total_borrowed = self.total_borrowed;
        let total_supply = self.total_supply()?; // WARN: Investigate how reserves affect UR
        let utilization_ratio = self.compute_utilization_ratio_bps()?;

        if utilization_ratio > self.config.health_config.utilization_ratio_limit_bps {
            events::utilization_ratio_exceeds_limit(
                e,
                utilization_ratio,
                self.config.health_config.utilization_ratio_limit_bps,
            );

            return Ok(0);
        }

        let available_percentage_to_withdraw_bps =
            self.config.health_config.utilization_ratio_limit_bps; // safe

        let diff = total_borrowed
            .fixed_div_ceil(available_percentage_to_withdraw_bps, BPS_FACTOR)
            .map_over_or_underflow()?;

        if total_supply < diff {
            // TODO: Make it more specific?
            events::pool_contains_inconsistent_state(e, self);

            return Err(MCError::InternalError);
        }

        Ok(total_supply - diff)
    }

    /// Computes the maximum available amount for borrowing that doesn't exceed the utilization
    /// ratio limit on a pool
    pub fn compute_available_utilization_ratio_cap_borrow(&self, e: &Env) -> Result<i128, MCError> {
        let total_supply = self.total_supply()?; // WARN: Investigate how reserves affect UR
        let utilization_ratio = self.compute_utilization_ratio_bps()?;

        if utilization_ratio > self.config.health_config.utilization_ratio_limit_bps {
            // NB: This can happen when the `total_borrowed` amount on a pool has accrued over time
            // by itself, so for now, we simply emit an event. We can agree to stop
            // accruing interest on a pool if this happens
            events::utilization_ratio_exceeds_limit(
                e,
                utilization_ratio,
                self.config.health_config.utilization_ratio_limit_bps,
            );

            return Ok(0);
        }

        let available_percentage_to_borrow_bps =
            self.config.health_config.utilization_ratio_limit_bps - utilization_ratio; // safe

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

        let tokens_amount = shares_amount
            .fixed_mul_floor(total_tokens_amount, total_shares_amount)
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

            // This must hold when issuing new shares: shares_to_issue / (shares_to_issue + prev_total_shares) = tokens_added_amount /
            // (tokens_added_amount + prev_total_tokens_amount)
            // Which implies:
            //   shares_to_issue = prev_total_shares * (tokens_added_amount / prev_total_tokens_amount)
            // This must hold when burning issued shares:
            //   shares_to_burn = prev_total_shares * (tokens_removed_amount / prev_total_tokens_amount)
            total_shares_amount
                // Using 'ceil' here has advantages when withdrawing\repaying small amounts of
                // tokens. Namely, if the token amount is really small, with
                // `floor`, the respective amount of shares to burn is 0, and
                // doesn't make a difference
                .fixed_div_ceil(total_tokens_amount, tokens_amount)
                .map_over_or_underflow()?
        };

        Ok(shares_amount)
    }

    pub fn available_accumulated_reserve_fees(&self) -> i128 {
        let total_available = self.total_available;
        let accumulated_reserve_fees = self.accumulated_reserve_fees;

        i128::min(total_available, accumulated_reserve_fees)
    }

    pub fn total_available_minus_accumulated_reserve_fees(&self) -> Result<i128, MCError> {
        // TODO: Can we use `saturating_sub` here instead of `checked_sub`?
        let res = self.total_available.saturating_sub(self.accumulated_reserve_fees);

        Ok(res)
    }

    /// Calculates total debt (total_borrowed - accumulated reserve fees)
    pub fn total_debt(&self) -> Result<i128, MCError> {
        self.total_borrowed.checked_sub(self.accumulated_reserve_fees).map_over_or_underflow()
    }

    /// Calculates total supply (available + total_borrowed - accumulated reserve fees)
    pub fn total_supply(&self) -> Result<i128, MCError> {
        let total_funds =
            self.total_available.checked_add(self.total_borrowed).map_over_or_underflow()?;

        total_funds.checked_sub(self.accumulated_reserve_fees).map_over_or_underflow()
    }

    /// Checks if the pool is empty
    pub fn is_empty(&self) -> bool {
        let &Self {
            pool_address: _,
            token_address: _,
            token_ticker: _,
            last_accrual_timestamp: _,
            name: _,
            config: _,
            total_borrowed,
            total_d_tokens,
            total_j_tokens,
            total_available,
            total_collateral,
            accumulated_reserve_fees,
            accumulated_market_fees,
            accumulated_host_fees,
        } = self;

        if total_j_tokens == 0 && total_available != 0 {
            // These represent serious invariant violations that should be logged
            // it means funds exist but no one owns them (orphaned funds)
        }

        if total_d_tokens == 0 && total_borrowed != 0 {
            // These represent serious invariant violations that should be logged
            // it means debt exists but no one owes it (ghost debt)
        }

        total_j_tokens == 0
            && total_d_tokens == 0
            && total_borrowed == 0
            && total_available == 0
            && total_collateral == 0
            && accumulated_host_fees == 0
            && accumulated_reserve_fees == 0
            && accumulated_market_fees == 0
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

    /// Queues in pool's config update
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn queue_in_config_update(&self, e: &Env, config: &PoolConfig) -> Result<(), MCError> {
        storage::queue_in_pool_config_update(e, &self.pool_address, config)
    }

    /// Cancels pool's config update from the queue
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn cancel_pool_config_update(&self, e: &Env) -> Result<(), MCError> {
        storage::cancel_pool_config_update(e, &self.pool_address)
    }

    /// Applies the pool's config update from the queue if it exists and is seasoned
    ///
    /// # WARNING
    /// Modifies the contract's storage
    pub fn apply_pool_config_update(&mut self, e: &Env) -> Result<(), MCError> {
        let update_pool_config_period = storage::get_update_in_queue_period(e);
        let pool_config_update = self.get_pool_config_update(e)?;
        let current_time = e.ledger().timestamp();

        if current_time
            < pool_config_update
                .queued_in_timestamp
                .checked_add(update_pool_config_period)
                .map_over_or_underflow()?
        {
            return Err(MCError::PoolConfigUpdateIsNotSeasonedYet);
        }
        self.config = pool_config_update.new_config;

        self.set(e);

        Ok(())
    }

    /// Gets the pool's config update from the queue if it exists
    pub fn get_pool_config_update(&self, e: &Env) -> Result<PoolUpdate, MCError> {
        storage::get_pool_config_update(e, &self.pool_address)
            .ok_or(MCError::PoolDoesNotHaveQueuedInConfigUpdate)
    }

    pub fn compute_total_collateral_value(&self, e: &Env) -> Result<i128, MCError> {
        let deposited_tokens = self.compute_tokens_from_j_tokens(e, self.total_j_tokens)?;
        let collateral_sum =
            deposited_tokens.checked_add(self.total_collateral).map_over_or_underflow()?;

        self.get_assets_value(e, collateral_sum)
    }

    pub fn compute_total_debt_value(&self, e: &Env) -> Result<i128, MCError> {
        let debt = self.total_debt()?;

        self.get_assets_value(e, debt)
    }

    fn get_assets_value(&self, e: &Env, amount: i128) -> Result<i128, MCError> {
        let price = get_asset_price(e, &self.token_address)?;

        price.checked_mul(amount).map_over_or_underflow()
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
        storage::register_pool(e, &self.pool_address)
    }
}

#[contracttype]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PoolFeeConfig {
    pub borrow_fee_bps: u32,
    pub flash_loan_fee_bps: u32,

    pub deposit_fee_bps: u32,
    pub withdraw_fee_bps: u32,
    /// Over-utilization ratio cap withdrawal fee scalar
    pub over_ur_withdraw_fee_scalar: u32,
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
            over_ur_withdraw_fee_scalar: DEFAULT_OVER_UTILIZATION_RATIO_FEE_SCALAR,
        }
    }
}

#[contracttype]
#[derive(Default, Copy, Clone, Debug, Eq, PartialEq)]
pub struct PoolConfig {
    pub fee_config: PoolFeeConfig,
    pub health_config: PoolHealthConfig,
    pub accrual_model: AccrualModel,
    pub interest_rate_model: InterestRateModel,
}

impl PoolConfig {
    pub fn validate(&self) -> Result<(), MCError> {
        let PoolConfig { health_config, .. } = self;

        // NB: Is there a reason to validate the fee config?

        if health_config.validate().is_err() {
            return Err(MCError::InvalidLoanPoolConfig);
        }

        Ok(())
    }
}

#[contracttype]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PoolHealthConfig {
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
    /// Maximum percentage of a borrower's debt that can be liquidated
    pub liquidation_close_factor_bps: i128,
    /// Additional discount given to liquidators when purchasing collateral
    pub liquidation_incentive_bps: i128,
}

impl Default for PoolHealthConfig {
    fn default() -> Self {
        Self {
            supply_limit: DEFAULT_SUPPLY_LIMIT,
            utilization_ratio_limit_bps: DEFAULT_UTILIZATION_RATIO_LIMIT_BPS,
            open_ltv_bps: DEFAULT_OPEN_LTV_BPS,
            close_ltv_bps: DEFAULT_CLOSE_LTV_BPS,
            liability_factor_bps: DEFAULT_LIABILITY_FACTOR_BPS,
            liquidation_close_factor_bps: DEFAULT_CLOSE_FACTOR_BPS,
            liquidation_incentive_bps: DEFAULT_LIQUIDATION_SPREAD_BPS, // TODO: Rename?
        }
    }
}

impl PoolHealthConfig {
    fn validate(&self) -> Result<(), &str> {
        let &Self {
            supply_limit,
            utilization_ratio_limit_bps,
            open_ltv_bps,
            close_ltv_bps,
            liability_factor_bps,
            liquidation_close_factor_bps,
            liquidation_incentive_bps,
        } = self;

        if supply_limit < 0 {
            return Err("Supply limit must be non-negative");
        }

        if !is_valid_percent(utilization_ratio_limit_bps) {
            return Err("Utilization ratio limit must be between 0% and 100%");
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

        if !(0..(MAX_LIABILITY_FACTOR_BPS)).contains(&liability_factor_bps) {
            return Err("Invalid liability factor");
        }

        if !is_valid_percent(liquidation_close_factor_bps) {
            return Err("Liquidation close factor must be between 0% and 100%");
        }

        if !is_valid_percent(liquidation_incentive_bps) {
            return Err("Liquidation incentive must be between 0% and 100%");
        }

        Ok(())
    }
}

fn is_valid_percent(value: i128) -> bool {
    (0..=100 * BPS_IN_PERCENT).contains(&value)
}
