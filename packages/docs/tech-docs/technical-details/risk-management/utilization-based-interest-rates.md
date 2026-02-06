# Utilization-Based Interest Rates

Borrow APR increases as pool utilization rises. As a pool becomes more utilized, borrowing becomes more expensive and supplying becomes more attractive. This encourages repayments and new supply, pushing utilization back toward a healthy band without manual intervention.

## Borrow APR

Alula uses a kinked interest rate model with two kink points. Borrow APR is a piecewise-linear function of utilization.

**Below the first kink**

If `U < U_k1`:

`BorrowAPR = BaseAPR + (U/U_k1) × (APR_k1 - BaseAPR)`

**Between the kinks**

If `U ∈ [U_k1, U_k2)`:

`BorrowAPR = APR_k1 + [(U - U_k1)/(U_k2 - U_k1)] × (APR_k2 - APR_k1)`

**Above the second kink**

If `U ≥ U_k2`:

`BorrowAPR = APR_k2 + [(U - U_k2)/(1 - U_k2)] × (APR_max - APR_k2)`

**Per-second borrow rate**

`BorrowRate_per_second = BorrowAPR / SECONDS_IN_YEAR`

### Legend

* `U`: utilization ratio, defined as total borrowed amount / total supplied liquidity
* `U_k1`: utilization at the 1st kink point
* `U_k2`: utilization at the 2nd kink point
* `BaseAPR`: base APR that is always present
* `APR_k1`: borrow APR at U_k1
* `APR_k2`: borrow APR at U_k2
* `APR_max`: maximum borrow APR at U = 1

### Constraints

* `0 < U_k1 <= U_k2 <= 1`
* `BaseAPR ≤ APR_k1 ≤ APR_k2 ≤ APR_max`
