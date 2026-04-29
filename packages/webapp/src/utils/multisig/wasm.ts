/**
 * WASM-hash verification helpers.
 *
 * The Upgrade-class flow requires the operator to upload a .wasm file at
 * compose time; the page computes its SHA-256 client-side and shows the
 * result alongside the on-chain market_wasm_hash. Each signer's view also
 * accepts a re-uploaded artifact to re-verify.
 *
 * Spec §5.3 invariant 5.
 */

import { sha256Hex } from './hash'

export type WasmHashCheck = {
  /** SHA-256 of the uploaded bytes, lowercase hex */
  computed_hash: string
  /** Hash claimed by the proposal (typically the queue tx's `new_wasm_hash` arg) */
  claimed_hash: string
  matches: boolean
  byte_size: number
}

export async function verifyWasmAgainstClaim(
  wasmBytes: Uint8Array,
  claimedHashHex: string,
): Promise<WasmHashCheck> {
  const computed = await sha256Hex(wasmBytes)
  return {
    computed_hash: computed,
    claimed_hash: claimedHashHex.toLowerCase(),
    matches: computed === claimedHashHex.toLowerCase(),
    byte_size: wasmBytes.length,
  }
}

/** Convenience for File inputs from the compose page. */
export async function verifyWasmFile(file: File, claimedHashHex: string): Promise<WasmHashCheck> {
  const buf = new Uint8Array(await file.arrayBuffer())
  return verifyWasmAgainstClaim(buf, claimedHashHex)
}

export { bytesToHex } from './hash'
