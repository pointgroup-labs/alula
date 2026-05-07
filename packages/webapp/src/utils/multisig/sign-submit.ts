/**
 * Sign and submit implementations.
 *
 * `signProposal` is a thin adapter over a wallet-kit-style XDR signer: it
 * hands the wallet the unsigned envelope, takes back the wallet's signed
 * envelope, extracts the new signature from the envelope's signature list,
 * and packages it as a SigPayload.
 *
 * `submitProposal` attaches a list of validated SigPayloads to the unsigned
 * envelope and broadcasts via Soroban RPC.
 *
 * `verifySigPayload` is pure crypto — used by the aggregator page to validate
 * sigs at paste/relay time before letting them affect the dashboard.
 *
 * Spec §5.3 invariants 2 and 3.
 */

import type { ProposalPayload, SigPayload } from './types'
import { Keypair, Networks, Transaction, xdr } from '@stellar/stellar-sdk'
import { Server as RpcServer } from '@stellar/stellar-sdk/rpc'

export type SignProposalInput = {
  payload: ProposalPayload
  /**
   * Hands the unsigned envelope XDR to a wallet (Freighter, Lobstr, etc.) and
   * receives the same envelope with one new DecoratedSignature appended.
   *
   * The wallet is expected to honor the network passphrase passed in.
   */
  signEnvelopeXdr: (xdrBase64: string, networkPassphrase: string) => Promise<{
    signedXdr: string
    signerPubkey: string
  }>
}

export async function signProposal(input: SignProposalInput): Promise<SigPayload> {
  const { payload, signEnvelopeXdr } = input

  const beforeSigCount = countEnvelopeSignatures(payload.unsigned_xdr)

  const { signedXdr, signerPubkey } = await signEnvelopeXdr(
    payload.unsigned_xdr,
    payload.network_passphrase,
  )

  const newSig = extractAddedSignature(payload.unsigned_xdr, signedXdr, beforeSigCount)
  if (!newSig) {
    throw new Error('wallet returned XDR with no new signature')
  }

  // Verify the wallet's claimed pubkey actually produced the sig we extracted.
  const txHashBytes = transactionHash(payload.unsigned_xdr, payload.network_passphrase)
  if (!ed25519Verify(signerPubkey, txHashBytes, base64ToBytes(newSig))) {
    throw new Error('wallet returned a signature that does not verify against its claimed pubkey')
  }

  return {
    proposal_hash: payload.proposal_hash,
    signer_pubkey: signerPubkey,
    signature_b64: newSig,
  }
}

export type SubmitProposalInput = {
  payload: ProposalPayload
  /** Sigs that have already passed verifySigPayload */
  sigs: SigPayload[]
  rpcUrl: string
}

export type SubmitProposalResult = {
  txHash: string
  /** Soroban RPC returns 'PENDING' first; caller is responsible for polling getTransaction */
  status: string
}

export async function submitProposal(input: SubmitProposalInput): Promise<SubmitProposalResult> {
  const { payload, sigs, rpcUrl } = input

  const tx = new Transaction(payload.unsigned_xdr, payload.network_passphrase)
  for (const s of sigs) {
    const decoratedSig = makeDecoratedSignature(s.signer_pubkey, s.signature_b64)
    tx.signatures.push(decoratedSig)
  }

  const server = new RpcServer(rpcUrl)
  const res = await server.sendTransaction(tx)
  return { txHash: res.hash, status: res.status }
}

export type VerifySigPayloadInput = {
  payload: ProposalPayload
  sig: SigPayload
  /**
   * Snapshot of allowed signer keys (G…) for the multisig account.
   * Typically payload.signer_set_snapshot.map(s => s.key).
   */
  allowedSigners: string[]
}

export type VerifySigPayloadResult = {
  ok: boolean
  reason?: string
}

export function verifySigPayload(input: VerifySigPayloadInput): VerifySigPayloadResult {
  const { payload, sig, allowedSigners } = input

  if (sig.proposal_hash !== payload.proposal_hash) {
    return { ok: false, reason: 'proposal hash mismatch' }
  }
  if (!allowedSigners.includes(sig.signer_pubkey)) {
    return { ok: false, reason: 'signer is not in the allowed set' }
  }

  let txHashBytes: Uint8Array
  try {
    txHashBytes = transactionHash(payload.unsigned_xdr, payload.network_passphrase)
  } catch (error) {
    return { ok: false, reason: `cannot hash unsigned_xdr: ${(error as Error).message}` }
  }

  let sigBytes: Uint8Array
  try {
    sigBytes = base64ToBytes(sig.signature_b64)
  } catch {
    return { ok: false, reason: 'signature is not valid base64' }
  }

  if (!ed25519Verify(sig.signer_pubkey, txHashBytes, sigBytes)) {
    return { ok: false, reason: 'signature does not verify' }
  }

  return { ok: true }
}

// Internals ------------------------------------------------------------------

function countEnvelopeSignatures(envelopeXdrBase64: string): number {
  const env = xdr.TransactionEnvelope.fromXDR(envelopeXdrBase64, 'base64')
  switch (env.switch().value) {
    case xdr.EnvelopeType.envelopeTypeTx().value:
      return env.v1().signatures().length
    case xdr.EnvelopeType.envelopeTypeTxV0().value:
      return env.v0().signatures().length
    case xdr.EnvelopeType.envelopeTypeTxFeeBump().value:
      return env.feeBump().signatures().length
    default:
      return 0
  }
}

function extractAddedSignature(
  beforeXdrBase64: string,
  afterXdrBase64: string,
  beforeCount: number,
): string | null {
  const env = xdr.TransactionEnvelope.fromXDR(afterXdrBase64, 'base64')
  let sigs: ReadonlyArray<xdr.DecoratedSignature> = []
  switch (env.switch().value) {
    case xdr.EnvelopeType.envelopeTypeTx().value:
      sigs = env.v1().signatures()
      break
    case xdr.EnvelopeType.envelopeTypeTxV0().value:
      sigs = env.v0().signatures()
      break
    case xdr.EnvelopeType.envelopeTypeTxFeeBump().value:
      sigs = env.feeBump().signatures()
      break
  }
  if (sigs.length <= beforeCount) { return null }
  // Take the last appended sig — wallets append.
  const newSig = sigs.at(-1)
  if (!newSig) { return null }
  return bytesToBase64(new Uint8Array(newSig.signature()))
}

function transactionHash(unsignedXdrBase64: string, networkPassphrase: string): Uint8Array {
  const tx = new Transaction(unsignedXdrBase64, networkPassphrase)
  return new Uint8Array(tx.hash())
}

function makeDecoratedSignature(signerPubkeyG: string, signatureB64: string): xdr.DecoratedSignature {
  const kp = Keypair.fromPublicKey(signerPubkeyG)
  const hint = kp.signatureHint()
  return new xdr.DecoratedSignature({
    hint,
    signature: Buffer.from(base64ToBytes(signatureB64)),
  })
}

function ed25519Verify(signerPubkeyG: string, message: Uint8Array, sig: Uint8Array): boolean {
  try {
    const kp = Keypair.fromPublicKey(signerPubkeyG)
    return kp.verify(Buffer.from(message), Buffer.from(sig))
  } catch {
    return false
  }
}

function base64ToBytes(b64: string): Uint8Array {
  const bin = atob(b64)
  const out = new Uint8Array(bin.length)
  for (let i = 0; i < bin.length; i++) { out[i] = bin.codePointAt(i) ?? 0 }
  return out
}

function bytesToBase64(bytes: Uint8Array): string {
  let bin = ''
  for (const byte of bytes) { bin += String.fromCodePoint(byte ?? 0) }
  return btoa(bin)
}

// Re-export the network passphrases as a convenience for callers.
export const NETWORK_PASSPHRASES = {
  mainnet: Networks.PUBLIC,
  testnet: Networks.TESTNET,
  futurenet: Networks.FUTURENET,
} as const
