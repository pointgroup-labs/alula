# Business Logic

Alula’s revenue model uses a dual-layer fee structure implemented entirely within the market smart contract. All protocol-side recipients (including the insurance fund and treasury) are configured as beneficiaries, enabling a unified routing model while allowing different split configurations per fee source.

## **Take rate (streaming)**;

As interest accrues, a configured take rate factor diverts a portion of borrower interest before it reaches lenders. The accrued take rate remains in the market contract and is distributed lazily according to the pool’s take rate beneficiary configuration when the permissionless `distribute` method is called. Supply APY is always shown net of the take rate.

## **Operation fee (atomic)**

Certain user operations charge an operation fee in basis points of the operation’s principal (for example `borrow` or `flash_loan`). If a valid `referrer_address` is provided, the optional referrer payout is settled instantly in the same transaction (the share is defined by `referrer_share_bps` in the partner registry). The partner registry is a registry of partners’ wallet addresses and their referrer fee shares (for example, an address used by LOBSTR can receive 10% of each operation fee for users who access Alula via LOBSTR). The remaining net fee accrues in the market contract and is routed on `distribute` according to the pool’s `origination_beneficiaries` configuration.
