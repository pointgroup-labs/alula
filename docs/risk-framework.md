# Alula Protocol Risk Framework

> **Public risk documentation.** This document describes the risk model of the Alula lending protocol on Stellar/Soroban: the risks the protocol faces and the mechanisms that mitigate them. It is a living document and is updated as the protocol evolves.

## Status legend

| Tag            | Meaning                                                |
| -------------- | ------------------------------------------------------ |
| ✅ Live        | Implemented and active in production                   |
| 🔨 In progress | Implemented partially or on testnet / being rolled out |
| 🗓️ Planned     | On the roadmap, not yet implemented                    |

> ⚠️ Status tags reflect current design intent. Confirm each tag against the latest deployment before relying on it.

---

# 1. Overview & design principles

Alula is an institution-ready lending protocol on Stellar. Lenders supply assets to **segregated, isolated markets** to earn yield; borrowers draw against collateral, including tokenized real-world assets (RWAs). The protocol can also route idle liquidity into approved low-risk yield sources while keeping funds withdrawable when liquidity is available.

The risk framework rests on four principles:

- **Isolation by default.** Each market is isolated so that a default or oracle failure in one market cannot contaminate others.
- **Conservative-by-construction parameters.** Volatile assets get conservative discounting; stable, liquid assets get more efficient limits.
- **Defense in depth.** Audits and on-chain invariants are necessary but not sufficient — they are layered with real-time monitoring, circuit breakers, and graduated emergency controls.
- **Transparency.** Positions, collateral, and parameters are verifiable on-chain; risk parameters change only through time-locked governance.

---

# 2. Risk taxonomy

Alula evaluates every asset integrated as collateral or debt across three vectors.

## 2.1 Market risk ✅

Historical price volatility, liquidity depth on DEX/CEX venues, and expected slippage when liquidating large positions under stress. Liquidity on the Stellar DEX (SDEX) is a key input: it determines whether the market can absorb liquidation flow without creating bad debt. Volatile assets (e.g. XLM) receive conservative collateral discounting; stablecoins and highly-liquid assets receive more efficient limits.

## 2.2 Smart-contract risk ✅

Audit history, time in live circulation (“battle-testing”), and architectural complexity of the asset. Non-standard tokens (fee-on-transfer, rebasing) that would corrupt internal pool accounting are screened out by listing criteria during onboarding (§3).

## 2.3 Counterparty risk (RWA / regulated assets) ✅

Critical when integrating tokenized RWAs and regulated tokens. Issuer standing, the transparency and legal enforceability of the collateral structure, and a credible redemption path are assessed as preconditions for listing (§3), supporting isolated markets tailored to institutional participants.

---

# 3. Collateral & asset onboarding framework

Every collateral candidate passes four control vectors before listing.

1. **Legal & operational due diligence.** Issuer/management reputation, transparency of the collateral structure, financial-sector track record (critical for RWAs); KYC/KYB diligence on issuers and counterparties; ability to isolate risk via isolated risk modules.
2. **Technical & systems security.** Completed independent audits with no unresolved critical findings; admin-access controls — time-lock for code upgrades and multisig emergency switches (pause/freeze).
3. **Economic parameters & market resilience.** Order-book depth on DEX/CEX to bound liquidation slippage; tokenomics (FDV vs real market cap, vesting, whale-concentration risk); volatility stress on Health Factor, with strict conservative LTV caps for new tokens.
4. **Pricing & liquidation.** Integration only with SEP-40 price feeds — median-of-sources aggregation with a consecutive-deviation circuit breaker; for RWAs, a transparent, time-bound redemption path to fiat.

## Asset tiers 🔨

Assets are graded into usage tiers, so a wide range of tokens can be supported safely without fragmenting liquidity:

| Tier                | Can be collateral | Can be borrowed | Typical use             |
| ------------------- | ----------------- | --------------- | ----------------------- |
| General             | Yes               | Yes             | Most liquid assets      |
| Isolated collateral | Yes (isolated)    | No              | New/volatile collateral |
| Isolated debt       | No                | Yes (isolated)  | Long-tail debt assets   |

---

# 4. Position & pool risk parameters ✅

Solvency is evaluated in real time on a consolidated obligation (`ObligationKey`), with cross-pool evaluation aggregating an account's collateral and debt within a market to compute Borrowing Capacity and Health Factor.

| Parameter                          | Mechanism                                                                                                      | Invariant                                                          |
| ---------------------------------- | -------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------ |
| **Open LTV**                       | Max loan-to-value at position opening                                                                          | Caps initial leverage                                              |
| **Close LTV**                      | Threshold at which a position becomes liquidatable                                                             | Always > Open LTV                                                  |
| **LTV Gap**                        | Enforced separation between open and liquidation points                                                        | `open_ltv_bps ≠ close_ltv_bps` (strict; no configurable `Gap_min`) |
| **Liability Factor**               | Multiplier (≥100%) inflating debt for volatile assets                                                          | Speeds liquidation threshold                                       |
| **Min Collateral Value**           | Minimum USD collateral size                                                                                    | Blocks dust/spam positions                                         |
| **Supply Cap / Utilization Limit** | Absolute per-asset supply cap; borrowing bounded by a utilization-ratio limit (not an absolute borrow ceiling) | Bounds concentration and utilization                               |

---

# 5. Liquidation engine ✅

When Health Factor falls below 1.0, a position becomes liquidatable. Alula runs a **two-mode** engine.

- **Health-improving mode.** Used while LTV is below the Insolvency LTV threshold. Repayment is capped by a **Liquidation Close Factor** (per-pool default 50%); the collateral seized is separately capped so that each liquidation **strictly improves** the position's LTV (a 99.9% factor that guarantees improvement), minimizing borrower penalty. This is closer to Aave-V3's close-factor model plus a novel strict-improvement cap than to an Aave-V4-style target-health-factor solve. The liquidator earns a Liquidation Bonus in discounted collateral.
- **Insolvency mode.** Triggered when LTV reaches/exceeds Insolvency LTV. The Close Factor cap is removed (up to 100% in one transaction) to retire bad debt fast.

In **both** modes, collateral is seized in strict order: first **plain collateral** (non-interest-bearing), then **jTokens** (yield-bearing pool shares) only if needed.

---

# 6. Oracle security ✅ / 🗓️

Price feeds are the most sensitive dependency for any lending market.

- **SEP-40 feeds only.** Assets integrate exclusively via SEP-40-compliant feeds. ✅
- **Aggregated median + circuit breaker.** An optional aggregated oracle computes the median across configured sources and halts price discovery when the deviation between **consecutive** medians exceeds a configured maximum — but only when those updates arrive within a configured window (`max_dev_consecutive_diff_secs`). It is a **rapid-move** breaker (after a quiet gap, the next price is accepted without a deviation check), not an all-time price bound. When it engages, it freezes price-dependent actions (borrow, withdraw collateral, liquidate) in _both_ directions until prices resume, so positions can't be liquidated on a single anomalous tick. ✅
- **Provenance preservation (staleness).** The aggregator reports the timestamp of the **oldest** contributing **periodic** source, so if a periodic feed stalls, downstream consumers detect a `max_age` breach and block borrowing against that collateral. Heartbeat-style sources are excluded from this minimum by design; a market priced solely by heartbeat sources falls back to current ledger time. ✅
- **AMM-spot feeds not used in production.** Instantaneous AMM reserve ratios are sandwich-manipulable, so production markets are configured with SEP-40 feeds rather than AMM-spot adapters. This is an operational configuration policy, not a contract-level prohibition — the market consumes whatever SEP-40 oracle address is configured.
- **Off-chain NAV oracleization / book-to-market for RWAs.** 🗓️ For tokenized RWAs, a transparent NAV feed prices collateral to model-derived fair value rather than thin SDEX spot, with the on-chain book marked to market.

> **Why this matters — lesson from the Feb 2026 YieldBlox/USTRY incident on Stellar.** An attacker manipulated a low-liquidity tokenized T-bill's spot price on SDEX, the integration consumed last-trade spot without order-book-depth validation, and the pool accepted inflated collateral — draining reserves. Alula's design response: median-of-sources aggregation with anomaly rejection, collateral-concentration caps, and the SEP-40 + circuit-breaker stack above.

---

# 7. Bad debt, insurance & backstop

- **Insurance Fund (ControlledInsuranceFund).** ✅ First-loss buffer funded by the spread (Take Rate) and a share of Operation Fees.
- **Bad Debt Lock.** ✅ On confirmed bad debt, a permissionless call freezes deposits/withdrawals in the affected pool, removing front-running risk during resolution.
- **Loss socialization.** ✅ If reserves are insufficient, losses are written down pro-rata across LPs via a reduction in the pool's settlement-token exchange rate.
- **Backstop module.** 🗓️ A dedicated decentralized first-loss backstop (in the spirit of Blend's backstop) is on the roadmap to complement the insurance fund, with a withdrawal-delay queue and a bad-debt auction as the final coverage layer. _(Design discussion: TODO link.)_

---

# 8. Security: monitoring & incident response

Audits catch issues before launch; this layer catches them after.

- **Real-time monitoring (Hypernative).** 🗓️ Integration planned for all v3 pools and vaults — continuous on-chain threat detection across security, financial, and governance risk vectors, wired to automated responses (pause, parameter change, unwind).
- **Continuous invariant testing & fuzzing.** 🔨 Continuous invariant testing, fuzzing, and security checks across all core contracts as a standing part of the development pipeline (not a one-time audit).
- **Granular per-function pause.** 🔨 Per-function pause capabilities (rather than only a global kill-switch), so a single risky entry point can be halted without degrading the rest of the protocol.
- **Emergency controls.** ✅ Multisig-gated market freeze states — a graduated set of `Frozen` variants (including admin-protected and deposit/borrow-scoped freezes), authorized via `require_auth`; all parameter changes pass a time-lock (see §9).

---

# 9. Risk governance model 🔨

Alula uses a **hybrid market + governance** model.

- **Configurable guardrails for pool/market creators.** Market creators set parameters (rates, LTVs, oracle config, caps) within hard protocol invariants they cannot override (§11). This combines permissionless market creation with protocol-level safety.
- **Risk Council / admin.** A designated authority (multisig) governs asset onboarding, tier assignment, and parameter changes. Its active surface is risk-parameter changes — not arbitrary product changes.
- **Time-locks.** Parameter and config changes pass through a queue delay (pool-config default **24h**). This delay has **no enforced minimum floor** — a market creator can configure a shorter (even zero) delay, so the amount of advance notice depends on the market's configured value.

---

# 10. Additional risk layers (credit stack) 🗓️

For RWA and institutional markets, on-chain code is one layer of a broader credit stack:

- **Vault Curators & Managers** — allocate capital across markets, set vault-level risk profiles, rebalance as conditions shift.
- **Risk Analysis providers** — independent, continuous risk scoring across protocol, market, and strategy dimensions.
- **Underwriting providers** — borrower diligence and first-loss provisioning for credit markets.
- **Liquidators** — independent keepers/MEV bots incentivized by the liquidation bonus to clear unhealthy positions promptly.

---

# 11. Protocol invariants ✅

Standing, code-enforced rules (these codify the resolutions of prior security review — see §12):

- **Asymmetric rounding.** Debt, borrow limits, and health checks round **up** (in favor of protocol stability); collateral value and withdrawable amounts round **down**. Prevents precision-extraction exploits.
- **No zero LTV gap.** Deploying a market with `open_ltv_bps == close_ltv_bps` is forbidden (would cause instant liquidation on oracle noise/rounding).
- **Minimum liquidation-incentive ceiling.** The per-pool **maximum** liquidation incentive cannot be configured below 50 bps (0.5%), so pools retain room to keep clearing economical for independent liquidators even under high Stellar gas. This is a floor on the configurable maximum, not a guaranteed premium on every liquidation.
- **Share-inflation protection.** Pool share balances are tracked in internal accounting rather than read from `balanceOf`, so a direct token donation cannot move the share exchange rate; debt shares round **up** and zero-share mints are rejected. (There is no fixed initial-share constant — the first deposit mints 1:1.)
- **Liquidity accounting integrity.** `total_available` is updated by the exact amount on fee distribution and on liquidation repayment, keeping displayed liquidity in lockstep with real contract balances.
- **Oracle positivity.** On any price P ≤ 0, the aggregator and swap adapter **skip** that source (returning no price) rather than reverting; the market consumer then **reverts** when it receives no valid price. Net effect: a non-positive price cannot enter a position, but the revert happens at the consumer, not inside the aggregator.

---

# 12. Audits, formal verification & bug bounty

- **Independent audits.** ✅ Independent smart-contract audits have been completed; all critical and high-severity findings were remediated at the code level and codified as the standing invariants in §11. _(Audit reports: link.)_
- **Continuous fuzzing / invariant testing.** 🔨 See §8.
- **Formal verification of borrowing invariants.** 🗓️ Targeted formal verification of core borrowing invariants is on the roadmap. _(Not yet in place — do not represent as complete.)_
- **Bug bounty.** 🗓️ Public bug-bounty program planned.

---

# 13. Transparency & risk dashboard 🗓️

A public live risk dashboard is planned: current deposits/borrows, utilization and rates per market, global caps, per-position Health Factor / Open / Close LTV, and liquidations-at-risk under price-shock scenarios — alongside the fully on-chain, verifiable position book.

---

> **Disclaimer.** This document is for information only and is not financial, investment, or legal advice. Supplying and borrowing involve smart-contract and market risk, including liquidation and potential loss of funds. Parameters and statuses described here may change; always verify against the live protocol.

---

## Appendix — design lineage (internal reference)

Alula's risk design draws selectively on prior lending architectures: isolated markets and caps (Aave V3), Target-Health-Factor liquidation and risk-premium thinking (Aave V4), single-borrow-asset isolation and two-step absorb/auction liquidation (Compound V3), daily debt caps and auto-deleverage (Kamino), and the isolated-pool + first-loss backstop pattern (Blend). _Move the full comparative analysis to internal research — it does not belong in the public document._
