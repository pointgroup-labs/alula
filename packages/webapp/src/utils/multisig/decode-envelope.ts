/**
 * Decode and integrity-check a proposal envelope.
 *
 * The hash binds args + xdr together, but `decodeProposal` runs *before* the
 * sig page asks the wallet to sign — so we re-encode `payload.args` through
 * the catalog's argSchema and compare the resulting ScVal vector against the
 * one carried in the InvokeHostFunctionOp. Any drift = reject before signing.
 *
 * Also confirms:
 *  - exactly one operation, of kind `invokeHostFunction`
 *  - host function is `invokeContract`
 *  - contract function name matches `fn.function`
 */

import { Transaction, xdr } from '@stellar/stellar-sdk'
import type { DecodedProposal } from './build'
import type { ProposalPayload } from './types'
import { getFunctionDef } from './catalog'
import { encodeArgsToScVals } from './encode'

export function decodeProposalEnvelope(payload: ProposalPayload): DecodedProposal {
  const fn = getFunctionDef(payload.function_id)
  if (!fn) {
    throw new Error(`unknown function_id "${payload.function_id}" — catalog mismatch`)
  }
  if (fn.multisig !== payload.multisig) {
    throw new Error(
      `multisig role mismatch: payload says "${payload.multisig}", catalog says "${fn.multisig}"`,
    )
  }

  const tx = new Transaction(payload.unsigned_xdr, payload.network_passphrase)
  if (tx.operations.length !== 1) {
    throw new Error(`expected exactly 1 operation, got ${tx.operations.length}`)
  }

  const op = tx.operations[0]
  if (!op || op.type !== 'invokeHostFunction') {
    throw new Error(`expected invokeHostFunction op, got ${op?.type ?? 'undefined'}`)
  }

  const hostFn = op.func
  if (hostFn.switch().value !== xdr.HostFunctionType.hostFunctionTypeInvokeContract().value) {
    throw new Error('host function is not invokeContract')
  }

  const invokeArgs = hostFn.invokeContract()
  const fnNameSym = invokeArgs.functionName()
  const fnName = typeof fnNameSym === 'string' ? fnNameSym : fnNameSym.toString()
  if (fnName !== fn.function) {
    throw new Error(`function name mismatch: xdr "${fnName}", catalog "${fn.function}"`)
  }

  const xdrArgs = invokeArgs.args()
  const reEncoded = encodeArgsToScVals(fn.argSchema, payload.args)
  if (!scvalArraysEqual(xdrArgs, reEncoded)) {
    throw new Error('args drift: payload.args does not re-encode to the ScVal vector in unsigned_xdr')
  }

  return { payload, fn }
}

function scvalArraysEqual(a: ReadonlyArray<xdr.ScVal>, b: ReadonlyArray<xdr.ScVal>): boolean {
  if (a.length !== b.length) { return false }
  for (let i = 0; i < a.length; i++) {
    const av = a[i]
    const bv = b[i]
    if (!av || !bv) { return false }
    // XDR byte-level equality is the canonical comparison for ScVals.
    if (av.toXDR('base64') !== bv.toXDR('base64')) { return false }
  }
  return true
}
