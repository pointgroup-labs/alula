use soroban_sdk::{Vec, contracttype};

use crate::{
    interest_rate::AnnualPercentageYields, multiply_pair::MultiplyPair, pool::Pool,
    storage::GlobalState,
};
#[contracttype]
#[derive(Debug, PartialEq, Eq, Clone)]

// Represents the pool's plain data with additionally computed info. Intended to be used as a result of simulated read-only
// invocations
pub struct PoolData {
    pub pool: Pool,
    pub apy: AnnualPercentageYields,
    pub total_supply: i128,
    pub total_available_adjusted: i128,
    pub j_token_rate_floor_bps: i128,
    pub d_token_rate_ceil_bps: i128,
    pub oracle_asset_price: i128,
}

// Represents the entire market's data(for every pool) with additionally computed info. Intended to be used as a result of simulated read-only
// invocations
#[contracttype]
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct MarketData {
    pub pools_data: Vec<PoolData>,
    pub multiply_pairs: Vec<MultiplyPair>,
    pub global_state: GlobalState,
    pub asset_decimals: u32,
    pub oracle_price_decimals: u32,
}
