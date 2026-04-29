/**
 * Shared types for Alula multisig coordination.
 *
 * See docs/superpowers/specs/2026-04-29-alula-multisig-design.md §11
 * for the on-the-wire formats these types describe.
 */

export type MultisigRole = 'upgrade' | 'program' | 'ops'

export type ContractKind = 'market' | 'market_manager' | 'controlled_insurance_fund'

export type SignerEntry = {
  /** Stellar G… public key */
  key: string
  /** Signer weight (1 in the v1 design; carried for forward-compat) */
  weight: number
}

export type ThresholdsSnapshot = {
  low: number
  med: number
  high: number
}

export type ProposalPayload = {
  v: 1
  kind: 'proposal'
  network_passphrase: string
  multisig: MultisigRole
  /** Catalog identifier, e.g. "market_manager.queue_in_market_upgrade" */
  function_id: string
  /** Decoded args matching the catalog entry's ArgSchema */
  args: Record<string, unknown>
  /** Optional "before" snapshot from chain at compose time */
  snapshot: Record<string, unknown> | null
  /** Base64-encoded Stellar TransactionEnvelope XDR (unsigned) */
  unsigned_xdr: string
  /** SHA-256 over canonical {network_passphrase, function_id, args, unsigned_xdr, created_at} */
  proposal_hash: string
  /** Composer's G… address */
  created_by: string
  /** Unix seconds */
  created_at: number
  /** Multisig account's signer set at compose time (for rotation detection) */
  signer_set_snapshot: SignerEntry[]
  /** Multisig account's thresholds at compose time */
  thresholds_snapshot: ThresholdsSnapshot
  /** For apply-stage proposals, the queue proposal's hash */
  parent_proposal_hash: string | null
}

export type SigPayload = {
  proposal_hash: string
  signer_pubkey: string
  signature_b64: string
}

/**
 * Catalog entry describing one privileged function.
 *
 * One file per entry under utils/multisig/catalog/{role}/.
 */
export type FunctionDef<Args = Record<string, unknown>, Snapshot = unknown> = {
  multisig: MultisigRole
  contract: ContractKind
  function: string
  /** Stable identifier used in ProposalPayload.function_id */
  id: string
  argSchema: ArgSchema
  isTimelocked: boolean
  pairWith?: { queue: string, apply: string, cancel?: string }
  fetchBeforeSnapshot?: (env: ChainEnv, args: Args) => Promise<Snapshot>
  renderSummary: (args: Args, snapshot: Snapshot | null) => HumanDiff
}

/**
 * Minimal schema description used to render the compose form
 * and decode args back from XDR for review.
 */
export type ArgSchema = Record<string, ArgFieldSchema>

export type ArgFieldSchema
  = | { kind: 'address' }
    | { kind: 'wasm-hash' }
    | { kind: 'u32' | 'u64' | 'i128' }
    | { kind: 'bool' }
    | { kind: 'string', maxLen?: number }
    | { kind: 'bytes', length?: number }
    | { kind: 'enum', variants: readonly string[] }
    | { kind: 'struct', fields: ArgSchema }
    | { kind: 'vec', element: ArgFieldSchema }

/**
 * Human-readable diff rendered on the sign page.
 */
export type HumanDiff = {
  title: string
  rows: Array<{
    label: string
    before?: string
    after: string
    /** Visual emphasis hint */
    severity?: 'info' | 'warning' | 'critical'
  }>
}

/**
 * Chain access surface passed to fetchBeforeSnapshot.
 * Implementations live in utils/multisig/chain.ts.
 */
export type ChainEnv = {
  rpcUrl: string
  networkPassphrase: string
  /** Resolved contract addresses for the current deployment */
  addresses: {
    market_manager: string
    market: string
    controlled_insurance_fund?: string
  }
}
