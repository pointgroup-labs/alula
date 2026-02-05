# How to Borrow Assets

1. Go to **Markets**.
2. Press **Borrow** for the asset you want to borrow.
3. Enter an amount (or press **MAX**).
4. Review the details:

   * **Health Factor:** A safety score for your account after this borrow. Higher is safer.
   * **Pool Liquidity Available:** The pool’s currently available lending liquidity for this asset (caps how much anyone can borrow right now).
   * **Your Borrowing Capacity:** The maximum additional amount you can borrow based on your collateral, current LTV limits, and risk settings. It decreases as you borrow more and increases when you add collateral or repay debt. 
   * **Open LTV**: The maximum loan-to-value allowed when opening/adding to a borrow. It drives your borrowing capacity.
   * **Close LTV**: A risk parameter used in liquidation checks.
   * **Liquidation Penalty**: The collateral discount applied during liquidation (an incentive paid to liquidators).
   * **Operation Fee:** A protocol fee charged for the action.
   * **Transaction Fee:** The network fee to submit the transaction on Stellar.
   * **Borrow APY**: The annualized interest rate you pay on the borrowed amount; interest adds up over time until you repay.

::: info
If you see “You cannot open a loan in the same pool where you have a deposit,” you can’t borrow from that pool from your current position. Use a different pool/market or withdraw the deposit that is blocking the borrow.
:::

5. Check the acknowledgement box, press **Borrow** and confirm the transaction in your wallet.
