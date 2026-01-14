# Borrow-Related Risks

## Liquidation risk&#x20;

The liquidation risk increases when your debt grows relative to your collateral value. This can happen because interest accrues, collateral prices move, or pool parameters change. Close LTV and Health Factor are the two key indicators to watch:

* Close LTV is the line you don’t want to cross.&#x20;
* Health Factor is the early-warning indicator that tells you how close you are. If your Health Factor gets near 1.0, your position can be liquidated, which means part of your collateral may be sold to repay the loan and you may pay a liquidation penalty.&#x20;

{% hint style="warning" %}
**If your Health Factor starts trending down**

The simplest way to reduce risk is to keep a buffer by borrowing less than your maximum, repaying some of the loan, or adding more collateral when needed.
{% endhint %}

## Market and parameter changes&#x20;

Such changes can affect your position. Rates, LTV thresholds, fees, and other pool settings may update over time, which can increase borrowing costs or bring you closer to liquidation. If you borrow, plan to monitor your position, especially during volatile periods or after parameter updates.

## Smart contract risk

Alula runs on smart contracts, which means your positions are governed by code. While the protocol is built to be secure, on-chain apps can still face risks such as bugs, unexpected edge cases, or attacks during extreme market conditions. Only supply what you’re comfortable using in DeFi, and keep your risk lower by avoiding overexposure (for example, by using less leverage in Multiply).
