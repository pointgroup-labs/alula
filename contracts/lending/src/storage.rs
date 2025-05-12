use {
    crate::error::LendingContractError,
    soroban_sdk::{contracttype, Address, Env, Map},
};

#[contracttype]
pub struct GlobalState {
    pub admin: Address,
    pub status: bool,
    // @TODO: Oracle address + ....
}

pub type PoolAddress = Address;
pub type UserAddress = Address;

#[contracttype]
pub enum DataKey {
    GlobalState,
    Pool(PoolAddress),
    PoolConfig(PoolAddress),
    Obligation(UserAddress),
    // @TODO: We also must be able to retrieve all pools and all user addresses
}

#[contracttype]
#[derive(Debug)]
pub struct PoolData {
    pub token_address: Address, // <---- this will be read frequently
    pub borrowed: i128,         // <---- this will be changed frequently
    pub supply: i128,           // <---- this will be changed frequently
                                // <---- pool_config?
                                // @TODO: For now it's not clear what's better - store `PoolConfig` as `Pool` field or separately
                                // and index into it with `PoolAddress` as well

                                // <---- available_supply for collateral deposits?
}

#[contracttype]
#[derive(Debug)]
pub struct PoolConfig {
    /// Positive Base Rate percentage
    pub base_rate: i128,
    /// Positive Optimal Utilization Ration percentage
    pub optimal_utilization_ratio: i128,
    pub slope1: i128,
    pub slope2: i128,
    /// Non-negative Reserve Ration percentage (< 100)
    pub reserve_ratio: i128,
    /// Non-negative Liquidation Threshold percentage (<= 100)
    pub liquidation_threshold: i128,
}

// 100_000 - 100%
impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            base_rate: 3_000,
            optimal_utilization_ratio: 70_000,
            slope1: 5_000,
            slope2: 100_000,
            reserve_ratio: 10_000,
            liquidation_threshold: 80_000,
        }
    }
}

#[contracttype]
pub struct Obligation {
    pub deposits: Map<PoolAddress, i128>,
    pub borrows: Map<PoolAddress, i128>,
}

#[allow(unused)]
pub(crate) fn read_global_state(e: &Env) -> GlobalState {
    e.storage()
        .instance()
        .get(&DataKey::GlobalState)
        .expect("Global State must be instantiated at this point")
}

pub(crate) fn write_global_state(e: &Env, global_state: &GlobalState) {
    e.storage()
        .instance()
        .set(&DataKey::GlobalState, global_state);
}

// --- Pool ---
pub(crate) fn set_pool(
    e: &Env,
    pool_address: &PoolAddress,
    token_address: &Address,
    pool_config: Option<PoolConfig>,
) -> Result<(), LendingContractError> {
    e.storage().instance().set(
        &DataKey::Pool(pool_address.clone()),
        &PoolData {
            token_address: token_address.clone(),
            supply: 0,
            borrowed: 0,
        },
    );

    let pool_config = if let Some(pool_config) = pool_config {
        if !is_pool_config_valid(&pool_config) {
            return Err(LendingContractError::InvalidLoanPoolConfig);
        }

        pool_config
    } else {
        Default::default()
    };
    e.storage()
        .instance()
        .set(&DataKey::PoolConfig(pool_address.clone()), &pool_config);

    Ok(())
}

// @TODO
// pub(crate) fn set_pool_config(e: &Env, pool_address: &Address, interest_rate_config: PoolConfig) {

//     // Maybe, store interest rate config separately???
// }

fn is_pool_config_valid(pool_config: &PoolConfig) -> bool {
    let &PoolConfig {
        base_rate,
        optimal_utilization_ratio,
        slope1,
        slope2,
        reserve_ratio,
        liquidation_threshold,
    } = pool_config;

    (0 < base_rate) // BR must be > 0%
        && (0 < optimal_utilization_ratio) // OUR must be > 0%
        && (0 <= reserve_ratio && reserve_ratio < 100_000) // RR must be [0%; 100%)
        && (0 <= liquidation_threshold && liquidation_threshold <= 100_000) // LT must be [0%; 100%]
        && (slope1 < slope2) // (slope1 < slope2) is necessary for kinked model to work
}

pub(crate) fn pool_exists(e: &Env, pool_address: &PoolAddress) -> bool {
    e.storage()
        .instance()
        .has(&DataKey::Pool(pool_address.clone()))
}

pub(crate) fn get_pool_data(e: &Env, pool_address: &PoolAddress) -> Option<PoolData> {
    e.storage()
        .instance()
        .get(&DataKey::Pool(pool_address.clone()))
}

pub(crate) fn get_pool_config(e: &Env, pool_address: &Address) -> Option<PoolConfig> {
    e.storage()
        .instance()
        .get(&DataKey::PoolConfig(pool_address.clone()))
}

pub(crate) fn set_pool_data(e: &Env, pool_address: &Address, pool_data: &PoolData) {
    e.storage()
        .instance()
        .set(&DataKey::Pool(pool_address.clone()), pool_data);
}

pub(crate) fn adjust_pool_supply(
    e: &Env,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<(), LendingContractError> {
    let PoolData {
        token_address,
        supply,
        borrowed,
        ..
    } = get_pool_data(e, pool_address).ok_or(LendingContractError::PoolDoesNotExist)?;
    let new_supply = supply
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    let new_pool_data = PoolData {
        supply: new_supply,
        token_address,
        borrowed,
    };
    set_pool_data(e, pool_address, &new_pool_data);

    Ok(())
}

// --- Obligation ---
pub(crate) fn set_obligation(e: &Env, user: &Address, obligation: &Obligation) {
    // let x: f32 = 123_f32;
    // let y = (x + 123.3);
    e.storage()
        .instance()
        .set(&DataKey::Obligation(user.clone()), obligation);
}

pub(crate) fn get_obligation(e: &Env, user: &Address) -> Option<Obligation> {
    e.storage()
        .instance()
        .get(&DataKey::Obligation(user.clone()))
}

pub(crate) fn adjust_deposit(
    e: &Env,
    user: &Address,
    pool_address: &PoolAddress,
    amount: i128,
) -> Result<i128, LendingContractError> {
    let Obligation {
        mut deposits,
        borrows,
    } = get_obligation(e, user).unwrap_or(Obligation {
        deposits: Map::new(e),
        borrows: Map::new(e),
    });
    let pool_obligation_deposit = deposits.get(pool_address.clone()).unwrap_or(0);
    let new_deposit_amount = pool_obligation_deposit
        .checked_add(amount)
        .ok_or(LendingContractError::OverOrUnderflow)?;
    deposits.set(pool_address.clone(), new_deposit_amount);
    let new_obligation = Obligation { deposits, borrows }; // @TODO: this sucks a bit
    set_obligation(e, user, &new_obligation);

    Ok(new_deposit_amount)
}

pub(crate) fn deposit_exists(
    e: &Env,
    user: &Address,
    pool_address: &PoolAddress,
) -> Result<bool, LendingContractError> {
    let Obligation {
        deposits,
        borrows: _,
    } = get_obligation(e, user).ok_or(LendingContractError::MisslingObligation)?;

    Ok(deposits.contains_key(pool_address.clone()))
}
