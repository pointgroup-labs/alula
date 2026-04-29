/**
 * Build and decode transaction shells.
 *
 * `signProposal` and `submitProposal` have real implementations in
 * `sign-submit.ts` and are re-exported below for back-compat. `buildProposal`
 * delegates to `build-envelope.ts`; `decodeProposal` delegates to
 * `decode-envelope.ts`. The catalog-args ↔ XDR layer lives in `encode.ts` /
 * `decode.ts` and is currently wasm-hash-only (Phase 1, Plan 2 scope).
 *
 * See `docs/superpowers/specs/2026-04-29-alula-multisig-design.md` §6 for
 * the catalog shape these operate on.
 */

import type { ChainEnv, FunctionDef, ProposalPayload } from './types'
import { buildProposalEnvelope } from './build-envelope'
import { decodeProposalEnvelope } from './decode-envelope'

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

export async function decodeProposal(payload: ProposalPayload): Promise<DecodedProposal> {
  return decodeProposalEnvelope(payload)
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
