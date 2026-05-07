# Changelog

All notable changes to the `moderc3156` flash-loan receiver interface are
documented here.

## [Unreleased]

> **Context:** these are pre-launch interface changes. As of this writing no
> third-party receivers are known to be deployed against the prior interface,
> so the practical migration burden is expected to be zero. The changes are
> nonetheless documented as breaking because they would be silent runtime
> failures for any receiver that *was* written against the old interface.

### Changed

- **`exec_op` parameter `caller` renamed to `initiator`** to align with EIP-3156
  semantics. The argument now contains the address that called `flash_loan` on
  the market, not the market's own address. Receivers should validate this
  against their trusted-initiator set:

  ```rust
  fn exec_op(e: Env, initiator: Address, token: Address, amount: i128, fee: i128) {
      assert_eq!(initiator, Self::trusted_initiator(&e), "untrusted initiator");
      // ...strategy...
  }
  ```

- **`exec_op` parameter `fee` is now the absolute fee in token units, not basis
  points.** Previously the market passed `pool.flash_loan_fee_bps` into a
  parameter the trait already named `fee` — an internal inconsistency the prior
  mock acknowledged by renaming the parameter to `fee_bps` in its impl. The
  trait now matches its declared semantics and EIP-3156:

  ```rust
  // Old internal behavior (parameter named `fee`, value was bps):
  let total = amount + amount * fee / BPS_FACTOR;

  // Now (parameter named `fee`, value is absolute token amount):
  let total = amount + fee;
  ```

  This is a same-name / same-type / different-meaning change. Any pre-existing
  receiver doing the bps math would silently under-pay by ~10000/fee_bps and
  the market's `transfer_from` would revert. Receivers using the new pattern
  ship correctly with no manual conversion.

### Security

- Closes audit finding "Steal Fees from users via Flash_Loan" (Medium).
  Before this change the market passed its own address (instead of the user's)
  to the receiver callback, leaving receivers without the EIP-3156-standard
  defense (`assert_eq!(initiator, trusted)`). With initiator forwarding in
  place, well-written receivers reject attacker-initiated flash loans before
  any fee is charged. See
  `contracts/flash_loan_taker_mock/src/lib.rs::StrictFlashLoanTakerContract`
  for the canonical receiver pattern, and the corresponding regression tests
  for the proof.

### Notes for receiver implementers

- Soroban has no `msg.sender` equivalent inside `exec_op`. Receivers must
  obtain the lender (market) address out-of-band — typically via a constructor
  parameter or instance storage — to call `approve` on the repayment token.
- Prefer just-in-time approvals scoped to exactly `amount + fee` over standing
  allowances. A standing allowance combined with a missing initiator check is
  the exact vector the audit finding describes.
