/**
 * SHA-256 helpers used both for proposal hashing and WASM-hash verification.
 *
 * Uses the Web Crypto API (available in browsers and in Cloudflare Workers).
 * No Node-only APIs.
 */

import type { ProposalPayload } from './types'

export async function sha256Hex(bytes: Uint8Array): Promise<string> {
  const buf = bytes.buffer.slice(bytes.byteOffset, bytes.byteOffset + bytes.byteLength) as ArrayBuffer
  const digest = await crypto.subtle.digest('SHA-256', buf)
  return bytesToHex(new Uint8Array(digest))
}

export function bytesToHex(bytes: Uint8Array): string {
  let s = ''
  for (let i = 0; i < bytes.length; i++) s += (bytes[i] ?? 0).toString(16).padStart(2, '0')
  return s
}

export function hexToBytes(hex: string): Uint8Array {
  if (hex.length % 2 !== 0) throw new Error('hex must be even-length')
  const out = new Uint8Array(hex.length / 2)
  for (let i = 0; i < out.length; i++) {
    const byte = parseInt(hex.slice(i * 2, i * 2 + 2), 16)
    if (Number.isNaN(byte)) throw new Error('invalid hex digit')
    out[i] = byte
  }
  return out
}

/**
 * Canonical proposal hash as defined in spec §11.1:
 *   sha256(canonical_json({network_passphrase, function_id, args, unsigned_xdr, created_at}))
 *
 * Canonical JSON = JSON.stringify with sorted object keys at every level.
 * The hash binds a sig payload to a specific proposal.
 */
export async function computeProposalHash(p: Pick<
  ProposalPayload,
  'network_passphrase' | 'function_id' | 'args' | 'unsigned_xdr' | 'created_at'
>): Promise<string> {
  const canonical = canonicalJsonStringify({
    network_passphrase: p.network_passphrase,
    function_id: p.function_id,
    args: p.args,
    unsigned_xdr: p.unsigned_xdr,
    created_at: p.created_at,
  })
  const bytes = new TextEncoder().encode(canonical)
  return sha256Hex(bytes)
}

/**
 * JSON.stringify with deterministic key order — required so the same payload
 * hashes identically on every signer's machine regardless of object key
 * insertion order.
 */
function canonicalJsonStringify(value: unknown): string {
  if (value === null || typeof value !== 'object') return JSON.stringify(value)
  if (Array.isArray(value)) return `[${value.map(canonicalJsonStringify).join(',')}]`
  const keys = Object.keys(value as Record<string, unknown>).sort()
  const parts = keys.map(k =>
    `${JSON.stringify(k)}:${canonicalJsonStringify((value as Record<string, unknown>)[k])}`,
  )
  return `{${parts.join(',')}}`
}
