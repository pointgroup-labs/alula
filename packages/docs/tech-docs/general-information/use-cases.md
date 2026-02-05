# Use Cases

:::: half
::: card Liquidity provision
**Who:** Retail/whales, ecosystem treasuries

**Flow:** Supply USDC/XLM/EURC to an Alula pool → supplied assets are allocated to RWA markets

**Outcome:** Earn safe and transparent supply yield
<br>
<br>
:::
::::

:::: half
::: card Leveraged exposure
**Who:** Funds, family offices, market makers

**Flow:** Supply RWA asset as collateral → automatically borrow USDC via a flash loan and swap into more of the same RWA asset using AMM DEX

**Outcome:** Gain leveraged exposure to the chosen position
:::
::::

:::: half
::: card Hedging & risk management
**Who:** Institutions, hedge funds, corporates

**Flow:** Supply RWA as collateral into a permissioned pool → borrow USDC → use it to short RWAs via perps/synths or diversify exposure (borrow-and-sell only in permissioned, allow-listed venues)

**Outcome:** Hedge risk while keeping RWAs on the balance sheet
<br>
<br>
<br>
<br>
<br>
:::
::::

:::: half
::: card Issuer / originator credit line (RWA-backed borrowing)
**Who:** Tokenized T-bill/MMF issuers, trade-finance originators, structured-credit vaults

**Flow:** Supply eligible RWA collateral → borrow USDC up to the pool’s LTV limit → use for new originations, redemption bridges, or working capital → draw/repay within borrower caps; gated by NAV freshness and pool health

**Outcome:** Predictable working capital without selling RWAs, with clear limits and automated risk controls
:::
::::

:::: half
::: card Market-maker funding (short-dated, revolving)
**Who:** KYC’d market makers and arb desks on allow-listed venues

**Flow:** Post eligible collateral (e.g., tokenized T-bill/MMF units; XLM if enabled per pool policy) → borrow USDC → deploy on listed venues → at term, repay or roll; limits and rates auto-adjust with utilization and risk caps

**Outcome:** Low-friction short-term funding with clear caps and guardrails; exits remain open under stress
:::
::::
