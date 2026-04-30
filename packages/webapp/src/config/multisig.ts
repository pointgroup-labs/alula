/**
 * Known multisig accounts by network + role.
 *
 * Operators register a freshly-deployed multisig here so the Compose page
 * can auto-select it when a function with the matching role is picked.
 * Empty entries are fine — the picker falls back to manual address entry
 * whenever a lookup returns undefined.
 *
 * Setup ceremony for new multisigs lives in `docs/multisig.md`.
 */

import type { MultisigRole } from '~/utils/multisig'
import { Network } from '~/store/rpc'

export const KNOWN_MULTISIGS: Partial<Record<Network, Partial<Record<MultisigRole, string>>>> = {
  [Network.Testnet]: {
    // ops: 'G…',
    // program: 'G…',
    // upgrade: 'G…',
  },
  [Network.Public]: {
    // ops: 'G…',
    // program: 'G…',
    // upgrade: 'G…',
  },
}
