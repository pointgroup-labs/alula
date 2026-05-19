/**
 * Multisig coordination store.
 *
 * Holds the currently-loaded proposal + the running list of validated
 * signatures. Pages call into this; the store handles relay sync, wallet
 * signing, and submission. Signature verification happens at the seam
 * (relay-fetch and paste-time) so invalid sigs never enter `sigs`.
 *
 * The store is intentionally per-tab state, not persisted: a proposal
 * lives in the URL fragment and the relay; reload regenerates it.
 */

import type {
  ProposalPayload,
  SigPayload,
  SubmitProposalResult,
} from '~/utils/multisig'
import { defineStore } from 'pinia'
import {
  decodeProposal,
  decodeProposalFromFragment,
  fetchSigs,
  postSig,
  signProposal,
  submitProposal,
  verifySigPayload,
} from '~/utils/multisig'

type RelayError = {
  when: number
  message: string
}

export const useMultisigStore = defineStore('multisig', () => {
  const config = useRuntimeConfig()
  const rpcStore = useRpcStore()
  const { signEnvelopeXdr } = useMultisigSigner()

  const proposal = ref<ProposalPayload | null>(null)
  const sigs = ref<SigPayload[]>([])
  const decodingError = ref<string | null>(null)
  const loading = ref(false)
  const submitting = ref(false)
  const lastRelayError = ref<RelayError | null>(null)
  const lastSubmit = ref<SubmitProposalResult | null>(null)

  const relayBaseUrl = computed(() => {
    const fromConfig = (config as { public?: { MULTISIG_RELAY_URL?: string } }).public?.MULTISIG_RELAY_URL
    return fromConfig || 'https://multisig-relay.alula.workers.dev'
  })

  const allowedSigners = computed(() =>
    proposal.value?.signer_set_snapshot.map(s => s.key) ?? [],
  )

  // Signer-weight lookup snapshot. Threshold uses the medium threshold by
  // convention (Soroban host functions count as medium-threshold ops).
  const signerWeight = computed(() => {
    const m = new Map<string, number>()
    for (const s of proposal.value?.signer_set_snapshot ?? []) {
      m.set(s.key, s.weight)
    }
    return m
  })

  const requiredThreshold = computed(() => proposal.value?.thresholds_snapshot.med ?? 0)

  const collectedWeight = computed(() => {
    let total = 0
    for (const sig of sigs.value) {
      total += signerWeight.value.get(sig.signer_pubkey) ?? 0
    }
    return total
  })

  const thresholdMet = computed(() =>
    requiredThreshold.value > 0 && collectedWeight.value >= requiredThreshold.value,
  )

  function reset() {
    proposal.value = null
    sigs.value = []
    decodingError.value = null
    lastRelayError.value = null
    lastSubmit.value = null
  }

  async function loadFromFragment(fragment: string) {
    reset()
    loading.value = true
    try {
      const payload = decodeProposalFromFragment(fragment)
      // Integrity check: payload.args must re-encode to the ScVal vector
      // in unsigned_xdr; otherwise refuse to surface the proposal.
      await decodeProposal(payload)
      proposal.value = payload
      await refreshSigs()
    } catch (error) {
      decodingError.value = (error as Error).message
    } finally {
      loading.value = false
    }
  }

  async function refreshSigs() {
    if (!proposal.value) { return }
    const result = await fetchSigs(
      { baseUrl: relayBaseUrl.value },
      proposal.value.proposal_hash,
    )
    if (!result.ok) {
      lastRelayError.value = { when: Date.now(), message: result.error ?? 'unknown relay error' }
      return
    }

    // Re-validate every sig the relay returns; the relay is untrusted.
    const validated: SigPayload[] = []
    const seen = new Set<string>()
    for (const sig of result.data ?? []) {
      if (seen.has(sig.signer_pubkey)) { continue }
      const verdict = verifySigPayload({
        payload: proposal.value,
        sig,
        allowedSigners: allowedSigners.value,
      })
      if (verdict.ok) {
        validated.push(sig)
        seen.add(sig.signer_pubkey)
      }
    }
    sigs.value = validated
  }

  async function signCurrent(): Promise<SigPayload> {
    if (!proposal.value) { throw new Error('no proposal loaded') }
    const sig = await signProposal({
      payload: proposal.value,
      signEnvelopeXdr,
    })

    // Local verification before relay POST. This catches a signer that's
    // been rotated off the multisig since the snapshot was taken.
    const verdict = verifySigPayload({
      payload: proposal.value,
      sig,
      allowedSigners: allowedSigners.value,
    })
    if (!verdict.ok) {
      throw new Error(`refusing to relay invalid sig: ${verdict.reason}`)
    }

    const post = await postSig(
      { baseUrl: relayBaseUrl.value },
      sig,
    )
    if (!post.ok) {
      lastRelayError.value = { when: Date.now(), message: post.error ?? 'unknown relay error' }
    }

    // Add to local list whether or not the relay accepted it; the user can
    // always paste it manually as a fallback.
    if (!sigs.value.some(s => s.signer_pubkey === sig.signer_pubkey)) {
      sigs.value.push(sig)
    }
    return sig
  }

  function addSigPayload(sig: SigPayload): { ok: true } | { ok: false, reason: string } {
    if (!proposal.value) { return { ok: false, reason: 'no proposal loaded' } }
    const verdict = verifySigPayload({
      payload: proposal.value,
      sig,
      allowedSigners: allowedSigners.value,
    })
    if (!verdict.ok) { return { ok: false, reason: verdict.reason ?? 'invalid sig' } }
    if (sigs.value.some(s => s.signer_pubkey === sig.signer_pubkey)) {
      return { ok: false, reason: 'duplicate signer' }
    }
    sigs.value.push(sig)
    return { ok: true }
  }

  async function submitCurrent(): Promise<SubmitProposalResult> {
    if (!proposal.value) { throw new Error('no proposal loaded') }
    if (!thresholdMet.value) { throw new Error('threshold not yet met') }
    submitting.value = true
    try {
      const res = await submitProposal({
        payload: proposal.value,
        sigs: sigs.value,
        rpcUrl: rpcStore.sorobanRPCUrl,
      })
      lastSubmit.value = res
      return res
    } finally {
      submitting.value = false
    }
  }

  return {
    proposal,
    sigs,
    decodingError,
    loading,
    submitting,
    lastRelayError,
    lastSubmit,

    allowedSigners,
    requiredThreshold,
    collectedWeight,
    thresholdMet,
    relayBaseUrl,

    reset,
    loadFromFragment,
    refreshSigs,
    signCurrent,
    addSigPayload,
    submitCurrent,
  }
})
