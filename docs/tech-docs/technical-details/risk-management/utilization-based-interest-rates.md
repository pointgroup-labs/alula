# Utilization-Based Interest Rates

Borrow APR increases as pool utilization rises. As a pool becomes more utilized, borrowing becomes more expensive and supplying becomes more attractive. This encourages repayments and new supply, pushing utilization back toward a healthy band without manual intervention.

## Borrow APR

Alula uses a kinked interest rate model with two kink points. Borrow APR is a piecewise-linear function of utilization.

**Below the first kink**

If _U_ < _U\_k1_:

BorrowAPR = BaseAPR + (U/U\_k1) × (APR\_k1 - BaseAPR)

**Between the kinks**

If _U ∈ \[U\_k1, U\_k2)_:

BorrowAPR = APR\_k1 + \[(U - U\_k1)/(U\_k2 - U\_k1)] × (APR\_k2 - APR\_k1)

**Above the second kink**

If _U_ ≥ _U\_k2_:

BorrowAPR = APR\_k2 + \[(U - U\_k2)/(1 - U\_k2)] × (APR\_max - APR\_k2)

**Per-second borrow rate**

BorrowRate\_per\_second = BorrowAPR / SECONDS\_IN\_YEAR

### Legend

* _U_: utilization ratio, defined as total borrowed amount / total supplied liquidity
* _U\_k1_: utilization at the 1st kink point
* _U\_k2_: utilization at the 2nd kink point
* _BaseAPR_: base APR that is always present
* _APR\_k1_: borrow APR at _U\_k1_
* _APR\_k2_: borrow APR at _U\_k2_
* _APR\_max_: maximum borrow APR at _U = 1_

### Constraints

* _0 < U\_k1 <= U\_k2 <= 1_
* _BaseAPR ≤ APR\_k1 ≤ APR\_k2 ≤ APR\_max_
