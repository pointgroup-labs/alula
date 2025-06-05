use {
    crate::{
        constants::{
            LCError, BPS_IN_PERCENT, DEFAULT_BASE_RATE_PER_SECOND, DEFAULT_CLOSE_FACTOR,
            DEFAULT_LIQUIDATION_SPREAD, DEFAULT_OPTIMAL_UTILIZATION_RATIO, DEFAULT_RESERVE_RATIO,
            DEFAULT_SLOPE1, DEFAULT_SLOPE2, HEALTH_FACTOR_THRESHOLD_BPS, INDIVIDUAL_BUMP,
            INDIVIDUAL_THRESHOLD, INSTANCE_BUMP, INSTANCE_THRESHOLD, REFLECTOR_TESTNET_ADDRESS,
            SHARED_BUMP, SHARED_THRESHOLD,
        },
        oracle,
    },
    soroban_sdk::{contracttype, Address, Env, Map, String, Symbol},
};

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
pub struct GlobalState {
    pub admin: Address,
    pub status: bool,
    pub liquidation_threshold_bps: i128,
    // TODO: Oracle addresses?
}

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(PoolAddress),
    Obligation(UserAddress),
    Accrual,
    AllPools,
    // AllUsers,
}

#[contracttype]
#[derive(Debug)]
pub struct Pool {
    /// The address of the token associated with the pool.
    pub token_address: Address,
    /// The ticker symbol of the associated token, which is used to identify the token in the pool.
    pub token_ticker: Symbol,
    /// The total amount of borrowed assets
    pub total_borrowed: i128,
    /// The total amount of deposited assets that accrue interest
    pub total_supply: i128,
    /// The total amount of deposited collateral assets that don't accrue interest
    pub total_collateral: i128,
    /// Configuration settings for the pool.
    pub config: PoolConfig,
    /// Details about the accrual settings and state of the pool, which govern how interest is computed or accumulated
    /// over time for deposited and borrowed assets.
    pub accrual: Accrual,
}

#[contracttype]
#[derive(Debug)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
#[contracttype]
pub struct Obligation {
    pub deposits: Map<PoolAddress, DepositObligation>,
    pub borrows: Map<PoolAddress, BorrowObligation>,
}

impl Obligation {
    pub fn new(e: &Env) -> Self {
        Self {
            deposits: Map::new(e),
            borrows: Map::new(e),
        }
    }

    pub fn accrue_interest(&mut self, e: &Env) -> Result<(), LCError> {
        for (pool_address, borrow_obligation) in self.borrows.iter() {
            self.accrue_borrow_obligation(e, &pool_address, borrow_obligation)?;
        }

        for (pool_address, deposit_obligation) in self.deposits.iter() {
            self.accrue_deposit_obligation(e, &pool_address, deposit_obligation)?;
        }

        Ok(())
    }

    pub fn is_healthy(&self, e: &Env) -> Result<bool, LCError> {
        Ok(self.compute_health_factor_bps(e)? >= HEALTH_FACTOR_THRESHOLD_BPS)
    }

    pub fn is_empty(&self) -> bool {
        self.deposits.is_empty() && self.borrows.is_empty()
    }

    pub fn compute_health_factor_bps(&self, e: &Env) -> Result<i128, LCError> {
        let liquidation_threshold_bps = get_global_state(e).liquidation_threshold_bps;

        let reflector_address =
            Address::from_string(&String::from_str(e, REFLECTOR_TESTNET_ADDRESS));
        let reflector_contract = oracle::Client::new(e, &reflector_address);

        let (mut collateral_value_sum, mut borrowed_value_sum) = (0i128, 0i128);

        for (pool_address, deposit_obligation) in self.deposits.iter() {
            let DepositObligation {
                deposited,
                collateral,
                ..
            } = deposit_obligation;

            // TODO: Get it from the token client?
            let ticker = get_pool_ticker(e, &pool_address)?;
            let asset = oracle::Asset::Other(ticker); // TODO: What about XLM?
            let price = reflector_contract
                .lastprice(&asset)
                .ok_or(LCError::OracleDoesNotKnowAssetPrice)?
                .price;

            // Add deposited value as a collateral
            collateral_value_sum = collateral_value_sum
                .checked_add(
                    price
                        .checked_mul(deposited)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;

            // Add plain collateral
            collateral_value_sum = collateral_value_sum
                .checked_add(
                    price
                        .checked_mul(collateral)
                        .ok_or(LCError::OverOrUnderflow)?,
                )
                .ok_or(LCError::OverOrUnderflow)?;
        }

        for (pool_address, borrow_obligation) in self.borrows.iter() {
            let borrowed = borrow_obligation.borrowed;

            let ticker = get_pool_ticker(e, &pool_address).expect("Pool must exist at this point");
            let asset = oracle::Asset::Other(ticker);
            let price = reflector_contract
                .lastprice(&asset)
                .ok_or(LCError::OracleDoesNotKnowAssetPrice)?
                .price;

            borrowed_value_sum = borrowed_value_sum
                .checked_add(
                    price
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

    fn accrue_borrow_obligation(
        &mut self,
        e: &Env,
        pool_address: &Address,
        borrow_obligation: BorrowObligation,
    ) -> Result<(), LCError> {
        let borrow_accrual = accrue_interest(e, pool_address)?.borrow_accrual;

        let borrowed = borrow_obligation.borrowed;
        let new_borrowed = borrowed
            .checked_mul(borrow_accrual / 10)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(borrow_obligation.last_accrual / 10)
            .ok_or(LCError::OverOrUnderflow)?;

        let new_borrow_obligation = BorrowObligation {
            borrowed: new_borrowed,
            last_accrual: borrow_accrual,
        };

        self.borrows
            .set(pool_address.clone(), new_borrow_obligation);

        Ok(())
    }

    fn accrue_deposit_obligation(
        &mut self,
        e: &Env,
        pool_address: &Address,
        deposit_obligation: DepositObligation,
    ) -> Result<(), LCError> {
        let deposit_accrual = accrue_interest(e, pool_address)?.deposit_accrual;

        let deposited = deposit_obligation.deposited;
        let new_deposited = deposited
            .checked_mul(deposit_accrual)
            .ok_or(LCError::OverOrUnderflow)?
            .checked_div(deposit_obligation.last_accrual)
            .ok_or(LCError::OverOrUnderflow)?;

        let new_deposit_obligation = DepositObligation {
            collateral: deposit_obligation.collateral,
            deposited: new_deposited,
            last_accrual: deposit_accrual,
        };

        self.deposits
            .set(pool_address.clone(), new_deposit_obligation);

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Default)]
#[contracttype]
pub struct BorrowObligation {
    pub borrowed: i128,
    /// The numerical value that is used to determine the scaling factor required for updating the position amount
    /// with interest, i.e. new_borrowed = (current_accrual \ last_accrual) * borrowed
    pub last_accrual: i128,
}

#[derive(Debug, Clone, Copy, Default)]
#[contracttype]
pub struct DepositObligation {
    pub collateral: i128,
    pub deposited: i128,
    /// The numerical value that is used to determine the scaling factor required for updating the position amount
    /// with interest, i.e. new_deposited = (current_accrual \ last_accrual) * deposited
    pub last_accrual: i128,
}

impl DepositObligation {
    pub fn is_empty(&self) -> bool {
        self.deposited == 0 && self.collateral == 0
    }
}

#[contracttype]
#[derive(Debug)]
pub struct Accrual {
    pub timestamp: u64,
    pub borrow_accrual: i128,
    pub deposit_accrual: i128,
}

/// Instance bumper
pub fn extend_instance_storage(e: &Env) {
    e.storage()
        .instance()
        .extend_ttl(INSTANCE_THRESHOLD, INSTANCE_BUMP);
}

/// Persistent individual resource bumper
pub fn extend_individual_storage(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, INDIVIDUAL_THRESHOLD, INDIVIDUAL_BUMP);
}

/// Persistent shared resource bumper
pub fn extend_shared_storage(e: &Env, key: &DataKey) {
    e.storage()
        .persistent()
        .extend_ttl(key, SHARED_THRESHOLD, SHARED_BUMP);
}

#[allow(unused)]
pub fn get_global_state(e: &Env) -> GlobalState {
    extend_instance_storage(e);

    e.storage()
        .instance()
        .get(&DataKey::GlobalState)
        .expect("Global State must be instantiated at this point")
}

pub fn set_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);

    extend_instance_storage(e);
}

// --- Pool ---

pub fn get_all_pools(e: &Env) -> soroban_sdk::Vec<PoolAddress> {
    let res = e.storage().persistent().get(&DataKey::AllPools);
    if let Some(pools) = res {
        extend_shared_storage(e, &DataKey::AllPools);
        pools
    } else {
        soroban_sdk::Vec::new(e)
    }
}

pub fn register_pool(e: &Env, pool_address: &Address) -> bool {
    let mut all_pools = get_all_pools(e);
    // for existing_pool in all_pools.iter() {
    //     if &existing_pool == pool_address {
    //         return false;
    //     }
    // }
    all_pools.push_back(pool_address.clone());
    e.storage().persistent().set(&DataKey::AllPools, &all_pools);
    extend_shared_storage(e, &DataKey::AllPools);
    true
}

pub fn set_pool(e: &Env, pool_address: &Address, pool: &Pool) -> Result<(), LCError> {
    e.storage()
        .persistent()
        .set(&DataKey::Pool(pool_address.clone()), pool);

    extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));

    Ok(())
}

pub fn pool_exists(e: &Env, pool_address: &Address) -> bool {
    let res = e
        .storage()
        .persistent()
        .has(&DataKey::Pool(pool_address.clone()));

    if res {
        extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
    }

    res
}

pub fn get_pool(e: &Env, pool_address: &Address) -> Option<Pool> {
    let res = e
        .storage()
        .persistent()
        .get(&DataKey::Pool(pool_address.clone()));

    if res.is_some() {
        extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
    }

    res
}

pub fn get_pool_ticker(e: &Env, pool_address: &Address) -> Result<Symbol, LCError> {
    let pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    Ok(pool.token_ticker)
}

pub fn set_pool_data(e: &Env, pool_address: &Address, pool_data: &Pool) {
    e.storage()
        .persistent()
        .set(&DataKey::Pool(pool_address.clone()), pool_data);

    extend_shared_storage(e, &DataKey::Pool(pool_address.clone()));
}

pub fn adjust_pool_borrowed(e: &Env, pool_address: &Address, amount: i128) -> Result<(), LCError> {
    let mut pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    pool.total_borrowed = pool
        .total_borrowed
        .checked_add(amount)
        .ok_or(LCError::OverOrUnderflow)?;
    set_pool_data(e, pool_address, &pool);

    Ok(())
}

pub fn adjust_pool_deposited(e: &Env, pool_address: &Address, amount: i128) -> Result<(), LCError> {
    let mut pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    pool.total_supply = pool
        .total_supply
        .checked_add(amount)
        .ok_or(LCError::OverOrUnderflow)?;
    set_pool_data(e, pool_address, &pool);

    Ok(())
}

pub fn adjust_pool_collateral(
    e: &Env,
    pool_address: &Address,
    amount: i128,
) -> Result<(), LCError> {
    let mut pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;

    pool.total_collateral = pool
        .total_collateral
        .checked_add(amount)
        .ok_or(LCError::OverOrUnderflow)?;
    set_pool_data(e, pool_address, &pool);

    Ok(())
}

// --- Obligation ---
pub fn set_obligation(e: &Env, user: &Address, obligation: &Obligation) {
    e.storage()
        .persistent()
        .set(&DataKey::Obligation(user.clone()), obligation);

    extend_individual_storage(e, &DataKey::Obligation(user.clone()));
}

pub fn get_obligation(e: &Env, user: &Address) -> Option<Obligation> {
    let res = e
        .storage()
        .persistent()
        .get(&DataKey::Obligation(user.clone()));

    if res.is_some() {
        extend_individual_storage(e, &DataKey::Obligation(user.clone()));
    }

    res
}

pub fn remove_obligation(e: &Env, user: &Address) {
    e.storage()
        .persistent()
        .remove(&DataKey::Obligation(user.clone()));
}

// --- Accrual ---
pub fn accrue_interest(e: &Env, pool_address: &Address) -> Result<Accrual, LCError> {
    let mut pool = get_pool(e, pool_address).ok_or(LCError::PoolDoesNotExist)?;
    pool.accrue_interest(e)?;

    set_pool(e, pool_address, &pool)?;

    Ok(pool.accrual)
}
