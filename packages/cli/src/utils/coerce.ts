/**
 * commander argument coercers — used via `Option.argParser(fn)`.
 *
 * Live in utils/ rather than inline in command files so the same
 * parsers can be reused by future command groups (audit, deploy, etc.)
 * and so they're trivially unit-testable without spinning up commander.
 */

import { StrKey } from '@stellar/stellar-sdk'

export function csv(v: string): string[] {
  return v.split(',').map(x => x.trim()).filter(Boolean)
}

export function int(v: string): number {
  const n = Number(v)
  if (!Number.isInteger(n) || n < 0) {
    throw new Error(`expected non-negative integer, got "${v}"`)
  }
  return n
}

/**
 * CSV of Stellar G-addresses (ed25519 public keys). Validates each
 * entry up-front with a targeted error message when an S-key (secret)
 * is supplied by mistake — that's the most common copy-paste error
 * (`stellar keys secret X` vs. `stellar keys public-key X`) and the
 * SDK's own error (`signer.ed25519PublicKey is invalid`) doesn't help
 * the operator spot the swap.
 */
export function gAddressList(v: string): string[] {
  const items = csv(v)
  for (const [i, item] of items.entries()) {
    if (StrKey.isValidEd25519PublicKey(item)) {
      continue
    }
    if (item.startsWith('S') && StrKey.isValidEd25519SecretSeed(item)) {
      throw new Error(
        `signer #${i + 1} looks like a SECRET key (S…), expected a PUBLIC key (G…). `
        + `Use \`stellar keys public-key <name>\` (not \`stellar keys secret <name>\`).`,
      )
    }
    throw new Error(
      `signer #${i + 1} "${item}" is not a valid Stellar public key (expected G…, 56 chars).`,
    )
  }
  return items
}
