/**
 * Governance / upgrade timelock parameters mirrored from the on-chain
 * contracts. These MUST stay in sync with the Rust constants — if the
 * contracts ship a different value, the transparency page will lie about
 * unlock times.
 *
 * Follow-up: expose this via a `upgrade_queue_period()` view function on
 * the market_manager contract and read it at runtime instead of mirroring.
 *
 * Source: `contracts/market_manager/src/constants.rs::UPGRADE_IN_QUEUE_SECONDS`.
 */
export const UPGRADE_QUEUE_PERIOD_SECONDS = 2 * 60
