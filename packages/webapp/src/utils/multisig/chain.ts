/**
 * Soroban RPC helpers used by the multisig lib.
 *
 * Thin wrappers over `@stellar/stellar-sdk/rpc` that return only the shapes
 * the lib cares about. Keeps the SDK surface contained to one file so future
 * SDK upgrades have one place to touch.
 */

import type { Account } from '@stellar/stellar-sdk'
import type { SignerEntry, ThresholdsSnapshot } from './types'
import { Address, rpc, scValToNative, StrKey, TransactionBuilder, xdr } from '@stellar/stellar-sdk'

// Import the rpc namespace from the SDK *root* (not `@stellar/stellar-sdk/rpc`).
// The package's browser export condition routes `.` to a self-contained bundle
// but leaves `./rpc` going through `lib/rpc/index.js` — two module graphs, two
// `Transaction` class identities. `instanceof Transaction` inside the rpc
// Server then fails on txs built with the root's `TransactionBuilder`.
const RpcServer = rpc.Server

export type ChainAccount = {
  /** G… */
  address: string
  /** Current sequence number as a decimal string */
  sequence: string
  /** A constructed Account ready to feed to TransactionBuilder */
  account: Account
}

export async function loadAccount(rpcUrl: string, address: string): Promise<ChainAccount> {
  const server = new RpcServer(rpcUrl)
  const account = await server.getAccount(address)
  return {
    address,
    sequence: account.sequenceNumber(),
    account,
  }
}

export type MultisigAccountState = {
  signers: SignerEntry[]
  thresholds: ThresholdsSnapshot
}

/**
 * Reads the multisig account's signer list and thresholds from chain.
 *
 * Notes:
 *  - We deliberately use the RPC `getAccountEntry` raw XDR rather than the
 *    Horizon JSON representation so we don't take a Horizon dependency.
 *  - The classic-account "master signer" (the account itself) is included if
 *    its weight is non-zero, since it counts toward thresholds at submission
 *    time.
 */
export async function loadMultisigState(rpcUrl: string, address: string): Promise<MultisigAccountState> {
  const server = new RpcServer(rpcUrl)
  const accountEntry = await server.getAccountEntry(address)

  const thresholds = accountEntry.thresholds()
  const masterWeight = thresholds[0] ?? 0
  const lowThreshold = thresholds[1] ?? 0
  const medThreshold = thresholds[2] ?? 0
  const highThreshold = thresholds[3] ?? 0

  const signers: SignerEntry[] = []
  if (masterWeight > 0) {
    signers.push({ key: address, weight: masterWeight })
  }
  for (const s of accountEntry.signers()) {
    const key = s.key()
    // Only ED25519 signers are usable for tx envelope signing.
    if (key.switch().value === xdr.SignerKeyType.signerKeyTypeEd25519().value) {
      const ed = key.ed25519()
      signers.push({
        key: StrKey.encodeEd25519PublicKey(Buffer.from(ed)),
        weight: s.weight(),
      })
    }
  }

  return {
    signers,
    thresholds: {
      low: lowThreshold,
      med: medThreshold,
      high: highThreshold,
    },
  }
}

export type QueuedUpgrade = {
  /** 64 lowercase hex chars. */
  wasmHash: string
  /** Unix seconds (u64 from contract). Decoded as `number`; safe for now. */
  queuedAtUnix: number
}

export type ManagerState = {
  /** True iff the contract instance exists on chain. */
  exists: boolean
  /** Admin G…/C… — the address the contract requires `require_auth` from. */
  admin?: string
  /** Current Market wasm hash the manager deploys / upgrades to. */
  marketWasmHash?: string
  /** Pending Market wasm upgrade queued by `queue_in_market_upgrade`. */
  queuedMarketUpgrade?: QueuedUpgrade
  /** Pending Manager self-upgrade queued by `queue_in_manager_upgrade`. */
  queuedManagerUpgrade?: QueuedUpgrade
  /** Pending admin rotation, if `set_pending_admin` was called. */
  pendingAdmin?: string
  /** Number of registered Market contracts under this manager. */
  marketsCount?: number
  /** Set if introspection partially failed but the entry exists. Non-fatal. */
  parseError?: string
}

/**
 * Reads the market_manager contract's instance storage in one round-trip.
 *
 * Soroban packs every `env.storage().instance().set(...)` into the contract
 * *instance* ledger entry's `ScContractInstance.storage` SCMap, so one
 * `getLedgerEntries` call returns admin + market wasm hash + both queued
 * upgrade slots + pending admin + the markets map.
 *
 * Unit-variant `#[contracttype] enum DataKey` keys serialize as
 * `ScVal::Vec([Symbol("Variant")])`. We match by the first symbol and
 * decode each value with `scValToNative`, then convert addresses and
 * BytesN<32> hashes into UI-friendly strings.
 */
export async function loadManagerState(rpcUrl: string, address: string): Promise<ManagerState> {
  const server = new RpcServer(rpcUrl)

  const key = xdr.LedgerKey.contractData(
    new xdr.LedgerKeyContractData({
      contract: Address.fromString(address).toScAddress(),
      key: xdr.ScVal.scvLedgerKeyContractInstance(),
      durability: xdr.ContractDataDurability.persistent(),
    }),
  )

  let entries: Awaited<ReturnType<typeof server.getLedgerEntries>>
  try {
    entries = await server.getLedgerEntries(key)
  } catch (error) {
    return { exists: false, parseError: (error as Error).message ?? String(error) }
  }
  if (entries.entries.length === 0) {
    return { exists: false }
  }

  const val = entries.entries[0]!.val
  if (val.switch().name !== 'contractData') {
    return { exists: false, parseError: `unexpected ledger entry type: ${val.switch().name}` }
  }
  const data = val.contractData().val()
  if (data.switch().name !== 'scvContractInstance') {
    return { exists: true, parseError: `unexpected contract data val: ${data.switch().name}` }
  }

  const storage = data.instance().storage()
  if (!storage) {
    // Contract exists but has empty instance storage. That's only possible
    // before `initialize` runs; surface it explicitly rather than as `ok`.
    return { exists: true, parseError: 'contract instance storage is empty (uninitialized?)' }
  }

  const out: ManagerState = { exists: true }
  try {
    for (const entry of storage) {
      const keyName = readDataKeyName(entry.key())
      if (!keyName) { continue }
      const v = entry.val()
      switch (keyName) {
        case 'Admin':
          out.admin = decodeAddress(v) ?? undefined
          break
        case 'MarketWasmHash':
          out.marketWasmHash = decodeBytesHex(v) ?? undefined
          break
        case 'QueuedInMarketUpgrade':
          out.queuedMarketUpgrade = decodeQueuedUpgrade(v) ?? undefined
          break
        case 'QueuedInManagerUpgrade':
          out.queuedManagerUpgrade = decodeQueuedUpgrade(v) ?? undefined
          break
        case 'PendingAdmin':
          out.pendingAdmin = decodeAddress(v) ?? undefined
          break
        case 'MarketsList':
          out.marketsCount = decodeMapSize(v)
          break
      }
    }
  } catch (error) {
    out.parseError = `instance storage decode failed: ${(error as Error).message ?? String(error)}`
  }
  return out
}

/**
 * Match `ScVal::Vec([Symbol("X")])` — the on-chain shape of a Soroban
 * `#[contracttype]` enum unit variant — and return "X". Anything else
 * returns null so the caller can skip non-DataKey entries safely.
 */
function readDataKeyName(key: xdr.ScVal): string | null {
  if (key.switch().name !== 'scvVec') { return null }
  const vec = key.vec()
  if (!vec || vec.length === 0) { return null }
  const head = vec[0]!
  if (head.switch().name !== 'scvSymbol') { return null }
  return head.sym().toString()
}

function decodeAddress(v: xdr.ScVal): string | null {
  if (v.switch().name !== 'scvAddress') { return null }
  try { return Address.fromScVal(v).toString() } catch { return null }
}

function decodeBytesHex(v: xdr.ScVal): string | null {
  if (v.switch().name !== 'scvBytes') { return null }
  return Buffer.from(v.bytes()).toString('hex')
}

function decodeQueuedUpgrade(v: xdr.ScVal): QueuedUpgrade | null {
  if (v.switch().name !== 'scvMap') { return null }
  const map = v.map()
  if (!map) { return null }
  let wasmHash: string | null = null
  let queuedAtUnix: number | null = null
  for (const entry of map) {
    const k = entry.key()
    if (k.switch().name !== 'scvSymbol') { continue }
    const name = k.sym().toString()
    if (name === 'wasm_hash') { wasmHash = decodeBytesHex(entry.val()) }
    else if (name === 'queued_in_timestamp') {
      // u64 lands as bigint via scValToNative; clamp to number for UI.
      const native = scValToNative(entry.val())
      queuedAtUnix = typeof native === 'bigint' ? Number(native) : Number(native ?? 0)
    }
  }
  if (!wasmHash || queuedAtUnix === null) { return null }
  return { wasmHash, queuedAtUnix }
}

function decodeMapSize(v: xdr.ScVal): number | undefined {
  if (v.switch().name !== 'scvMap') { return undefined }
  return v.map()?.length ?? 0
}

export type ProposalAddresses = {
  /** Tx source — the multisig account that owns this proposal. */
  multisigAccount: string
  /** Operation target — the contract the host function invokes (C…). */
  targetContract: string | null
  /** Host function name, e.g. "queue_in_market_upgrade". */
  invokedFunction: string | null
}

/**
 * Decodes a proposal's unsigned envelope and surfaces the two addresses
 * the sign page needs for inspection: the multisig account (tx source)
 * and the target contract (the invokeContract op's contract address).
 *
 * Single SDK call, runs once on page load. We deliberately use
 * `TransactionBuilder.fromXDR` rather than raw XDR walks so future op
 * additions (multi-op envelopes, fee bumps) fail loudly rather than
 * returning wrong addresses silently.
 */
export function extractProposalAddresses(
  unsignedXdr: string,
  networkPassphrase: string,
): ProposalAddresses {
  const tx = TransactionBuilder.fromXDR(unsignedXdr, networkPassphrase)
  // `tx` may be a `FeeBumpTransaction` if compose ever wraps; the inner
  // tx is what carries the source and ops in that case. We don't compose
  // fee-bumps today, so reject loudly to surface a future schema change.
  if ('innerTransaction' in tx) {
    throw new Error('fee-bump envelopes are not supported here')
  }
  const multisigAccount = tx.source

  let targetContract: string | null = null
  let invokedFunction: string | null = null
  const op = tx.operations[0]
  if (op && op.type === 'invokeHostFunction') {
    const hostFn = op.func
    if (hostFn.switch().name === 'hostFunctionTypeInvokeContract') {
      const invokeContract = hostFn.invokeContract()
      try {
        targetContract = Address.fromScAddress(invokeContract.contractAddress()).toString()
      } catch {
        targetContract = null
      }
      try {
        invokedFunction = invokeContract.functionName().toString()
      } catch {
        invokedFunction = null
      }
    }
  }

  return { multisigAccount, targetContract, invokedFunction }
}
