/**
 * Pure quorum logic for `multisig check-quorum`. Lives here (instead
 * of inline in the command) so it's testable without Horizon:
 *   - Input: list of on-chain signers (key+weight) + list of decorated
 *     signature hints from a parsed Transaction.
 *   - Output: which signers were matched, total weight, unrecognized
 *     hints.
 *
 * Hints are the last 4 bytes of the signer's ed25519 public key.
 * Collisions are theoretically possible (P ≈ N²/2³³) but vanishingly
 * rare for ≤10-signer multisigs.
 */

import { Keypair } from '@stellar/stellar-sdk'

export interface SignerEntry {
  key: string
  weight: number
  type: string
}

export interface QuorumResult {
  matchedSigners: string[]
  unrecognizedHints: string[]
  reachedWeight: number
}

export function matchSignaturesToSigners(
  signers: readonly SignerEntry[],
  signatureHints: readonly Buffer[],
): QuorumResult {
  const byHint = new Map<string, SignerEntry>()
  for (const s of signers) {
    if (s.type !== 'ed25519_public_key') {
      continue
    }
    const hint = Keypair.fromPublicKey(s.key).signatureHint().toString('base64')
    byHint.set(hint, s)
  }

  const matchedSigners: string[] = []
  const unrecognizedHints: string[] = []
  let reachedWeight = 0
  for (const sigHint of signatureHints) {
    const hint = sigHint.toString('base64')
    const known = byHint.get(hint)
    if (!known) {
      unrecognizedHints.push(hint)
      continue
    }
    matchedSigners.push(known.key)
    reachedWeight += known.weight
  }

  return { matchedSigners, unrecognizedHints, reachedWeight }
}
