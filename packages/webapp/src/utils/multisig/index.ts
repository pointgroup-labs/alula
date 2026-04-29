/**
 * Public entry point for the multisig lib.
 *
 * Pages and stores import from here, not from individual files. Lets us
 * reorganize internals without breaking consumers.
 */

export {
  buildProposal,
  decodeProposal,
  NETWORK_PASSPHRASES,
  NotImplementedError,
  signProposal,
  submitProposal,
  verifySigPayload,
} from './build'

export type {
  BuildProposalInput,
  DecodedProposal,
  SignProposalInput,
  SubmitProposalInput,
  SubmitProposalResult,
  VerifySigPayloadInput,
  VerifySigPayloadResult,
} from './build'

export { getFunctionDef, listAllFunctions, listFunctionsByRole } from './catalog'

export { bytesToHex, computeProposalHash, hexToBytes, sha256Hex } from './hash'
export { fetchSigs, postSig } from './relay'

export type { RelayConfig, RelayResult } from './relay'
export type {
  ArgFieldSchema,
  ArgSchema,
  ChainEnv,
  ContractKind,
  FunctionDef,
  HumanDiff,
  MultisigRole,
  ProposalPayload,
  SignerEntry,
  SigPayload,
  ThresholdsSnapshot,
} from './types'

export {
  decodeProposalFromFragment,
  encodeProposalToFragment,
  extractSigPayloads,
  isWellFormedSigPayload,
  parseSigPayload,
  serializeSigPayload,
} from './url'
export { verifyWasmAgainstClaim, verifyWasmFile } from './wasm'

export type { WasmHashCheck } from './wasm'
