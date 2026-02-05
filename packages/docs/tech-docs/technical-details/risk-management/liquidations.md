# Liquidations

Liquidations protect the protocol from undercollateralized debt by allowing third parties to repay risky obligations in exchange for collateral at a parameterized discount. A liquidation is permitted only when the obligation’s liquidation health factor (LHF) falls below 1. Markets also define an insolvency LTV (constrained to the 95–98.5% range) that distinguishes health-improving liquidations from insolvency handling.

## Liquidator Interface

A liquidator specifies the obligation to act on, the borrowed-asset pool to repay, the collateral-asset pool to receive collateral from, the amount of borrowed tokens to repay, and a minimum amount of collateral to receive. The transaction proceeds only if the protocol can deliver at least that minimum while respecting all active constraints; otherwise it reverts.

Liquidations execute in slices rather than as a single wipeout. Each call can extinguish only up to the pool’s liquidation close factor (for example, 40% of the outstanding debt). This limits market impact and makes liquidations less destructive for the borrower. In the health-improving scenario, the close-factor cap is enforced; in insolvency handling, the close-factor constraint is not applied. The collateral received includes a collateral purchase discount (liquidation bonus) bounded by the pool’s maximum liquidation incentive (for example, up to 5%), and never exceeds that parameter.

## Modes by Solvency State

When `LHF < 1`, the obligation is liquidatable and the protocol applies one of two modes based on its unparameterized LTV relative to the market’s insolvency LTV.

**Health-improving mode (LTV < insolvency LTV).** Liquidations must improve the position’s health and are subject to three constraints: the pool’s maximum liquidation incentive, the pool’s liquidation close factor (how much of the debt can be liquidated per call), and the amount of collateral available in the obligation (both plain collateral and deposited tokens). If all constraints can be met, the liquidator receives at least the stated minimum collateral. If any constraint cannot be satisfied, the liquidation fails.

**Insolvency mode (LTV ≥ insolvency LTV).** The obligation is considered insolvent. Liquidations in this state aim to reduce bad debt and are constrained by the pool’s maximum liquidation incentive and the collateral actually available. Health improvement is not guaranteed. As above, the liquidation proceeds only if the protocol can transfer at least the liquidator’s minimum requested collateral; otherwise it fails.

## Collateral Delivery

Collateral is delivered in a defined order. Plain collateral is transferred first and is prioritized to satisfy the liquidator’s minimum. If plain collateral is insufficient, the protocol transfers the liquidated position’s jTokens (supply shares of the collateral pool) to the liquidator’s obligation (created if needed). The liquidator may then withdraw the underlying tokens, bearing the additional risk that the pool might not have sufficient available liquidity at that moment.

Optionally, the protocol can be configured to automatically redeem jTokens when plain collateral is insufficient. In that mode, jTokens are transferred to the liquidator’s obligation only when the pool lacks enough available tokens to complete the withdrawal immediately.

## Minimum Collateral Value Guarantee

Every deposit position that improves borrowing capacity (as defined by the borrowing capacity formula) must have a value greater than the minimum collateral value. If a position is below this minimum, it can still earn supply interest (if it contains yield-bearing supply in addition to plain collateral), but it does not contribute to borrowing capacity and cannot be used as redeemable collateral by the liquidator.

## Bad Debt Resolution

For insolvent obligations, liquidations are used to reduce bad debt. After liquidations can no longer proceed, remaining bad debt is first covered by the insurance fund. If that is insufficient, any residual is socialized across the pool’s lenders according to the protocol’s loss-sharing rules.
