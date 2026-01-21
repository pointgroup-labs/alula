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
