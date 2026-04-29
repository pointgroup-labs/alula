/**
 * Catalog-args → Soroban ScVal encoding.
 *
 * Each catalog entry's argSchema describes the shape of its args; this
 * encoder turns the operator's typed input into the ScVal vector that
 * `InvokeHostFunctionOp` expects.
 *
 * Phase 1 implements only the kinds the Upgrade catalog actually uses
 * (`wasm-hash`); other kinds throw NotImplementedError until Plan 3
 * (Program catalog) and Plan 4 (Ops catalog) need them.
 */

import { xdr } from '@stellar/stellar-sdk'
import type { ArgFieldSchema, ArgSchema } from './types'

export class UnsupportedArgKindError extends Error {
  constructor(kind: string) {
    super(`arg kind "${kind}" is not yet supported by the multisig encoder`)
    this.name = 'UnsupportedArgKindError'
  }
}

/**
 * Encode an args object into the positional ScVal vector that
 * `Operation.invokeContractFunction` expects, in the field order declared by
 * `schema`.
 */
export function encodeArgsToScVals(schema: ArgSchema, args: Record<string, unknown>): xdr.ScVal[] {
  const out: xdr.ScVal[] = []
  for (const [name, field] of Object.entries(schema)) {
    if (!(name in args)) throw new Error(`missing arg: ${name}`)
    out.push(encodeField(field, args[name], name))
  }
  return out
}

function encodeField(schema: ArgFieldSchema, value: unknown, path: string): xdr.ScVal {
  switch (schema.kind) {
    case 'wasm-hash': {
      if (typeof value !== 'string') throw new TypeError(`${path}: wasm-hash must be a hex string`)
      const hex = value.toLowerCase()
      if (!/^[0-9a-f]{64}$/.test(hex)) throw new Error(`${path}: wasm-hash must be 64 hex chars`)
      const bytes = Buffer.alloc(32)
      for (let i = 0; i < 32; i++) bytes[i] = Number.parseInt(hex.slice(i * 2, i * 2 + 2), 16)
      return xdr.ScVal.scvBytes(bytes)
    }
    default:
      // Every other kind is owed by Plans 3 and 4. Fail loudly at compose time
      // rather than producing a malformed XDR that signers might rubber-stamp.
      throw new UnsupportedArgKindError(schema.kind)
  }
}
