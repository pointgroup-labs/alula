use soroban_sdk::{Env, I256};

use crate::{constants::SCALE_FACTOR, error::FCError};

// `rewards × SCALE_FACTOR` leaves i128 long before the result does: a dust stake alone in a farm drives
// the index past i128 within ~170 tokens of an 18-decimal reward. Positions checkpoint the index and
// multiply only its growth: `stake × index` grows with the farm's age and leaves even I256.

pub fn zero(e: &Env) -> I256 {
    I256::from_i128(e, 0)
}

/// Index increase for `rewards` shared over `denominator` (floor).
pub fn rps_delta(e: &Env, rewards: i128, denominator: i128) -> I256 {
    I256::from_i128(e, rewards)
        .mul(&I256::from_i128(e, SCALE_FACTOR))
        .div(&I256::from_i128(e, denominator))
}

/// What `stake` earned while the index rose from `checkpoint` to `index` (floor). The growth is split
/// into whole and fractional parts so neither product leaves I256.
pub fn earned(e: &Env, stake: i128, index: &I256, checkpoint: &I256) -> Result<i128, FCError> {
    if stake == 0 {
        return Ok(0);
    }
    let scale = I256::from_i128(e, SCALE_FACTOR);
    let stake = I256::from_i128(e, stake);
    let growth = index.sub(checkpoint);
    stake
        .mul(&growth.div(&scale))
        .add(&stake.mul(&growth.rem_euclid(&scale)).div(&scale))
        .to_i128()
        .ok_or(FCError::OverOrUnderflow)
}
