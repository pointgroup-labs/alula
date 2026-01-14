# How to Supply Assets

{% stepper %}
{% step %}
Go to **Markets**.
{% endstep %}

{% step %}
Press **Supply** for the asset you want to supply.
{% endstep %}

{% step %}
Enter an amount (or press **MAX**).
{% endstep %}

{% step %}
Review the details:

* **Supply Limit** (if the pool has a cap): The maximum total amount the pool can accept for this asset. If the pool is at (or near) the limit, you may not be able to supply your full amount.
* **Operation Fee:** A protocol fee charged for the action.
* **Transaction Fee:** The network fee to submit the transaction on Stellar.
{% endstep %}

{% step %}
(Optional) Enable **Collateral Only** if you want this deposit to be treated as collateral rather than general pool liquidity. Use this when your goal is to improve your borrowing power.
{% endstep %}

{% step %}
Press **Supply** and confirm in your wallet.
{% endstep %}
{% endstepper %}

{% hint style="warning" %}
**If you see a warning that you can’t supply into a pool with an active loan**

It usually means you already have an open borrow in that same asset pool. Repay that borrow first, then supply.
{% endhint %}
