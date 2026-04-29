/**
 * URL fragment and signature payload codecs.
 *
 * The proposal payload travels as a base64url-encoded JSON blob in a URL
 * fragment (#p=…). The sig payload travels as a short colon-delimited string
 * (alula-sig:v1:<hash>:<G…>:<sig>) suitable for either chat paste or relay
 * POST. See spec §11.
 */

import type { ProposalPayload, SigPayload } from './types'

const SIG_PREFIX = 'alula-sig:v1:'
const SIG_REGEX = /^alula-sig:v1:([0-9a-f]{64}):(G[A-Z2-7]{55}):([A-Za-z0-9+/]+={0,2})$/

/** Base64url encode without padding. Browser-safe (no Buffer dependency). */
function bytesToB64url(bytes: Uint8Array): string {
  let bin = ''
  for (const byte of bytes) { bin += String.fromCharCode(byte ?? 0) }
  return btoa(bin).replaceAll('+', '-').replaceAll('/', '_').replace(/=+$/, '')
}

function b64urlToBytes(s: string): Uint8Array {
  const pad = s.length % 4 === 0 ? '' : '='.repeat(4 - (s.length % 4))
  const bin = atob(s.replaceAll('-', '+').replaceAll('_', '/') + pad)
  const out = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) { out[i] = bin.charCodeAt(i) }
  return out
}

export function encodeProposalToFragment(p: ProposalPayload): string {
  const json = JSON.stringify(p)
  const bytes = new TextEncoder().encode(json)
  return bytesToB64url(bytes)
}

export function decodeProposalFromFragment(fragment: string): ProposalPayload {
  if (!fragment) { throw new Error('empty fragment') }
  if (!/^[\w-]+$/.test(fragment)) { throw new Error('fragment is not base64url') }
  const bytes = b64urlToBytes(fragment)
  const json = new TextDecoder().decode(bytes)
  let parsed: unknown
  try {
    parsed = JSON.parse(json)
  } catch {
    throw new Error('fragment is not JSON')
  }
  return validateProposalPayload(parsed)
}

function validateProposalPayload(x: unknown): ProposalPayload {
  if (!x || typeof x !== 'object') { throw new Error('payload not an object') }
  const o = x as Record<string, unknown>
  if (o.v !== 1) { throw new Error(`unsupported payload version: ${String(o.v)}`) }
  if (o.kind !== 'proposal') { throw new Error(`unsupported payload kind: ${String(o.kind)}`) }
  // Structural sanity; full schema validation is the catalog's job at decode time.
  for (const key of [
    'network_passphrase',
    'multisig',
    'function_id',
    'unsigned_xdr',
    'proposal_hash',
    'created_by',
  ]) {
    if (typeof o[key] !== 'string') { throw new TypeError(`payload.${key} must be a string`) }
  }
  if (typeof o.created_at !== 'number') { throw new TypeError('payload.created_at must be a number') }
  if (!Array.isArray(o.signer_set_snapshot)) { throw new TypeError('payload.signer_set_snapshot must be an array') }
  return o as unknown as ProposalPayload
}

export function isWellFormedSigPayload(s: string): boolean {
  return SIG_REGEX.test(s.trim())
}

export function parseSigPayload(s: string): SigPayload {
  const m = SIG_REGEX.exec(s.trim())
  if (!m || !m[1] || !m[2] || !m[3]) { throw new Error('malformed sig payload') }
  return { proposal_hash: m[1], signer_pubkey: m[2], signature_b64: m[3] }
}

export function serializeSigPayload(sig: SigPayload): string {
  const out = `${SIG_PREFIX}${sig.proposal_hash}:${sig.signer_pubkey}:${sig.signature_b64}`
  if (!isWellFormedSigPayload(out)) {
    throw new Error('refusing to serialize malformed sig payload')
  }
  return out
}

/**
 * Extract one or more sig payloads from a free-form blob (e.g. a chat paste).
 * Returns only well-formed entries; silently skips garbage between them.
 */
export function extractSigPayloads(blob: string): SigPayload[] {
  const out: SigPayload[] = []
  for (const line of blob.split(/\r?\n/)) {
    const trimmed = line.trim()
    if (isWellFormedSigPayload(trimmed)) { out.push(parseSigPayload(trimmed)) }
  }
  return out
}
