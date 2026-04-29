/**
 * Build, sign, and submit transaction shells.
 *
 * Stub-level: each function is shaped for the real implementation but throws
 * `NotImplemented` until the Stellar SDK integration lands. This file is here
 * so the catalog and pages can import against a stable surface from day one.
 *
 * Concrete behavior is owed to a follow-up commit; see the implementation
 * plan at docs/superpowers/plans/.
 */

import type { ChainEnv, FunctionDef, ProposalPayload, SigPayload } from './types'

export class NotImplementedError extends Error {
  constructor(what: string) {
    super(`${what} is not yet implemented`)
    this.name = 'NotImplementedError'
  }
}

export interface BuildProposalInput<Args = Record<string, unknown>> {
  fn: FunctionDef<Args>
  args: Args
  /** G… of the multisig account whose seqnum and source we use */
  multisigAccountAddress: string
  env: ChainEnv
  /** Optional minTime override (epoch seconds) — used for apply-stage timelock */
  minTime?: number
  /** Optional maxTime override (epoch seconds) — defaults to minTime + 7d, or now + 30d */
  maxTime?: number
  /** Composer's G…, recorded in the payload for audit */
  composerAddress: string
  /** For apply-stage proposals, the queue proposal's hash */
  parentProposalHash?: string | null
}

export async function buildProposal<Args>(_input: BuildProposalInput<Args>): Promise<ProposalPayload> {
  throw new NotImplementedError('buildProposal')
}

export interface DecodedProposal {
  payload: ProposalPayload
  /** Resolved catalog entry */
  fn: FunctionDef
}

export async function decodeProposal(_payload: ProposalPayload): Promise<DecodedProposal> {
  throw new NotImplementedError('decodeProposal')
}

export interface SignProposalInput {
  payload: ProposalPayload
  /** Returns the base64 signature for the tx envelope's hash */
  signEnvelopeXdr: (xdrBase64: string, networkPassphrase: string) => Promise<{
    signedXdr: string
    signerPubkey: string
  }>
}

export async function signProposal(_input: SignProposalInput): Promise<SigPayload> {
  throw new NotImplementedError('signProposal')
}

export interface SubmitProposalInput {
  payload: ProposalPayload
  sigs: SigPayload[]
  rpcUrl: string
}

export interface SubmitProposalResult {
  txHash: string
  ledger: number
}

export async function submitProposal(_input: SubmitProposalInput): Promise<SubmitProposalResult> {
  throw new NotImplementedError('submitProposal')
}
