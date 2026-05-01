/**
 * Build an unsigned proposal envelope for a catalog function call.
 *
 * Implements the assembly half of buildProposal: account fetch, signer-set
 * snapshot, timeBounds, fee policy, payload + hash. The catalog-args→ScVal
 * encoding is delegated to encode.ts (which today only knows wasm-hash).
 *
 * `simulateProposalEnvelope` is a separate exported helper rather than an
 * implicit step inside buildProposal — the compose page wants the envelope
 * even when simulation fails (so the operator can see the diff and decide),
 * and signers may want to re-simulate before signing without rebuilding.
 */

import type { BuildProposalInput } from './build'
import type {
  ChainEnv,
  FunctionDef,
  ProposalPayload,
} from './types'
import {
  Address,
  Operation,
  Transaction,
  TransactionBuilder,
} from '@stellar/stellar-sdk'
import { Api, Server as RpcServer } from '@stellar/stellar-sdk/rpc'
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

export type SimulateResult
  = | { ok: true, minResourceFee: string | null }
    | { ok: false, error: string }

/**
 * Re-hydrates a built proposal envelope and asks Soroban RPC to simulate
 * it. Catches three classes of issue at compose-time that would otherwise
 * only surface at submit-time:
 *
 *  - Auth failures (e.g. the multisig isn't actually the manager's admin).
 *  - Contract-side reverts (e.g. queueing while another upgrade is pending).
 *  - Argument decoding errors on the contract side.
 *
 * Returns a string error rather than throwing so the caller can render it
 * inline next to the proposal preview without try/catch noise.
 */
export async function simulateProposalEnvelope(
  rpcUrl: string,
  networkPassphrase: string,
  unsignedXdr: string,
): Promise<SimulateResult> {
  try {
    const tx = new Transaction(unsignedXdr, networkPassphrase)
    const server = new RpcServer(rpcUrl)
    const sim = await server.simulateTransaction(tx)
    if (Api.isSimulationError(sim)) {
      return { ok: false, error: sim.error || 'simulation failed (no error message)' }
    }
    const minResourceFee = 'minResourceFee' in sim ? String(sim.minResourceFee) : null
    return { ok: true, minResourceFee }
  } catch (error) {
    return { ok: false, error: (error as Error).message ?? String(error) }
  }
}

// Note: this builds the envelope without simulation. The compose page is
// expected to call `simulateProposalEnvelope` after build to surface
// auth/revert errors early; signers may re-simulate before signing.
// Sizing the resource fee from the simulation result (DEFAULT_BASE_FEE ×
// DEFAULT_RESOURCE_FEE_BUFFER_MULT, capped at the simulator's
// minResourceFee) is the next polish step — see spec §10 week 4.
export const RESERVED_FOR_FUTURE = { DEFAULT_RESOURCE_FEE_BUFFER_MULT }
