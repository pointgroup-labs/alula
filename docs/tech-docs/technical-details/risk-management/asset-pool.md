# Asset Pool

Markets maintain per-asset pools tracked in pool state and governed by pool config. Each pool is initialized separately, and its parameters can be queued for update (for managed pools), and updates can be applied through administrative methods (for example `queue_in_pool_config_update`, `apply_pool_config_update`). This keeps risk isolated at the pool level. The separation is especially important for permissioned or RWA-heavy pools, where participation may be gated through allow-lists and policy-aligned settings.\
