/**
 * Soroban RPC helpers used by the multisig lib.
 *
 * Thin wrappers over `@stellar/stellar-sdk/rpc` that return only the shapes
 * the lib cares about. Keeps the SDK surface contained to one file so future
 * SDK upgrades have one place to touch.
 */

import type { Account } from '@stellar/stellar-sdk'
import type { SignerEntry, ThresholdsSnapshot } from './types'
import { StrKey, xdr } from '@stellar/stellar-sdk'
import { Server as RpcServer } from '@stellar/stellar-sdk/rpc'

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
