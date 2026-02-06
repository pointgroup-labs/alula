# Insurance Fund

Insurance funds are protocol-owned safety buffers that absorb residual losses after liquidations, reducing the chance that lenders take sudden, socialized writedowns. In Alula, each market can route a portion of fee revenue (such as the take rate and configured portions of the operation fee) into an insurance fund contract, which can then be used to cover bad debt when a position becomes insolvent. This keeps pool accounting predictable, improves stress resilience, and makes loss handling explicit and auditable.

When `distribute` is called, the market computes each beneficiary’s share of accrued fees and routes the insurance fund’s allocation to the fund contract through the same routing flow as other protocol recipients.

`Rs = Rb × U × (1 − t)`

where:

* `Rs`: supply rate
* `Rb`: borrow rate
* `U`: utilization ratio
* `t`: take rate (fraction of borrower interest diverted as the take rate before reaching lenders)

For example, if a pool has:

* `Rb` = 20%
* `U` = 50%
* `t` = 10%

then:

`Rs = 20% × 50% × (1 − 10%) = 20% × 50% × 90% = 9%`

Here, borrowers in aggregate are paying _Rb × U = 20% × 50% = 10%_. Of that 10%, a share equal to _10% × t = 1%_ accrues as the pool’s take rate. When `distribute` is called, the accrued take rate is paid out across the configured take rate beneficiaries (which may include the insurance fund). The remaining 9% is distributed to lenders.

The insurance fund can also receive direct donations. Unlike a backstop funded by lenders, donors do not receive a claim on the market and have no economic incentive beyond supporting the platform. Operation fees can also be routed to the insurance fund via the pool’s operation beneficiary configuration.

## Insurance Fund Interface

Alula’s insurance fund uses a decoupled, interface-driven design for insolvency handling. The core market contract does not embed any specific asset-recovery mechanism (auctions, governance votes, reserves, etc.). Instead, it integrates with an insurance fund contract through a small standard interface, keeping the lending logic lightweight and allowing different fund implementations over time.

The insurance fund contract is defined by an `InsuranceFund` interface (trait), which standardizes two responsibilities: revenue inflow and bad-debt coverage.

For revenue inflow, the market pushes the fund’s share of accrued take rate via `add_revenue(token, amount)` during `distribute`. This categorizes the transfer as protocol earnings (rather than a generic transfer).

For bad-debt events, the entry point is `request_coverage`. When a bad-debt coverer detects bad debt and initiates coverage on the market side, the market requests coverage from the insurance fund. The fund may respond in a fast path by returning `IssueRequestResult::Immediate(covered_amount)` when it has sufficient liquid reserves, allowing the market to wipe the covered debt within the same transaction. If the fund needs time to liquefy assets or complete an external process, it responds in a slow path by returning `IssueRequestResult::Processing(request_id)`, creating a persistent link between a specific bad-debt event and its resolution process.

When `request_coverage` returns a processing result, settlement follows a strict request–claim pattern. Bad-debt coverers can poll `get_status` until the request transitions from `CoverageStatus::Pending` to `CoverageStatus::Ready`. Once `Ready`, the `claim_coverage` method is invoked. This atomic function transfers the authorized tokens and clears the request state.

## Reference Implementation: Account-Controlled Insurance Fund

`AccountControlledInsuranceFundContract` is the simplest canonical implementation of the interface and is designed for a 1:1 market relationship managed by a multisig or DAO. It behaves like a managed escrow with an approve–pull mechanism: fund admin validates requests off-chain and explicitly authorizes payouts, while the contract enforces a hard solvency guarantee.

Solvency safety is enforced by tracking locked liquidity. Fund admin cannot approve coverage unless the contract holds sufficient free liquidity, which is checked by comparing the verified token balance against the internal `total_locked_amount`. When a request is approved, the approved amount is reserved internally (increasing locked liquidity).

### **Asynchronous Processing Workflow**

1. **Initiation:** The market calls `request_coverage`; the fund records the request and returns a `request_id`. The market freezes the specific obligation and applies protective withdrawal restrictions on the affected pool while the request is pending.
2. **Assessment:** The fund admin reviews the bad-debt event off-chain.
3. **Approval:** The fund admin calls `mark_ready(request_id, covered_amount)`, which:
   * verifies `free_balance >= covered_amount`
   * increases the internal locked-liquidity counter (e.g., `total_locked_amount`)
   * sets the request status to `CoverageStatus::Ready`
4. **Settlement:** A bad-debt coverer triggers market finalization, which calls `claim_coverage`. The market receives the full or partial amount required to cover the bad debt, decreases locked liquidity, and clears the request. Any non-covered remainder is handled by the market’s lender loss-socialization rules.

Funds remain under the custody of the fund admin account (controlled by multi-sig or DAO). The fund admin may withdraw funds, but withdrawals are constrained by the solvency check: the contract prevents withdrawing any assets that are currently locked, ensuring that approved claims remain fully backed until they are claimed.

## Bad Debt Resolution via Insurance Fund

If an obligation’s LTV rises above the market’s insolvency LTV threshold, the obligation is treated as insolvent and the protocol enters insolvency handling. When an obligation is liquidatable but not insolvent, liquidations are restricted to health-improving actions. Once it is insolvent, the protocol may allow liquidations that do not necessarily improve health. If the obligation’s LTV exceeds 100% (borrowed value exceeds collateral value), liquidations are no longer guaranteed to be health-improving (LTV-reducing). In this state, the protocol may allow liquidations that can be health-factor-reducing (LTV-increasing) in order to reduce bad debt.

To prevent depositors from front-running the socialization of bad debt, which lowers the jToken exchange rate, withdrawals are automatically frozen whenever bad debt is detected. This ensures the loss is distributed fairly among all liquidity providers. Withdrawals remain paused until the bad debt is effectively processed (either socialized against the pool reserves or covered by the insurance fund), at which point the exchange rate is updated and normal operations resume.

For a single obligation, loan-to-value (LTV) is calculated as:

`LTV = Vb / Vc`

where:

* `LTV`: loan-to-value ratio of the obligation
* `Vb`: total borrowed value (sum of all borrowed asset values)
* `Vc`: total collateral value (sum of all collateral asset values)

Collateral includes both plain collateral and deposited tokens that accrue supply interest.

A liquidator repays part or all of the debt and receives collateral whose value exceeds the repaid debt (by the liquidation bonus). Over time, collateral may be exhausted while some debt remains. Any remaining debt with no backing collateral is bad debt.

Anyone can initiate a bad debt coverage procedure for a specific obligation once liquidations are no longer possible for that obligation. Losses are resolved in two steps:

* The system first attempts to cover the loss using the insurance fund allocated to the relevant market.
* If the insurance fund is insufficient to fully cover the remaining bad debt, the residual loss is socialized across current lenders of that market.

The process begins by covering as much as possible from the insurance fund. If it is fully drained, any remaining shortfall is socialized among lenders. For example:

* Before the bad debt event, Alice held 5% of the supply, corresponding to 100 USD in value.
* The entire supply was worth 2,000 USD.
* Socialized bad debt is 200 USD.
* Alice’s share is 5% of 1,800 USD, which is 90 USD.
