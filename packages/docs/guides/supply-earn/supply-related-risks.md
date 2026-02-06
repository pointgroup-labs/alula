# Supply-Related Risks

## Withdrawal limits / throttles

You might not be able to withdraw your full balance immediately if the pool is constrained or limits are active. Alula can apply withdrawal limits to protect pool liquidity, especially when utilization is high or when demand for withdrawals spikes.

**Available for Withdrawal** is the amount you can take out right now. It can be lower than your total supplied balance if the pool doesn’t have enough idle liquidity (for example, if most funds are currently borrowed), if a **Pool Withdrawal Limit** is active, or if part of your supply is tied up as collateral for an open borrow or a Multiply position. In those cases, withdrawing more usually requires waiting for liquidity to return, or reducing your debt so more of your collateral becomes withdrawable.

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
