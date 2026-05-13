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

export { simulateProposalEnvelope } from './build-envelope'
export type { SimulateResult } from './build-envelope'

export { getFunctionDef, listAllFunctions, listFunctionsByRole } from './catalog'

export { extractProposalAddresses, loadAccount, loadManagerState, loadMultisigState } from './chain'
export type { ChainAccount, ManagerState, MultisigAccountState, ProposalAddresses, QueuedUpgrade } from './chain'

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
export { verifyHashOnChain } from './wasm'

export type { OnChainWasmInfo, WasmCustomSection } from './wasm'
