# Data, Pricing, and Cross-Pool Evaluation

Health, liquidation, and high-utilization rules rely on two inputs: pool state (available liquidity, borrowed amounts) and the configured SEP-40 oracle (current asset prices). Cross-pool logic—cross-pool collateral (CPC), health factor (HF), liquidation (LIQ)—evaluates user obligations rather than individual pools, so collateral posted in one pool can be accounted for when assessing another, according to configured rules. This supports more capital-efficient, RWA-oriented markets while still enforcing conservative behavior under stress.

Taken together, these safeguards aim to degrade gracefully: users are warned early, liquidations execute in small steps, liquidity cannot be drained too quickly when utilization is high, and markets can slow down entry when prices are unstable, while keeping exits open.

<br>
