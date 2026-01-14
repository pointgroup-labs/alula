# Withdrawal Throttles

When a pool exceeds its configured utilization limit, stricter withdrawal rules apply to protect against rapid liquidity drains under stress.

* After a withdrawal from a specific obligation, a pool-defined cooldown prevents immediate sequential withdrawals from the same obligation while the pool remains above its limit.
* Each withdrawal is capped by a pool parameter such as a withdrawal scarcity limit, which allows withdrawing only up to a fixed percentage of remaining pool supply while the pool is under stress.

In extended high-utilization mode (for example above 85%), the system can further restrict withdrawals per address per block and apply a small exit fee that can be directed to an insurance or protocol fund. Users can still exit, but not in a way that empties the pool faster than it can recover.
