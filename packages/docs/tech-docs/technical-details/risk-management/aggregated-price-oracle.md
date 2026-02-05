# Aggregated Price Oracle

When a market is created, the Market Manager admin must specify an oracle compliant with SEP-40 (the oracle address is immutable and cannot be updated after deployment). This allows markets to integrate existing Stellar-based solutions, such as Reflector oracles, or integrate Alula’s aggregated oracle at deployment time.

For higher reliability, Alula provides an optional aggregated oracle that computes the median price across multiple SEP-40–compliant sources and can be configured with circuit breakers. Markets may integrate any SEP-40 oracle; use of the aggregated oracle is optional and set per-market at deployment.

## Circuit Breaker

The aggregated oracle includes a circuit breaker: if an asset’s price changes by more than a configured percentage within a short window (for example, 10% within 5 minutes), the oracle pauses updates and returns no price. This safeguard lives at the oracle layer, not in the market contract. Because markets can choose any SEP-40 oracle, use of a circuit-breaker-enabled oracle is optional.

Withdrawal behavior during a circuit-breaker event depends on the obligation type. For obligations with no open borrows, withdrawals may remain available so users can derisk while prices are guarded. For obligations with open borrows, withdrawals that would reduce collateral cannot proceed while the oracle returns no price, because the protocol cannot validate collateralization. In contrast, _Earn_ obligations are never blocked by this rule because they do not allow borrows. The initial implementation compares only two sequential prices, keeping the rule simple and auditable.
