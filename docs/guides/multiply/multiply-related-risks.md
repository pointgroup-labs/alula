# Multiply-Related Risks

## Liquidation risk

A higher multiplier increases liquidation risk because it increases the size of your borrow relative to your collateral. With less buffer, your Health Factor can drop faster when prices move against you or when interest grows your debt. If you want a safer position, choose a lower multiplier and keep extra headroom instead of borrowing close to your limit.

## Price movement risk

Multiply increases price movement risk because you’re taking on more exposure to the asset you’re multiplying. If the multiplied asset falls in value, your collateral-to-debt ratio can worsen and your Health Factor can decline quickly. The higher the multiplier, the more a given price move can impact your position.

## Borrow-side risk

Multiply relies on borrowing, so your costs can change over time. Borrow APY and pool conditions (especially utilization) can move, which can increase the ongoing cost of the position and put more pressure on your Health Factor. Market parameters may also change, which can shift risk thresholds.

{% hint style="warning" %}
**If your Health Factor starts trending down**

Act early! Reducing the position (withdrawing part of it), lowering the multiplier, or closing the position entirely can restore buffer and reduce liquidation risk.
{% endhint %}

## Smart contract risk

Alula runs on smart contracts, which means your positions are governed by code. While the protocol is built to be secure, on-chain apps can still face risks such as bugs, unexpected edge cases, or attacks during extreme market conditions. Only supply what you’re comfortable using in DeFi, and keep your risk lower by avoiding overexposure (for example, by using less leverage in Multiply).
