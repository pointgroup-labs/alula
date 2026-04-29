/**
 * Build and decode transaction shells.
 *
 * `signProposal` and `submitProposal` now have real implementations in
 * `sign-submit.ts` and are re-exported below for back-compat. `buildProposal`
 * and `decodeProposal` remain stubs until the catalog-args ↔ XDR encoding
 * layer lands; see `docs/superpowers/specs/2026-04-29-alula-multisig-design.md`
 * §6 for the catalog shape these will operate on.
 */

import type { ChainEnv, FunctionDef, ProposalPayload } from './types'
import { buildProposalEnvelope } from './build-envelope'

export class NotImplementedError extends Error {
  constructor(what: string) {
    super(`${what} is not yet implemented`)
    this.name = 'NotImplementedError'
  }
}

export type BuildProposalInput<Args = Record<string, unknown>> = {
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

export async function buildProposal<Args>(input: BuildProposalInput<Args>): Promise<ProposalPayload> {
  return buildProposalEnvelope(input)
}

export type DecodedProposal = {
  payload: ProposalPayload
  /** Resolved catalog entry */
  fn: FunctionDef
}

export async function decodeProposal(_payload: ProposalPayload): Promise<DecodedProposal> {
  throw new NotImplementedError('decodeProposal')
}

export {
  NETWORK_PASSPHRASES,
  signProposal,
  submitProposal,
  verifySigPayload,
} from './sign-submit'

export type {
  SignProposalInput,
  SubmitProposalInput,
  SubmitProposalResult,
  VerifySigPayloadInput,
  VerifySigPayloadResult,
} from './sign-submit'
