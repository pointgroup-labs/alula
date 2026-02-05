# Upcoming Capabilities

The current Alula design is modular, so new mechanisms can be added without rewriting the core contracts. The features below are not live yet but are planned on the protocol roadmap.

::: details Fixed rates
Lending and borrowing at pre-agreed fixed interest rates, giving participants more predictable cashflows compared to purely variable-rate markets.
:::

::: details Expanded insurance fund modules (mutualized + backstop options)
Extend the existing insurance fund interface approach so markets can select different insurance fund implementations over time, ranging from an account-controlled (multisig/DAO) model to more permissionless coverage flows and a mature backstop module (staking-style). Where appropriate, introduce _mutualized_ insurance configurations so multiple pools/markets can be backstopped by a shared insurance fund, without changing the core market contract.
:::

::: details Targeted incentive programs
Reward campaigns on selected pools, funded by third parties, that pay additive rewards without distorting utilization-driven rates. This can lift net APY for liquidity providers and/or effectively lower the borrow rate on targeted assets.
:::

::: details Senior–junior tranches
Pool structures where capital is split into higher-priority “senior” tranches and higher-yield “junior” tranches, enabling different risk/return profiles and limiting socialization exposure for capital-constrained participants.
:::

::: details Automatic investment of idle liquidity into yield-bearing assets
Additional automation on top of the existing orchestrator so unused liquidity can be continuously swept into designated yield-bearing instruments and returned to cash when needed for draws.
:::

::: details RWA-compatible liquidation execution (permissioned + auction/RFQ options)
Extend liquidation flows to support policy-aligned execution for regulated RWAs, including permissioned/allow-listed liquidation participation where required, and optional auction-style or RFQ-style execution modules for harder-to-trade collateral.
:::

::: details Dutch auction for liquid collateral
Auction-based sales for liquid collateral where price adjusts over time, aiming to improve execution quality compared to fixed-discount sales.
:::

::: details Isolated risk modules with unified liquidity where safe
More advanced isolation configurations that allow listing higher-risk assets with strict caps and restricted collateral usage, while preserving capital efficiency for safer assets and pools.
:::

::: details Cross-chain pools
Extending Alula’s pool model across multiple chains so capital on other networks can access Stellar-native credit rails and RWA flows through a unified pool design.
:::

::: details Institutional custody and workflow integrations
Support institutional participation patterns (custody, approvals, and operational controls) required by regulated funds and treasuries, enabling safer onboarding and execution while preserving market/pool policy constraints.
:::
