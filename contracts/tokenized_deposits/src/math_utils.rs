use soroban_fixed_point_math::SorobanFixedPoint;
use soroban_sdk::Env;

use crate::{constants::*, error::TDError};

pub trait MathUtils<T> {
    fn map_over_or_underflow(self) -> Result<T, TDError>;
}

impl<T> MathUtils<T> for Option<T> {
    fn map_over_or_underflow(self) -> Result<T, TDError> {
        self.ok_or(TDError::OverOrUnderflow)
    }
}

// The (shares, assets) pair every conversion is priced against, already including the virtual
// offset.
pub struct Rate {
    pub shares: i128,
    pub assets: i128,
}

impl Rate {
    pub fn new(total_shares: i128, total_assets: i128, offset: u32) -> Result<Self, TDError> {
        Ok(Self {
            shares: total_shares.checked_add(virtual_shares(offset)).map_over_or_underflow()?,
            assets: total_assets.checked_add(VIRTUAL_ASSETS).map_over_or_underflow()?,
        })
    }

    // Converts assets to shares, rounding **down**.
    pub fn assets_to_shares_floor(&self, e: &Env, assets: i128) -> i128 {
        if assets == 0 {
            return 0;
        }

        SorobanFixedPoint::fixed_mul_floor(&assets, e, &self.shares, &self.assets)
    }

    // Converts assets to shares, rounding **up**.
    pub fn assets_to_shares_ceil(&self, e: &Env, assets: i128) -> i128 {
        if assets == 0 {
            return 0;
        }

        SorobanFixedPoint::fixed_mul_ceil(&assets, e, &self.shares, &self.assets)
    }

    // Converts shares to assets, rounding **down**.
    pub fn shares_to_assets_floor(&self, e: &Env, shares: i128) -> i128 {
        if shares == 0 {
            return 0;
        }

        SorobanFixedPoint::fixed_mul_floor(&shares, e, &self.assets, &self.shares)
    }

    // Converts shares to assets, rounding **up**.
    pub fn shares_to_assets_ceil(&self, e: &Env, shares: i128) -> i128 {
        if shares == 0 {
            return 0;
        }

        SorobanFixedPoint::fixed_mul_ceil(&shares, e, &self.assets, &self.shares)
    }
}
