/**
 * Build an unsigned proposal envelope for a catalog function call.
 *
 * Implements the assembly half of buildProposal: account fetch, signer-set
 * snapshot, timeBounds, fee policy, payload + hash. The catalog-args→ScVal
 * encoding is delegated to encode.ts (which today only knows wasm-hash).
 */

import {
  Address,
  Operation,
  TransactionBuilder,
} from '@stellar/stellar-sdk'
import type { BuildProposalInput } from './build'
import type {
  ChainEnv,
  FunctionDef,
  ProposalPayload,
} from './types'
import { loadAccount, loadMultisigState } from './chain'
import { encodeArgsToScVals } from './encode'
import { computeProposalHash } from './hash'

const DEFAULT_BASE_FEE = 100_000 // 0.01 XLM, generous for inclusion
const DEFAULT_RESOURCE_FEE_BUFFER_MULT = 1.5
const DEFAULT_TIMEBOUNDS_VALIDITY_SECONDS = 30 * 24 * 3600 // 30d for non-timelocked
const DEFAULT_APPLY_VALIDITY_SECONDS = 7 * 24 * 3600 // 7d after minTime

export async function buildProposalEnvelope<Args>(input: BuildProposalInput<Args>): Promise<ProposalPayload> {
  const {
    fn,
    args,
    multisigAccountAddress,
    env,
    minTime,
    maxTime,
    composerAddress,
    parentProposalHash,
  } = input

  if (!env.networkPassphrase) {
    throw new Error('env.networkPassphrase is required')
  }

  const contractAddress = resolveContractAddress(env, fn)
  if (!contractAddress) {
    throw new Error(`no address configured for contract "${fn.contract}" in this env`)
  }

  // Snapshot signer set + thresholds at compose time so the signer page can
  // detect rotation later (spec §5.3 invariant 6).
  const [{ account }, signerState] = await Promise.all([
    loadAccount(env.rpcUrl, multisigAccountAddress),
    loadMultisigState(env.rpcUrl, multisigAccountAddress),
  ])

  const scvalArgs = encodeArgsToScVals(fn.argSchema, args as Record<string, unknown>)

  const op = Operation.invokeContractFunction({
    contract: new Address(contractAddress).toString(),
    function: fn.function,
    args: scvalArgs,
  })

  const now = Math.floor(Date.now() / 1000)
  const effectiveMinTime = minTime ?? 0
  const effectiveMaxTime = maxTime ?? (
    minTime
      ? minTime + DEFAULT_APPLY_VALIDITY_SECONDS
      : now + DEFAULT_TIMEBOUNDS_VALIDITY_SECONDS
  )

  const builder = new TransactionBuilder(account, {
    fee: String(DEFAULT_BASE_FEE),
    networkPassphrase: env.networkPassphrase,
    timebounds: { minTime: effectiveMinTime, maxTime: effectiveMaxTime },
  })
    .addOperation(op)

  const tx = builder.build()
  const unsignedXdr = tx.toEnvelope().toXDR('base64')

  const created_at = now
  const proposal_hash = await computeProposalHash({
    network_passphrase: env.networkPassphrase,
    function_id: fn.id,
    args: args as Record<string, unknown>,
    unsigned_xdr: unsignedXdr,
    created_at,
  })

  return {
    v: 1,
    kind: 'proposal',
    network_passphrase: env.networkPassphrase,
    multisig: fn.multisig,
    function_id: fn.id,
    args: args as Record<string, unknown>,
    snapshot: null,
    unsigned_xdr: unsignedXdr,
    proposal_hash,
    created_by: composerAddress,
    created_at,
    signer_set_snapshot: signerState.signers,
    thresholds_snapshot: signerState.thresholds,
    parent_proposal_hash: parentProposalHash ?? null,
  }
}

function resolveContractAddress(env: ChainEnv, fn: FunctionDef<any, any>): string | undefined {
  switch (fn.contract) {
    case 'market_manager': return env.addresses.market_manager
    case 'market': return env.addresses.market
    case 'controlled_insurance_fund': return env.addresses.controlled_insurance_fund
  }
}

// Note: this builds the envelope without simulation. A follow-up commit
// should add a `simulateTransaction` pass to size the resource fee
// (DEFAULT_BASE_FEE × DEFAULT_RESOURCE_FEE_BUFFER_MULT) and surface
// simulation failures at compose time. See spec §10 week 4 polish item.
export const RESERVED_FOR_FUTURE = { DEFAULT_RESOURCE_FEE_BUFFER_MULT }
