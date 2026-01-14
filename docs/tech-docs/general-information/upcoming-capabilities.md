# Upcoming Capabilities

The current Alula design is modular, so new mechanisms can be added without rewriting the core contracts. The features below are not live yet but are planned on the protocol roadmap.

<details>

<summary><strong>Fixed rates</strong></summary>

Lending and borrowing at pre-agreed fixed interest rates, giving participants more predictable cashflows compared to purely variable-rate markets.

</details>

<details>

<summary><strong>Expanded insurance fund modules (mutualized + backstop options)</strong></summary>

Extend the existing insurance fund interface approach so markets can select different insurance fund implementations over time, ranging from an account-controlled (multisig/DAO) model to more permissionless coverage flows and a mature backstop module (staking-style). Where appropriate, introduce _mutualized_ insurance configurations so multiple pools/markets can be backstopped by a shared insurance fund, without changing the core market contract.

</details>

<details>

<summary><strong>Targeted incentive programs</strong></summary>

Reward campaigns on selected pools, funded by third parties, that pay additive rewards without distorting utilization-driven rates. This can lift net APY for liquidity providers and/or effectively lower the borrow rate on targeted assets.

</details>

<details>

<summary><strong>Senior–junior tranches</strong></summary>

Pool structures where capital is split into higher-priority “senior” tranches and higher-yield “junior” tranches, enabling different risk/return profiles and limiting socialization exposure for capital-constrained participants.

</details>

<details>

<summary><strong>Automatic investment of idle liquidity into yield-bearing assets</strong></summary>

Additional automation on top of the existing orchestrator so unused liquidity can be continuously swept into designated yield-bearing instruments and returned to cash when needed for draws.

</details>

<details>

<summary><strong>RWA-compatible liquidation execution (permissioned + auction/RFQ options)</strong></summary>

Extend liquidation flows to support policy-aligned execution for regulated RWAs, including permissioned/allow-listed liquidation participation where required, and optional auction-style or RFQ-style execution modules for harder-to-trade collateral.

</details>

<details>

<summary><strong>Dutch auction for liquid collateral</strong></summary>

Auction-based sales for liquid collateral where price adjusts over time, aiming to improve execution quality compared to fixed-discount sales.

</details>

<details>

<summary><strong>Isolated risk modules with unified liquidity where safe</strong></summary>

More advanced isolation configurations that allow listing higher-risk assets with strict caps and restricted collateral usage, while preserving capital efficiency for safer assets and pools.

</details>

<details>

<summary><strong>Cross-chain pools</strong></summary>

Extending Alula’s pool model across multiple chains so capital on other networks can access Stellar-native credit rails and RWA flows through a unified pool design.

</details>

<details>

<summary><strong>Institutional custody and workflow integrations</strong></summary>

Support institutional participation patterns (custody, approvals, and operational controls) required by regulated funds and treasuries, enabling safer onboarding and execution while preserving market/pool policy constraints.

</details>
