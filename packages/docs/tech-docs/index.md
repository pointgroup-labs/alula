# Protocol Features

:::: half
::: card 📊 Configurable markets and pools
Markets define global constraints, while each asset pool is configured independently with per-asset risk parameters: open/close LTV, fixed (under development) or variable rates, liability factor, interest curve, fees, eligible collateral rules, and optional allow-lists where applicable. This produces predictable, policy-aligned credit lines suitable for institutional borrowers and RWA issuers.
:::
::::

:::: half
::: card 💸 Utilization & yield orchestrator
Idle liquidity can be routed to pre-approved, conservative yield-bearing assets and pulled back to cash when a borrow arrives. Curator vaults can optimize allocation across pools and risk tiers. This addresses the “new pool underutilization” problem and can improve realized APY for liquidity providers.
<br>
<br>
<br>
:::
::::

:::: half
::: card 🛡️ Guarded risk controls
Markets are isolated from each other. Utilization-based throttles and utilization-based interest curves help keep liquidity healthy. Circuit-breaker logic at the oracle layer prevents price-dependent actions (e.g., borrowing, collateral withdrawals, liquidations, and leveraged swaps) on obviously bad prices. Liquidations execute in slices rather than all at once, reducing price impact.
:::
::::

:::: half
::: card ⚙️ Stellar-native compliance primitives
Out-of-the-box support for SEP-12 (KYC), SEP-8 (regulated assets), anchors, and fiat ramps enables policy-aligned participation and simpler institutional onboarding on Stellar.
<br>
<br>
<br>
<br>
<br>
:::
::::
