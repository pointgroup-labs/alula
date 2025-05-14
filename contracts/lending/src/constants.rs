pub const REFLECTOR_TESTNET_ADDRESS: &str =
    "CCYOZJCOPG34LLQQ7N24YXBM7LL62R7ONMZ3G6WZAAYPB5OYKOMJRN63";

pub const BPS_IN_PERCENT: i128 = 100;

// TODO: think about extension
// pub trait BpsExtension {
//     fn to_bps(self) -> i128;
// }
// impl BpsExtension for i128 {
//     fn to_bps(self) -> i128 {
//         self * BPS_IN_PERCENT
//     }
// }

pub const DEFAULT_LIQUIDATION_THRESHOLD: i128 = 80;

pub const DEFAULT_BASE_RATE: i128 = 3;
pub const DEFAULT_OPTIMAL_UTILIZATION_RATIO: i128 = 70;
pub const DEFAULT_RESERVE_RATIO: i128 = 10;
pub const DEFAULT_SLOPE1: i128 = 2;
pub const DEFAULT_SLOPE2: i128 = 20;
