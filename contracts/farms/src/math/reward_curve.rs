use soroban_sdk::Vec;

use crate::{
    error::FarmsError,
    state::{RewardCurvePoint, RewardScheduleCurve},
};

impl RewardScheduleCurve {
    /// Calculates the total rewards to emit between two timestamps based on the curve.
    ///
    /// The curve defines segments with different emission rates. This function
    /// integrates the piecewise-constant function from `from_ts` to `to_ts`.
    ///
    /// # Arguments
    /// * `from_ts` - Start timestamp
    /// * `to_ts` - End timestamp
    ///
    /// # Returns
    /// Total rewards to emit in the time period
    pub fn calculate_rewards(&self, from_ts: u64, to_ts: u64) -> Result<i128, FarmsError> {
        if from_ts >= to_ts {
            // Shouldn't even be allowed here?
            return Ok(0);
        }

        if self.points.is_empty() {
            // why to have it?
            return Ok(0);
        }

        let mut total_rewards: i128 = 0;
        let points_len = self.points.len();

        for i in 0..points_len {
            let point = self.points.get(i).ok_or(FarmsError::InternalError)?;

            // Determine the end of this segment
            let segment_end = if i + 1 < points_len {
                let next_point = self.points.get(i + 1).ok_or(FarmsError::InternalError)?;
                next_point.ts_start
            } else {
                // Last segment extends indefinitely
                u64::MAX
            };

            // Calculate overlap between [from_ts, to_ts] and [point.ts_start, segment_end]
            let overlap_start = from_ts.max(point.ts_start);
            let overlap_end = to_ts.min(segment_end);

            // Is this even correct?

            // Can't you have something like this?

            if overlap_start < overlap_end {
                let duration = (overlap_end - overlap_start) as i128; // You've overlapped with a segment...
                let segment_rewards =
                    duration.checked_mul(point.reward_per_time_unit).ok_or(FarmsError::Overflow)?; // You are aware about
                // the reward per time unit here.... But this depends on the amount of your shares, doesn't it?
                total_rewards =
                    total_rewards.checked_add(segment_rewards).ok_or(FarmsError::Overflow)?;
            } // I am not sure if this is correct here...
        }

        Ok(total_rewards)
    }

    /// Gets the current reward rate at a given timestamp
    pub fn get_rate_at(&self, ts: u64) -> i128 {
        if self.points.is_empty() {
            return 0;
        }

        // Find the applicable rate (last point with ts_start <= ts)
        let mut rate = 0i128;
        for i in 0..self.points.len() {
            if let Some(point) = self.points.get(i) {
                if point.ts_start <= ts {
                    rate = point.reward_per_time_unit; // We have this, but we can increase it, right?
                } else {
                    break;
                }
            }
        }
        rate
    }

    /// Validates the reward schedule curve
    pub fn validate(&self) -> Result<(), FarmsError> {
        use crate::constants::MAX_CURVE_POINTS;

        if self.points.len() > MAX_CURVE_POINTS {
            return Err(FarmsError::InvalidRewardSchedule);
        }

        // Verify points are ordered by timestamp
        let mut last_ts: u64 = 0;
        for i in 0..self.points.len() {
            if let Some(point) = self.points.get(i) {
                if i > 0 && point.ts_start <= last_ts {
                    return Err(FarmsError::InvalidRewardSchedule);
                }
                if point.reward_per_time_unit < 0 {
                    return Err(FarmsError::InvalidRewardSchedule);
                }
                last_ts = point.ts_start;
            }
        }

        Ok(())
    }

    /// Creates an empty reward schedule
    pub fn empty(points: Vec<RewardCurvePoint>) -> Self {
        Self { points }
    }
}

#[cfg(test)]
mod tests {
    use soroban_sdk::{Env, vec};

    use super::*;

    #[test]
    fn test_empty_curve() {
        let env = Env::default();
        let curve = RewardScheduleCurve::empty(vec![&env]);
        assert_eq!(curve.calculate_rewards(0, 100).unwrap(), 0);
    }

    #[test]
    fn test_single_segment() {
        let env = Env::default();
        let curve = RewardScheduleCurve {
            points: vec![&env, RewardCurvePoint { ts_start: 0, reward_per_time_unit: 10 }],
        };

        // 100 seconds * 10 rewards/second = 1000 rewards
        assert_eq!(curve.calculate_rewards(0, 100).unwrap(), 1000);
        // 50 seconds * 10 rewards/second = 500 rewards
        assert_eq!(curve.calculate_rewards(50, 100).unwrap(), 500);
    }

    #[test]
    fn test_multiple_segments() {
        let env = Env::default();
        let curve = RewardScheduleCurve {
            points: vec![
                &env,
                RewardCurvePoint { ts_start: 0, reward_per_time_unit: 10 },
                RewardCurvePoint { ts_start: 100, reward_per_time_unit: 5 },
                RewardCurvePoint { ts_start: 200, reward_per_time_unit: 0 },
            ],
        };

        // 0-50: 50 * 10 = 500
        assert_eq!(curve.calculate_rewards(0, 50).unwrap(), 500);

        // 0-150: (100 * 10) + (50 * 5) = 1000 + 250 = 1250
        assert_eq!(curve.calculate_rewards(0, 150).unwrap(), 1250);

        // 50-250: (50 * 10) + (100 * 5) + (50 * 0) = 500 + 500 = 1000
        assert_eq!(curve.calculate_rewards(50, 250).unwrap(), 1000);
    }

    #[test]
    fn test_get_rate_at() {
        let env = Env::default();
        let curve = RewardScheduleCurve {
            points: vec![
                &env,
                RewardCurvePoint { ts_start: 0, reward_per_time_unit: 10 },
                RewardCurvePoint { ts_start: 100, reward_per_time_unit: 5 },
            ],
        };

        assert_eq!(curve.get_rate_at(0), 10);
        assert_eq!(curve.get_rate_at(50), 10);
        assert_eq!(curve.get_rate_at(100), 5);
        assert_eq!(curve.get_rate_at(200), 5);
    }
}
