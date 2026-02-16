use crate::{constants::*, error::FCError, utils::MathUtils};
use soroban_sdk::{Vec, contracttype};

#[contracttype]
#[derive(Clone)]
pub struct RewardCurvePoint {
    pub ts_start: u64,
    pub reward_per_time_unit: i128,
}

#[contracttype]
#[derive(Clone)]
pub struct RewardScheduleCurve {
    /// Points defining the curve (up to `[MAX_CURVE_POINTS]` and in ascending order)
    pub points: Vec<RewardCurvePoint>,
}

impl RewardScheduleCurve {
    pub fn require_valid(&self) -> Result<(), FCError> {
        if self.points.is_empty() || self.points.len() > MAX_CURVE_POINTS {
            return Err(FCError::InvalidRewardScheduleCurve);
        }

        for (p_current, p_next) in self.points.iter().zip(self.points.iter().skip(1)) {
            if p_current.ts_start >= p_next.ts_start {
                return Err(FCError::InvalidRewardScheduleCurve);
            }
        }

        Ok(())
    }

    /// Calculates the total rewards to emit between two timestamps based on the curve.
    ///
    /// The curve defines segments with different emission rates. This function
    /// integrates the piecewise-constant function from `from_ts` to `to_ts`.
    ///
    /// # Arguments
    /// * `from_ts` - start timestamp
    /// * `to_ts` - end timestamp
    ///
    /// # Returns
    /// Total rewards to emit in the time period
    pub fn calculate_rewards(&self, from_ts: u64, to_ts: u64) -> Result<i128, FCError> {
        if from_ts >= to_ts || self.points.is_empty() {
            return Ok(0);
        }

        let mut total_rewards = 0_i128;
        let len = self.points.len();

        for i in 0..len {
            let point = self.points.get(i).unwrap();
            if point.ts_start >= to_ts {
                break;
            }

            let segment_end = if i + 1 < len {
                self.points.get(i + 1).unwrap().ts_start
            } else {
                to_ts
            };

            let overlap_start = from_ts.max(point.ts_start);
            let overlap_end = to_ts.min(segment_end);

            if overlap_start < overlap_end {
                let duration = (overlap_end - overlap_start) as i128;
                let duration_rewards =
                    duration.checked_mul(point.reward_per_time_unit).map_over_or_underflow()?;
                total_rewards =
                    total_rewards.checked_add(duration_rewards).map_over_or_underflow()?;
            }
        }

        Ok(total_rewards)
    }
}
