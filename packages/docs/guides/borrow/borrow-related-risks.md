# Borrow-Related Risks

## Liquidation risk&#x20;

The liquidation risk increases when your debt grows relative to your collateral value. This can happen because interest accrues, collateral prices move, or pool parameters change. Close LTV and Health Factor are the two key indicators to watch:

* Close LTV is the line you don’t want to cross.&#x20;
* Health Factor is the early-warning indicator that tells you how close you are. If your Health Factor drops below 1.0, your position can be liquidated, which means part of your collateral may be sold to repay the loan and you may pay a liquidation penalty.&#x20;

::: warning
**If your Health Factor starts trending down**

The simplest way to reduce risk is to keep a buffer by borrowing less than your maximum, repaying some of the loan, or adding more collateral when needed.
:::

## Market and parameter changes&#x20;

Such changes can affect your position. Rates, LTV thresholds, fees, and other pool settings may update over time, which can increase borrowing costs or bring you closer to liquidation. If you borrow, plan to monitor your position, especially during volatile periods or after parameter updates.

## Oracle circuit breaker

During extreme price moves, the protocol’s price oracle may temporarily stop returning updated prices. When the protocol can’t get a reliable price, it can’t safely check collateralization.

What this can mean for you:

* If you have an active borrow, actions that depend on prices (such as borrowing more or withdrawing collateral) may be temporarily blocked.
* If you don’t have an active borrow (for example, you’re only supplying/earning), you may still be able to withdraw, depending on pool liquidity and limits.

Once price updates resume, normal actions become available again.

## Bad-debt freeze

In rare cases, a borrower’s collateral can become insufficient to cover their debt after liquidations. When the protocol detects bad debt, it may temporarily pause withdrawals for the affected pool.

Why this happens:

* It prevents users from withdrawing right before a loss is applied (which would shift the loss unfairly to remaining suppliers).
* It gives the protocol time to apply insurance coverage first. If insurance doesn’t fully cover the shortfall, any remaining loss may be shared across suppliers in that pool according to protocol rules.

Withdrawals resume after the bad-debt event is processed and the pool’s accounting is updated.

## Smart contract risk

Alula runs on smart contracts, which means your positions are governed by code. While the protocol is built to be secure, on-chain apps can still face risks such as bugs, unexpected edge cases, or attacks during extreme market conditions. Only supply what you’re comfortable using in DeFi, and avoid borrowing close to your limit, and monitor your position during volatility.
