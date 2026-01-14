# How to Borrow Assets

{% stepper %}
{% step %}
Go to **Markets**.
{% endstep %}

{% step %}
Press **Borrow** for the asset you want to borrow.
{% endstep %}

{% step %}
Enter an amount (or press **MAX**).
{% endstep %}

{% step %}
Review the details:

* **Health Factor:** A safety score for your account after this borrow. Higher is safer; closer to 1.0 means you’re closer to liquidation.
* **Pool Liquidity Available:** The pool’s currently available lending liquidity for this asset (caps how much anyone can borrow right now).
* **Your Borrowing Capacity:** Your account-side limit that reflects how much you can borrow based on your collateral and the current LTV values.
* **Open LTV**: The maximum loan-to-value allowed when opening/adding to a borrow. It drives your borrowing capacity.
* **Close LTV**: The liquidation threshold. If your position crosses this level, it can be liquidated.
* **Liquidation Penalty**: The extra cost applied if your position is liquidated (effectively a fee paid from your collateral).
* **Operation Fee:** A protocol fee charged for the action.
* **Transaction Fee:** The network fee to submit the transaction on Stellar.
* **Borrow APY**: The annualized interest rate you pay on the borrowed amount; interest adds up over time until you repay.
{% endstep %}

{% step %}
Check the acknowledgement box, press **Borrow** and confirm the transaction in your wallet.
{% endstep %}
{% endstepper %}
