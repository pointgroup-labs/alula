/**
 * CLI-wide constants. Anything that's a tunable default or a hard-coded
 * environment-specific value lives here so it's discoverable in one
 * grep and configurable in one edit.
 */

import type { NetworkName } from './context'
import { Networks } from '@stellar/stellar-sdk'

// ── multisig setup defaults ────────────────────────────────────────────────

export const DEFAULT_HOME_DOMAIN = 'alula.finance'
export const DEFAULT_FEE_STROOPS = '10000'
export const DEFAULT_TIMEOUT_SECONDS = 120

// ── network presets ────────────────────────────────────────────────────────
// Operator can override via NETWORK_PASSPHRASE / HORIZON_URL env vars.

export const NETWORK_PRESETS: Record<NetworkName, { passphrase: string, horizon: string }> = {
  testnet: { passphrase: Networks.TESTNET, horizon: 'https://horizon-testnet.stellar.org' },
  public: { passphrase: Networks.PUBLIC, horizon: 'https://horizon.stellar.org' },
}
