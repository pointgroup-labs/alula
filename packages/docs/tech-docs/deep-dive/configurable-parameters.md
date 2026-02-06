# Configurable Parameters

This section outlines the configurable parameter set for an Alula market and its pools. It distinguishes between health and risk parameters (solvency and liquidation rules), fee schedule parameters (user-facing operation fees), and market-wide constraints. Every parameter update is validated.

## Governance and Update Logic

All parameters below follow the market’s update rules.

* **Owned markets only:** Pool configuration updates are only possible on owned markets where a market admin role is defined.
* **Timed queue execution:** Updates to `PoolConfig` must be queued by the market admin into a timed queue (timelock). They cannot be applied instantly, which gives users time to exit if they disagree with the change. The timelock duration is configured at deployment and cannot be changed later.
* **Immutability:** Markets deployed without an update queue are immutable; their pool and market parameters cannot be changed.
* **Authorization:** All updates must be authorized by the market admin.

## Market-Wide Configuration

These are global parameters affecting the entire market contract and all obligations within it. They are managed via `update_market` (owned markets only) and are applied instantly.

<table><thead><tr><th width="245.72723388671875">Parameter</th><th width="86.36358642578125">Type</th><th width="80.6363525390625">Unit</th><th>Description</th></tr></thead><tbody><tr><td><code>max_positions</code></td><td><code>u32</code></td><td>Count</td><td>Maximum number of distinct assets (collateral + borrows) a single obligation can hold. This prevents resource exhaustion during liquidation and health checks.</td></tr><tr><td><code>min_collateral_value_cents</code></td><td><code>i128</code></td><td>USD cents</td><td>Minimum collateral value (e.g., 500 = $5.00) required for an obligation to begin receiving positive borrowing capacity. This helps guarantee liquidation viability.</td></tr></tbody></table>

## Health and Risk Configuration

These parameters are managed via `PoolConfig.health_config`. Updates must go through the timed queue.

<table><thead><tr><th width="264.6363525390625">Parameter</th><th width="83.36358642578125">Type</th><th width="91.45452880859375">Unit</th><th>Description</th></tr></thead><tbody><tr><td><code>supply_limit</code></td><td><code>i128</code></td><td>Token amount</td><td>Hard cap on total liquidity (available + borrowed). 0 implies unlimited.</td></tr><tr><td><code>utilization_ratio_limit_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Utilization threshold (e.g., 9000 = 90%) that triggers the withdrawal throttle. Above this threshold, borrows are prohibited and withdrawal limits and throttle fees apply.</td></tr><tr><td><code>withdraw_scarcity_limit_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Maximum percentage of the pool’s total supply that can be withdrawn in a single transaction while the withdrawal throttle is active.</td></tr><tr><td><code>withdraw_scarcity_cooldown_s</code></td><td><code>u64</code></td><td>Seconds</td><td>Minimum time required between consecutive withdrawals from the same obligation while the withdrawal throttle is active.</td></tr><tr><td><code>open_ltv_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Open LTV used when computing borrowing capacity (e.g., 7500 = 75%).</td></tr><tr><td><code>close_ltv_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Close LTV used to determine whether an obligation is eligible for liquidation (e.g., 8500 = 85%).</td></tr><tr><td><code>liability_factor_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Multiplier applied to debt value (e.g., 12000 = 1.2×). Used to conservatively inflate debt value for riskier borrowed assets and reduce borrowing capacity.</td></tr><tr><td><code>liquidation_close_factor_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Maximum percentage of a single debt position that a liquidator can repay in one liquidation transaction.</td></tr><tr><td><code>max_liquidation_incentive_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Maximum collateral discount awarded to liquidators (e.g., 500 = 5%).</td></tr><tr><td><code>insolvency_ltv_bps</code></td><td><code>i128</code></td><td>BPS</td><td>LTV threshold (e.g., 9800 = 98%) above which an obligation is considered insolvent for insolvency handling.</td></tr></tbody></table>

## Operation Fees

These parameters are managed via `PoolConfig.fee_config`. Updates must go through the timed queue.

<table><thead><tr><th width="232.90911865234375">Parameter</th><th width="84.1817626953125">Type</th><th width="78.63641357421875">Unit</th><th>Description</th></tr></thead><tbody><tr><td><code>borrow_fee_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Operation fee charged atomically on new borrows. Added to the debt principal.</td></tr><tr><td><code>flash_loan_fee_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Operation fee charged on flash loan principal.</td></tr><tr><td><code>deposit_fee_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Operational friction fee on deposits (if enabled). Deducted from the deposit amount.</td></tr><tr><td><code>withdraw_fee_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Operational friction fee on withdrawals (usually 0). Deducted from the receivable amount.</td></tr><tr><td><code>withdraw_scarcity_fee_sc_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Additional fee scalar applied to <code>withdraw_fee_bps</code> only while utilization is above <code>utilization_ratio_limit_bps</code>. Discourages exits during a liquidity crunch and can be routed as protocol revenue (for example, to the insurance fund).</td></tr><tr><td><code>add_collateral_fee_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Friction fee on adding collateral (usually 0).</td></tr><tr><td><code>remove_collateral_fee_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Friction fee on removing collateral (usually 0).</td></tr><tr><td><code>repay_fee_bps</code></td><td><code>u32</code></td><td>BPS</td><td>Friction fee on repayment (usually 0).</td></tr></tbody></table>

## Fee Routing and Splits

This configuration defines how revenue is routed to beneficiaries. It requires market admin authorization and follows the owned-market rules.

### **Take Rate**

<table><thead><tr><th width="225.81817626953125">Parameter</th><th width="135.54547119140625">Source</th><th>Description</th></tr></thead><tbody><tr><td><code>take_rate_bps</code></td><td><code>PoolFeeConfig</code></td><td>Percentage of borrower interest diverted as protocol revenue (e.g., <code>1000</code> = 10%).</td></tr><tr><td><code>take_rate_beneficiaries</code></td><td><code>PoolConfig</code></td><td>List of <code>(Address, share_bps)</code> tuples defining how the take rate is split (e.g., <code>[(InsuranceFund, 2000), (Treasury, 8000)]</code>).</td></tr></tbody></table>

### **Operation Fees**

<table><thead><tr><th width="243.45458984375">Parameter</th><th width="160.3636474609375">Source</th><th>Description</th></tr></thead><tbody><tr><td><code>referrer_share_bps</code></td><td>Partner registry</td><td>Percentage of the operation fee paid instantly to the referrer when <code>referrer_address</code> is eligible.</td></tr><tr><td><code>origination_beneficiaries</code></td><td><code>PoolConfig</code></td><td>List of <code>(Address, share_bps)</code> tuples defining how net operation fees are split when <code>distribute</code> is called (e.g., <code>[(Treasury, 10000)]</code>).</td></tr></tbody></table>

## Interest Rate and Accrual Models

Model configuration must go through the timed queue. Only the compounded accrual model is available at the moment.

The interest rate curve is managed via `KinkedIRConfig`. The current implementation uses a kinked utilization-based model.

<table><thead><tr><th width="160.3636474609375">Parameter</th><th width="97.72723388671875">Type</th><th width="101.54541015625">Unit</th><th>Description</th></tr></thead><tbody><tr><td><code>base_apr_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Minimum borrow APR accrued regardless of utilization.</td></tr><tr><td><code>kink1_ur_bps</code></td><td><code>i128</code></td><td>BPS</td><td>First utilization kink threshold where the slope increases (e.g., 80%).</td></tr><tr><td><code>kink1_apr_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Borrow APR exactly at <code>kink1_ur_bps</code>.</td></tr><tr><td><code>kink2_ur_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Second utilization kink threshold for high-demand pricing (e.g., 90%).</td></tr><tr><td><code>kink2_apr_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Borrow APR exactly at <code>kink2_ur_bps</code>.</td></tr><tr><td><code>max_apr_bps</code></td><td><code>i128</code></td><td>BPS</td><td>Maximum borrow APR when utilization reaches 100%.</td></tr></tbody></table>
