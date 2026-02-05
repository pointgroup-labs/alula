# Health Factor

Every account has a health factor (HF) that indicates proximity to liquidation. It is surfaced to users with simple, color-coded messages (for example, 1.2 as a caution zone and 1.05 as a critical zone). When HF drops below these thresholds, the system can notify the account owner so they can add collateral, repay, or unwind before liquidation begins.

Health factor is calculated as:

`HF = Vc / Vb  `

where:

* `HF`: health factor of the obligation
* `Vc`: total collateral value (sum of all collateral asset values)
* `Vb`: total borrowed value (sum of all borrowed asset values)

HF is the reciprocal of unweighted LTV; liquidation eligibility is determined by LHF.

## Liquidation Health Factor

The liquidation health factor (LHF) is calculated as:

`LHF = (Σ Vc_i × cLTV_i) / (Σ Vb_j × LF_j)`

where:

* `LHF`: liquidation health factor of the obligation
* `Vc_i`: value of collateral asset _i_
* `cLTV_i`: close LTV for collateral asset _i_
* `Vb_j`: value of borrowed asset _j_
* `LF_j`: liability factor parameter for borrowed asset _j_

Per-asset close LTV must lie in \[0%; 100%], and per-asset liability factor must lie in \[100%; 200%].

If LHF < 1, the obligation is eligible for liquidation. In addition, every market configures an insolvency LTV (currently constrained to \[95%; 98.5%]). If an obligation’s LTV exceeds this threshold, the position is considered insolvent and the protocol may allow liquidations that are not health-improving (i.e., can be health-factor-reducing / LTV-increasing) in order to reduce bad debt. Because of the insolvency LTV constraint, health-factor-reducing liquidations can become possible even when HF is only slightly above 1.

## Borrowing Capacity

Borrowing capacity is the remaining value an obligation can borrow without violating its risk limits. Receiving positive borrowing capacity is a prerequisite for an obligation to be able to maintain any active borrows. It is calculated per obligation as:

`BC = (Σ max(Vc_i − Vmin, 0) × oLTV_i) − (Σ Vb_j × LF_j)`

where:

* `BC`: borrowing capacity of the obligation
* `Vc_i`: value of collateral asset _i_ (includes plain collateral and deposited tokens that earn supply rate)
* `Vmin`: minimum collateral value configured for the market (must always remain and therefore does not contribute to capacity)
* `oLTV_i`: open LTV parameter for collateral asset _i_
* `Vb_j`: value of borrowed asset _j_
* `LF_j`: liability factor parameter for borrowed asset _j_ within \[100%, 200%] (scales each borrowed asset’s contribution to risk)

All new borrows and withdrawals are capped so that borrowing capacity ≥ 0 after the operation. Since open LTV parameter for collateral asset _i_ ≤ close LTV parameter for collateral asset _i_, these caps keep the obligation healthy for all user actions (liquidations excluded).

For example, if capacity is 300 USD and the liability factor for USDC is 200%, a request to borrow 200 USDC is reduced to 150 USDC because 150 × 200% = 300, leaving capacity at zero.
