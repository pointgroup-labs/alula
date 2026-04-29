/**
 * Soroban ScVal → catalog-args decoding.
 *
 * Inverse of `encode.ts`. Used by `decodeProposal` so the sign/aggregate
 * pages can render the human-readable args of a proposal before the wallet
 * is asked to sign — never display unsigned XDR opaquely.
 *
 * Phase 1 mirrors the encoder: only `wasm-hash` is supported. Other kinds
 * throw so a malformed-or-future proposal fails loudly at decode time
 * rather than rendering as `[object Object]`.
 */

import { xdr } from '@stellar/stellar-sdk'
import type { ArgFieldSchema, ArgSchema } from './types'
import { UnsupportedArgKindError } from './encode'

export function decodeScValsToArgs(schema: ArgSchema, scvals: xdr.ScVal[]): Record<string, unknown> {
  const fields = Object.entries(schema)
  if (scvals.length !== fields.length) {
    throw new Error(
      `arg arity mismatch: catalog expects ${fields.length}, envelope carries ${scvals.length}`,
    )
  }

  const out: Record<string, unknown> = {}
  for (let i = 0; i < fields.length; i++) {
    const entry = fields[i]
    const scval = scvals[i]
    if (!entry || !scval) {
      // Defensive — fields.length and scvals.length matched above so this
      // branch only fires under array-mutation pathologies.
      throw new Error(`internal: missing field or scval at index ${i}`)
    }
    const [name, field] = entry
    out[name] = decodeField(field, scval, name)
  }
  return out
}

function decodeField(schema: ArgFieldSchema, scval: xdr.ScVal, path: string): unknown {
  switch (schema.kind) {
    case 'wasm-hash': {
      if (scval.switch().value !== xdr.ScValType.scvBytes().value) {
        throw new TypeError(`${path}: expected ScVal bytes, got ${scval.switch().name}`)
      }
      const bytes = scval.bytes()
      if (bytes.length !== 32) {
        throw new Error(`${path}: wasm-hash must be 32 bytes, got ${bytes.length}`)
      }
      let hex = ''
      for (const b of bytes) {
        hex += (b ?? 0).toString(16).padStart(2, '0')
      }
      return hex
    }
    default:
      throw new UnsupportedArgKindError(schema.kind)
  }
}
