#![no_std]

use soroban_sdk::{Address, Env, contractclient};

/// Flash-loan receiver interface, modeled on EIP-3156's `IERC3156FlashBorrower`.
///
/// # Arguments
/// * `initiator` - the address that called `flash_loan` on the market. Receivers MUST
///   validate this against an allowlist of trusted initiators to prevent unauthorized
///   third parties from forcing flash loans against the receiver, e.g.:
///
///   ```ignore
///   if initiator != Self::trusted_initiator(&e) { panic!("untrusted initiator") }
///   ```
///
/// * `token`     - the token being borrowed.
/// * `amount`    - the amount borrowed (in token units).
/// * `fee`       - the absolute fee owed in addition to `amount`, in token units. The
///   receiver must hold/produce `amount + fee` of `token` and approve the market to
///   pull it before `exec_op` returns. Prefer just-in-time approvals scoped to the
///   exact `amount + fee` over standing allowances.
#[contractclient(name = "FlashLoanClient")]
pub trait ModErc3156 {
    fn exec_op(e: Env, initiator: Address, token: Address, amount: i128, fee: i128);
}
