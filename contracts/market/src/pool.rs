use soroban_fixed_point_math::FixedPoint;
use soroban_sdk::{Address, Env, Map, String, Vec, contracttype};

use crate::{
    accrual::AccrualModel,
    constants::*,
    error::MCError,
    events,
    interest_rate_model::InterestRateModel,
    math_utils::MathUtils,
    misc::PoolData,
    obligation::{
        AddCollateralResult, BorrowResult, DepositResult, LiquidationResult, OperationFees,
        RemoveCollateralResult, RepayResult, WithdrawResult,
    },
    oracle::{self, get_asset_price},
    storage::{self, PoolUpdate},
};

#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Pool {
    // The address of the loan pool
    pub pool_address: Address,
    // The address of the token contract associated with the pool
    pub token_address: Address,
    // The token symbol of the associated asset
    pub token_symbol: String,
    // The total amount of borrowed assets. This value increases with interest rate accrual
    pub total_borrowed: i128,
    // The total `dTokens` amount. Represents the sum of all debt shares distributed among debtors
    pub total_d_tokens: i128,
    // The total `jTokens` amount. Represents the sum of all yielding interest collateral shares
    // distributed among creditors
    pub total_j_tokens: i128,
    // The total amount of currently available tokens for borrowing
    pub total_available: i128,
    // The total amount of deposited collateral assets that don't accrue interest
    pub total_collateral: i128,
    // The result of `TokenClient::name(&self)` invocation: `native` string for XLM SAC and the
    // SAC's native asset code and asset issuer concatenated with `:` for other SACs(e.g,
    // "AQUA:GAHPYWLK6YRN7CVYZOO4H3VDRZ7PVF5UJGLZCSPAEIKJE2XSWF5LAGER")
    pub name: String,
    // Configuration settings for the pool
    pub config: PoolConfig,
    // The timestamp of the last accrual re-calculation
    pub last_accrual_timestamp: u64,
    // Remaining supply bootstrap amounts that are distributed evenly among specified periods
    pub bootstrap_periods: Map<(u64, u64), PoolBootstrapPeriod>,
    // Borrow annual percentage rate in basis points
    pub borrow_apr_bps: i128,
    // Supply annual percentage rate in basis points
    pub supply_apr_bps: i128,
    // Maintained sum of the accumulated per-operation beneficiaries' fees
    pub operation_fees_sum: i128,
    //  Maintained sum of the accumulated per take rate beneficiaries' fees
    pub take_rate_fees_sum: i128,
    // Target utilization ratio in basis points
    pub target_utilization_ratio_bps: i128,
    // Interest rate modifier in basis points
    pub interest_rate_modifier: i128,
}

macro_rules! generate_adjust_method {
    ($method_name:ident, $field:ident) => {
        pub fn $method_name(&mut self, e: &Env, amount: i128) -> Result<(), MCError> {
            let new_amount = self.$field.checked_add(amount).map_over_or_underflow()?;
            if new_amount.is_negative() {
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
    generate_adjust_method!(adjust_operation_fees_sum, operation_fees_sum);

    fn adjust_accumulated_fees_with_computed_per_operation(
        &mut self,
        fees: &OperationFees,
    ) -> Result<(), MCError> {
        let referrer_fees = fees.referrer_fee.unwrap_or(0);
        let net_protocol_fees = fees.fee_sum - referrer_fees;

        self.operation_fees_sum =
            self.operation_fees_sum.checked_add(net_protocol_fees).map_over_or_underflow()?;

        Ok(())
    }

    // Bootstraps the pool by distributing additional rewards to suppliers
    pub fn bootstrap(&mut self, amount: i128, period: (u64, u64)) -> Result<(), MCError> {
        if self.bootstrap_periods.get(period).is_some() {
            return Err(MCError::InvalidBootstrapPeriod);
        }

        self.bootstrap_periods.set(period, PoolBootstrapPeriod::new(amount));

        Ok(())
    }

    // Deposits assets to the pool
    pub fn deposit(&mut self, e: &Env, deposit_result: &DepositResult) -> Result<(), MCError> {
        self.adjust_total_j_tokens(e, deposit_result.j_tokens_to_issue)?;
        self.adjust_total_available(e, deposit_result.deposited)?;

        self.adjust_accumulated_fees_with_computed_per_operation(&deposit_result.operation_fees)?;

        Ok(())
    }

    // Withdraws yielding assets from the pool
    pub fn withdraw(&mut self, e: &Env, withdraw_result: &WithdrawResult) -> Result<(), MCError> {
        self.adjust_total_available(
            e,
            withdraw_result.deposit_decrease.checked_neg().map_over_or_underflow()?,
        )?;
        self.adjust_total_j_tokens(
            e,
            withdraw_result.j_tokens_to_burn.checked_neg().map_over_or_underflow()?,
        )?;

        self.adjust_accumulated_fees_with_computed_per_operation(&withdraw_result.operation_fees)?;

        Ok(())
    }

    // Borrows assets from the pool
    pub fn borrow(&mut self, e: &Env, borrow_result: &BorrowResult) -> Result<(), MCError> {
        self.adjust_total_d_tokens(e, borrow_result.d_tokens_to_issue)?;
        self.adjust_total_borrowed(e, borrow_result.borrower_new_debt)?;
        self.adjust_total_available(
            e,
            borrow_result.borrower_new_debt.checked_neg().map_over_or_underflow()?,
        )?;

        self.adjust_accumulated_fees_with_computed_per_operation(&borrow_result.operation_fees)?;

        Ok(())
    }

    // Adds collateral to the pool
    pub fn add_collateral(
        &mut self,
        e: &Env,
        add_collateral_result: &AddCollateralResult,
    ) -> Result<(), MCError> {
        self.adjust_total_collateral(e, add_collateral_result.added_collateral)?;

        self.adjust_accumulated_fees_with_computed_per_operation(
            &add_collateral_result.operation_fees,
        )?;

        Ok(())
    }

    // Repays debt to the pool
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

        self.adjust_accumulated_fees_with_computed_per_operation(&repay_result.operation_fees)?;

        Ok(())
    }

    // Removes collateral from the pool
    pub fn remove_collateral(
        &mut self,
        e: &Env,
        remove_collateral_result: &RemoveCollateralResult,
    ) -> Result<(), MCError> {
        self.adjust_total_collateral(
            e,
            remove_collateral_result.collateral_decrease.checked_neg().map_over_or_underflow()?,
        )?;

        self.adjust_accumulated_fees_with_computed_per_operation(
            &remove_collateral_result.operation_fees,
        )?;

        Ok(())
    }

    // Repays liquidated obligation's debt to the pool
    pub fn liquidation_repay_debt(
        &mut self,
        e: &Env,
        liquidation_result: &LiquidationResult,
    ) -> Result<(), MCError> {
        self.adjust_total_borrowed(
            e,
            liquidation_result.debt_repaid.checked_neg().map_over_or_underflow()?,
        )?;
        self.adjust_total_d_tokens(
            e,
            liquidation_result.d_tokens_burned.checked_neg().map_over_or_underflow()?,
        )?;

        Ok(())
    }

    // Removes liquidated plain collateral from the pool as a part of the liquidator's incentive
    pub fn liquidation_redeem_collateral(
        &mut self,
        e: &Env,
        liquidation_result: &LiquidationResult,
    ) -> Result<(), MCError> {
        self.adjust_total_collateral(
            e,
            liquidation_result.plain_collateral_seized.checked_neg().map_over_or_underflow()?,
        )?;

        Ok(())
    }

    // ---- Shares <—> Tokens Calculations ----

    pub fn compute_tokens_from_d_tokens_ceil(
        &self,
        e: &Env,
        d_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        let tokens = Self::compute_tokens_from_shares_ceil(
            e,
            d_tokens_amount,
            self.total_d_tokens,
            self.total_borrowed,
        )?;

        Ok(tokens)
    }

    // # Returns
    // `(d_tokens, tokens)`: `[(i128, i128)]`, where `tokens` is the amount that is guaranteed to be represented by the `d_tokens`
    pub fn compute_d_tokens_from_tokens_ceil(
        &self,
        e: &Env,
        tokens_amount: i128,
    ) -> Result<(i128, i128), MCError> {
        let d_tokens = Self::compute_shares_from_tokens_ceil(
            tokens_amount,
            self.total_d_tokens,
            self.total_borrowed,
        )?;

        let new_d_tokens = self.total_d_tokens.checked_add(d_tokens).map_over_or_underflow()?;
        let new_total_borrowed =
            self.total_borrowed.checked_add(tokens_amount).map_over_or_underflow()?;
        let tokens =
            Self::compute_tokens_from_shares_ceil(e, d_tokens, new_d_tokens, new_total_borrowed)?;

        Ok((d_tokens, tokens))
    }

    pub fn compute_d_tokens_from_tokens_floor(&self, tokens_amount: i128) -> Result<i128, MCError> {
        let d_tokens = Self::compute_shares_from_tokens_floor(
            tokens_amount,
            self.total_d_tokens,
            self.total_borrowed,
        )?;

        Ok(d_tokens)
    }

    pub fn compute_tokens_from_j_tokens_floor(
        &self,
        e: &Env,
        j_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        let tokens = Self::compute_tokens_from_shares_floor(
            e,
            j_tokens_amount,
            self.total_j_tokens,
            self.total_supply()?,
        )?;

        Ok(tokens)
    }

    pub fn compute_tokens_from_j_tokens_ceil(
        &self,
        e: &Env,
        j_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        let tokens = Self::compute_tokens_from_shares_ceil(
            e,
            j_tokens_amount,
            self.total_j_tokens,
            self.total_supply()?,
        )?;

        Ok(tokens)
    }

    // # Returns
    // `(j_tokens, tokens)`: `[(i128, i128)]`, where `tokens` is the amount that is guaranteed to be represented by the `j_tokens`
    pub fn compute_j_tokens_from_tokens_floor(
        &self,
        e: &Env,
        tokens_amount: i128,
    ) -> Result<(i128, i128), MCError> {
        let total_supply = self.total_supply()?;

        let j_tokens = Self::compute_shares_from_tokens_floor(
            tokens_amount,
            self.total_j_tokens,
            total_supply,
        )?;

        let new_j_tokens = self.total_j_tokens.checked_add(j_tokens).map_over_or_underflow()?;
        let new_total_supply = total_supply.checked_add(tokens_amount).map_over_or_underflow()?;
        let tokens =
            Self::compute_tokens_from_shares_floor(e, j_tokens, new_j_tokens, new_total_supply)?;

        Ok((j_tokens, tokens))
    }

    pub fn compute_j_tokens_from_tokens_ceil(&self, tokens_amount: i128) -> Result<i128, MCError> {
        let j_tokens = Self::compute_shares_from_tokens_ceil(
            tokens_amount,
            self.total_j_tokens,
            self.total_supply()?,
        )?;

        Ok(j_tokens)
    }

    // Computes the number of tokens proportional to the given share of the tokens in the pool ceiled.
    // Intended to be used for both `jTokens` and `dTokens` related calculations
    pub fn compute_tokens_from_shares_ceil(
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
            .fixed_mul_ceil(total_tokens_amount, total_shares_amount)
            .map_over_or_underflow()?;

        Ok(tokens_amount)
    }

    // Computes the number of tokens proportional to the given share of the tokens in the pool floored.
    // Intended to be used for both `jTokens` and `dTokens` related calculations
    pub fn compute_tokens_from_shares_floor(
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

    // Computes the shares amount which must be issued or burnt from a specific obligation based on
    // the provided tokens amount ceiled. Intended to be used for both `jTokens` and `dTokens` related
    // calculations
    fn compute_shares_from_tokens_ceil(
        tokens_amount: i128,
        total_shares_amount: i128,
        total_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        if tokens_amount == 0 {
            return Ok(0);
        }

        let shares_amount = if total_shares_amount == 0 {
            INITIAL_SHARES_AMOUNT
        } else {
            // This must hold when issuing new shares:
            // ----
            // shares_to_issue / (shares_to_issue + prev_total_shares) = tokens_added_amount /
            // (tokens_added_amount + prev_total_tokens_amount)
            // ----
            // Which implies:
            // ----
            //   shares_to_issue = prev_total_shares * (tokens_added_amount / prev_total_tokens_amount)
            // ----
            // This must hold when burning issued shares:
            // ----
            //   shares_to_burn = prev_total_shares * (tokens_removed_amount / prev_total_tokens_amount)
            // ----
            total_shares_amount
                .fixed_div_ceil(total_tokens_amount, tokens_amount)
                .map_over_or_underflow()?
        };

        Ok(shares_amount)
    }

    // Computes the shares amount which must be issued or burnt from a specific obligation based on
    // the provided tokens amount floored. Intended to be used for both `jTokens` and `dTokens` related
    // calculations
    fn compute_shares_from_tokens_floor(
        tokens_amount: i128,
        total_shares_amount: i128,
        total_tokens_amount: i128,
    ) -> Result<i128, MCError> {
        if tokens_amount == 0 {
            return Ok(0);
        }

        let shares_amount = if total_shares_amount == 0 {
            INITIAL_SHARES_AMOUNT
        } else {
            total_shares_amount
                .fixed_div_floor(total_tokens_amount, tokens_amount)
                .map_over_or_underflow()?
        };

        Ok(shares_amount)
    }

    // ---- `require_` circuits ----

    pub fn require_total_available(&self, required: i128) -> Result<(), MCError> {
        if required > self.total_available {
            return Err(MCError::NotEnoughPoolFunds);
        }

        Ok(())
    }

    pub fn require_does_not_exist(e: &Env, pool_address: &Address) -> Result<(), MCError> {
        if Self::exists(e, pool_address) {
            return Err(MCError::InvalidInitialization);
        }

        Ok(())
    }

    pub fn require_deposit_enabled(&self) -> Result<(), MCError> {
        if !self.config.status.is_deposit_enabled() {
            return Err(MCError::OperationForbiddenOnPool);
        }

        Ok(())
    }

    pub fn require_borrow_enabled(&self) -> Result<(), MCError> {
        if !self.config.status.is_borrow_enabled() {
            return Err(MCError::OperationForbiddenOnPool);
        }

        Ok(())
    }

    pub fn require_add_collateral_enabled(&self) -> Result<(), MCError> {
        if !self.config.status.is_add_collateral_enabled() {
            return Err(MCError::OperationForbiddenOnPool);
        }

        Ok(())
    }

    pub fn require_remove_collateral_enabled(&self) -> Result<(), MCError> {
        if !self.config.status.is_remove_collateral_enabled() {
            return Err(MCError::OperationForbiddenOnPool);
        }

        Ok(())
    }

    pub fn require_withdraw_enabled(&self) -> Result<(), MCError> {
        if !self.config.status.is_withdraw_enabled() {
            return Err(MCError::OperationForbiddenOnPool);
        }

        Ok(())
    }

    pub fn require_repay_enabled(&self) -> Result<(), MCError> {
        if !self.config.status.is_repay_enabled() {
            return Err(MCError::OperationForbiddenOnPool);
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

    pub fn require_collateral_is_seizable(&self) -> Result<(), MCError> {
        if self.config.health_config.close_ltv_bps == 0 {
            return Err(MCError::AssetCannotBeUsedAsCollateral);
        }

        Ok(())
    }

    // ---- MISC ----

    pub fn get_pool_data(self, e: &Env) -> Result<PoolData, MCError> {
        let apy = self.get_apy()?;
        let j_token_rate_floor_bps = self.get_j_token_rate_floor()?;
        let d_token_rate_ceil_bps = self.get_d_token_rate_ceil()?;
        let total_supply = self.total_supply()?;
        let total_available_adjusted = self.total_available()?;
        let oracle_asset_price = oracle::get_asset_price(e, &self.token_address)?;

        Ok(PoolData {
            pool: self,
            apy,
            j_token_rate_floor_bps,
            d_token_rate_ceil_bps,
            total_supply,
            total_available_adjusted,
            oracle_asset_price,
        })
    }

    // Returns a `jTokenRate` with floor rounding in basis points (i.e., 10001 for 1.0001, etc)
    fn get_j_token_rate_floor(&self) -> Result<i128, MCError> {
        let total_supply = self.total_supply()?;
        let total_j_tokens = self.total_j_tokens;

        if total_j_tokens == 0 {
            Ok(0)
        } else {
            total_supply.fixed_div_floor(total_j_tokens, BPS_FACTOR).map_over_or_underflow()
        }
    }

    // Returns a `dTokenRate` with ceil rounding in basis points (i.e. 10023 for 1.0023, etc)
    fn get_d_token_rate_ceil(&self) -> Result<i128, MCError> {
        let total_borrowed = self.total_borrowed;
        let total_d_tokens = self.total_d_tokens;

        if total_d_tokens == 0 {
            Ok(0)
        } else {
            total_borrowed.fixed_div_ceil(total_d_tokens, BPS_FACTOR).map_over_or_underflow()
        }
    }

    // Computes the maximum available amount for borrowing that doesn't exceed the utilization
    // ratio limit on a pool
    pub fn compute_available_utilization_ratio_cap_borrow(&self, e: &Env) -> Result<i128, MCError> {
        let total_supply = self.total_supply()?;
        let utilization_ratio = self.compute_utilization_ratio_bps()?;

        if utilization_ratio > self.config.health_config.utilization_ratio_limit_bps {
            // NB: This can happen when the `total_borrowed` amount on a pool has accrued over time
            // by itself, so we emit an event for now. We can agree to stop
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
            .fixed_mul_floor(available_percentage_to_borrow_bps, BPS_FACTOR)
            .map_over_or_underflow()
    }

    pub fn operation_fees_sum(&self) -> i128 {
        self.operation_fees_sum
    }

    pub fn total_available(&self) -> Result<i128, MCError> {
        Ok(i128::max(
            self.total_available.checked_sub(self.take_rate_fees_sum).map_over_or_underflow()?,
            0,
        ))
    }

    // Calculates total debt
    pub fn total_debt(&self) -> Result<i128, MCError> {
        Ok(self.total_borrowed)
    }

    // Calculates total supply (available + total_borrowed - accumulated reserve fees)
    pub fn total_supply(&self) -> Result<i128, MCError> {
        self.total_available()?.checked_add(self.total_borrowed).map_over_or_underflow()
    }

    // Tries to get the pool from the contract's storage
    //
    // # Returns
    // - [`Ok(Pool)`] if a pool with the given address exists in the contract's storage
    // - [`Err(MCError::PoolDoesNotExist)`] otherwise
    pub fn try_get(e: &Env, pool_address: &Address) -> Result<Self, MCError> {
        storage::get_pool(e, pool_address).ok_or(MCError::PoolDoesNotExist)
    }

    pub fn get_all(e: &Env) -> Vec<Address> {
        storage::get_all_pools(e)
    }

    fn exists(e: &Env, address: &Address) -> bool {
        storage::pool_exists(e, address)
    }

    // Queues in pool's config update
    //
    // # WARNING
    // Modifies the contract's storage
    pub fn queue_in_config_update(&self, e: &Env, config: &PoolConfig) -> Result<(), MCError> {
        storage::queue_in_pool_config_update(e, &self.pool_address, config)
    }

    // Removes pool's config update from the queue
    //
    // # WARNING
    // Modifies the contract's storage
    pub fn remove_pool_config_update(&self, e: &Env) -> Result<(), MCError> {
        storage::remove_pool_config_update(e, &self.pool_address)
    }

    // Applies the pool's config update from the queue if it exists and is matured
    //
    // # WARNING
    // Modifies the contract's storage
    pub fn apply_pool_config_update(&mut self, e: &Env) -> Result<(), MCError> {
        let update_pool_config_period =
            storage::get_update_in_queue_period(e).expect("Must be present for an owned pool");
        let pool_config_update = self.get_pool_config_update(e)?;
        let current_time = e.ledger().timestamp();

        if current_time
            < pool_config_update
                .queued_in_timestamp
                .checked_add(update_pool_config_period)
                .map_over_or_underflow()?
        {
            return Err(MCError::PoolConfigUpdateIsNotYetApplicable);
        }

        self.config = pool_config_update.new_config;
        self.set(e);
        storage::remove_pool_config_update(e, &self.pool_address)?;

        Ok(())
    }

    // Gets the pool's config update from the queue if it exists
    pub fn get_pool_config_update(&self, e: &Env) -> Result<PoolUpdate, MCError> {
        storage::get_pool_config_update(e, &self.pool_address)
            .ok_or(MCError::PoolDoesNotHaveQueuedInConfigUpdate)
    }

    pub fn compute_total_collateral_value(&self, e: &Env) -> Result<i128, MCError> {
        let deposited_tokens = self.compute_tokens_from_j_tokens_floor(e, self.total_j_tokens)?;
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

    // Refreshes the pool with the contract's storage data
    pub fn refresh(&mut self, e: &Env) -> Result<(), MCError> {
        let Some(refreshed_pool) = storage::get_pool(e, &self.pool_address) else {
            events::pool_is_unexpectedly_missing_in_storage(e, &self.pool_address);

            return Err(MCError::InternalError);
        };
        *self = refreshed_pool;

        Ok(())
    }

    // Saves/updates pool in the contract's storage
    //
    // # WARNING
    // Modifies the contract's storage
    pub fn set(&self, e: &Env) {
        storage::set_pool(e, &self.pool_address, self);
    }

    // Registers pool in the pools list
    //
    // # WARNING
    // Modifies the contract's storage
    pub fn register(&self, e: &Env) -> u32 {
        storage::register_pool(e, &self.pool_address)
    }
}

#[contracttype]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PoolFeeConfig {
    pub borrow_fee_bps: u32,
    pub flash_loan_fee_bps: u32,

    pub deposit_fee_bps: u32,
    pub withdraw_fee_bps: u32,
    // Additional scalar (in basis points) used for the additional withdrawal fee when the utilization ratio
    // exceeds `utilization_ratio_limit_bps`
    pub withdraw_scarcity_fee_sc_bps: u32,
    pub add_collateral_fee_bps: u32,
    pub remove_collateral_fee_bps: u32,
    pub repay_fee_bps: u32,

    // Borrow rate percentage that is taken from the suppliers and distributed among the `take_rate` beneficiaries
    pub take_rate_bps: u32,
    // A map of beneficiaries who split the `take_rate` and their distribution proportions(in basis points). Proportions must add up to 10_000
    pub take_rate_beneficiaries: Option<Map<Address, u32>>,
    // A map of beneficiaries who split the `origination fee` left after removing the possible referrer's cut and their distribution proportions.
    // Proportions must add up to 10_000
    pub operation_fee_beneficiaries: Option<Map<Address, u32>>,
    // A map of allowed referrers and their immediately received percentage of the origination fee
    pub referrers: Option<Map<Address, u32>>,
}

impl Default for PoolFeeConfig {
    fn default() -> Self {
        Self {
            borrow_fee_bps: DEFAULT_BORROW_FEE_BPS,
            flash_loan_fee_bps: DEFAULT_FLASH_LOAN_FEE_BPS,

            deposit_fee_bps: DEFAULT_DEPOSIT_FEE_BPS,
            withdraw_fee_bps: DEFAULT_WITHDRAW_FEE_BPS,
            withdraw_scarcity_fee_sc_bps: DEFAULT_WITHDRAW_SCARCITY_FEE_SCALAR_BPS,
            add_collateral_fee_bps: DEFAULT_ADD_COLLATERAL_FEE_BPS,
            remove_collateral_fee_bps: DEFAULT_REMOVE_COLLATERAL_FEE_BPS,
            repay_fee_bps: DEFAULT_REPAY_FEE_BPS,

            take_rate_bps: DEFAULT_TAKE_RATE_BPS,
            take_rate_beneficiaries: None,
            operation_fee_beneficiaries: None,
            referrers: None,
        }
    }
}

impl PoolFeeConfig {
    fn validate(&self) -> Result<(), &str> {
        let &Self {
            borrow_fee_bps,
            flash_loan_fee_bps,
            deposit_fee_bps,
            withdraw_fee_bps,
            add_collateral_fee_bps,
            remove_collateral_fee_bps,
            repay_fee_bps,
            take_rate_bps,
            take_rate_beneficiaries,
            operation_fee_beneficiaries,
            ..
        } = &self;

        let individual_fees = [
            borrow_fee_bps,
            flash_loan_fee_bps,
            deposit_fee_bps,
            withdraw_fee_bps,
            add_collateral_fee_bps,
            remove_collateral_fee_bps,
            repay_fee_bps,
        ];

        for &fee in individual_fees {
            if fee as i128 > BPS_FACTOR {
                return Err("Individual fees must not exceed 100%");
            }
        }

        if *take_rate_bps as i128 > BPS_FACTOR {
            return Err("Take Rate must not exceed 100%");
        }

        if let Some(take_rate_beneficiaries) = take_rate_beneficiaries {
            if take_rate_beneficiaries.is_empty() {
                return Err("Provided Take Rate beneficiaries are empty");
            }

            let mut share_sum: u32 = 0;
            for (_, share_bps) in take_rate_beneficiaries.iter() {
                share_sum = match share_sum.checked_add(share_bps) {
                    Some(sum) => sum,
                    None => return Err("Take Rate shares overflow"),
                };
            }

            if share_sum as i128 != BPS_FACTOR {
                return Err("Provided Take Rate shares don't add up to 100%");
            }
        }

        if let Some(operation_fee_beneficiaries) = operation_fee_beneficiaries {
            if operation_fee_beneficiaries.is_empty() {
                return Err("Provided Operation fees beneficiaries are empty");
            }

            let mut share_sum: u32 = 0;
            for (_, share_bps) in operation_fee_beneficiaries.iter() {
                share_sum = match share_sum.checked_add(share_bps) {
                    Some(sum) => sum,
                    None => return Err("Operation fee shares overflow"),
                };
            }

            if share_sum as i128 != BPS_FACTOR {
                return Err("Provided Operation fees shares don't add up to 100%");
            }
        }

        Ok(())
    }
}

#[contracttype]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub struct PoolStatus {
    pub flags: u32,
}

impl Default for PoolStatus {
    fn default() -> Self {
        Self { flags: POOL_STATUS_ALL_ENABLED }
    }
}

impl PoolStatus {
    pub fn new_all_disabled() -> Self {
        Self { flags: 0 }
    }

    pub fn is_deposit_enabled(&self) -> bool {
        (self.flags & POOL_STATUS_DEPOSIT_ENABLED) > 0
    }

    pub fn is_borrow_enabled(&self) -> bool {
        (self.flags & POOL_STATUS_BORROW_ENABLED) > 0
    }

    pub fn is_withdraw_enabled(&self) -> bool {
        (self.flags & POOL_STATUS_WITHDRAW_ENABLED) > 0
    }

    pub fn is_repay_enabled(&self) -> bool {
        (self.flags & POOL_STATUS_REPAY_ENABLED) > 0
    }

    pub fn is_add_collateral_enabled(&self) -> bool {
        (self.flags & POOL_STATUS_ADD_COLLATERAL_ENABLED) > 0
    }

    pub fn is_remove_collateral_enabled(&self) -> bool {
        (self.flags & POOL_STATUS_REMOVE_COLLATERAL_ENABLED) > 0
    }
}

#[contracttype]
#[derive(Default, Clone, Debug, Eq, PartialEq)]
pub struct PoolConfig {
    pub status: PoolStatus,
    pub fee_config: PoolFeeConfig,
    pub health_config: PoolHealthConfig,
    pub accrual_model: AccrualModel,
    pub interest_rate_model: InterestRateModel,
    pub ir_reactivity_constant: u32,
}

impl PoolConfig {
    pub fn validate(&self) -> Result<(), MCError> {
        let PoolConfig { health_config, fee_config, ir_reactivity_constant, .. } = self;

        if health_config.validate().is_err()
            || fee_config.validate().is_err()
            || !(MIN_REACTIVITY_CONSTANT..=MAX_REACTIVITY_CONSTANT).contains(ir_reactivity_constant)
        {
            return Err(MCError::InvalidLoanPoolConfig);
        }

        Ok(())
    }
}

#[contracttype]
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct PoolHealthConfig {
    // The maximum amount of supplied tokens that can be supplied in the pool(i.e., `available` +
    // `total_borrowed`). 0 denotes unlimited supply
    pub supply_limit: i128,
    // The maximum utilization ratio that is allowed to be reached via borrowing
    pub utilization_ratio_limit_bps: i128,
    // Basis points of the pool's total supply that can be withdrawn in a single operation when the pool's utilization ratio exceeds
    // `utilization_ratio_limit_bps`
    pub withdraw_scarcity_limit_bps: i128,
    // Cooldown period(in seconds) required between a pair of sequential withdrawals when the pool's utilization ratio exceeds
    // `utilization_ratio_limit_bps`
    pub withdraw_scarcity_cooldown_s: u64,
    // The maximum percentage of an asset's value that can be borrowed in basis points(e.g, 7000 =
    // 70%, etc) with respect to a total obligation's collateral value
    pub open_ltv_bps: i128,
    // The maximum percentage of an asset's value that can be held in an individual obligation in
    // basis points with respect to a total obligation's collateral value. LTV greater than
    // that makes borrow position eligible to liquidation
    pub close_ltv_bps: i128,
    // The factor used to calculate the current borrow limit by multiplying the collateral value
    // by it before subtracting this value from the obligation's max borrow limit. Volatile
    // assets' pools are expected to have this value set way above 100%
    pub liability_factor_bps: i128,
    // Maximum percentage of a borrower's debt that can be liquidated at once
    pub liquidation_close_factor_bps: i128,
    // Maximum additional value in the received tokens that can be given to liquidators when purchasing collateral
    pub max_liquidation_incentive_bps: i128,
    // LTV calculated for unparameterized obligation positions(i.e., no openLTV/liability factors scaling) that marks
    // position as insolvent. Used as a means to avoid unprofitable health-improving liquidations
    pub insolvency_ltv_bps: i128,
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
            max_liquidation_incentive_bps: DEFAULT_LIQUIDATION_INCENTIVE_BPS,
            withdraw_scarcity_limit_bps: DEFAULT_WITHDRAW_SCARCITY_LIMIT_BPS,
            withdraw_scarcity_cooldown_s: DEFAULT_WITHDRAW_SCARCITY_COOLDOWN_SECS,
            insolvency_ltv_bps: DEFAULT_INSOLVENCY_LTV_BPS,
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
            max_liquidation_incentive_bps,
            withdraw_scarcity_limit_bps,
            withdraw_scarcity_cooldown_s,
            insolvency_ltv_bps,
        } = self;

        if supply_limit.is_negative() {
            return Err("Supply limit must be non-negative");
        }

        if !is_valid_bps_percent(utilization_ratio_limit_bps) {
            return Err("Utilization ratio limit must be between 0% and 100%");
        }

        if !is_valid_bps_percent(open_ltv_bps) {
            return Err("Open LTV must be between 0% and 100%");
        }

        if !is_valid_bps_percent(close_ltv_bps) {
            return Err("Close LTV must be between 0% and 100%");
        }

        if close_ltv_bps < open_ltv_bps {
            return Err("Open LTV mustn't be bigger than close LTV");
        }

        if !(BPS_FACTOR..=(MAX_LIABILITY_FACTOR_BPS)).contains(&liability_factor_bps) {
            return Err("Invalid liability factor");
        }

        if !is_valid_bps_percent(liquidation_close_factor_bps) {
            return Err("Liquidation close factor must be between 0% and 100%");
        }

        if !is_valid_bps_percent(max_liquidation_incentive_bps) {
            return Err("Liquidation incentive must be between 0% and 100%");
        }

        if !is_valid_bps_percent(withdraw_scarcity_limit_bps) {
            return Err("Withdrawal scarcity limit must be between 0% and 100%");
        }

        if !(0..MAX_WITHDRAW_SCARCITY_COOLDOWN_SECS).contains(&withdraw_scarcity_cooldown_s) {
            return Err("Withdrawal scarcity cooldown seconds exceed limit");
        }

        if !(MIN_INSOLVENCY_LTV_BPS..=MAX_INSOLVENCY_LTV_BPS).contains(&insolvency_ltv_bps) {
            return Err("Insolvency LTV bps must be within 95% to 100%");
        }

        Ok(())
    }
}

#[contracttype]
#[derive(Debug, Clone)]
pub struct PoolBootstrapPeriod {
    // Total provided bootstrap amount
    pub total_amount: i128,
    // Remaining bootstrap amount
    pub remaining_amount: i128,
}

impl PoolBootstrapPeriod {
    pub fn new(total_amount: i128) -> Self {
        Self { total_amount, remaining_amount: total_amount }
    }
}

fn is_valid_bps_percent(value: i128) -> bool {
    (0..=BPS_FACTOR).contains(&value)
}
