# FAQ

:::details Why won’t my wallet connect, or why does it keep disconnecting?

Make sure you’re on the right network (testnet vs mainnet), refresh the page, and reconnect from the wallet extension/app. If it still drops, close other dapps using the wallet and try again.

:::

:::details What should I do if a transaction is pending for a long time or fails after I sign it?

Don’t keep re-signing the same transaction. Check your wallet activity/history, make sure you have enough XLM for fees, then retry once. If it keeps failing, try a smaller amount (you may be hitting a limit or cap).

:::

:::details What happens if I don’t have enough XLM to pay network fees?

Your transaction won’t submit or will fail. Add a small amount of XLM to your wallet, then try again.

:::

:::details Why can’t I supply or borrow more even when I have balance and the market shows liquidity?

You may be blocked by a pool rule, not your wallet balance. Common reasons: the pool hit its Supply Limit, borrowing is constrained by available liquidity, utilization-based throttles are active, or the market is temporarily paused for certain actions.

:::

:::details Why is an asset marked Restricted, and what does it mean for me?

Restricted assets require extra rules (like KYC/allow-listing). If you’re not eligible, you may be unable to supply, borrow, or use that asset as collateral until you meet the requirements.

:::

:::details Why can’t I borrow in the same pool where I have a deposit (or supply to a pool where I have an active loan)?

Alula can separate “earning” positions from borrowing positions depending on how you supplied. If you supplied in a way that’s meant to be borrow-disabled (for example, an earn-only path), you won’t be able to borrow from that same position. Use the standard supply/borrow flow instead.

:::

:::details Why does a withdrawal get blocked even when I’m not trying to take out “too much”?

Withdrawals can be blocked if the pool doesn’t have enough available liquidity right now, if withdrawal throttles are active at high utilization, or if withdrawing would reduce your collateral too much (hurting your health factor).

:::

:::details Why does Multiply show a negative APY sometimes?

Multiply APY combines what you earn on the position and what you pay to maintain it. If borrow costs are high (often from high utilization) or the yield in the vault is low, the net result can go negative.

:::
